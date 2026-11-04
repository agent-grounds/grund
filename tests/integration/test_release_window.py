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


class WarningWindowCommandTests(unittest.TestCase):
    """Isolated report fixtures survive the YAML closing. §FS-distribution.4.2.2"""

    GOLDENS = (
        ("check-yaml-value-source-anchor-deferred", 1),
        ("check-yaml-value-source-issue-example", 2),
        ("check-yaml-value-source-json-duplicate-deferred", 1),
        ("check-yaml-value-source-nested-too-deep", 1),
    )

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="grund-1132-warning-window-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def write(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")

    def messages(self, clause, grund="grund "):
        # Like the report's template, the release is interpolated in source;
        # only the goldens carry a complete, readable pending claim.
        self.write("crates/a/src/window.rs",
                   'const RELEASE: &str = "0.22.0";\n'
                   f'const MESSAGE: &str = "findings {clause} {grund}{{RELEASE}}";\n')
        for case, line in self.GOLDENS:
            self.write(f"tests/e2e/cases/{case}/expected.stdout",
                       "intro\n" * (line - 1)
                       + f"values/limits.yaml:2: warning: findings {clause} {grund}0.22.0\n")

    def assert_refused_with_every_golden(self, result, clause):
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertEqual(result.stdout, "")
        for case, line in self.GOLDENS:
            location = f"tests/e2e/cases/{case}/expected.stdout:{line}: `{clause} 0.22.0`"
            self.assertEqual(result.stderr.count(location), 1, result.stderr)

    def test_warning_window_accepts_below_but_refuses_deadline_and_later(self):
        self.write(FLOOR, "0.19.0\n")
        for grund in ("", "grund "):
            # The same paths and line numbers work with an existing clause.
            self.messages("becomes an error in", grund)
            for release in ("0.22.0", "1.0.0"):
                self.assert_refused_with_every_golden(
                    run_guard(self.root, release), "becomes an error in",
                )
            self.messages("are warnings until", grund)
            below = run_guard(self.root, "0.21.1")
            self.assertEqual(below.returncode, 0, below.stdout + below.stderr)
            self.assertEqual(below.stderr, "")
            version_free = run_guard(self.root)
            self.assertEqual(version_free.returncode, 0, version_free.stdout + version_free.stderr)
            self.assertEqual(version_free.stderr, "")
            for release in ("0.22.0", "1.0.0"):
                with self.subTest(grund=grund, release=release):
                    result = run_guard(self.root, release)
                    self.assert_refused_with_every_golden(result, "are warnings until")
                    self.assertIn("at or above 0.19.0 and below 0.22.0", result.stderr)

    def test_warning_window_conflicts_with_shared_or_landed_floor(self):
        """Equal and crossed bounds have no release. §FS-distribution.4.2.5"""
        self.messages("are warnings until")
        for source in ("shared", "landed"):
            for floor in ("0.22.0", "0.23.0"):
                self.write(FLOOR, floor + "\n" if source == "shared" else "0.19.0\n")
                self.write(LANDED, f'const T: &str = "was removed in grund {floor}";\n'
                           if source == "landed" else "")
                for arguments in ((), ("0.22.0",), ("1.0.0",)):
                    with self.subTest(source=source, floor=floor, arguments=arguments):
                        result = run_guard(self.root, *arguments)
                        self.assert_refused_with_every_golden(result, "are warnings until")
                        self.assertIn("no release at all", result.stderr)
                        self.assertIn(f"at or above {floor}", result.stderr)
                        self.assertIn("below 0.22.0", result.stderr)
                        lower = (f"{FLOOR}:1: minimum publishable release is {floor}."
                                 if source == "shared"
                                 else f"{LANDED}:1: `was removed in {floor}`")
                        # Candidate refusals name only disagreeing landed claims;
                        # version-free mode names floor setters (§FS-distribution.4.2.5).
                        if (source == "shared" or not arguments
                                or tuple(map(int, arguments[0].split(".")))
                                < tuple(map(int, floor.split(".")))):
                            self.assertIn(lower, result.stderr)


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
