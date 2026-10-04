"""Force and schedule Git's actual pack write, as the #420 reproducer does.

§AR-ci.10.4: no synthetic writer or cleanup exception; the socket only chooses
when Git's real open(O_CREAT) happens. Imported by the lifetime regression.
"""

import errno
import hashlib
import json
import os
import signal
import socket
import subprocess
import tempfile
import zlib
from pathlib import Path
from unittest.mock import patch

RUN = subprocess.run
RMDIR = os.rmdir


def seed(repo):
    """28 valid loose blobs in shard 17 cross old and new Git heuristics."""
    bucket = repo / ".git/objects/17"
    bucket.mkdir(exist_ok=True)
    count = 0
    for number in range(100000):
        payload = f"grund-420-object-{number}\n".encode()
        data = f"blob {len(payload)}\0".encode() + payload
        oid = hashlib.sha1(data).hexdigest()
        if oid.startswith("17"):
            (bucket / oid[2:]).write_bytes(zlib.compress(data))
            count += 1
            if count == 28:
                return
    raise AssertionError("could not seed the real automatic maintenance threshold")


def policy(repo):
    """Read local policy independently of the command wrapper being checked."""
    return tuple(
        RUN(["git", "-C", str(repo), "config", "--local", "--get", key],
            capture_output=True, text=True, check=False).stdout.strip()
        for key in ("gc.auto", "maintenance.auto")
    )


def cleanup(case):
    # Invoke the registered callbacks strictly; doCleanups would capture errors.
    while case._cleanups:
        function, arguments, keywords = case._cleanups.pop()
        function(*arguments, **keywords)


class PackBoundary:
    def __init__(self, scratch, library):
        self.scratch = scratch
        self.library = library
        self.trace = scratch / "trace.json"
        self.endpoint = scratch / "pack.sock"
        self.server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.server.bind(str(self.endpoint))
        self.server.listen()
        self.server.settimeout(10)
        self.connection = None
        self.writer = None
        self.created = False
        self.policies = []

    def environment(self, template):
        return {
            "LD_PRELOAD": str(self.library),
            "GRUND_TEST_PACK_SOCKET": str(self.endpoint),
            "GRUND_TEST_SESSION": str(os.getsid(0)),
            "GIT_TRACE2_EVENT": str(self.trace),
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_TEMPLATE_DIR": str(template),
            "GIT_AUTHOR_DATE": "2026-09-01T12:00:00+00:00",
            "GIT_COMMITTER_DATE": "2026-09-01T12:00:00+00:00",
        }

    def observe(self, arguments, *args, seed_initial=False, **kwargs):
        if arguments[0] == "git" and "-C" in arguments:
            repo = Path(arguments[arguments.index("-C") + 1])
            if "commit" in arguments or "rebase" in arguments:
                self.policies.append(policy(repo))
        result = RUN(arguments, *args, **kwargs)
        if seed_initial and arguments[0] == "git" and "init" in arguments:
            destination = Path(arguments[-1])
            seed(destination if (destination / ".git").is_dir() else repo)
        return result

    def unfinished(self):
        events = [json.loads(line) for line in self.trace.read_text().splitlines()]
        started = {event["sid"] for event in events if event["event"] == "start"}
        exited = {event["sid"] for event in events if event["event"] == "exit"}
        return started - exited

    def returned(self, repo):
        # Disabled policy returns with every traced Git process finished. If
        # policy or process lifetime is wrong, require an actual pack handshake.
        if policy(repo) == ("0", "false") and not self.unfinished():
            self.server.settimeout(0)
            try:
                self.connection, _ = self.server.accept()
            except BlockingIOError:
                return
        else:
            self.connection, _ = self.server.accept()
        self.connection.settimeout(10)
        with self.connection.makefile("rb") as ready:
            self.writer = int(ready.readline())
            self.pack = ready.readline().decode().strip()
        assert Path(f"/proc/{self.writer}/cmdline").read_bytes().find(b"pack-objects") >= 0
        assert os.getsid(self.writer) != os.getsid(0), "writer did not detach"

    def rmdir(self, path, *, dir_fd=None):
        if self.writer and not self.created and Path(os.fspath(path)).name == "pack":
            self.connection.sendall(b"R")
            assert self.connection.recv(1) == b"C", "Git did not create its pack"
            self.created = True
        try:
            return RMDIR(path, dir_fd=dir_fd)
        except OSError as error:
            if self.created and error.errno == errno.ENOTEMPTY:
                error.add_note(
                    f"Detached git pack-objects PID {self.writer} survived the returned "
                    f"fixture commands and created {self.pack} after cleanup's scan.")
            raise

    def close(self):
        if self.writer:
            # Kill the whole detached maintenance group before reclaiming the
            # experiment's outer tree; never leave our deliberately held writer.
            try:
                os.killpg(os.getpgid(self.writer), signal.SIGKILL)
            except ProcessLookupError:
                pass
        if self.connection:
            self.connection.close()
        self.server.close()
        # Also reclaim descendants if an operation or handshake failed before
        # we learned the writer PID. Only this experiment owns this unique root.
        for process in Path("/proc").iterdir():
            if process.name.isdigit() and int(process.name) != os.getpid():
                try:
                    if (process / "cwd").resolve().is_relative_to(self.scratch):
                        os.kill(int(process.name), signal.SIGKILL)
                except (FileNotFoundError, ProcessLookupError, PermissionError):
                    pass


def exercise(test, case, construct, operation, library, template, seed_initial):
    base = Path.home() / "ag/tmp"
    base.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="g420-", dir=base) as root:
        boundary = PackBoundary(Path(root), library)
        try:
            with patch.dict(os.environ, boundary.environment(template)), \
                    patch.object(tempfile, "tempdir", root), \
                    patch("subprocess.run", side_effect=lambda *a, **kw: boundary.observe(
                        *a, seed_initial=seed_initial, **kw)):
                construct()
                if operation:
                    operation()
                boundary.returned(case.repo)
                # This is the issue's real failure: enumerate, let Git create
                # its pack, then let the OS reject the original strict rmdir.
                with patch("os.rmdir", boundary.rmdir):
                    cleanup(case)
                test.assertIsNone(boundary.writer, "Git writer outlived fixture commands")
                test.assertFalse(boundary.unfinished(), "Git descendants still running")
                test.assertTrue(boundary.policies, "no real commit/rebase was exercised")
                test.assertEqual({("0", "false")}, set(boundary.policies),
                                 "local policy must precede every commit and rebase")
                test.assertFalse(case.repo.exists(), "strict cleanup left the tree behind")
        finally:
            boundary.close()
            cleanup(case)
