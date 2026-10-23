"""§FS-distribution.4.4.1 — the `-dev` advance pushes the tested commit onto a `main`
that is still the release commit, and refuses only a `main` that has moved.

Each case runs the advance the way both helpers reach it: their candidates step
hands everything from the candidates on to `scripts/release_candidates.py`, and
each case calls that controller's own `advance`, which runs the one script the
advance has (the fixture clone links this checkout's `scripts/`). Its origin is
a local bare repository whose `main` is the release commit R, with the advance D
over R on the candidate branch and a pre-receive hook that refuses pushes to
`main` as the case asks. The retry backoff is zeroed with
`GRUND_ADVANCE_RETRY_DELAY=0`, so no case waits and none reaches the network."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT, environment
from test_release_candidate_runs import CANDIDATES, CONTROLLER_CALL, HELPERS, helper_steps
from test_release_workflow import step_index

RELEASED, DEV = "0.19.0", "0.19.1-dev"
BRANCH = "release-candidate/dev-0.19.1"
REJECTION = "missing necessary objects (simulated transient rejection)"
MOVED = "main moved"
# Where the case puts the advance's commit, which only the fixture knows.
DEV_SHA = "@DEV_SHA@"
# Refuses pushes to main while `refuse` holds a count above zero (`always` refuses every one),
# and counts every push to main that reaches it.
HOOK = """#!/bin/sh
here=$(dirname "$0")
while read old new ref; do
  [ "$ref" = refs/heads/main ] || continue
  n=$(cat "$here/pushes" 2>/dev/null || echo 0); echo $((n + 1)) > "$here/pushes"
  left=$(cat "$here/refuse" 2>/dev/null || echo 0)
  if [ "$left" = always ]; then echo "%s" >&2; exit 1; fi
  if [ "$left" -gt 0 ]; then echo $((left - 1)) > "$here/refuse"; echo "%s" >&2; exit 1; fi
done
exit 0
""" % (REJECTION, REJECTION)


def advance_script(workflow):
    """The controller's advance, as the helper's candidates step reaches it."""
    found = helper_steps(workflow)
    index = step_index(found, CANDIDATES)
    if index == -1 or CONTROLLER_CALL not in found[index]:
        raise AssertionError(f"{workflow} does not hand its candidates to {CONTROLLER_CALL}")
    call = (f"import sys; sys.path.insert(0, 'scripts'); import release_candidates; "
            f"sys.exit(release_candidates.advance('{RELEASED}', '{DEV}', '{BRANCH}', '{DEV_SHA}'))")
    return f'"{sys.executable}" -c "{call}"\n'


@unittest.skipIf(sys.platform == "win32", "the step is a bash script with a shell pre-receive hook")
@unittest.skipUnless(shutil.which("bash") and shutil.which("git"), "needs bash and git")
class DevAdvanceTests(unittest.TestCase):
    """The advance retries an unmoved main and blames a moved main only when it moved."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="grund-dev-advance-"))
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.env = environment({
            "GIT_CONFIG_GLOBAL": str(self.root / "gitconfig"),
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_AUTHOR_NAME": "Fixture", "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
            "GIT_COMMITTER_NAME": "Fixture", "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
            "GRUND_ADVANCE_RETRY_DELAY": "0",
        })

    def git(self, repo, *arguments):
        return subprocess.run(["git", "-C", str(repo), *arguments], check=True, capture_output=True,
                              text=True, env=self.env).stdout.strip()

    def fixture(self, workflow):
        """origin's main is the release commit R; D over R is on the candidate branch."""
        case = self.root / workflow
        origin, work = case / "origin.git", case / "work"
        case.mkdir()
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(origin)], check=True, env=self.env)
        subprocess.run(["git", "init", "-q", "-b", "main", str(work)], check=True, env=self.env)
        self.git(work, "commit", "-q", "--allow-empty", "-m", f"Release v{RELEASED}")
        self.git(work, "remote", "add", "origin", str(origin))
        self.git(work, "push", "-q", "origin", "main")
        self.git(work, "checkout", "-q", "-b", BRANCH)
        self.git(work, "commit", "-q", "--allow-empty", "-m", f"Open {DEV}")
        self.git(work, "push", "-q", "origin", BRANCH)
        (work / "scripts").symlink_to(REPO_ROOT / "scripts", target_is_directory=True)
        hook = origin / "hooks" / "pre-receive"
        hook.write_text(HOOK, encoding="utf-8")
        hook.chmod(0o755)
        return origin, work, self.git(work, "rev-parse", "main"), self.git(work, "rev-parse", "HEAD")

    def advance(self, workflow, origin, work, dev_sha, refuse):
        (origin / "hooks" / "refuse").write_text(str(refuse), encoding="utf-8")
        step = work.parent / "step.sh"
        step.write_text(advance_script(workflow).replace(DEV_SHA, dev_sha), encoding="utf-8")
        result = subprocess.run(["bash", str(step)], cwd=work, capture_output=True, text=True,
                                env=self.env, timeout=120)
        pushes = origin / "hooks" / "pushes"
        count = int(pushes.read_text(encoding="utf-8")) if pushes.exists() else 0
        return result, self.git(origin, "rev-parse", "main"), count

    def test_a_transient_rejection_onto_an_unmoved_main_is_retried(self):
        for workflow in HELPERS:
            with self.subTest(workflow=workflow):
                origin, work, _release, dev = self.fixture(workflow)
                result, tip, _pushes = self.advance(workflow, origin, work, dev, refuse=1)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertEqual(dev, tip, "main did not take the tested advance")
                self.assertNotIn(MOVED, result.stderr)

    def test_a_main_that_moved_refuses_and_says_so(self):
        for workflow in HELPERS:
            with self.subTest(workflow=workflow):
                origin, work, release, dev = self.fixture(workflow)
                self.git(work, "checkout", "-q", "-b", "elsewhere", release)
                self.git(work, "commit", "-q", "--allow-empty", "-m", "A change merged meanwhile")
                moved = self.git(work, "rev-parse", "HEAD")
                self.git(work, "push", "-q", "origin", "elsewhere:main")
                self.git(work, "checkout", "-q", BRANCH)
                (origin / "hooks" / "pushes").unlink()
                result, tip, _pushes = self.advance(workflow, origin, work, dev, refuse=0)
                self.assertNotEqual(0, result.returncode)
                self.assertIn(MOVED, result.stderr)
                self.assertEqual(moved, tip, "the advance moved a main that had moved")

    def test_a_push_that_keeps_failing_onto_an_unmoved_main_names_gits_error(self):
        for workflow in HELPERS:
            with self.subTest(workflow=workflow):
                origin, work, release, dev = self.fixture(workflow)
                result, tip, pushes = self.advance(workflow, origin, work, dev, refuse="always")
                self.assertNotEqual(0, result.returncode)
                self.assertIn(REJECTION, result.stderr)
                self.assertNotIn(MOVED, result.stderr)
                self.assertEqual(release, tip, "main left the release commit")
                self.assertGreater(pushes, 1, "the push onto an unmoved main was not retried")


if __name__ == "__main__":
    unittest.main()
