"""§FS-distribution.release-floor: one tree-wide floor survives message retirement.

Synthetic trees deliberately carry no landed messages that could accidentally
enforce the floor. These tests exercise the existing publication guard's CLI.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GUARD = ROOT / "scripts/check_release_ramps.py"


class SharedReleaseFloorTests(unittest.TestCase):
    def test_this_tree_installs_one_single_line_floor(self):
        floor = ROOT / "scripts/release-floor.txt"
        self.assertTrue(floor.is_file(), "shared release floor is not installed")
        lines = floor.read_text().splitlines()
        self.assertEqual(len(lines), 1, "one shared value, with no per-window entries")
        self.assertRegex(lines[0], r"^\d+\.\d+\.\d+$")

    def run_guard(self, release, *, floor="0.19.0", message=""):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="release-floor-", dir=scratch) as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            (root / "scripts/release-floor.txt").write_text(floor + "\n")
            (root / "crates/core/src").mkdir(parents=True)
            (root / "crates/core/src/message.rs").write_text(message)
            # A release helper has already rewritten this development version.
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "' + release + '"\n')
            return subprocess.run(
                [sys.executable, str(GUARD), release, "--root", str(root)],
                capture_output=True, text=True, check=False,
            )

    def test_floor_alone_rejects_a_lower_release_after_manifest_rewrite(self):
        result = self.run_guard("0.18.9")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("scripts/release-floor.txt", result.stderr)
        self.assertIn("0.19.0", result.stderr)
        self.assertIn("at or above 0.19.0", result.stderr)

    def test_the_floor_itself_is_releasable_without_pending_promises(self):
        result = self.run_guard("0.19.0")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")

    def test_floor_and_independent_pending_promise_make_an_empty_window(self):
        result = self.run_guard(
            "0.19.0", message='const PENDING: &str = "becomes an error in grund 0.19.0";\n'
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("no release at all", result.stderr)
        self.assertIn("at or above 0.19.0", result.stderr)
        self.assertIn("below 0.19.0", result.stderr)
        self.assertIn("scripts/release-floor.txt", result.stderr)
        self.assertIn("crates/core/src/message.rs:1", result.stderr)

    def test_landed_message_can_raise_the_shared_floor(self):
        result = self.run_guard(
            "0.19.0", message='const LANDED: &str = "was removed in grund 0.20.0";\n'
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("at or above 0.20.0", result.stderr)

    def test_the_floor_is_read_from_the_checked_tree(self):
        result = self.run_guard("0.20.0", floor="0.21.0")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("at or above 0.21.0", result.stderr)


if __name__ == "__main__":
    unittest.main()
