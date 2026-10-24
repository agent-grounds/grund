"""§AR-ci.14 — the macOS suites run once a day in `macos.yml` and on every release
candidate in `ci.yml`, both through `macos-suites.yml`, and a failed scheduled run
opens or edits one issue.

Read as text, the way `test_ci_language_jobs.py` reads `ci.yml`: the CI Python has no
YAML parser. The report and its open-or-edit decision are functions of the forge's
JSON, so they are held offline, against canned jobs, and the kind label the issue
carries is read from the `gh` calls `main` makes against a recording `gh` (§AR-ci.14.4)."""

import importlib.util
import re
import sys
import unittest
from pathlib import Path

from test_ci_duration import RecordingGh, calls_of, labels, load_tracker
from test_ci_language_jobs import CI, instances, leg_commands, runs
from test_release_workflow import WORKFLOWS, jobs, squash

ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = ROOT / "scripts" / "macos_daily.py"
MACOS = WORKFLOWS / "macos.yml"
SUITES = WORKFLOWS / "macos-suites.yml"
CALL = "uses: ./.github/workflows/macos-suites.yml"
MARKER = "<!-- macos-daily-failure -->"


def load_report():
    sys.path.insert(0, str(SCRIPT_PATH.parent))
    spec = importlib.util.spec_from_file_location("macos_daily", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def triggers(text):
    """The event names under `on:`."""
    block = text.split("\non:\n", 1)[1].split("\n\n", 1)[0]
    return re.findall(r"(?m)^  ([a-z_]+):", block)


def job_head(body):
    return body.split("\n    steps:", 1)[0]


def failed_job(name, *steps, conclusion="failure"):
    return {"name": name, "conclusion": conclusion,
            "steps": [{"name": step, "conclusion": result} for step, result in steps]}


def callers(text):
    """The job heads of `text` that call the reusable macOS suite workflow."""
    return [squash(job_head(body)) for body in jobs(text).values() if CALL in body]


class WorkflowTests(unittest.TestCase):
    """§AR-ci.14.1 and §AR-ci.14.2 on the three workflows that run or call the macOS suites."""

    @classmethod
    def setUpClass(cls):
        cls.text = MACOS.read_text(encoding="utf-8")
        cls.jobs = jobs(cls.text)
        cls.suites = SUITES.read_text(encoding="utf-8")
        cls.ci = CI.read_text(encoding="utf-8")

    def test_it_runs_on_a_schedule_and_by_hand_and_never_as_a_gate(self):
        self.assertEqual(sorted(triggers(self.text)), ["schedule", "workflow_dispatch"])
        self.assertRegex(self.text, r'(?m)^    - cron: "[^"]+"$')
        self.assertEqual(len(callers(self.text)), 1)

    def test_the_suite_workflow_is_only_called_and_asks_for_contents_read_alone(self):
        self.assertEqual(triggers(self.suites), ["workflow_call"])
        self.assertRegex(self.suites, r"(?m)^permissions:\n  contents: read\n\n")
        self.assertNotRegex(self.suites, r"(?m)^ +permissions:")

    @unittest.skip("red until #978 macos-suites.yml runs every shard and selection; implement removes this")
    def test_one_job_runs_every_suite_once(self):
        suites = [i for i in instances(self.suites) if i["runs_on"] == "macos-latest"]
        self.assertEqual([i["name"] for i in suites], ["suites (macos-latest)"])
        for key, command in leg_commands("macos-latest").items():
            with self.subTest(command=key):
                found = suites[0]["text"].count(command) if command.startswith('"') \
                    else suites[0]["commands"].count(command)
                self.assertEqual(found, 1)

    def test_ci_calls_the_suites_on_a_dispatch_and_on_no_other_event(self):
        self.assertIn("workflow_dispatch", triggers(self.ci))
        self.assertEqual(callers(self.ci), [callers(self.ci)[0]])
        self.assertIn("if: github.event_name == 'workflow_dispatch'", callers(self.ci)[0])

    def test_ci_runs_no_macos_job_of_its_own(self):
        self.assertNotIn("macos-latest", self.ci)
        self.assertFalse(any(runs(i, command) for i in instances(self.ci)
                             for command in leg_commands("macos-latest").values()
                             if i["runs_on"].startswith("macos")))

    def test_the_report_runs_after_a_failed_scheduled_run_with_issue_write(self):
        reports = [body for body in self.jobs.values() if "scripts/macos_daily.py" in body]
        self.assertEqual(len(reports), 1)
        head = squash(job_head(reports[0]))
        self.assertIn("needs: [macos]", head)
        self.assertIn("if: failure() && github.event_name == 'schedule'", head)
        self.assertIn("issues: write", head)
        self.assertIn("actions: read", head)
        self.assertNotIn("issues: write", self.text.split("\njobs:", 1)[0])


class ReportTests(unittest.TestCase):
    """§AR-ci.14.3: what the report says and which issue it lands on."""

    @classmethod
    def setUpClass(cls):
        cls.report = load_report()
        cls.tracker = load_tracker()

    def test_the_report_names_the_commit_the_run_and_each_failed_step(self):
        jobs_ = [failed_job("suites (macos-latest)", ("cargo build", "success"),
                            ("cargo test", "failure"), ("Save cargo", "skipped")),
                 {"name": "report the failure", "conclusion": None, "steps": []}]
        body = self.report.failure_report(jobs_, "agent-grounds/grund", "a" * 40,
                                          "https://github.com/agent-grounds/grund/actions/runs/7")
        self.assertTrue(body.startswith(MARKER + "\n"))
        self.assertIn("`aaaaaaaaaaaa`", body)
        self.assertIn("actions/runs/7", body)
        self.assertIn("- `cargo test`", body)
        self.assertNotIn("cargo build", body)
        self.assertNotIn("report the failure", body)

    def test_a_run_with_no_failed_job_still_says_so(self):
        body = self.report.failure_report([], "o/r", "b" * 40, "https://example/run")
        self.assertIn("no failed job", body)

    def test_an_open_failure_issue_is_edited_and_none_opens_one(self):
        # The report writes through `ci_duration.put_on_issue`, so this is its decision.
        action = self.tracker.issue_action
        issues = [{"number": 3, "body": "<!-- ci-duration-drift -->\nslow"},
                  {"number": 9, "body": f"{MARKER}\nold"}]
        self.assertEqual(action(issues, MARKER), ("update", 9))
        self.assertEqual(action(issues[:1], MARKER), ("open", None))
        self.assertEqual(action([{"number": 4, "body": f"It quotes {MARKER} here."}], MARKER),
                         ("open", None))


class FailureIssueKindTests(RecordingGh, unittest.TestCase):
    """§AR-ci.14.3: the macOS failure issue carries its kind, `bug`, whether `main`
    opens it or edits it."""

    @classmethod
    def setUpClass(cls):
        cls.daily = load_report()

    def report(self, issues):
        jobs_ = [failed_job("suites (macos-latest)", ("cargo test", "failure")),
                 {"name": "report the failure", "conclusion": None, "steps": []}]
        answers = {"api": {"actions/runs/202/jobs": {"jobs": jobs_}}, "issues": issues}
        return self.run_against_forge(self.daily.main, answers, GITHUB_RUN_ID="202",
                                      GITHUB_REPOSITORY="agent-grounds/grund", GITHUB_SHA="a" * 40)

    def test_the_failure_issue_is_opened_with_the_kind_label_bug(self):
        calls = self.report([{"number": 3, "body": "<!-- ci-duration-drift -->\nslow"}])
        creates = calls_of(calls, "issue", "create")
        self.assertEqual(len(creates), 1, calls)
        self.assertEqual(calls_of(calls, "issue", "edit"), [])
        self.assertIn("bug", labels(creates[0], "--label", "-l"),
                      f"the macOS failure issue is opened without its kind label: {creates[0]}")

    def test_an_edit_of_the_failure_issue_adds_the_kind_label_bug_again(self):
        calls = self.report([{"number": 3, "body": "<!-- ci-duration-drift -->\nslow"},
                             {"number": 9, "body": f"{MARKER}\nold"}])
        edits = calls_of(calls, "issue", "edit")
        self.assertEqual([argv[:3] for argv in edits], [["issue", "edit", "9"]], calls)
        self.assertEqual(calls_of(calls, "issue", "create"), [])
        self.assertIn("bug", labels(edits[0], "--add-label"),
                      f"the macOS failure issue is edited without adding its kind label: {edits[0]}")


if __name__ == "__main__":
    unittest.main()
