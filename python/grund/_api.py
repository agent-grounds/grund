"""Disk-backed synchronous API (§FS-distribution.3.3.5)."""

import json
import os
from collections.abc import Sequence
from typing import Any

from . import _native
from ._convert import convert
from .errors import (ConfigError, FilesystemError, OperationError, QueryError,
                     PathEncodingError)
from .types import (BatchResult, CheckResult, CompletionResult, ConfigResult,
                    CoverResult, CoverTextResult, Failure, FetchResult, FmtResult,
                    IdProposal, InitResult, IntegrationsResult, ListResult, SizesResult, ReferenceStyle,
                    RefsResult, ScanResult, SetupInstructions, ShowQuery, ShowResult)

PathInput = str | os.PathLike[str]


def _path(value: PathInput | None) -> tuple[str, bool]:
    """Snapshot cwd once and reject invalid encodings (§FS-distribution.3.3.3)."""
    explicit = value is not None
    cwd = os.getcwd()
    text = cwd if value is None else os.fspath(value)
    if not isinstance(text, str):
        raise TypeError("paths must be str or os.PathLike[str]")
    if any(0xD800 <= ord(c) <= 0xDFFF for c in text):
        raise PathEncodingError("paths must contain Unicode scalar values")
    if "\0" in text:
        raise ValueError("paths must not contain NUL")
    # §FS-distribution.3.3.3: symlink/.. must reach engine filesystem resolution.
    return os.path.join(cwd, text), explicit


def _str(value: Any, name: str, optional: bool = False) -> None:
    if optional and value is None:
        return
    if not isinstance(value, str):
        raise TypeError(f"{name} must be str")
    if any(0xD800 <= ord(c) <= 0xDFFF for c in value):
        raise ValueError(f"{name} must contain Unicode scalar values")
    if "\0" in value:
        raise ValueError(f"{name} must not contain NUL")


def _strings(value: Any, name: str) -> tuple[str, ...]:
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise TypeError(f"{name} must be a sequence of strings")
    for item in value:
        _str(item, name)
    return tuple(value)


def _call(operation: str, cls: type, root: PathInput | None,
          args: tuple = (), **options: Any) -> Any:
    """Conversion only; engine outcomes choose exceptions (§FS-distribution.3.3.2)."""
    path, explicit = ("", False) if operation in ("integrations", "agent_setup_instructions") else _path(root)
    for name, value in options.items():
        if name in ("write", "check", "docs", "force", "no_vcs", "suggestions", "full",
                    "require_grounding", "only_rule", "unused", "sections", "text",
                    "marker", "cross_refs", "descendants") and type(value) is not bool:
            raise TypeError(f"{name} must be bool")
        if name in ("section", "rule", "selector", "name", "description",
                    "conversation", "conversation_target", "agent"):
            _str(value, name, optional=True)
    envelope = json.loads(_native.call(operation, path, explicit,
                                       json.dumps(args), json.dumps(options)))
    if envelope["failure"] is not None:
        failure = convert(Failure, envelope["failure"])
        if failure.kind == "path-encoding":
            raise PathEncodingError(failure.message)
        exception = {"config": ConfigError, "filesystem": FilesystemError,
                     "query": QueryError}.get(failure.kind, OperationError)
        raise exception(failure)
    return convert(cls, envelope["result"])


def check(root: PathInput | None = None, *, require_grounding: bool = False,
          suggestions: bool = False, full: bool = False, rule: str | None = None,
          only: Sequence[str] = (), ignore: Sequence[str] = (),
          only_rule: bool = False) -> CheckResult:
    """Return complete and selected reports; findings are normal results."""
    if type(only_rule) is not bool:
        raise TypeError("only_rule must be bool")
    if only_rule and rule is None:
        raise ValueError("only_rule requires rule")
    only, ignore = _strings(only, "only"), _strings(ignore, "ignore")
    if any(not _native.valid_option("code", code) for code in only + ignore):
        raise ValueError("unknown check finding code")
    return _call("check", CheckResult, root, require_grounding=require_grounding,
                 suggestions=suggestions, full=full, rule=rule, only=only,
                 ignore=ignore, only_rule=only_rule)


def scan(root: PathInput) -> ScanResult:
    """Read a scanner snapshot at an explicit path."""
    if root is None:
        raise TypeError("scan requires an explicit path")
    return _call("scan", ScanResult, root)


def _mode(mode: str) -> None:
    _str(mode, "mode")
    if mode not in ("lead", "brief", "toc", "full"):
        raise ValueError("mode must be lead, brief, toc or full")


def show(id: str, *, section: str | None = None, mode: str = "lead",
         format: str = "text", root: PathInput | None = None) -> ShowResult:
    """Read one coordinate, or raise a structured QueryError."""
    _str(id, "id")
    _mode(mode)
    _str(format, "format")
    if format not in ("text", "md", "json"):
        raise ValueError("format must be text, md or json")
    return _call("show", ShowResult, root, (id,), section=section, mode=mode, format=format)


def show_batch(queries: Sequence[str | ShowQuery] | None = None, *,
               mode: str = "lead", root: PathInput | None = None) -> BatchResult:
    """None discovers all coordinates; empty input avoids config loading."""
    _mode(mode)
    if queries is not None:
        if isinstance(queries, (str, bytes)) or not isinstance(queries, Sequence):
            raise TypeError("queries must be a sequence of strings or ShowQuery")
        converted = []
        for query in queries:
            if isinstance(query, ShowQuery):
                _str(query.id, "id")
                _str(query.section, "section", optional=True)
                converted.append({"id": query.id, "section": query.section})
            else:
                _str(query, "query")
                converted.append(query)
        operands = (converted,)
    else:
        operands = ()
    return _call("show_batch", BatchResult, root, operands, mode=mode)


