"""§FS-distribution.4.4.1.1 — the watch on a round's runs gives a round up the
moment `main` moves, and the publisher is held to the commit those runs passed.

`scripts/candidate_runs.py` runs here against `release_queue_forge.py`, the
stateful stub forge the helper acceptance uses, in a clone of a local bare
origin, so no case reaches the network:

    candidate_runs.py watch [--main <sha>] [--deadline <unix-seconds>] <run-id>...

exits 0 when every run passed; 1 on a run that did not pass, after cancelling
the others still going - and a failure seen in the same poll as a move is a
failure; 3 when `main`, read with `git ls-remote origin refs/heads/main`, is no
longer `--main`, after cancelling every run still going; 4 when the deadline
passes first, after the same cancels. A cancel the forge refuses because the
run has just finished ends nothing (§AR-ci.11).

`release.yml` is read as text, as `test_release_workflow.py` reads it: it takes
an optional `expected_release_sha` and refuses a source that is not that commit,
and only a publishing run removes the candidate branch it was dispatched from
(§FS-distribution.4.3)."""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT
from release_queue_fixture import FORGE, Condition, _isolated, git, parse_step
from test_release_workflow import WORKFLOWS, jobs, steps

SCRIPT = REPO_ROOT / "scripts" / "candidate_runs.py"
RELEASE = WORKFLOWS / "release.yml"


@unittest.skipIf(sys.platform == "win32", "the stub forge is run through a shell wrapper")
@unittest.skipUnless(shutil.which("git"), "needs git")
class WatchTests(unittest.TestCase):
    """A round's three runs, watched over a `main` the stub can move."""

    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="grund-983-watch-")
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        self.env = _isolated(root)
        self.origin, self.work, self.state, seed = root / "origin.git", root / "work", root / "forge", root / "seed"
        for path in (self.state, root / "bin"):
            path.mkdir()
        git("init", "-q", "--bare", str(self.origin), env=self.env)
        git("init", "-q", str(seed), env=self.env)
        who = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid")
        for name in ("a", "b"):
            git("-C", str(seed), *who, "commit", "-q", "--allow-empty", "-m", name, env=self.env)
            setattr(self, name, git("-C", str(seed), "rev-parse", "HEAD", env=self.env))
        git("-C", str(seed), "push", "-q", str(self.origin), f"{self.a}:refs/heads/main",
            f"{self.a}:refs/heads/release-candidate/v0.20.0-7-1-r1", f"{self.b}:refs/queue/b", env=self.env)
        git("clone", "-q", str(self.origin), str(self.work), env=self.env)
        (root / "bin" / "gh").write_text(f'#!/bin/sh\nexec "{sys.executable}" "{FORGE}" "$@"\n')
        (root / "bin" / "gh").chmod(0o755)
        self.env.update({"PATH": f"{root / 'bin'}{os.pathsep}{os.environ.get('PATH', '')}",
                         "GH_STUB_STATE": str(self.state), "GH_STUB_ORIGIN": str(self.origin),
                         "CANDIDATE_RUNS_POLL": "0", "CANDIDATE_RUNS_WATCH": "0", "PYTHONUTF8": "1"})
        branch = "release-candidate/v0.20.0-7-1-r1"
        self.runs = [self.run_record(1000 + n, workflow, branch) for n, workflow in
                     enumerate(("release.yml", "ci.yml", "ci.yml"))]
        self.save()

    def run_record(self, run_id, workflow, branch, **extra):
        return {"id": run_id, "workflow": workflow, "branch": branch, "sha": self.a, "fields": {},
                "event": "workflow_dispatch", "status": "queued", "conclusion": None, "main": self.a,
                "round": 1, "looks": 0, "hidden": 0, **extra}

    def save(self, **scenario):
        (self.state / "runs.json").write_text(json.dumps(self.runs))
        (self.state / "scenario.json").write_text(json.dumps(scenario))

    def script(self, *arguments):
        return subprocess.run([sys.executable, str(SCRIPT), *arguments], cwd=self.work, env=self.env,
                              capture_output=True, text=True, timeout=60)

    def watch(self, *extra):
        return self.script("watch", *extra, *[str(run["id"]) for run in self.runs])

    def after(self):
        return json.loads((self.state / "runs.json").read_text())

    def calls(self):
        return [json.loads(line) for line in (self.state / "calls").read_text().splitlines()]

    def assertNoneGoing(self, result):
        going = [run["id"] for run in self.after() if run["status"] != "completed"]
        self.assertEqual([], going, f"runs left going\n{result.stderr}")

    def test_a_move_cancels_every_pending_run_and_exits_moved(self):
        self.save(move={"1": self.b})
        result = self.watch("--main", self.a)
        self.assertEqual(3, result.returncode, result.stderr)
        self.assertIn("main moved", result.stderr)
        self.assertIn(self.b, result.stderr)
        self.assertNoneGoing(result)
        self.assertTrue(all(run.get("cancelled") for run in self.after()), self.after())

    def test_a_failed_run_cancels_the_others_still_going(self):
        self.save(fail={"1": "ci.yml"})
        result = self.watch()
        self.assertEqual(1, result.returncode, result.stderr)
        self.assertIn("ended failure", result.stderr)
        self.assertNoneGoing(result)

    def test_a_failed_run_wins_over_a_move_in_the_same_poll(self):
        git("--git-dir", str(self.origin), "update-ref", "refs/heads/main", self.b, env=self.env)
        self.save(fail={"1": "ci.yml"})
        result = self.watch("--main", self.a)
        self.assertEqual(1, result.returncode, result.stderr)
        self.assertIn("ended failure", result.stderr)
        self.assertNoneGoing(result)

    def test_a_cancel_refused_on_a_run_that_just_finished_ends_nothing(self):
        self.save(move={"1": self.b}, finish_before_cancel=True)
        result = self.watch("--main", self.a)
        self.assertEqual(3, result.returncode, result.stderr)
        self.assertIn("main moved", result.stderr)

    def test_main_is_read_with_git_not_the_forge_api(self):
        self.save(move={"1": self.b})
        result = self.watch("--main", self.a)
        self.assertEqual(3, result.returncode, result.stderr)
        asked = {tuple(call[:2]) for call in self.calls()}
        self.assertLessEqual(asked, {("run", "view"), ("run", "cancel")}, self.calls())

    def test_the_deadline_cancels_the_runs_still_going(self):
        self.save(slow=True)
        self.env["CANDIDATE_RUNS_WATCH"] = "0.2"
        started = time.time()
        result = self.watch("--main", self.a, "--deadline", str(int(started) + 2))
        self.assertEqual(4, result.returncode, result.stderr)
        self.assertIn("deadline", result.stderr)
        self.assertLess(time.time() - started, 30)
        self.assertNoneGoing(result)

    def test_dispatch_ignores_a_run_that_existed_before_it(self):
        """The publisher runs on the branch and commit the dry run ran on (§AR-ci.11)."""
        branch = "release-candidate/v0.20.0-7-1-r1"
        self.runs = [self.run_record(1000, "release.yml", branch, status="completed", conclusion="success")]
        self.save()
        result = self.script("dispatch", "release.yml", branch, self.a, "version=0.20.0", "publish_crates=true",
                             f"expected_release_sha={self.a}")
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("1001", result.stdout.strip(), "dispatch named the dry run, not the run it started")


