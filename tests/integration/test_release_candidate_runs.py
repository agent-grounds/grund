"""§FS-distribution.4.4.1 — nothing reaches `main` that CI has not passed.

Both release helpers build the release commit and the `-dev` advance on
candidate branches, run CI on each beside the release dry run, and push exactly
those commits once every run has passed. The workflow half is read as text, the
way `test_release_workflow.py` reads it; the half that talks to the forge,
`scripts/candidate_runs.py`, runs against a stub `gh` on `PATH` that answers
from a table the case writes, so no case reaches the network."""

import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT, environment
from test_release_workflow import WORKFLOWS, jobs, squash, step_index, step_name, steps

SCRIPT = REPO_ROOT / "scripts" / "candidate_runs.py"
HELPERS = ("auto-bump.yml", "release-minor.yml")
RUNS = "Dry-run the release and run CI on both candidates"
DEV = "Commit and push the dev candidate"
PUBLISH = "Publish release commit and dispatch release"
ADVANCE = "Advance main to the next dev version"


def helper_steps(name):
    (body,) = jobs((WORKFLOWS / name).read_text(encoding="utf-8")).values()
    return steps(body)


class CandidateWorkflowTests(unittest.TestCase):
    """The helpers dispatch CI on both candidates and push only those commits."""

    def test_ci_can_be_dispatched_on_a_candidate(self):
        text = (WORKFLOWS / "ci.yml").read_text(encoding="utf-8")
        triggers = text.split("\non:\n", 1)[1].split("\nenv:", 1)[0]
        self.assertRegex(triggers, r"(?m)^  workflow_dispatch:", "ci.yml cannot be run on a candidate branch")

    def test_both_helpers_wait_on_ci_for_both_candidates_and_the_dry_run(self):
        for name in HELPERS:
            with self.subTest(workflow=name):
                found = helper_steps(name)
                index = step_index(found, RUNS)
                self.assertNotEqual(-1, index, f"{name} has no step named {RUNS!r}")
                runs = squash(found[index].replace("\\\n", " "))  # one command across a continuation
                self.assertIn('sha="${{ steps.candidate.outputs.sha }}"', runs)
                self.assertIn('dev_sha="${{ steps.dev_candidate.outputs.sha }}"', runs)
                self.assertIn('dispatch release.yml "$branch" "$sha" version="$next" '
                              'publish_crates=false create_github_release=false', runs)
                self.assertIn('release_ci="$(python3 scripts/candidate_runs.py dispatch ci.yml "$branch" "$sha")"',
                              runs)
                self.assertIn('dev_ci="$(python3 scripts/candidate_runs.py dispatch ci.yml '
                              '"$dev_branch" "$dev_sha")"', runs)
                self.assertIn('python3 scripts/candidate_runs.py watch "$dry_run" "$release_ci" "$dev_ci"', runs)

    def test_the_dev_advance_is_built_before_the_runs(self):
        for name in HELPERS:
            with self.subTest(workflow=name):
                found = helper_steps(name)
                dev, runs = step_index(found, DEV), step_index(found, RUNS)
                self.assertNotEqual(-1, dev, f"{name} has no step named {DEV!r}")
                self.assertLess(dev, runs, f"{name} builds the -dev advance after CI ran")
                self.assertIn("id: dev_candidate", found[dev])
                self.assertIn('git commit -m "Open ${dev} for development"', found[dev])

    def test_no_commit_is_made_after_the_runs(self):
        for name in HELPERS:
            with self.subTest(workflow=name):
                found = helper_steps(name)
                for step in found[step_index(found, RUNS) + 1:]:
                    for made in ("git commit", "set-version", "cargo update", "git checkout", "git switch"):
                        self.assertNotIn(made, step, f"{name}: {step_name(step)!r} changes the tree after CI ran")

    def test_every_push_to_main_is_a_commit_the_runs_passed(self):
        """The release commit and then the advance, each by the sha CI ran on."""
        expected = {PUBLISH: "steps.candidate.outputs.sha", ADVANCE: "steps.dev_candidate.outputs.sha"}
        for name in HELPERS:
            with self.subTest(workflow=name):
                found = helper_steps(name)
                runs = step_index(found, RUNS)
                pushed, bodies = {}, {}
                for index, step in enumerate(found):
                    for target in re.findall(r"git push origin ([^\s;]+)", step):
                        bare = target.strip('"')
                        if bare == "main" or bare.endswith((":main", ":refs/heads/main")):
                            self.assertGreater(index, runs, f"{name} pushes to main before CI ran")
                            pushed.setdefault(step_name(step), []).append(target)
                            bodies[step_name(step)] = step
                self.assertEqual(set(expected), set(pushed), f"{name} pushes to main from other steps")
                for step, output in expected.items():
                    self.assertEqual(['"${sha}:refs/heads/main"'], pushed[step], f"{name}: {step!r}")
                    self.assertIn(f'sha="${{{{ {output} }}}}"', bodies[step])

    def test_a_quiet_scheduled_run_builds_no_dev_candidate(self):
        """§FS-distribution.4.1: nothing is opened before a release."""
        found = helper_steps("auto-bump.yml")
        for step in (DEV, RUNS):
            with self.subTest(step=step):
                self.assertIn("if: steps.gate_substantive.outputs.ok == 'true'", found[step_index(found, step)])


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
        dispatched = self.calls()[0]
        self.assertEqual(["workflow", "run", "ci.yml", "--ref", branch, "-f", "version=0.18.0", "-f", "dry=true"],
                         dispatched)
        listed = self.calls()[1]
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
        result = self.run_script({"dispatch_error": "HTTP 403: Resource not accessible"},
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
