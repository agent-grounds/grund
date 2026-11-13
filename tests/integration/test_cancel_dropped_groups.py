"""§AR-ci.12.2 — a dropped merge-queue group's run is cancelled.

`scripts/cancel_dropped_groups.py <owner/repo> <own-run-id>` is the sweep `ci.yml` runs on
every group: it lists the `ci.yml` runs and the queue's live `gh-readonly-queue` branches,
and cancels each unfinished `merge_group` run, not its own, whose group is not live: whose
branch the queue deleted, or whose branch now holds another commit because the queue reused
its name. It runs here against a stub `gh` on `PATH` that answers from a table the case
writes and logs every call, so no case reaches the network. A run the stub lists carries its
event, so the script's own filter, not the list's flags, is what leaves another event's run
alone, and every run and branch carries a commit of its own."""

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT, environment

SCRIPT = REPO_ROOT / "scripts" / "cancel_dropped_groups.py"
REPOSITORY = "agent-grounds/grund"
OWN = 500
QUEUE = "gh-readonly-queue/main/pr-"

# The stub models the three calls of §AR-ci.12.2 and refuses every other one, so a sweep
# that asks the forge some other way fails visibly rather than passing on an empty answer.
STUB = r'''#!{python}
import json, os, re, sys

args = sys.argv[1:]
with open(os.environ["GH_STUB_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(args) + "\n")
with open(os.environ["GH_STUB_ANSWERS"], encoding="utf-8") as table:
    answers = json.load(table)

def fail(key):
    sys.stderr.write(answers[key] + "\n")
    sys.exit(1)

if args[:2] == ["run", "list"]:
    if "runs_error" in answers:
        fail("runs_error")
    fields = args[args.index("--json") + 1].split(",") if "--json" in args else []
    print(json.dumps([{{key: run[key] for key in fields if key in run}}
                      for run in answers["runs"]]))
    sys.exit(0)
if args[:1] == ["api"] and "--jq" not in args and "-q" not in args and any(
        "/git/matching-refs/heads/gh-readonly-queue" in arg for arg in args):
    if "refs_error" in answers:
        fail("refs_error")
    print(json.dumps([{{"ref": "refs/heads/" + branch, "object": {{"sha": sha}}}}
                      for branch, sha in answers["live"].items()]))
    sys.exit(0)
if args[:2] == ["run", "cancel"]:
    run = next(arg for arg in args[2:] if re.fullmatch(r"\d+", arg))
    if run in answers.get("cancel_errors", {{}}):
        sys.stderr.write(answers["cancel_errors"][run] + "\n")
        sys.exit(1)
    sys.exit(0)
sys.stderr.write(f"gh stub: no answer for {{args!r}}\n")
sys.exit(3)
'''


def tip(name):
    """A commit of its own for every name: the one a live branch points at, by default."""
    return hashlib.sha1(name.encode("utf-8")).hexdigest()


def run(database_id, branch, status="in_progress", event="merge_group", head=None):
    """One run as `gh run list --json` gives it, on its branch's tip unless `head` names
    another commit; `head=""` leaves the run with no `headSha` at all."""
    found = {"databaseId": database_id, "headBranch": branch, "status": status, "event": event,
             "workflowName": "CI", "headSha": tip(branch) if head is None else head}
    if head == "":
        del found["headSha"]
    return found


def group(number, sha="a"):
    return f"{QUEUE}{number}-{sha * 40}"


def live(*branches):
    """The queue's live branches, each pointing at its own tip, as matching-refs gives them."""
    return {branch: tip(branch) for branch in branches}


