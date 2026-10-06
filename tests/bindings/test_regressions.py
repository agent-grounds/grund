"""Fetch sources and filesystem roots (§FS-distribution.3.3.2, §FS-distribution.3.3.3)."""

import errno
import os
from pathlib import Path
import unittest

from support import binding, canonical, fixture, plain, python_call, rust_call, temporary, tree_bytes


class RegressionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = binding()

    def fetch_fixture(self, temp):
        root = fixture(temp, "fetch-workspace-folder")
        config = root / "grund.toml"
        config.write_text(config.read_text().replace(
            "[workspace]", '[scan]\ninclude = ["unread"]\n\n[workspace]')
            + "include_root = false\n")
        (root / "unread").mkdir()
        (root / "unread/note.md").write_text("This block is intentionally not scanned.\n")
        return root

    def assert_fetch_filesystem_failure(self, root, path, os_error, *, check_bytes=True):
        args, options = ("alpha/TICKET-1234",), {"write": True}
        before = tree_bytes(root) if check_bytes else None
        expected, wire, _ = rust_call("fetch", root, args, options)
        actual = python_call(self.module, "fetch", root, args, options)
        self.assertEqual(expected, actual, "complete failure/caution parity")
        self.assertEqual(wire, canonical(actual), "canonical failure bytes")
        with self.assertRaises(self.module.FilesystemError) as caught:
            self.module.fetch(*args, root=root, **options)
        failure = caught.exception.failure
        self.assertEqual(actual["failure"], plain(failure))
        self.assertEqual("filesystem", failure.kind)
        self.assertEqual("io", failure.code)
        self.assertEqual(path.as_posix(), failure.path)
        self.assertIsNone(failure.line)
        self.assertIsNone(failure.column)
        self.assertEqual(os_error, failure.details["os_error"])
        self.assertTrue(failure.causes, "the original I/O source must survive")
        self.assertTrue(any(os.strerror(os_error) in cause for cause in failure.causes))
        self.assertTrue(failure.run_cautions, "workspace cautions must survive the I/O failure")
        self.assertEqual(actual["run_cautions"], plain(failure.run_cautions))
        if check_bytes:
            self.assertEqual(before, tree_bytes(root), "failed fetch must preserve bytes")
        return failure

    @unittest.skipUnless(os.name == "posix", "fixture fetcher is a POSIX shell executable")
    def test_missing_fetch_executable_retains_io_source(self):
        with temporary() as temp:
            root = self.fetch_fixture(temp)
            executable = root / "packages/alpha/scripts/fetch-ticket"
            executable.unlink()
            self.assert_fetch_filesystem_failure(root, executable, errno.ENOENT)

    @unittest.skipUnless(os.name == "posix", "requires POSIX directory write permissions")
    def test_denied_fetch_snapshot_write_retains_io_source(self):
        if os.geteuid() == 0:
            self.skipTest("root bypasses directory write permissions; run as an unprivileged user")
        with temporary() as temp:
            root = self.fetch_fixture(temp)
            home = root / "packages/alpha/docs/tickets"
            home.mkdir(parents=True, exist_ok=True)
            permissions = home.stat().st_mode
            home.chmod(0o555)
            try:
                # Unexpected success is a failure, never a platform skip.
                self.assert_fetch_filesystem_failure(root, home / "TICKET-1234.md", errno.EACCES)
            finally:
                home.chmod(permissions)

    @unittest.skipUnless(os.name == "posix", "requires POSIX directory traversal permissions")
    def test_denied_fetch_snapshot_traversal_retains_io_source(self):
        if os.geteuid() == 0:
            self.skipTest("root bypasses traversal permissions; run as an unprivileged user")
        with temporary() as temp:
            root = self.fetch_fixture(temp)
            home = root / "packages/alpha/docs/tickets"
            home.mkdir(parents=True, exist_ok=True)
            before = tree_bytes(root)
            permissions = home.stat().st_mode
            home.chmod(0o000)
            try:
                with self.assertRaises(PermissionError) as caught:
                    with os.scandir(home) as entries:
                        list(entries)
                self.assertEqual(errno.EACCES, caught.exception.errno)
                # Inspect bytes only after restoring the unreadable directory.
                failure = self.assert_fetch_filesystem_failure(
                    root, home, caught.exception.errno, check_bytes=False)
                self.assertIn(
                    f"{os.strerror(caught.exception.errno)} (os error {caught.exception.errno})",
                    failure.causes, "retain the original I/O cause, not only the scan reason")
            finally:
                home.chmod(permissions)
            self.assertEqual(before, tree_bytes(root), "failed traversal must preserve bytes")

    @unittest.skipUnless(os.name == "posix", "requires POSIX directory symlink semantics")
    def test_explicit_symlink_parent_root_matches_engine(self):
        with temporary() as first, temporary() as second:
            clean = fixture(first, "clean")
            erroneous = fixture(second)
            (erroneous / "subdir").mkdir()
            (clean / "link").symlink_to(erroneous / "subdir", target_is_directory=True)
            # Keep the literal operand: resolve/abspath would erase the regression.
            absolute = str(clean / "link") + "/.."
            expected, wire, _ = rust_call("check", absolute)
            self.assertIn("dangling", [f["code"] for f in expected["result"]["report"]["errors"]])
            cwd = Path.cwd()
            try:
                os.chdir(clean)
                for operand in (absolute, "link/..", Path("link/..")):
                    with self.subTest(operand=str(operand)):
                        actual = python_call(self.module, "check", operand)
                        self.assertEqual(expected, actual, "filesystem root parity")
                        self.assertEqual(wire, canonical(actual), "canonical root bytes")
            finally:
                os.chdir(cwd)
