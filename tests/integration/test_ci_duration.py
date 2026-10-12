"""§AR-ci.requirements.3 — the CI-duration tracker, offline: per-gate averages over
the window, a drift report naming gate, average, window and slowest steps, and one
drift issue opened or edited. §AR-ci.requirements.4 — threshold and window are read
from `.github/ci-duration.json`, never from a literal in the script."""

import importlib.util
import json
import re
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = ROOT / "scripts" / "ci_duration.py"
CONFIG_PATH = ROOT / ".github" / "ci-duration.json"
WORKFLOW_PATH = ROOT / ".github" / "workflows" / "ci-duration.yml"
MARKER = "<!-- ci-duration-drift -->"


def load_tracker():
    spec = importlib.util.spec_from_file_location("ci_duration", SCRIPT_PATH)
    if spec is None:
        raise FileNotFoundError(f"no tracker at {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def stamp(minute: float) -> str:
    whole = int(minute * 60)
    return f"2026-10-11T{10 + whole // 3600:02d}:{whole // 60 % 60:02d}:{whole % 60:02d}Z"


def job(name: str, minutes: float, steps=(), conclusion: str = "success") -> dict:
    """A job as `GET /repos/{o}/{r}/actions/runs/{id}/jobs` lists it, starting at minute 0."""
    at, rendered = 0.0, []
    for step, length in steps:
        rendered.append({"name": step, "conclusion": "success",
                         "started_at": stamp(at), "completed_at": stamp(at + length)})
        at += length
    return {"name": name, "conclusion": conclusion, "started_at": stamp(0),
            "completed_at": stamp(minutes), "steps": rendered}


def ubuntu(minutes: float, lychee: float) -> dict:
    return job("cargo test (ubuntu-latest)", minutes,
               [("pre-commit", lychee), ("cargo test", minutes - lychee)])


# Newest first, as the tracker reads them: each entry is one run's jobs.
RUNS = [
    [ubuntu(40, 30), job("cargo test (macos-latest)", 6), job("cargo bench (instruction counts)", 4)],
    [ubuntu(44, 32), job("cargo test (macos-latest)", 8), job("cargo bench (instruction counts)", 6)],
    [ubuntu(42, 28), job("cargo test (macos-latest)", 7),
     job("cargo bench (instruction counts)", 0, conclusion="skipped")],
    # Outside a window of 3: would pull every average down if it were counted.
    [ubuntu(1, 0.5), job("cargo test (macos-latest)", 1), job("cargo bench (instruction counts)", 1)],
]


class TrackerTests(unittest.TestCase):
    """The pure half of `scripts/ci_duration.py`, which the workflow drives through `gh`."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.tracker = load_tracker()

    def config(self, threshold: float = 10, window: int = 3):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ci-duration.json"
            path.write_text(json.dumps({"threshold_minutes": threshold, "window": window}))
            return self.tracker.load_config(path)

    def test_each_matrix_leg_is_its_own_gate_averaged_over_the_window(self) -> None:
        averages = self.tracker.gate_averages(RUNS, window=3)
        self.assertEqual(set(averages), {"cargo test (ubuntu-latest)", "cargo test (macos-latest)",
                                         "cargo bench (instruction counts)"})
        self.assertAlmostEqual(averages["cargo test (ubuntu-latest)"].average_minutes, 42.0)
        self.assertAlmostEqual(averages["cargo test (macos-latest)"].average_minutes, 7.0)
        self.assertEqual(averages["cargo test (ubuntu-latest)"].samples, 3)

    def test_a_skipped_job_is_not_a_sample(self) -> None:
        bench = self.tracker.gate_averages(RUNS, window=3)["cargo bench (instruction counts)"]
        self.assertEqual(bench.samples, 2)
        self.assertAlmostEqual(bench.average_minutes, 5.0)

    def test_steps_are_averaged_slowest_first(self) -> None:
        steps = self.tracker.gate_averages(RUNS, window=3)["cargo test (ubuntu-latest)"].slowest_steps
        self.assertEqual([name for name, _ in steps], ["pre-commit", "cargo test"])
        self.assertAlmostEqual(steps[0][1], 30.0)
        self.assertAlmostEqual(steps[1][1], 12.0)

    def test_config_supplies_threshold_and_window(self) -> None:
        config = self.config(threshold=7.5, window=5)
        self.assertEqual((config.threshold_minutes, config.window), (7.5, 5))

    def test_a_missing_key_is_an_error_that_names_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ci-duration.json"
            path.write_text(json.dumps({"window": 8}))
            with self.assertRaisesRegex(Exception, "threshold_minutes"):
                self.tracker.load_config(path)

    def test_a_gate_over_the_threshold_is_reported_with_its_window_and_slowest_steps(self) -> None:
        config = self.config(threshold=10, window=3)
        text = self.tracker.drift_report(self.tracker.gate_averages(RUNS, config.window), config)
        self.assertIsNotNone(text)
        self.assertIn(MARKER, text)
        self.assertIn("cargo test (ubuntu-latest)", text)
        self.assertRegex(text, r"42(\.0)?\b")
        self.assertRegex(text, r"\b3\b")
        self.assertIn("pre-commit", text)
        self.assertNotIn("cargo test (macos-latest)", text)

    def test_every_gate_under_the_threshold_reports_nothing(self) -> None:
        config = self.config(threshold=60, window=3)
        self.assertIsNone(self.tracker.drift_report(self.tracker.gate_averages(RUNS, 3), config))

    def test_an_open_drift_issue_is_edited_rather_than_a_second_opened(self) -> None:
        issues = [{"number": 12, "body": "unrelated"}, {"number": 31, "body": f"{MARKER}\nold"}]
        self.assertEqual(self.tracker.issue_action(issues), ("update", 31))

    def test_without_an_open_drift_issue_one_is_opened(self) -> None:
        self.assertEqual(self.tracker.issue_action([{"number": 12, "body": "unrelated"}]),
                         ("open", None))

    def test_an_issue_that_only_quotes_the_marker_is_not_the_drift_issue(self) -> None:
        issues = [{"number": 5, "body": f"It looks for {MARKER} in the body."}]
        self.assertEqual(self.tracker.issue_action(issues), ("open", None))

    def test_a_window_outside_one_to_a_hundred_is_an_error_that_names_it(self) -> None:
        for window in (0, -1, 101):
            with self.subTest(window=window), self.assertRaisesRegex(Exception, "window"):
                self.config(window=window)

    def test_no_run_measured_fails_rather_than_passing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ci-duration.json"
            path.write_text(json.dumps({"threshold_minutes": 10, "window": 8}))
            original = self.tracker.gh
            self.tracker.gh = lambda *arguments, stdin=None: json.dumps({"workflow_runs": []})
            try:
                self.assertEqual(self.tracker.main(["ci_duration.py", str(path)]), 1)
            finally:
                self.tracker.gh = original

    def test_the_link_to_ar_ci_is_absolute_or_plain(self) -> None:
        config = self.config(threshold=10, window=3)
        averages = self.tracker.gate_averages(RUNS, config.window)
        linked = self.tracker.drift_report(averages, config, "agent-grounds/grund")
        self.assertIn("(https://github.com/agent-grounds/grund/blob/main/docs/architecture/AR-ci.md)",
                      linked)
        plain = self.tracker.drift_report(averages, config)
        self.assertNotIn("](docs/", plain)


class WiringTests(unittest.TestCase):
    """Where the meter sits: its config, its workflow, and not in the local gate."""

    def test_the_repository_config_carries_threshold_and_window(self) -> None:
        config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
        self.assertEqual(config["threshold_minutes"], 15)
        self.assertIsInstance(config["window"], int)

    def test_the_workflow_runs_the_tracker_on_a_schedule_and_never_as_a_gate(self) -> None:
        text = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("scripts/ci_duration.py", text)
        self.assertIn("schedule:", text)
        self.assertIn("workflow_dispatch", text)
        self.assertNotRegex(text, r"(?m)^\s*(push|pull_request):")
        self.assertRegex(text, r"issues:\s*write")

    def test_the_tracker_stays_out_of_the_pre_commit_hooks(self) -> None:
        self.assertNotIn("ci_duration", (ROOT / ".pre-commit-config.yaml").read_text(encoding="utf-8"))

    def test_ar_ci_declares_the_four_requirements(self) -> None:
        text = (ROOT / "docs" / "architecture" / "AR-ci.md").read_text(encoding="utf-8")
        self.assertRegex(text, r"(?m)^## requirements: ")
        for n in range(1, 5):
            self.assertRegex(text, rf"(?m)^### requirements\.{n}: ")


if __name__ == "__main__":
    unittest.main()