class CancelDroppedGroupsTests(unittest.TestCase):
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
            (self.bin / "gh.cmd").write_text(f'@"{sys.executable}" "{stub}" %*\r\n',
                                             encoding="utf-8")
        self.log = self.scratch / "gh.log"

    def sweep(self, answers):
        table = self.scratch / "answers.json"
        table.write_text(json.dumps(answers), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT), REPOSITORY, str(OWN)],
            capture_output=True, encoding="utf-8", errors="replace",
            env=environment({"PATH": str(self.bin), "PYTHONUTF8": "1",
                             "GH_STUB_ANSWERS": str(table), "GH_STUB_LOG": str(self.log)}))

    def calls(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]

    def cancelled(self):
        return [next(arg for arg in call[2:] if arg.isdigit()) for call in self.calls()
                if call[:2] == ["run", "cancel"]]

    def test_the_script_exists(self):
        self.assertTrue(SCRIPT.is_file(), f"{SCRIPT.relative_to(REPO_ROOT)} does not exist")

    def test_a_dropped_groups_run_is_cancelled_and_named(self):
        alive, dropped = group(1380), group(1371, "7")
        result = self.sweep({"runs": [run(OWN, alive), run(401, dropped, status="queued")],
                             "live": live(alive)})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["401"], self.cancelled())
        output = result.stdout + result.stderr
        self.assertIn("401", output)
        self.assertIn(dropped, output, "the cancellation does not name the group's branch")

    def test_the_runs_listed_are_ci_ymls_on_the_queue(self):
        self.sweep({"runs": [], "live": {}})
        (listed,) = [call for call in self.calls() if call[:2] == ["run", "list"]]
        for flag, value in (("--workflow", "ci.yml"), ("--event", "merge_group"),
                            ("--repo", REPOSITORY), ("--limit", "100")):
            with self.subTest(flag=flag):
                self.assertIn(flag, listed)
                self.assertEqual(value, listed[listed.index(flag) + 1])

    def test_every_other_run_is_left_alone(self):
        # §AR-ci.12.2: a live group's run, its own, another event's and a finished one.
        alive = group(1375)
        runs = [run(301, alive), run(OWN, group(1365, "b")),
                run(302, "main", event="push"), run(303, group(1360, "c"), event="push"),
                run(304, "release-candidate/v0.23.0-1", event="workflow_dispatch"),
                run(305, group(1356, "d"), status="completed")]
        result = self.sweep({"runs": runs, "live": live(alive)})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual([], self.cancelled())

    def test_a_dropped_groups_run_is_cancelled_when_its_name_is_reused(self):
        # §AR-ci.12.2: queued again on the same commit, a pull request's new group takes the
        # dropped one's branch name, and only the branch's commit tells their runs apart.
        reused, other = group(894, "6"), group(1173, "d")
        old = tip("the dropped group's head")
        runs = [run(OWN, reused), run(401, reused, head=old), run(301, other)]
        result = self.sweep({"runs": runs, "live": live(reused, other)})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["401"], self.cancelled(),
                         "a run on a live name but an older head is a dropped group's run")
        self.assertIn(reused, result.stdout + result.stderr)

    def test_a_run_with_no_head_commit_is_not_live(self):
        # §AR-ci.12.2: without a headSha nothing says the run is the live group's.
        alive = group(1380)
        result = self.sweep({"runs": [run(OWN, alive), run(401, alive, head="")],
                             "live": live(alive)})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["401"], self.cancelled())

    def test_every_dropped_group_is_cancelled_in_one_pass(self):
        alive = group(1380)
        runs = [run(OWN, alive), run(401, group(1371, "7")), run(402, group(1365, "8"), "queued"),
                run(403, group(1355, "9"), "waiting"), run(404, alive)]
        result = self.sweep({"runs": runs, "live": live(alive)})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["401", "402", "403"], sorted(self.cancelled()))

    def test_a_failed_run_list_cancels_nothing_and_passes(self):
        result = self.sweep({"runs": [run(401, group(1371))], "live": {},
                             "runs_error": "HTTP 502: Bad Gateway"})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual([], self.cancelled())
        self.assertIn("HTTP 502: Bad Gateway", result.stdout + result.stderr)

    def test_a_failed_branch_list_cancels_nothing_and_passes(self):
        # §AR-ci.12.2: a branch list it could not read would make every group look dropped.
        result = self.sweep({"runs": [run(OWN, group(1380)), run(401, group(1371))],
                             "live": live(group(1380)), "refs_error": "HTTP 403: Forbidden"})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual([], self.cancelled())
        self.assertIn("HTTP 403: Forbidden", result.stdout + result.stderr)

    def test_a_failed_cancel_goes_on_to_the_next_and_passes(self):
        alive = group(1380)
        runs = [run(OWN, alive), run(401, group(1371)), run(402, group(1365))]
        result = self.sweep({"runs": runs, "live": live(alive),
                             "cancel_errors": {"401": "Cannot cancel a workflow run that is "
                                                      "completed. (HTTP 409)"}})
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["401", "402"], sorted(self.cancelled()))
        self.assertIn("HTTP 409", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
