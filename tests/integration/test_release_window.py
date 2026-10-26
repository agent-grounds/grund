"""Version-free release-window gate. §FS-distribution.4.2.5

Every case invokes the ordinary script, including the actual unfiltered tree
that the Python pull-request gate discovers (§AR-ci.7, §FS-distribution.4.2.6). Fixtures
exercise source and golden scans without copying the bound calculation.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GUARD = ROOT / "scripts/check_release_ramps.py"
LANDED = "crates/grund-core/src/lib.rs"
PENDING = "tests/e2e/cases/promise/expected.stderr"
FLOOR = "scripts/release-floor.txt"
LEDGER = "docs/discussions/proposals/2026-09-30-core-concerns.md"


def run_guard(root, *arguments):
    return subprocess.run(
        [sys.executable, str(GUARD), *arguments, "--root", str(root)],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )


class ReleaseWindowCommandTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="grund-1019-release-window-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def write(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")

    def bounds(self, landed=None, pending=None, floor=None):
        if landed:
            self.write(LANDED, f'const MESSAGE: &str = "was removed in grund {landed}";\n')
        if pending:
            self.write(PENDING, f"warning: wording changes in grund {pending}\n")
        if floor:
            self.write(FLOOR, floor + "\n")

    def assert_empty_window(self, result, floor, ceiling, lower_location):
        # A usage error (2) is not the required empty-window refusal (1).
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("no release at all", result.stderr)
        self.assertIn(f"at or above {floor}", result.stderr)
        self.assertIn(f"below {ceiling}", result.stderr)
        self.assertIn(lower_location, result.stderr)
        self.assertIn(
            f"{PENDING}:1: `wording changes in {ceiling}`", result.stderr,
        )

    def assert_releasable(self, result):
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")

    def test_equal_landed_and_pending_bounds_refuse_with_both_locations(self):
        self.bounds(landed="0.21.0", pending="0.21.0")
        self.assert_empty_window(
            run_guard(self.root), "0.21.0", "0.21.0",
            f"{LANDED}:1: `was removed in 0.21.0`",
        )

    def test_crossed_landed_and_pending_bounds_refuse_with_both_locations(self):
        self.bounds(landed="0.22.0", pending="0.21.0")
        self.assert_empty_window(
            run_guard(self.root), "0.22.0", "0.21.0",
            f"{LANDED}:1: `was removed in 0.22.0`",
        )

    def test_shared_floor_conflicting_with_pending_refuses_with_both_locations(self):
        for floor in ("0.21.0", "0.22.0"):
            with self.subTest(floor=floor):
                self.bounds(pending="0.21.0", floor=floor)
                self.assert_empty_window(
                    run_guard(self.root), floor, "0.21.0",
                    f"{FLOOR}:1: minimum publishable release is {floor}.",
                )

    def test_nonempty_window_passes_above_a_refused_patch(self):
        self.bounds(landed="0.21.0", pending="0.22.0", floor="0.19.0")
        pinned = run_guard(self.root, "0.20.1")
        self.assertEqual(pinned.returncode, 1, pinned.stdout + pinned.stderr)
        self.assertIn("at or above 0.21.0 and below 0.22.0", pinned.stderr)
        self.assert_releasable(run_guard(self.root))

    def test_nonempty_window_compares_versions_numerically(self):
        self.bounds(landed="0.9.0", pending="0.10.0")
        self.assert_releasable(run_guard(self.root))

    def test_completely_unbounded_window_passes(self):
        self.assert_releasable(run_guard(self.root))

    def test_window_with_only_a_lower_bound_passes(self):
        self.bounds(landed="0.21.0")
        self.assert_releasable(run_guard(self.root))

    def test_window_with_only_a_pending_ceiling_passes(self):
        self.bounds(pending="0.21.0")
        self.assert_releasable(run_guard(self.root))

    def test_shared_floor_without_messages_passes_above_a_refused_patch(self):
        self.bounds(floor="0.21.0")
        pinned = run_guard(self.root, "0.20.1")
        self.assertEqual(pinned.returncode, 1, pinned.stdout + pinned.stderr)
        self.assertIn(f"{FLOOR}:1: minimum publishable release is 0.21.0.", pinned.stderr)
        self.assert_releasable(run_guard(self.root))

    def test_version_free_check_does_not_evaluate_a_notice_at_an_invented_release(self):
        self.bounds(landed="0.21.0", pending="0.22.0", floor="0.19.0")
        self.write(
            LEDGER,
            "### 9.9. The deprecation ledger\n\n"
            "| Surface | Notice | Removal | Status |\n"
            "| --- | --- | --- | --- |\n"
            "| example | `0.19.0` | `0.22.0` | planned |\n",
        )
        pinned = run_guard(self.root, "0.21.0")
        self.assertEqual(pinned.returncode, 1, pinned.stdout + pinned.stderr)
        self.assertIn("notice `0.19.0` is still planned", pinned.stderr)
        self.assert_releasable(run_guard(self.root))

    def test_malformed_shared_floor_is_a_configuration_error(self):
        self.write(FLOOR, "not-a-release\n")
        result = run_guard(self.root)
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn(f"{FLOOR} must contain one release", result.stderr)
        self.assertNotIn("no release at all", result.stderr)
        self.assertNotIn("usage:", result.stderr)

    def test_unreadable_shared_floor_is_an_io_error(self):
        # A directory at the floor path fails on every platform, including root.
        (self.root / FLOOR).mkdir(parents=True)
        result = run_guard(self.root)
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("error:", result.stderr)
        self.assertIn("release-floor.txt", result.stderr)
        self.assertNotIn("no release at all", result.stderr)
        self.assertNotIn("usage:", result.stderr)


class ThisRepositoryReleaseWindowTests(unittest.TestCase):
    def test_every_scanned_ramp_in_this_repository_permits_some_release(self):
        """The PR gate checks all real ramps. §FS-distribution.4.2.6"""
        result = subprocess.run(
            [sys.executable, str(GUARD)], cwd=ROOT,
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
