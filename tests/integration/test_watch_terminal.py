"""PTY byte contracts for §FS-check.6.2.2 and CLI ownership (§AR-bindings.3)."""

import errno
import os
from pathlib import Path
import select
import signal
import subprocess
import tempfile
import time
import unittest

if os.name == "posix":
    import pty
    import tty

ROOT = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag/tmp"
TARGET = SCRATCH / "grund-issue-473-target"
ENTER = b"\x1b[?1049h"
CLEAR = b"\x1b[H\x1b[2J"
RESTORE = b"\x1b[?1049l"


@unittest.skipUnless(os.name == "posix", "PTY/SIGINT tests require POSIX")
class WatchTerminalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "grund", "--target-dir", str(TARGET)],
            cwd=ROOT, check=True, timeout=120,
        )
        cls.binary = TARGET / "debug/grund"

    def setUp(self):
        self.fixture = tempfile.TemporaryDirectory(prefix="grund-473-pty-", dir=SCRATCH)
        self.addCleanup(self.fixture.cleanup)
        self.root = Path(self.fixture.name)
        (self.root / "docs/functional-spec").mkdir(parents=True)
        (self.root / "src").mkdir()
        (self.root / "grund.toml").write_text('grund_config_version = 1\n[id]\nformat = "{kind}-{slug}"\n')
        (self.root / "docs/functional-spec/FS-live.md").write_text("# FS-live: Live\n")
        (self.root / "docs/functional-spec/README.md").write_text("# Index\n\n- [\u00a7FS-live](FS-live.md#fs-live-live)\n")
        (self.root / "src/main.rs").write_text("// \u00a7FS-live\n")

    def terminal(self):
        master, slave = pty.openpty()
        # Disable line-discipline translation: compare application bytes exactly.
        tty.setraw(slave)
        self.addCleanup(os.close, slave)
        self.addCleanup(os.close, master)
        return master, slave

    def check_pair(self, mode, args=(), term="xterm-256color", invalid=False):
        if invalid:
            (self.root / "grund.toml").write_text('grund_config_version = "invalid"\n')
        ordinary = subprocess.run(
            [str(self.binary), "check", *args], cwd=self.root,
            capture_output=True, timeout=10, check=False,
        )
        self.assertEqual(ordinary.returncode, 2 if invalid else (1 if "--format=json" in args else 0))
        master, slave = self.terminal()
        other_master, other_slave = self.terminal()
        choices = {
            "shared": (slave, slave, [master]),
            "mixed": (slave, subprocess.PIPE, [master]),
            "distinct": (slave, other_slave, [master, other_master]),
            "stdout_pipe": (subprocess.PIPE, slave, [master]),
        }
        stdout, stderr, terminals = choices[mode]
        environment = dict(os.environ, TERM=term)
        child = subprocess.Popen(
            [str(self.binary), "check", "--watch", *args], cwd=self.root,
            stdout=stdout, stderr=stderr, env=environment,
        )

        def cleanup():
            if child.poll() is None:
                child.kill()
            child.wait(timeout=10)
            if child.stdout:
                child.stdout.close()
            if child.stderr:
                child.stderr.close()

        self.addCleanup(cleanup)
        streams = list(terminals)
        if child.stdout:
            streams.append(child.stdout.fileno())
        if child.stderr:
            streams.append(child.stderr.fileno())
        output = {fd: bytearray() for fd in streams}
        out_fd = child.stdout.fileno() if child.stdout else master
        err_fd = child.stderr.fileno() if child.stderr else (other_master if mode == "distinct" else master)
        owns = mode in ("shared", "mixed") and term != "dumb" and "--format=json" not in args
        expected = {fd: b"" for fd in streams}
        expected[out_fd] += (ENTER + CLEAR if owns else b"") + ordinary.stdout
        expected[err_fd] += ordinary.stderr

        def receive(timeout):
            for fd in select.select(streams, [], [], timeout)[0]:
                try:
                    data = os.read(fd, 65536)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    data = b""
                if data:
                    output[fd].extend(data)
                else:
                    streams.remove(fd)

        deadline = time.monotonic() + 10
        while not all(bytes(output[fd]).startswith(value) for fd, value in expected.items()):
            receive(0.02)
            if child.poll() is not None:
                receive(0)
                self.fail(f"watch exited {child.returncode} before PTY report; captured={list(map(bytes, output.values()))!r}, expected={list(expected.values())!r}")
            self.assertLess(time.monotonic(), deadline, f"PTY report deadline: {output!r}")
        self.assertIsNone(child.poll(), "watch must remain resident after publishing")
        child.send_signal(signal.SIGINT)
        while child.poll() is None:
            receive(0.02)
            self.assertLess(time.monotonic(), deadline, "PTY interrupt deadline")
        receive(0)
        self.assertEqual(child.returncode, ordinary.returncode)
        if owns:
            expected[out_fd] += RESTORE
        self.assertEqual({fd: bytes(value) for fd, value in output.items()}, expected)

    def test_watch_shared_terminal_enters_clears_and_restores(self):
        self.check_pair("shared")

    def test_watch_mixed_terminal_keeps_error_on_redirected_stderr(self):
        self.check_pair("mixed", args=("--format=text",), invalid=True)

    def test_watch_distinct_terminals_append_without_controls(self):
        self.check_pair("distinct")

    def test_watch_dumb_terminal_appends_without_controls(self):
        self.check_pair("shared", term="dumb")

    def test_watch_redirected_stdout_appends_without_controls(self):
        self.check_pair("stdout_pipe")

    def test_watch_json_terminal_never_enters_or_clears(self):
        (self.root / "src/main.rs").write_text("// \u00a7FS-live\n// \u00a7FS-missing\n")
        self.check_pair("shared", args=("--format=json",))


if __name__ == "__main__":
    unittest.main()
