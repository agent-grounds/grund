"""Acceptance helpers and future Node seam (§FS-distribution.3.0.3).

The oracle is a test driver over core, supplied by the implementer, never a CLI
adapter. It reads one JSON request and returns data plus independently encoded
canonical bytes. Tests intentionally do not implement the missing core adapters.
"""

import base64
import dataclasses
import importlib
import importlib.metadata
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Mapping

REPO = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag/tmp"
SCRATCH.mkdir(parents=True, exist_ok=True)
ADAPTERS = ("rust", "python")  # Node adds an adapter here when #469 lands.


def binding():
    module = importlib.import_module("grund")
    location = Path(module.__file__).resolve()
    if location.name != "__init__.py" or location.parent.name != "grund":
        raise AssertionError(f"not the approved mixed Python package: {location}")
    native = importlib.import_module("grund._native")
    if not Path(native.__file__).resolve().is_relative_to(location.parent):
        raise AssertionError("native extension and Python package have different origins")
    direct = importlib.metadata.distribution("grund").read_text("direct_url.json")
    from urllib.parse import urlparse
    from urllib.request import url2pathname
    if not direct:
        raise AssertionError("acceptance requires a local-source install, not a registry package")
    source = Path(url2pathname(urlparse(json.loads(direct)["url"]).path)).resolve()
    expected = Path(os.environ.get("GRUND_ACCEPTANCE_SOURCE", REPO)).resolve()
    if source != expected:
        raise AssertionError(f"unrelated installed grund: {source}; expected {expected}")
    return module


def temporary():
    return tempfile.TemporaryDirectory(prefix="grund-python-contract-", dir=SCRATCH)


def fixture(parent, case="json-report"):
    root = Path(parent) / "repo"
    source = "json-report" if case == "clean" else case
    shutil.copytree(REPO / "tests/e2e/cases" / source / "repo", root)
    if case == "clean":
        declaration = root / "docs/functional-spec/FS-001-alpha.md"
        declaration.write_text(declaration.read_text().replace("FS-999-missing", "FS-002-beta"))
    return root


def tree_bytes(root):
    return {str(p.relative_to(root)): p.read_bytes()
            for p in sorted(Path(root).rglob("*")) if p.is_file()}


def plain(value):
    """Lossless host records: no selected-field or message-only projection."""
    if dataclasses.is_dataclass(value):
        return {f.name: plain(getattr(value, f.name)) for f in dataclasses.fields(value)}
    if isinstance(value, Mapping):
        return {k: plain(v) for k, v in value.items()}
    if isinstance(value, tuple):
        return [plain(v) for v in value]
    if value is None or type(value) in (str, bool, int, float):
        return value
    raise AssertionError(f"untyped/non-contract result: {type(value)!r}")


def canonical(value):
    def encode(item):
        if isinstance(item, str):
            escapes = {'"': '\\"', '\\': '\\\\', '\n': '\\n', '\r': '\\r', '\t': '\\t'}
            return '"' + ''.join(escapes.get(c, f'\\u{ord(c):04x}' if ord(c) < 32 else c)
                                 for c in item) + '"'
        if isinstance(item, dict):
            return '{' + ','.join(encode(k) + ':' + encode(item[k])
                                  for k in sorted(item, key=lambda k: k.encode())) + '}'
        if isinstance(item, list):
            return '[' + ','.join(encode(v) for v in item) + ']'
        return json.dumps(item, allow_nan=False, separators=(',', ':'))
    return (encode(value) + "\n").encode()


def python_call(module, operation, root, args=(), options=None):
    # Capture native file descriptors as well as Python writes on every corpus
    # operation. This helper is used serially; concurrency tests call the API directly.
    with temporary() as temp, open(Path(temp) / "streams", "w+b") as streams:
        sys.stdout.flush()
        sys.stderr.flush()
        saved = [os.dup(fd) for fd in (1, 2)]
        cwd = Path.cwd()
        try:
            for fd in (1, 2):
                os.dup2(streams.fileno(), fd)
            result = _python_call(module, operation, root, args, options)
            sys.stdout.flush()
            sys.stderr.flush()
        finally:
            for fd, original in zip((1, 2), saved):
                os.dup2(original, fd)
                os.close(original)
        streams.seek(0)
        output = streams.read()
        if output:
            raise AssertionError(f"{operation} wrote process streams: {output!r}")
        if Path.cwd() != cwd:
            raise AssertionError(f"{operation} changed global cwd")
        return result


def _python_call(module, operation, root, args=(), options=None):
    keywords = dict(options or {})
    if operation not in ("integrations", "agent_setup_instructions"):
        if operation == "scan":
            args = (root,)
        elif operation == "init":
            args = (root,)
        else:
            keywords["root"] = root
    try:
        result = getattr(module, operation)(*args, **keywords)
    except module.GrundError as error:
        failure = plain(error.failure)
        return {"failure": failure, "result": None,
                "run_cautions": failure["run_cautions"]}
    data = plain(result)
    return {"failure": None, "result": data,
            "run_cautions": data.get("run_cautions", [])}


def rust_call(operation, root, args=(), options=None, home=None):
    oracle = REPO / "target/debug/grund-binding-oracle"
    if os.name == "nt":
        oracle = oracle.with_suffix(".exe")
    if not oracle.is_file():
        raise AssertionError("missing core-only test driver target/debug/grund-binding-oracle; "
                             "implement the tests/bindings/README.md protocol")
    request = {"operation": operation, "root": str(root), "args": list(args),
               "options": options or {}, "home": str(home) if home else None}
    run = subprocess.run([str(oracle)], input=json.dumps(request), text=True,
                         capture_output=True, check=True, cwd=REPO)
    if run.stderr:
        raise AssertionError(f"oracle wrote stderr: {run.stderr}")
    response = json.loads(run.stdout)
    data = response["data"]
    wire = base64.b64decode(response["canonical"], validate=True)
    if wire != canonical(data):
        raise AssertionError("Rust canonical bytes violate the specified encoding")
    return data, wire, response
