"""§FS-distribution.4.4.1 — nothing reaches `main` that CI has not passed.

Both release helpers hand everything from their candidates on to one controller,
`scripts/release_candidates.py` (§FS-distribution.4.4): it builds the release
commit and the `-dev` advance on candidate branches, runs CI on each beside the
release dry run, and pushes exactly those commits once every run has passed. The
workflow half is read as text, the way `test_release_workflow.py` reads it; the
controller runs each helper's whole job against the local origin and stub forge
of `release_queue_fixture.py`; and the half that talks to the forge,
`scripts/candidate_runs.py`, runs against a stub `gh` on `PATH` that answers
from a table the case writes, so no case reaches the network."""

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT, environment
from release_queue_fixture import Case
from test_release_workflow import WORKFLOWS, jobs, squash, step_index, step_name, steps

SCRIPT = REPO_ROOT / "scripts" / "candidate_runs.py"
HELPERS = ("auto-bump.yml", "release-minor.yml")
CANDIDATES = "Build, run and publish the candidates"
# The one controller both helpers call (§AR-system.principle), and the round it builds.
CONTROLLER = REPO_ROOT / "scripts" / "release_candidates.py"
CONTROLLER_CALL = "python3 scripts/release_candidates.py"
ROUND = REPO_ROOT / "scripts" / "release_round.sh"


def helper_steps(name):
    (body,) = jobs((WORKFLOWS / name).read_text(encoding="utf-8")).values()
    return steps(body)


class CandidateWorkflowTests(unittest.TestCase):
    """The helpers hand their candidates to the controller, and push nothing beside it."""

    def test_ci_can_be_dispatched_on_a_candidate(self):
        text = (WORKFLOWS / "ci.yml").read_text(encoding="utf-8")
        triggers = text.split("\non:\n", 1)[1].split("\nenv:", 1)[0]
        self.assertRegex(triggers, r"(?m)^  workflow_dispatch:", "ci.yml cannot be run on a candidate branch")

    def test_both_helpers_hand_the_candidates_to_the_one_controller(self):
        for name in HELPERS:
            with self.subTest(workflow=name):
                found = helper_steps(name)
                index = step_index(found, CANDIDATES)
                self.assertNotEqual(-1, index, f"{name} has no step named {CANDIDATES!r}")
                call = squash(found[index].replace("\\\n", " "))  # one command across a continuation
                self.assertIn(f"{CONTROLLER_CALL} --helper {name}", call)
                self.assertIn('--next "${{ steps.versions.outputs.next }}"', call)
                self.assertEqual(len(found) - 1, index, f"{name} runs a step after the controller")

    def test_no_helper_commits_pushes_or_dispatches_beside_the_controller(self):
        for name in HELPERS:
            with self.subTest(workflow=name):
                for step in helper_steps(name):
                    step = "\n".join(line for line in step.splitlines() if not line.strip().startswith("#"))
                    for made in ("git commit", "git push", "set-version", "cargo update", "gh workflow run",
                                 "scripts/candidate_runs.py", "scripts/advance_dev.sh"):
                        self.assertNotIn(made, step, f"{name}: {step_name(step)!r} acts beside the controller")

    def test_the_controller_advances_main_through_the_one_script(self):
        """§FS-distribution.4.4.1: the retrying advance has one implementation, not a copy each."""
        text = CONTROLLER.read_text(encoding="utf-8")
        self.assertIn('"advance_dev.sh"', text)
        self.assertIn('"scripts/release_round.sh"', text)
        self.assertNotIn("git push", ROUND.read_text(encoding="utf-8"), "the round pushes beside the controller")

    def test_a_quiet_scheduled_run_builds_no_dev_candidate(self):
        """§FS-distribution.4.1: nothing is opened before a release."""
        found = helper_steps("auto-bump.yml")
        self.assertIn("if: steps.gate_substantive.outputs.ok == 'true'", found[step_index(found, CANDIDATES)])


