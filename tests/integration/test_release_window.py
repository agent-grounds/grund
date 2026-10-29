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


@unittest.skip("red until #1112 lists every pending line at the ceiling; implement removes this")
class EveryCeilingLineIsNamedTests(unittest.TestCase):
    """An empty window names every pending line at the ceiling, once. §FS-distribution.4.2.5"""

    PENDING_LINES = (
        "crates/a/src/config_findings.rs:1: `becomes an error in 0.22.0`",
        "crates/a/src/config_findings.rs:2: `becomes an error in 0.22.0`",
        "crates/a/src/value_mismatch.rs:1: `wording changes in 0.22.0`",
        "crates/a/src/whose.rs:1: `wording changes in 0.22.0`",
        "crates/a/tests/string_closes_early.rs:1: `becomes an error in 0.22.0`",
    )

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="grund-1112-release-window-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        # One ramp landed at 0.22.0, five still promising it, across four files and both clauses.
        files = {
            FLOOR: "0.19.0\n",
            "crates/a/src/landed.rs":
                'const T: &str = "a trailing comma became an error in grund 0.22.0";\n',
            "crates/a/tests/string_closes_early.rs":
                'const A: &str = "text after the quote becomes an error in grund 0.22.0";\n',
            "crates/a/src/config_findings.rs":
                'const W1: &str = "this form becomes an error in grund 0.22.0";\n'
                'const W2: &str = "this other form becomes an error in grund 0.22.0";\n',
            "crates/a/src/whose.rs":
                'const W: &str = "this whose-clause wording changes in grund 0.22.0";\n',
            "crates/a/src/value_mismatch.rs":
                'const R: &str = "the values root wording changes in grund 0.22.0";\n',
        }
        for path, text in files.items():
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")

    def assert_names_every_ceiling_line_once(self, result):
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("no release at all", result.stderr)
        missing = [line for line in self.PENDING_LINES if line not in result.stderr]
        self.assertEqual(missing, [], result.stderr)
        repeated = [line for line in self.PENDING_LINES if result.stderr.count(line) > 1]
        self.assertEqual(repeated, [], result.stderr)

    def test_version_free_refusal_names_every_pending_line_at_the_ceiling(self):
        result = run_guard(self.root)
        self.assert_names_every_ceiling_line_once(result)
        self.assertIn("crates/a/src/landed.rs:1: `became an error in 0.22.0`", result.stderr)

    def test_release_refusal_below_the_ceiling_names_every_pending_line_at_it(self):
        self.assert_names_every_ceiling_line_once(run_guard(self.root, "0.21.0"))

    def test_release_refusal_at_the_ceiling_names_each_pending_line_once(self):
        self.assert_names_every_ceiling_line_once(run_guard(self.root, "0.22.0"))


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
