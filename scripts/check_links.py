#!/usr/bin/env python3
"""Check links while resolving this repository's main-branch URLs locally (§AR-ci.3.1), and
settle a github.com 5xx through GitHub's API (§AR-ci.3.1.2) and any other transient failure
through one more lychee pass (§AR-ci.3.1.3).

With no flag this is the gate: the internal pass over the whole tree, offline, then only the
external URLs on the lines the change touches (§AR-ci.3.1.4). `--all` is the sweep: the
internal pass, then every external URL of the tree (§AR-ci.3.1.5).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Sequence
from urllib.parse import unquote, urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_scope import (  # noqa: E402
    NameNotUtf8, changed_lines, changed_urls, diff_base, dump_lines,
)
from link_settle import check_urls, report_failures, settle, write_report  # noqa: E402


# The one set of self-link forms every step reads (§AR-ci-self-links.1).
SELF_PATH_PREFIXES = {
    "blob": "/agent-grounds/grund/blob/main/",
    "tree": "/agent-grounds/grund/tree/main/",
}
SELF_PREFIXES = tuple(f"https://github.com{prefix}" for prefix in SELF_PATH_PREFIXES.values())
RECHECK_PAUSE = 5  # seconds before the transient re-check (§AR-ci.3.1.3)
# `subprocess.run` or a test's stand-in. Lychee's two dumps read its output as bytes
# (§AR-ci.3.1.4.1), the `gh api` re-check reads it as text, and the other calls read none.
Runner = Callable[..., subprocess.CompletedProcess[Any]]


class SelfLinkError(Exception):
    pass


def self_links(dumped: bytes) -> list[str]:
    """Every self-link of a `lychee --dump`, in either form (§AR-ci-self-links.1), read as the
    gate reads every dump (§AR-ci.3.1.4.1)."""
    return sorted({line for line in dump_lines(dumped) if line.startswith(SELF_PREFIXES)})


def self_link_target(url: str, repo_root: Path) -> Path:
    """The checkout path a `blob/main` (file) or `tree/main` (directory) self-link names.

    Shared with the printed-link test, so the CLI's guide links and this gate
    agree on what a self-link names (§FS-cli.2.3).
    """
    parsed = urlsplit(url)
    kind = next(
        (k for k, prefix in SELF_PATH_PREFIXES.items() if parsed.path.startswith(prefix)), None
    )
    if (
        parsed.scheme != "https"
        or parsed.netloc != "github.com"
        or kind is None
        or parsed.query
    ):
        raise SelfLinkError(f"malformed canonical self-link: {url}")
    if kind == "tree" and parsed.fragment:
        # §AR-ci-self-links.3: a directory link names no heading; link its file instead.
        raise SelfLinkError(
            f"malformed canonical self-link: a tree/main link carries no fragment: {url}"
        )

    encoded = parsed.path.removeprefix(SELF_PATH_PREFIXES[kind])
    try:
        relative = unquote(encoded, errors="strict")
    except UnicodeError as exc:
        raise SelfLinkError(f"invalid path encoding in canonical self-link: {url}") from exc
    parts = PurePosixPath(relative).parts
    if not relative or any(part in ("", ".", "..") for part in parts):
        raise SelfLinkError(f"unsafe path in canonical self-link: {url}")

    root = repo_root.resolve()
    target = (root / Path(*parts)).resolve()
    try:
        target.relative_to(root)
    except ValueError as exc:
        raise SelfLinkError(f"self-link escapes the repository: {url}") from exc
    # §AR-ci-self-links.2: `blob` names a file, `tree` a directory.
    exists = target.is_file() if kind == "blob" else target.is_dir()
    if not exists:
        raise SelfLinkError(f"canonical self-link target is missing: {relative} ({url})")
    return target


def _local_url(url: str, repo_root: Path) -> str:
    """The `file://` URI a self-link of either form maps to in the checkout
    (§AR-ci-self-links.1); `self_link_target()` alone decides what is a self-link."""
    target = self_link_target(url, repo_root)
    fragment = urlsplit(url).fragment
    fragment = f"#{fragment}" if fragment else ""
    return f"{target.as_uri()}{fragment}"


def internal_pass(
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
    runner: Runner = subprocess.run,
) -> tuple[int, list[str]]:
    """Every internal link of the inputs, without the network: same-repository URLs against
    the checkout, then relative paths and fragments offline (§AR-ci.3.1). Returns lychee's
    exit code and the self-links that passed, which no network pass may request."""
    dumped = runner(
        [lychee, "--dump", *inputs],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        check=False,
    )
    if dumped.returncode != 0:
        return dumped.returncode, []
    urls = self_links(dumped.stdout)
    if urls:
        local_document = "".join(f"[self-link](<{_local_url(url, repo_root)}>)\n" for url in urls)
        local = runner(
            [lychee, "--no-progress", "--include-fragments", "-"],
            cwd=repo_root,
            input=local_document,
            text=True,
            check=False,
        )
        if local.returncode != 0:
            return local.returncode, urls
    offline = runner(
        [lychee, "--no-progress", "--offline", "--include-fragments", *inputs],
        cwd=repo_root,
        check=False,
    )
    return offline.returncode, urls


def config_excludes(repo_root: Path) -> list[str]:
    """`lychee.toml`'s `exclude` list, which every network pass that names exclusions of its own
    must carry first, because lychee 0.23.0 lets a command-line `exclude` replace the config's
    list (§AR-ci.3.1.1.1). It reads the file lychee itself loads: lychee takes `lychee.toml` from
    its working directory, and every pass runs with `cwd=repo_root` and no `--config`. A missing
    file or `exclude` key gives none; the patterns stay declared in `lychee.toml` alone."""
    try:
        text = (repo_root / "lychee.toml").read_text(encoding="utf-8")
    except FileNotFoundError:
        return []
    return list(tomllib.loads(text).get("exclude", []))


def check_links(
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
    runner: Runner = subprocess.run,
    kept: list[tuple[str, str, dict, str | None]] | None = None,
) -> int:
    """The sweep, `--all`: the internal pass, then one network pass over every external URL
    of the inputs (§AR-ci.3.1.5). `kept` collects the failures the run keeps."""
    returncode, urls = internal_pass(inputs, repo_root, lychee, runner)
    if returncode != 0:
        return returncode

    command = [lychee, "--no-progress", "--include-fragments"]
    # The self-links extend the config's exclusions, never replace them (§AR-ci.3.1.1.1).
    for pattern in [*config_excludes(repo_root), *(f"^{re.escape(url)}$" for url in urls)]:
        command.extend(["--exclude", pattern])
    command.extend(["--scheme", "http", "--scheme", "https"])
    with tempfile.TemporaryDirectory(prefix="grund-links-") as scratch:
        report_path = Path(scratch) / "lychee.json"
        command.extend(["--format", "json", "--output", str(report_path), *inputs])
        returncode = runner(command, cwd=repo_root, check=False).returncode
        if returncode == 0:
            return 0
        failures = report_failures(report_path)
    if failures is None:
        return returncode
    return settle(
        failures, repo_root, runner, returncode, lychee, report=kept, pause=RECHECK_PAUSE
    )


def check_changed(
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
    runner: Runner = subprocess.run,
    link_base: str | None = None,
) -> int:
    """The gate: the internal pass, then a network pass over only the external URLs on lines
    the change adds or modifies (§AR-ci.3.1.4); none at all when it touches none."""
    returncode, self_urls = internal_pass(inputs, repo_root, lychee, runner)
    if returncode != 0:
        return returncode
    base = diff_base(repo_root, link_base)
    if base is None:
        print(
            "no diff base: LINK_BASE names no commit, or is unset and HEAD has no merge base "
            "with origin/main, so no external URL is checked; the scheduled sweep checks them all"
        )
        return 0
    changed = changed_lines(repo_root, base, inputs)
    for name in sorted(name for name in changed if "\n" in name):
        print(  # lychee's dump writes this name across two lines (§AR-ci.3.1.4.1)
            f"not requested: {json.dumps(name, ensure_ascii=False)} holds a newline, which "
            "lychee's dump cannot name, so no URL on its changed lines is checked; the "
            "scheduled sweep checks them"
        )
    if not changed:
        return 0
    dumped = runner(  # bytes: text mode would turn a \r in a file name into \n (§AR-ci.3.1.4.1)
        [lychee, "--dump", "--verbose", *inputs],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if dumped.returncode != 0:
        return dumped.returncode
    locations = changed_urls(dumped.stdout, changed, set(self_urls))
    if not locations:
        return 0
    returncode, failures = check_urls(list(locations), repo_root, runner, lychee, "link")
    if returncode == 0:
        return 0
    if failures is None:
        return returncode
    # The document lychee read is stdin; name the changed lines instead (§AR-ci.3.1.4).
    failures = [
        (", ".join(sorted(set(locations.get(url, [source])))), url, status)
        for source, url, status in failures
    ]
    return settle(failures, repo_root, runner, returncode, lychee, pause=RECHECK_PAUSE)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check links, resolving canonical same-repository main-branch URLs locally."
        " With no flag, only the external URLs on lines changed since the diff base go to the"
        " network (LINK_BASE, else the merge base with origin/main)."
    )
    parser.add_argument("--lychee", default="lychee", help="lychee executable (default: lychee)")
    parser.add_argument(
        "--all", action="store_true",
        help="the full pass: every external URL of the inputs, as the nightly sweep runs it",
    )
    parser.add_argument(
        "--report", type=Path, metavar="PATH",
        help="with --all, write each failing URL with every file and line that carries it",
    )
    parser.add_argument("inputs", nargs="+", help="files and directories passed to lychee")
    args = parser.parse_args(argv)
    if args.report is not None and not args.all:
        parser.error("--report needs --all")

    repo_root = Path.cwd()
    try:
        if not args.all:
            return check_changed(
                args.inputs, repo_root, args.lychee, link_base=os.environ.get("LINK_BASE")
            )
        kept: list[tuple[str, str, dict, str | None]] = []
        returncode = check_links(args.inputs, repo_root, args.lychee, kept=kept)
        if args.report is not None:
            write_report(args.report, kept, args.inputs, repo_root, args.lychee)
        return returncode
    except (SelfLinkError, NameNotUtf8) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