@unittest.skipIf(sys.platform == "win32", "the helpers' steps are bash scripts with shell hooks")
@unittest.skipUnless(shutil.which("bash") and shutil.which("git") and shutil.which("jq"), "needs bash, git and jq")
class EveryPushToMainTests(unittest.TestCase):
    """Each helper's whole job, run against a local origin: the dry run on the release
    commit and CI on it and on the advance built over it, and then `main` takes
    exactly those two commits, in that order, and nothing else the job made."""

    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="grund-candidate-runs-")
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)

    def pushed_to_main(self, case):
        lines = [line.split() for line in (case.state / "pushes").read_text().splitlines()]
        return [new for _old, new, ref in lines if ref == "refs/heads/main"]

    def check(self, workflow, scenario, round_):
        inputs = {"version": "next-minor", "expected_main_sha": ""} if workflow == "release-minor.yml" else {}
        case = Case(self.scratch, workflow, scenario, inputs).run()
        report = case.report()
        self.assertTrue(case.ok, report)
        runs = case.runs(round_)
        dry = [run for run in runs if run["workflow"] == "release.yml"]
        self.assertEqual(1, len(dry), report)
        self.assertEqual("false", dry[0]["fields"].get("publish_crates"), report)
        release = dry[0]["sha"]
        ci = sorted(run["sha"] for run in runs if run["workflow"] == "ci.yml")
        (dev,) = [sha for sha in ci if sha != release]
        self.assertEqual(sorted([release, dev]), ci, report)
        self.assertEqual(release, case.parent(dev), "the advance is not built over the release commit")
        self.assertEqual([release, dev], self.pushed_to_main(case), report)
        for sha in (release, dev):
            self.assertTrue(case.passed(sha, "ci.yml", round_), report)
        self.assertTrue(case.passed(release, "release.yml", round_), report)

    def test_every_push_to_main_is_a_commit_the_runs_passed(self):
        for workflow in HELPERS:
            with self.subTest(workflow=workflow):
                self.check(workflow, None, 1)

    def test_a_moved_main_pushes_only_what_the_last_round_passed(self):
        for workflow in HELPERS:
            with self.subTest(workflow=workflow):
                self.check(workflow, {"move": {"1": "m1"}}, 2)


STUB = r'''#!{python}
import json, os, sys

args = sys.argv[1:]
with open(os.environ["GH_STUB_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(args) + "\n")
with open(os.environ["GH_STUB_LOG"], encoding="utf-8") as log:
    seen = sum(1 for line in log if json.loads(line) == args) - 1
with open(os.environ["GH_STUB_ANSWERS"], encoding="utf-8") as table:
    answers = json.load(table)

def answer(sequence):
    print(json.dumps(sequence[min(seen, len(sequence) - 1)]))
    sys.exit(0)

if args[:2] == ["workflow", "run"]:
    if "dispatch_error" in answers:
        sys.exit(answers["dispatch_error"])
    sys.exit(0)
if args[:2] == ["run", "list"]:
    key = args[args.index("--workflow") + 1] + "@" + args[args.index("--branch") + 1]
    answer(answers["lists"].get(key, [[]]))
if args[:2] == ["run", "view"]:
    answer(answers["views"][args[2]])
sys.exit(f"gh stub: no answer for {{args!r}}")
'''


def view(run_id, status, conclusion, workflow="CI", branch="release-candidate/v0.18.0-7"):
    return {"status": status, "conclusion": conclusion, "workflowName": workflow, "headBranch": branch,
            "headSha": "c" * 40, "url": f"https://github.com/agent-grounds/grund/actions/runs/{run_id}"}