def release_jobs():
    return jobs(RELEASE.read_text(encoding="utf-8"))


def job_condition(body):
    found = re.search(r"(?m)^    if:\s*(.+)$", body)
    return found[1] if found else None


class DryRun:
    """The values a non-publishing dispatch gives every expression; anything else reads empty."""

    def value(self, expr):
        if re.search(r"(publish_crates|create_github_release|create_tag)$", expr):
            return "false"
        return "success" if expr.endswith(".result") else ""


class PublisherTests(unittest.TestCase):
    """`release.yml` publishes only the commit the helper tested, and only it cleans up."""

    def test_release_yml_takes_an_optional_expected_release_sha(self):
        text = RELEASE.read_text(encoding="utf-8")
        inputs = text.split("workflow_dispatch:", 1)[1].split("\npermissions:", 1)[0]
        found = re.search(r"(?m)^      expected_release_sha:\n((?:        .*\n)+)", inputs)
        self.assertIsNotNone(found, "release.yml has no expected_release_sha input")
        self.assertNotRegex(found[1], r"required:\s*true")

    @unittest.skipIf(sys.platform == "win32", "the step is a bash script")
    def test_a_source_other_than_expected_release_sha_is_refused(self):
        body = release_jobs()["verify"]
        guards = [parse_step(step) for step in steps(body) if "expected_release_sha" in step]
        self.assertTrue(guards, "no step of the verify job reads expected_release_sha")
        with tempfile.TemporaryDirectory(prefix="grund-983-publisher-") as scratch:
            repo, env = Path(scratch) / "repo", _isolated(Path(scratch))
            git("init", "-q", str(repo), env=env)
            git("-C", str(repo), "-c", "user.name=F", "-c", "user.email=f@x.invalid", "commit", "-q",
                "--allow-empty", "-m", "release", env=env)
            head = git("-C", str(repo), "rev-parse", "HEAD", env=env)
            other = "0123456789abcdef0123456789abcdef01234567"

            def run(expected):
                codes = []
                for step in guards:
                    script = re.sub(r"\$\{\{\s*inputs\.expected_release_sha\s*\}\}", expected, step["run"])
                    script = re.sub(r"\$\{\{\s*github\.sha\s*\}\}", head, script)
                    step_env = {k: v.replace("${{ inputs.expected_release_sha }}", expected)
                                .replace("${{ github.sha }}", head) for k, v in step.get("env", {}).items()}
                    codes.append(subprocess.run(["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
                                                cwd=repo, env={**env, **step_env, "GITHUB_SHA": head},
                                                capture_output=True, text=True).returncode)
                return codes

            self.assertNotEqual([0] * len(guards), run(other), "a mismatched expected_release_sha was accepted")
            self.assertEqual([0] * len(guards), run(head), "the tested sha itself was refused")
            self.assertEqual([0] * len(guards), run(""), "a manual dispatch without the input was refused")

    def test_only_a_publishing_run_removes_the_candidate_branch(self):
        removals = []
        for name, body in release_jobs().items():
            for step in steps(body):
                if "release-candidate/" in step or ("expected_release_sha" in step and "delete" in step.lower()):
                    if re.search(r"--delete|-X\s*DELETE|--method\s+DELETE", step):
                        removals.append((name, job_condition(body), parse_step(step).get("if")))
        self.assertTrue(removals, "no step of release.yml removes the candidate branch")
        for name, job_if, step_if in removals:
            ran = all(Condition(DryRun(), expr, True).read() for expr in (job_if, step_if) if expr)
            self.assertFalse(ran, f"job {name!r} removes the candidate branch on a dry run")


if __name__ == "__main__":
    unittest.main()
