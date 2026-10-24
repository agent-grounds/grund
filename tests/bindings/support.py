"""The oracle side of binding acceptance, shared by every binding (§FS-distribution.3.0.3).

The oracle is a test driver over core, supplied by the implementer, never a CLI
adapter. It reads one JSON request and returns data plus independently encoded
canonical bytes. Each binding's own side lives in its directory: Python's is
`python/adapter.py`, as Node's is `node/adapter.mjs` (§AR-bindings.1).
"""

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag/tmp"
SCRATCH.mkdir(parents=True, exist_ok=True)
ADAPTERS = ("rust", "python")  # Node adds an adapter here when #469 lands.


def python_root():
    """The shared root every checkout's Python target lives under (§AR-ci.3.4.5).

    `GRUND_PYTHON_TARGET` when given, else a directory outside the checkout kept between
    runs, so the third-party dependency tree is compiled once rather than on every install.
    """
    given = os.environ.get("GRUND_PYTHON_TARGET")
    return Path(given).expanduser() if given else SCRATCH / "grund-python-target"


def python_target():
    """This checkout's Cargo target under `python_root()`, which every Python acceptance
    build shares (§AR-ci.3.4.5)."""
    return checkout_target(python_root())


def checkout_key(checkout):
    """Keep the existing resolved-source digest (§FS-repository-maintenance.2.1)."""
    return hashlib.sha256(os.fsencode(checkout)).hexdigest()[:16]


def _replace_record(temporary, record):
    """Wait out a Windows reader's open handle on the record (§FS-repository-maintenance.2.4)."""
    deadline = time.monotonic() + 10
    while True:
        try:
            temporary.replace(record)
            return
        except PermissionError:
            if os.name != "nt" or time.monotonic() > deadline:
                raise
            time.sleep(0.01)


def _record_checkout(target):
    """Publish complete ownership atomically (§FS-repository-maintenance.2.4)."""
    record = target / "checkout.json"
    # Preserve nonregular ownership records (§FS-repository-maintenance.2.3).
    try:
        if not stat.S_ISREG(record.lstat().st_mode):
            raise ValueError(f"build target ownership is not a regular file: {record}")
    except FileNotFoundError:
        pass
    output = tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=target,
                                        prefix=".checkout-", suffix=".json", delete=False)
    temporary = Path(output.name)
    try:
        with output:
            json.dump({"checkout": str(REPO)}, output)
            output.write("\n")
        _replace_record(temporary, record)
    finally:
        temporary.unlink(missing_ok=True)


def _checkout_is_missing(target):
    """Only trustworthy ownership permits deletion (§FS-repository-maintenance.2.3)."""
    try:
        if not stat.S_ISDIR(target.lstat().st_mode):
            return False
        record = target / "checkout.json"
        if not stat.S_ISREG(record.lstat().st_mode):
            return False
        data = json.loads(record.read_text(encoding="utf-8"))
        owner = data.get("checkout") if isinstance(data, dict) else None
        if not isinstance(owner, str):
            return False
        checkout = Path(owner)
        if not checkout.is_absolute() or str(checkout.resolve()) != owner:
            return False
        if checkout_key(checkout) != target.name:
            return False
        try:
            checkout.stat()
        except FileNotFoundError:
            return True
    except (OSError, ValueError, RuntimeError):
        # Unknown ownership or an uncertain lookup never authorizes removal.
        pass
    return False


def _ignore_removed_output(operation, path, error):
    """Continue concurrent traversal after removals (§FS-repository-maintenance.2.4)."""
    if not isinstance(error[1], FileNotFoundError):
        raise error[1]


def _remove_output(target):
    """Bound Windows deletion contention per target (§FS-repository-maintenance.2.4).

    Retry the traversal, not the callback operation: shutil may report a failed
    lookup or scan as well as an unlink or rmdir. Share a ten-second budget across
    the target and its descendants, matching the ownership-publication wait.
    """
    deadline = time.monotonic() + 10
    while True:
        try:
            shutil.rmtree(target, onerror=_ignore_removed_output)
            return
        except PermissionError:
            if os.name != "nt":
                raise
            try:
                target.lstat()
            except FileNotFoundError:
                return
            except PermissionError:
                # Delete-pending output may deny lookup too (§FS-repository-maintenance.2.4).
                pass
            if time.monotonic() >= deadline:
                raise
            time.sleep(0.01)


def _reclaim_targets(root, current):
    """Reclaim recorded dead siblings on every call (§FS-repository-maintenance.2.2)."""
    for candidate in root.iterdir():
        if candidate == current or not re.fullmatch(r"[0-9a-f]{16}", candidate.name):
            continue
        if _checkout_is_missing(candidate):
            _remove_output(candidate)


def checkout_target(root):
    """This checkout's own persistent build output under the shared `root`
    (§FS-repository-maintenance.2.1).

    One per source checkout, keyed by a digest of its resolved root (§AR-ci.3.4.1.2); both
    bindings' acceptances derive their target here (§AR-ci.3.4.5).
    """
    target = root / checkout_key(REPO)
    target.mkdir(parents=True, exist_ok=True)
    # Never write ownership through a target symlink (§FS-repository-maintenance.2.3).
    if not stat.S_ISDIR(target.lstat().st_mode):
        raise ValueError(f"build target is not a regular directory: {target}")
    _record_checkout(target)
    _reclaim_targets(root, target)
    return target.resolve()


def prune_python_target(target):
    """Drop the workspace crates' artifacts from the shared target (§AR-ci.3.4.5).

    Each clean checkout and sdist compiles them from a fresh path, so they would pile up
    in a target kept between runs; the dependencies' artifacts, the point of sharing it,
    stay. A prune that fails leaves the target as it is: it is a cache, never a verdict.
    """
    try:
        metadata = json.loads(subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
            cwd=REPO, capture_output=True, text=True).stdout)
        packages = [arg for package in metadata["packages"] for arg in ("-p", package["name"])]
    except (OSError, ValueError, KeyError, TypeError):
        return
    if packages:
        subprocess.run(["cargo", "clean", "--locked", "--quiet", "--target-dir", str(target),
                        "--profile", "dev", *packages], cwd=REPO, capture_output=True)


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


def checked(argv, cwd=REPO, env=None, input=None):
    """Run one step with its output captured, failing with that output (§AR-bindings.6.2.2).

    A non-zero exit raises an AssertionError whose own text carries the command, its exit
    status, its stdout and its stderr, so a failed step is diagnosable from the job log.
    """
    args = [str(arg) for arg in argv]
    result = subprocess.run(args, cwd=cwd, env=env, input=input, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"command failed ({result.returncode}): {' '.join(args)}\n"
                             f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    return result


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
    run = checked([oracle], input=json.dumps(request))
    if run.stderr:
        raise AssertionError(f"oracle wrote stderr: {run.stderr}")
    response = json.loads(run.stdout)
    data = response["data"]
    wire = base64.b64decode(response["canonical"], validate=True)
    if wire != canonical(data):
        raise AssertionError("Rust canonical bytes violate the specified encoding")
    return data, wire, response