class CandidateRunsScriptTests(unittest.TestCase):
    """`candidate_runs.py` finds the run it started and fails on the first that did not pass."""

    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)
        self.bin = self.scratch / "bin"
        self.bin.mkdir()
        stub = self.bin / "gh"
        stub.write_text(STUB.format(python=sys.executable), encoding="utf-8")
        stub.chmod(0o755)
        if sys.platform == "win32":
            # `which` finds a .cmd through PATHEXT; a shebang means nothing here.
            (self.bin / "gh.cmd").write_text(f'@"{sys.executable}" "{stub}" %*\r\n', encoding="utf-8")
        self.log = self.scratch / "gh.log"

    def run_script(self, answers, *arguments):
        table = self.scratch / "answers.json"
        table.write_text(json.dumps(answers), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT), *arguments],
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            env=environment({
                "PATH": str(self.bin),
                "PYTHONUTF8": "1",
                "GH_STUB_ANSWERS": str(table),
                "GH_STUB_LOG": str(self.log),
                "CANDIDATE_RUNS_POLL": "0",
                "CANDIDATE_RUNS_WATCH": "0",
            }),
        )

    def calls(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]

    def test_dispatch_prints_the_run_started_on_exactly_the_candidate(self):
        branch, sha = "release-candidate/v0.18.0-7", "a" * 40
        answers = {"lists": {f"ci.yml@{branch}": [
            [],
            [{"databaseId": 11, "headSha": "b" * 40}],
            [{"databaseId": 11, "headSha": "b" * 40}, {"databaseId": 12, "headSha": sha}],
        ]}}
        result = self.run_script(answers, "dispatch", "ci.yml", branch, sha, "version=0.18.0", "dry=true")
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("12\n", result.stdout)
        # §AR-ci.11: the runs already on the branch are listed before the dispatch.
        listed, dispatched = self.calls()[:2]
        self.assertEqual(["workflow", "run", "ci.yml", "--ref", branch, "-f", "version=0.18.0", "-f", "dry=true"],
                         dispatched)
        for flag, value in (("--workflow", "ci.yml"), ("--branch", branch), ("--event", "workflow_dispatch")):
            self.assertEqual(value, listed[listed.index(flag) + 1])

    def test_dispatch_fails_naming_the_run_that_never_appeared(self):
        branch, sha = "release-candidate/v0.18.0-7", "a" * 40
        result = self.run_script({"lists": {}}, "dispatch", "release.yml", branch, sha)
        self.assertEqual(1, result.returncode)
        self.assertIn(f"error: dispatched release.yml on {branch}, but no run on {sha} appeared", result.stderr)
        self.assertEqual("", result.stdout)

    def test_dispatch_refuses_a_field_that_is_not_an_assignment(self):
        result = self.run_script({}, "dispatch", "ci.yml", "b", "a" * 40, "version")
        self.assertEqual(1, result.returncode)
        self.assertIn("error: field 'version' is not <field>=<value>", result.stderr)
        self.assertEqual([], self.calls(), "nothing is dispatched for a malformed field")

    def test_a_failed_gh_call_is_named(self):
        result = self.run_script({"lists": {}, "dispatch_error": "HTTP 403: Resource not accessible"},
                                 "dispatch", "ci.yml", "b", "a" * 40)
        self.assertEqual(1, result.returncode)
        self.assertIn("error: `gh workflow run ci.yml --ref b` exited 1: HTTP 403: Resource not accessible",
                      result.stderr)

    def test_watch_passes_once_every_run_has_passed(self):
        answers = {"views": {
            "1": [view(1, "queued", ""), view(1, "in_progress", ""), view(1, "completed", "success")],
            "2": [view(2, "in_progress", ""), view(2, "completed", "success")],
        }}
        result = self.run_script(answers, "watch", "1", "2")
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(3, sum(1 for call in self.calls() if call[:3] == ["run", "view", "1"]))

    def test_watch_fails_on_the_first_run_that_did_not_pass(self):
        """Fail early: a failed run ends the wait while another still runs."""
        answers = {"views": {
            "1": [view(1, "in_progress", "", workflow="Release")],
            "2": [view(2, "completed", "failure", branch="release-candidate/v0.18.0-7-dev")],
        }}
        result = self.run_script(answers, "watch", "1", "2")
        self.assertEqual(1, result.returncode)
        self.assertIn("error: CI on release-candidate/v0.18.0-7-dev (cccccccccccc) ended failure, "
                      "so main does not move: https://github.com/agent-grounds/grund/actions/runs/2",
                      result.stderr)
        self.assertEqual(1, sum(1 for call in self.calls() if call[:3] == ["run", "view", "1"]))

    def test_a_cancelled_run_does_not_pass(self):
        answers = {"views": {"3": [view(3, "completed", "cancelled")]}}
        result = self.run_script(answers, "watch", "3")
        self.assertEqual(1, result.returncode)
        self.assertIn("ended cancelled, so main does not move", result.stderr)

    def test_an_unknown_command_prints_the_usage(self):
        result = self.run_script({}, "watch")
        self.assertEqual(1, result.returncode)
        self.assertIn("error: usage: candidate_runs.py dispatch", result.stderr)


if __name__ == "__main__":
    unittest.main()
