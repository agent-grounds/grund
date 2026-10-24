"""Filesystem-boundary fixture for §FS-repository-maintenance.2.4.

Port issue #986's triage reproducer: copy the real helper, preserve eligibility
and shutil traversal, and inject only OS operations. The Windows delete-pending
sequence is simulated, including on Linux; this does not reproduce kernel timing.
"""

import errno
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import traceback
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]


class WindowsOS:
    """Select the helper's Windows branches without changing pathlib's host OS."""
    name = "nt"

    def __getattr__(self, name):
        return getattr(os, name)


def seed(root, owner, record=True):
    target = root / hashlib.sha256(os.fsencode(owner)).hexdigest()[:16]
    target.mkdir()
    if record:
        (target / "checkout.json").write_text(json.dumps({"checkout": str(owner)}))
    (target / "debug").mkdir()
    (target / "debug" / "artifact").write_bytes(b"build output\x00\xff")
    return target


def scenario(work, mode, location):
    checkout = work / "caller"
    harness = checkout / "tests" / "bindings" / "node"
    harness.mkdir(parents=True)
    for filename in ("support.py", "launch.py"):
        shutil.copyfile(REPO / "tests/bindings/node" / filename, harness / filename)
    # The shared harness owns the target's lifetime for both bindings (§AR-ci.3.4.1.2).
    shutil.copyfile(REPO / "tests/bindings/support.py", harness.parent / "support.py")
    sys.path.insert(0, str(harness))
    spec = importlib.util.spec_from_file_location("removal_support", harness / "support.py")
    support = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(support)
    sys.path.pop(0)
    root = work / "targets"
    root.mkdir()
    own = seed(root, checkout)
    live = work / "live"
    live.mkdir()
    other = seed(root, live)
    legacy = seed(root, work / "legacy-owner", record=False)
    untrusted = seed(root, work / "untrusted-owner")
    (untrusted / "checkout.json").write_text("{")
    tools = root / "tools"
    tools.mkdir()
    (tools / "compiler").write_bytes(b"shared tools")
    uncertain_owner = work / "uncertain-owner"
    uncertain = seed(root, uncertain_owner)
    dead = seed(root, work / "gone-owner")
    protected = {path: path.read_bytes()
                 for target in (own, other, legacy, untrusted, tools, uncertain)
                 for path in target.rglob("*") if path.is_file()}
    protected.pop(own / "checkout.json")  # publication may change whitespace
    boundary = {"root": dead, "directory": dead / "debug",
                "file": dead / "debug/artifact"}[location]
    original_stat, original_lstat = os.stat, os.lstat
    original_rmdir, original_unlink = os.rmdir, os.unlink
    remove = original_unlink if location == "file" else original_rmdir
    denied = False
    peer_finished = False
    injected = False
    lookup_denied = False

    def matches(path, kwargs):
        if isinstance(path, int):
            return False
        # shutil uses leaf names with dir_fd on POSIX and full paths on Windows.
        if kwargs.get("dir_fd") is not None:
            return os.fsdecode(path) == boundary.name
        return os.path.abspath(os.fsdecode(path)) == str(boundary)

    def peer():
        nonlocal peer_finished
        if mode == "peer" and denied and not peer_finished:
            remove(boundary)
            peer_finished = True

    def removal(operation, path, *args, **kwargs):
        nonlocal denied, injected
        if matches(path, kwargs):
            injected = True
            if mode == "missing":
                remove(boundary)
                raise FileNotFoundError(errno.ENOENT, "peer already removed output", str(boundary))
            if mode == "unrelated":
                raise OSError(errno.EIO, "unrelated I/O failure", str(boundary))
            if not denied or mode in ("persistent", "uncertain"):
                denied = True
                error = PermissionError(errno.EACCES, "simulated Windows deletion contention",
                                        str(boundary))
                error.winerror = 5
                raise error
            peer()
        return operation(path, *args, **kwargs)

    def lookup(operation, path, *args, **kwargs):
        nonlocal lookup_denied
        if not isinstance(path, int) and os.path.abspath(os.fsdecode(path)) == str(uncertain_owner):
            raise PermissionError(errno.EACCES, "uncertain checkout lookup", str(path))
        if denied and matches(path, kwargs):
            if mode == "uncertain":
                lookup_denied = True
                raise PermissionError(errno.EACCES, "cannot confirm output absence", str(path))
            peer()
        return operation(path, *args, **kwargs)

    error = None
    returned = None
    absent_at_return = False
    with patch.dict(os.environ, GRUND_NODE_TARGET=str(root)), \
            patch.object(support.bindings, "os", WindowsOS()), \
            patch.object(os, "rmdir", lambda p, *a, **kw: removal(original_rmdir, p, *a, **kw)), \
            patch.object(os, "unlink", lambda p, *a, **kw: removal(original_unlink, p, *a, **kw)), \
            patch.object(os, "stat", lambda p, *a, **kw: lookup(original_stat, p, *a, **kw)), \
            patch.object(os, "lstat", lambda p, *a, **kw: lookup(original_lstat, p, *a, **kw)):
        try:
            returned = support.node_target()
            # Use the saved OS lookup: the assertion must not itself finish the peer.
            try:
                original_lstat(dead)
            except FileNotFoundError:
                absent_at_return = True
        except OSError as caught:
            error = {"type": type(caught).__name__, "errno": caught.errno,
                     "traceback": traceback.format_exc()}

    # Finish the simulated peer after an early failure, as triage's timer did.
    # This cannot turn an early return into a passing absence-at-return assertion.
    peer()
    safe = all(path.read_bytes() == data for path, data in protected.items())
    owner = json.loads((own / "checkout.json").read_text())
    return {"error": error, "injected": injected, "absent_at_return": absent_at_return,
            "boundary_retained": boundary.exists(), "peer_finished": peer_finished,
            "lookup_denied": lookup_denied, "protected": safe,
            "owner": owner == {"checkout": str(checkout)}, "returned": returned == own}


if __name__ == "__main__":
    scratch = Path.home() / "ag/tmp"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="grund-986-removal-", dir=scratch) as temporary:
        print(json.dumps(scenario(Path(temporary).resolve(), *sys.argv[1:])))