def refs(id: str, *, section: str | None = None, descendants: bool = False,
         root: PathInput | None = None) -> RefsResult:
    """Read citation sites, file summaries and totals."""
    _str(id, "id")
    return _call("refs", RefsResult, root, (id,), section=section, descendants=descendants)


def list_ids(*, kinds: Sequence[str] = (), projects: Sequence[str] = (),
             unused: bool = False, selector: str | None = None,
             root: PathInput | None = None) -> ListResult:
    """Read the declaration/chapter catalog and kind summaries."""
    return _call("list_ids", ListResult, root, kinds=_strings(kinds, "kinds"),
                 projects=_strings(projects, "projects"), unused=unused, selector=selector)


def list_sizes(*, kinds: Sequence[str] = (), projects: Sequence[str] = (),
               unused: bool = False, selector: str | None = None,
               units: Sequence[str] = ("lines", "words", "bytes"),
               top: int | None = None, root: PathInput | None = None) -> SizesResult:
    """Read lead/full measurements, preserving requested unit order."""
    units = _strings(units, "units")
    if not units or any(u not in ("lines", "words", "bytes") for u in units):
        raise ValueError("units must contain lines, words or bytes")
    if top is not None:
        if type(top) is not int:
            raise TypeError("top must be int")
        if top <= 0:
            raise ValueError("top must be positive")
    return _call("list_sizes", SizesResult, root, kinds=_strings(kinds, "kinds"),
                 projects=_strings(projects, "projects"), unused=unused, selector=selector,
                 units=units, top=top)


def cover(*, text: bool = False, root: PathInput | None = None) -> CoverResult | CoverTextResult:
    """Read file-grouped coverage, with an optional text-oriented record."""
    return _call("cover", CoverTextResult if text else CoverResult, root, text=text)


def fmt(*, write: bool = False, marker: bool = False, cross_refs: bool = False,
        root: PathInput | None = None) -> FmtResult:
    """Preview formatting; write=True applies only engine-owned edits."""
    return _call("fmt", FmtResult, root, write=write, marker=marker, cross_refs=cross_refs)


def propose_id(kind: str, title: str, *, width: int = 3,
               root: PathInput | None = None) -> IdProposal:
    """Propose a conflict-free ID without writing."""
    _str(kind, "kind")
    _str(title, "title")
    if type(width) is not int:
        raise TypeError("width must be int")
    if width < 0:
        raise ValueError("width must be non-negative")
    return _call("propose_id", IdProposal, root, (kind, title), width=width)


def init(target: PathInput | None = None, *, name: str | None = None,
         description: str | None = None, docs: bool = False, force: bool = False,
         write: bool = False, check: bool = False, no_vcs: bool = False,
         agents: Sequence[str] | None = None) -> InitResult:
    """Preview scaffolding; check always suppresses writes; force preserves config."""
    if agents is not None:
        agents = _strings(agents, "agents")
        if any(a not in ("canonical", "codex", "agents", "claude", "gemini", "pi",
                         "copilot", "cursor", "windsurf", "zed") for a in agents):
            raise ValueError("unknown init agent")
    return _call("init", InitResult, target, name=name, description=description, docs=docs,
                 force=force, write=write, check=check, no_vcs=no_vcs, agents=agents)


def effective_config(*, root: PathInput | None = None) -> ConfigResult:
    """Read effective settings as recursively read-only schema mappings."""
    return _call("effective_config", ConfigResult, root)


def validate_config(*, root: PathInput | None = None) -> ConfigResult:
    """Validate config and workspace members without scanning source files."""
    return _call("validate_config", ConfigResult, root)


def fetch(id: str, *, write: bool = False, root: PathInput | None = None) -> FetchResult:
    """Materialize a snapshot only with explicit execution opt-in."""
    _str(id, "id")
    if type(write) is not bool:
        raise TypeError("write must be bool")
    if not write:
        raise ValueError("fetch requires write=True; it has no preview")
    return _call("fetch", FetchResult, root, (id,), write=write)


def integrations(client: str | None = None, *, write: bool = False,
                 conversation: str | None = None, conversation_target: str | None = None,
                 agent: str | None = None) -> IntegrationsResult:
    """Inspect or install user-global integration artifacts and guidance."""
    _str(client, "client", optional=True)
    if type(write) is not bool:
        raise TypeError("write must be bool")
    for name, value in (("client", client), ("conversation", conversation),
                        ("conversation_target", conversation_target), ("agent", agent)):
        _str(value, name, optional=True)
        if value is not None and not _native.valid_option(name, value):
            raise ValueError(f"unknown {name}")
    if (not write and any(v is not None for v in (conversation, conversation_target, agent))) \
            or (agent is not None and conversation_target is None) \
            or (write and client is None and conversation is None and conversation_target is None):
        raise ValueError("invalid integration preference combination")
    return _call("integrations", IntegrationsResult, None, (client or "",), write=write,
                 conversation=conversation, conversation_target=conversation_target, agent=agent)


def complete_ids(prefix: str = "", *, sections: bool = False,
                 root: PathInput | None = None) -> CompletionResult:
    """Complete IDs, aliases and optionally sections."""
    _str(prefix, "prefix")
    return _call("complete_ids", CompletionResult, root, (prefix,), sections=sections)


def reference_style(*, root: PathInput | None = None) -> ReferenceStyle:
    """Read this path's marker and typing trigger."""
    return _call("reference_style", ReferenceStyle, root)


def agent_setup_instructions() -> SetupInstructions:
    """Return the engine's canonical setup payload."""
    return _call("agent_setup_instructions", SetupInstructions, None)
