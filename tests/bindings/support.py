"""The oracle side of binding acceptance, shared by every binding (§FS-distribution.3.0.3).

The oracle is a test driver over core, supplied by the implementer, never a CLI
adapter. It reads one JSON request and returns data plus independently encoded
canonical bytes. Each binding's own side lives in its directory: Python's is
`python/adapter.py`, as Node's is `node/adapter.mjs` (§AR-bindings.1).
"""

import base64
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag/tmp"
SCRATCH.mkdir(parents=True, exist_ok=True)
ADAPTERS = ("rust", "python")  # Node adds an adapter here when #469 lands.


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


def rust_call(operation, root, args=(), options=None, home=None):
    # §AR-bindings.6.2: the gate's same-source oracle when it hands one, else the checkout's.
    oracle = Path(os.environ.get("GRUND_BINDINGS_ORACLE") or REPO / "target/debug/grund-binding-oracle")
    if os.name == "nt" and not os.environ.get("GRUND_BINDINGS_ORACLE"):
        oracle = oracle.with_suffix(".exe")
    if not oracle.is_file():
        raise AssertionError(f"missing core-only test driver {oracle}; "
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
