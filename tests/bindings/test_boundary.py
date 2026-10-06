"""Host isolation and concurrency (§FS-distribution.3.3.3, §FS-distribution.3.3.4)."""

from concurrent.futures import ThreadPoolExecutor
import contextlib
import io
import os
from pathlib import Path
import subprocess
import sys
import threading
import time
import unittest
from unittest.mock import patch

from support import REPO, binding, fixture, plain, temporary, tree_bytes


def workload(parent):
    root = fixture(parent)
    # Long comment input, few result objects: exercise scanning rather than Python
    # conversion, with enough work to distinguish progress during Rust from entry.
    folder = root / "src"
    folder.mkdir(exist_ok=True)
    block = '# Ordinary non-citation text for scan work.\n' * 10000
    for number in range(400):
        (folder / f"file_{number}.py").write_text(block)
    return root


class BoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.g = binding()

    def test_silent_no_argv_no_exit_no_cwd_and_independent_roots(self):
        with temporary() as first, temporary() as second:
            root = fixture(first)
            clean = fixture(second, "clean")
            cwd, argv = Path.cwd(), sys.argv[:]
            out, err = io.StringIO(), io.StringIO()
            with patch.object(sys, "argv", ["unrelated", "--garbage"]), \
                    contextlib.redirect_stdout(out), contextlib.redirect_stderr(err), \
                    patch.object(sys, "exit", side_effect=AssertionError("API called sys.exit")), \
                    patch.object(os, "chdir", side_effect=AssertionError("API called chdir")):
                for _ in range(3):
                    self.assertTrue(self.g.check(root).report.errors)
                    self.assertFalse(self.g.check(clean).report.errors)
                with self.assertRaises(self.g.QueryError):
                    self.g.show("FS-999-missing", root=root)
            self.assertEqual("", out.getvalue())
            self.assertEqual("", err.getvalue())
            self.assertEqual(cwd, Path.cwd())
            self.assertEqual(argv, sys.argv)
            # Capture OS file descriptors too; Python redirects alone miss native prints.
            code = 'import grund; grund.check(__import__("pathlib").Path(__import__("sys").argv[1])); print("alive")'
            run = subprocess.run([sys.executable, "-c", code, str(root)], capture_output=True,
                                 text=True, cwd=REPO, check=True)
            self.assertEqual("alive\n", run.stdout)
            self.assertEqual("", run.stderr)

    def test_omitted_root_snapshots_cwd_and_explicit_file_scope(self):
        with temporary() as temp:
            root = fixture(temp)
            code = ('import grund, os, pathlib; before=os.getcwd(); '
                    'assert grund.check().report.errors; '
                    'assert grund.show("FS-001-alpha").body; '
                    'assert os.getcwd()==before')
            subprocess.run([sys.executable, "-c", code], cwd=root, check=True,
                           capture_output=True, text=True)
            source = root / "docs/functional-spec/FS-001-alpha.md"
            result = self.g.check(source)
            self.assertTrue(result.report.errors)
            self.assertTrue(all(f.path.endswith("FS-001-alpha.md") for f in result.report
                                if f.path))

    def test_concurrent_reads_keep_per_call_scope(self):
        with temporary() as first, temporary() as second:
            roots = [fixture(first), fixture(second, "clean")]
            expected = [plain(self.g.check(root)) for root in roots]
            inputs = roots * 20
            with ThreadPoolExecutor(max_workers=4) as pool:
                actual = list(pool.map(lambda root: plain(self.g.check(root)), inputs))
            self.assertEqual(expected * 20, actual)

    def test_gil_is_released_during_long_rust_work(self):
        with temporary() as temp:
            root = workload(temp)
            stop = threading.Event()
            beats = []
            def heartbeat():
                while not stop.wait(0.002):
                    beats.append(time.monotonic())
            worker = threading.Thread(target=heartbeat)
            worker.start()
            started = time.monotonic()
            try:
                self.g.check(root)
            finally:
                ended = time.monotonic()
                stop.set()
                worker.join()
            self.assertGreater(ended - started, 0.1, "fixture too short to establish GIL evidence")
            self.assertGreater(len([t for t in beats if started + .025 < t < ended - .025]), 2,
                               "Python thread made no progress during Rust work")

    @unittest.skipUnless(os.name == "posix", "POSIX signal delivery test")
    def test_pending_interrupt_is_delivered_when_operation_returns(self):
        with temporary() as temp:
            root = workload(temp)
            # Sender is a separate process, so GIL behavior cannot prevent delivery.
            code = '''import grund, os, signal, subprocess, sys, time
sender = subprocess.Popen([sys.executable, '-c',
    'import os,signal,time; time.sleep(.1); os.kill(int(__import__("sys").argv[1]), signal.SIGINT)', str(os.getpid())])
started = time.monotonic()
try:
    grund.check(sys.argv[1])
except KeyboardInterrupt:
    assert time.monotonic() - started >= .1
    assert grund.check(sys.argv[2]).report is not None
else:
    raise AssertionError('pending interrupt was swallowed')
finally:
    sender.wait()
'''
            with temporary() as other:
                small = fixture(other)
                result = subprocess.run([sys.executable, "-c", code, str(root), str(small)],
                                        capture_output=True, text=True, timeout=120)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertEqual("", result.stdout)
                self.assertEqual("", result.stderr)

    @unittest.skipUnless(os.name == "posix", "non-Unicode filesystem names require POSIX bytes paths")
    def test_unsafe_unnamed_non_unicode_workspace_alias_is_rejected(self):
        with temporary() as temp:
            root = Path(temp) / "workspace"
            root.mkdir()
            (root / "grund.toml").write_text('grund_config_version = 1\n[workspace]\nmembers = ["packages/*"]\n')
            (root / "packages").mkdir()
            member = os.fsencode(root / "packages") + b"/bad-\xff"
            os.mkdir(member)
            with open(member + b"/grund.toml", "wb") as config:
                config.write(b"grund_config_version = 1\n")
            with self.assertRaises(self.g.PathEncodingError):
                self.g.check(root)
