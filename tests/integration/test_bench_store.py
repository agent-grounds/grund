"""§AR-ci.5.5.3, §AR-ci.5.5.2, §AR-ci.5.5.5, §AR-benchmarks.7.5 — `scripts/bench_store.py`
keeps a push's run record on the `benchmark-data` branch as `pr/<commit>.json`. It only ever
adds that one file on top of the branch's tip, lets a commit's first record stand, never
creates the branch, and retries a push a race refused a bounded number of times, without
force. A missing record passes only when the record job did not succeed.

It runs by hand on a downloaded record and its commit, so this module runs it the way a
person does: in a checkout whose `origin` is a bare repository the module makes, with no
workflow context. `test_bench_store_workflow.py` holds the workflow that calls it."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from test_bench_record import IDENTITY, load as load_recorder
from test_release_workflow import REPO_ROOT

SCRIPT = REPO_ROOT / "scripts" / "bench_store.py"
BRANCH = "benchmark-data"
# §AR-ci.5.5.2: the conclusion of CI's record job, as the forge reports it.
CONCLUSION = "--bench-record-conclusion"


class TheStoreScript(unittest.TestCase):
    """§AR-ci.5.5.2, §AR-ci.5.5.3, §AR-ci.5.5.5: `bench_store.py` run by hand, as a person runs
    it: in a checkout whose `origin` holds `benchmark-data`, on a downloaded record and its
    commit, with no workflow context."""

    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="bench-store-")
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        config = self.root / "gitconfig"
        # §AR-ci.10.4 for every repository the store makes too, and nothing of the user's.
        config.write_text("[gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n"
                          "[core]\n\tfsmonitor = false\n[commit]\n\tgpgsign = false\n",
                          encoding="utf-8")
        # No workflow context: nothing of CI's, the forge's or the hook's reaches the store.
        ambient = ("GITHUB_", "RUNNER_", "ACTIONS_", "GH_", "GIT_", "PRE_COMMIT_")
        self.env = {key: value for key, value in os.environ.items() if not key.startswith(ambient)}
        self.env.update(GIT_CONFIG_GLOBAL=str(config), GIT_CONFIG_NOSYSTEM="1",
                        GIT_TERMINAL_PROMPT="0", GIT_AUTHOR_NAME="Fixture",
                        GIT_AUTHOR_EMAIL="fixture@example.invalid", GIT_COMMITTER_NAME="Fixture",
                        GIT_COMMITTER_EMAIL="fixture@example.invalid")
        self.remote, self.work = self.root / "remote.git", self.root / "work"
        self.git(self.root, "init", "-q", "--bare", str(self.remote))
        self.git(self.root, "init", "-q", "-b", "main", str(self.work))
        for repo in (self.remote, self.work):  # §AR-ci.10.4
            self.git(repo, "config", "gc.auto", "0")
            self.git(repo, "config", "maintenance.auto", "false")
        (self.work / "README.md").write_text("measured\n", encoding="utf-8")
        self.git(self.work, "add", "README.md")
        self.git(self.work, "commit", "-q", "-m", "A commit on main")
        self.commit = self.git(self.work, "rev-parse", "HEAD")
        self.git(self.work, "remote", "add", "origin", str(self.remote))
        self.git(self.work, "push", "-q", "origin", "main")
        self.recorder = load_recorder()

    def git(self, repo, *arguments, stdin=None):
        return subprocess.run(["git", "-C", str(repo), *arguments], check=True, capture_output=True,
                              text=True, input=stdin, env=self.env).stdout.strip()

    def start_branch(self):
        """The branch as a person makes it: one empty root commit (§AR-ci.5.5.4)."""
        tree = self.git(self.remote, "mktree", stdin="")
        root = self.git(self.remote, "commit-tree", tree, "-m", "Start benchmark-data")
        self.git(self.remote, "update-ref", f"refs/heads/{BRANCH}", root)
        return root

    def record(self, name="first.json", commit=None, ir=1000, series=None):
        record = self.recorder.build_record(commit=commit or self.commit, binary_sha256="e" * 64,
                                            identity=IDENTITY, ir={"check_large_10k": ir}, refused=[])
        if series:
            record["series"] = series
        path = self.root / "download" / name
        self.recorder.write(record, path)
        return path

    def store(self, record, *arguments, race=None):
        self.assertTrue(SCRIPT.is_file(),
                        f"no {SCRIPT.relative_to(REPO_ROOT)}: §AR-ci.5.5.3 has no writer")
        env = dict(self.env)
        if race:
            env.update(self.racer(race))
        result = subprocess.run([sys.executable, str(SCRIPT), "--record", str(record),
                                 "--commit", self.commit, *arguments], cwd=self.work, env=env,
                                capture_output=True, text=True, timeout=120)
        self.assertNotIn("Traceback", result.stderr, "the store crashed rather than refused")
        return result

    def racer(self, mode):
        """A `git` on PATH that lands another record on the branch before the push it wraps:
        before the first push, or before every push."""
        clone = self.root / "racer"
        self.git(self.root, "clone", "-q", "-b", BRANCH, str(self.remote), str(clone))
        shim = self.root / "bin" / "git"
        shim.parent.mkdir()
        shim.write_text(f"#!{sys.executable}\n" + RACING_GIT, encoding="utf-8")
        shim.chmod(0o755)
        self.race_log = self.root / "pushes"
        return {"PATH": f"{shim.parent}{os.pathsep}{self.env['PATH']}", "STORE_RACE": mode,
                "STORE_REAL_GIT": shutil.which("git"), "STORE_RACER": str(clone),
                "STORE_RACE_LOG": str(self.race_log)}

    def tip(self):
        return self.git(self.remote, "rev-parse", f"refs/heads/{BRANCH}")

    def stored(self, path):
        return self.git(self.remote, "show", f"refs/heads/{BRANCH}:{path}") + "\n"

    def assertUntouched(self, root, result):
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(root, self.tip())

    def test_by_hand_it_adds_exactly_its_record_on_top_of_the_tip(self):
        root, record = self.start_branch(), self.record()
        head = self.git(self.work, "rev-parse", "HEAD")
        result = self.store(record)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(f"{self.tip()} {root}",
                         self.git(self.remote, "rev-list", "--parents", "-n1", self.tip()))
        self.assertEqual(f"A\tpr/{self.commit}.json",
                         self.git(self.remote, "diff-tree", "-r", "--name-status", root, self.tip()))
        self.assertEqual(record.read_text(encoding="utf-8"), self.stored(f"pr/{self.commit}.json"))
        # §AR-ci.5.5.3: the branch is never checked out.
        checkout = [self.git(self.work, *probe) for probe in (
            ("rev-parse", "--symbolic-full-name", "HEAD"), ("rev-parse", "HEAD"), ("status", "--porcelain"))]
        self.assertEqual(["refs/heads/main", head, ""], checkout)

    def test_the_first_record_of_a_commit_stands(self):
        self.start_branch()
        first = self.record()
        self.assertEqual(0, self.store(first).returncode)
        tip = self.tip()
        for again in (first, self.record("rerun.json", ir=1004)):
            result = self.store(again)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertEqual(tip, self.tip())
        self.assertEqual(first.read_text(encoding="utf-8"), self.stored(f"pr/{self.commit}.json"))

    def test_without_the_branch_it_fails_and_creates_nothing(self):
        refs = self.git(self.remote, "for-each-ref")
        result = self.store(self.record())
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn(BRANCH, result.stdout + result.stderr)
        self.assertEqual(refs, self.git(self.remote, "for-each-ref"))

    def test_a_record_of_another_commit_or_series_is_refused(self):
        root = self.start_branch()
        self.assertUntouched(root, self.store(self.record(commit="f" * 40)))
        self.assertUntouched(root, self.store(self.record(series="0" * 64)))

    def test_no_record_passes_only_when_the_record_job_did_not_succeed(self):
        root, missing = self.start_branch(), self.root / "download" / "first.json"
        for conclusion in ("failure", "cancelled", "skipped"):
            result = self.store(missing, CONCLUSION, conclusion)
            self.assertEqual(0, result.returncode, f"{conclusion}: {result.stdout}{result.stderr}")
            self.assertEqual(root, self.tip())
        # §AR-ci.5.5.2: a record the job wrote and the store cannot find is a record lost.
        for lost in ((CONCLUSION, "success"), (), (CONCLUSION, "")):
            self.assertUntouched(root, self.store(missing, *lost))

    @unittest.skipIf(os.name == "nt", "the racing git is a POSIX script; the store runs on Linux")
    def test_a_store_that_loses_a_race_adds_its_record_on_the_new_tip(self):
        root, record = self.start_branch(), self.record()
        result = self.store(record, race="once")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertTrue(self.race_log.is_file(), "the store ran no `git push` that a race could beat")
        history = self.git(self.remote, "rev-list", "--parents", self.tip()).splitlines()
        self.assertEqual(3, len(history), "the history is not root, the racer's record, then this one")
        self.assertTrue(all(len(line.split()) <= 2 for line in history), "the history is not linear")
        raced = history[1].split()[0]
        self.assertEqual(f"A\tpr/{self.commit}.json",
                         self.git(self.remote, "diff-tree", "-r", "--name-status", raced, self.tip()))
        self.assertEqual("{}\n", self.stored(f"pr/{1:040x}.json"))
        self.assertEqual(root, history[2])

    @unittest.skipIf(os.name == "nt", "the racing git is a POSIX script; the store runs on Linux")
    def test_a_store_that_keeps_losing_fails_after_a_bounded_number_of_pushes(self):
        self.start_branch()
        result = self.store(self.record(), race="always")
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        pushes = len(self.race_log.read_text(encoding="utf-8").splitlines())
        self.assertTrue(2 <= pushes <= 10, f"the store pushed {pushes} times")
        listed = self.git(self.remote, "ls-tree", "-r", "--name-only", self.tip())
        self.assertNotIn(f"pr/{self.commit}.json", listed)


# The racing `git` of `TheStoreScript.racer`: before a `push`, another store lands a record.
RACING_GIT = r'''import os, subprocess, sys
real, rest = os.environ["STORE_REAL_GIT"], sys.argv[1:]
while rest and rest[0].startswith("-"):
    rest = rest[2:] if rest[0] in ("-C", "-c", "--git-dir", "--work-tree") else rest[1:]
if rest[:1] == ["push"]:
    BRANCH = "benchmark-data"
    with open(os.environ["STORE_RACE_LOG"], "a+", encoding="utf-8") as log:
        log.seek(0)
        count = len(log.read().splitlines())
        log.write("push\n")
    if os.environ["STORE_RACE"] == "always" or count == 0:
        racer, name = os.environ["STORE_RACER"], f"pr/{count + 1:040x}.json"
        os.makedirs(os.path.join(racer, "pr"), exist_ok=True)
        with open(os.path.join(racer, "pr", name[3:]), "w", encoding="utf-8") as record:
            record.write("{}\n")
        push = ["push", "-q", "origin", f"HEAD:refs/heads/{BRANCH}"]
        for step in (["add", name], ["commit", "-q", "-m", name], push):
            subprocess.run([real, "-C", racer, *step], check=True)
os.execv(real, [real, *sys.argv[1:]])
'''


if __name__ == "__main__":
    unittest.main()
