"""§AR-ci.3.1.4 — a dispatched run measures the link gate's changed lines from the
merge base with the upstream repository's `main`, never from a fork's stale one,
and an unresolvable upstream is no diff base rather than a fallback (#981).

An upstream repository, a bare fork whose `main` stopped at the upstream's first
commit, and a runner checked out from the fork the way `actions/checkout` does
on a fork's dispatch. Real lychee and a loopback host that counts requests: no
real host is reached.
"""

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_gate_fixture import LYCHEE, ROOT, LinkFixture, isolated_env  # noqa: E402


DISPATCH_BASE = ROOT / "scripts" / "dispatch_link_base.py"


def git(cwd: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
         "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main", *args],
        cwd=cwd, env=isolated_env(), check=True, capture_output=True, text=True,
    ).stdout.strip()


class ForkDispatch:
    """Upstream A -> B -> C; the fork's `main` is A; the runner is at C with `origin` = fork.

    B adds `/upstream-added` on upstream `main` after the fork synced; C, the dispatched
    change, adds `/change-added` and nothing else."""

    def __init__(self, test: unittest.TestCase):
        self.gate = LinkFixture()
        test.addCleanup(self.gate.close)
        side = tempfile.TemporaryDirectory(prefix="grund-981-dispatch-", dir=self.gate.root.parent)
        test.addCleanup(side.cleanup)
        self.upstream = Path(side.name) / "upstream"
        self.fork = Path(side.name) / "fork.git"
        self.upstream.mkdir()
        host = self.gate.base

        git(self.upstream, "init", "-q")
        self._write("docs/guide.md", "# Guide\n\nPlain text.\n")
        git(self.upstream, "add", "-A")
        git(self.upstream, "commit", "-q", "-m", "A")
        git(self.upstream.parent, "clone", "-q", "--bare", str(self.upstream), str(self.fork))
        self._write("docs/guide.md", f"# Guide\n\nPlain text.\n\nSee <{host}/upstream-added>.\n")
        git(self.upstream, "commit", "-q", "-am", "B")
        self.upstream_main = git(self.upstream, "rev-parse", "HEAD")
        self._write("docs/notes.md", f"# Notes\n\nSee <{host}/change-added>.\n")
        git(self.upstream, "add", "-A")
        git(self.upstream, "commit", "-q", "-m", "C")
        change = git(self.upstream, "rev-parse", "HEAD")
        git(self.upstream, "push", "-q", str(self.fork), "HEAD:refs/heads/measure")

        runner = self.gate.root
        git(runner, "init", "-q")
        git(runner, "remote", "add", "origin", str(self.fork))
        git(runner, "fetch", "-q", "origin", "+refs/heads/*:refs/remotes/origin/*")
        git(runner, "checkout", "-q", "--detach", change)
        self.stale_main = git(runner, "rev-parse", "origin/main")

    def _write(self, relative: str, text: str) -> None:
        path = self.upstream / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def link_base(self, upstream_url: str) -> str:
        """What the job hands the hook as LINK_BASE on a dispatch. Without the script, that is
        what ci.yml's event expression gives a dispatch today: the empty string."""
        if not DISPATCH_BASE.is_file():
            return ""
        result = subprocess.run(
            [sys.executable, str(DISPATCH_BASE), upstream_url], cwd=self.gate.root,
            env=isolated_env(), capture_output=True, text=True, timeout=120,
        )
        if result.returncode != 0:
            raise AssertionError(f"{DISPATCH_BASE.name} exited {result.returncode}:\n{result.stderr}")
        return result.stdout.strip()


@unittest.skipUnless(LYCHEE, "Link gate fixture requires CI-pinned lychee 0.23.0 on PATH")
class DispatchBaseTests(unittest.TestCase):
    def test_the_fork_fixture_has_a_stale_main(self):
        """The precondition the defect needs: `origin/main` is behind upstream `main`."""
        fork = ForkDispatch(self)
        self.assertNotEqual(fork.upstream_main, fork.stale_main)
        self.assertEqual(fork.stale_main, git(fork.gate.root, "merge-base", "HEAD", "origin/main"))

    def test_dispatch_on_a_fork_resolves_the_upstream_merge_base(self):
        fork = ForkDispatch(self)

        base = fork.link_base(str(fork.upstream))

        self.assertEqual(
            fork.upstream_main, base,
            f"a fork dispatch must measure from the upstream merge base {fork.upstream_main[:10]},"
            f" not the fork's stale main {fork.stale_main[:10]}; LINK_BASE was {base!r}",
        )

    def test_dispatch_on_a_fork_requests_only_the_url_the_change_added(self):
        fork = ForkDispatch(self)

        code, output = fork.gate.run("docs", link_base=fork.link_base(str(fork.upstream)))

        self.assertEqual(0, code, output)
        self.assertEqual(["/change-added"], sorted(fork.gate.requested()), output)

    def test_unresolvable_upstream_requests_no_external_url_and_says_so(self):
        fork = ForkDispatch(self)
        missing = fork.upstream.parent / "no-such-upstream.git"

        code, output = fork.gate.run("docs", link_base=fork.link_base(str(missing)))

        self.assertEqual({}, fork.gate.requested(), output)
        self.assertEqual(0, code, output)
        self.assertEqual(1, len(re.findall(r"(?m)^no diff base:", output)), output)

    def test_a_link_base_naming_no_commit_is_no_diff_base(self):
        """A set LINK_BASE that resolves to nothing never falls back to `origin/main`."""
        fork = ForkDispatch(self)

        code, output = fork.gate.run("docs", link_base="none")

        self.assertEqual({}, fork.gate.requested(), output)
        self.assertEqual(0, code, output)
        self.assertEqual(1, len(re.findall(r"(?m)^no diff base:", output)), output)


class CiGivesADispatchItsBaseTests(unittest.TestCase):
    """§AR-ci.3.1.4 — ci.yml resolves a dispatch's base and feeds it into LINK_BASE."""

    def test_pre_commit_feeds_the_dispatch_base_into_link_base(self):
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        job = re.search(r"(?ms)^  pre-commit:\n(?P<body>.*?)(?=^  [a-z][a-z0-9-]*:\n)", workflow)
        self.assertIsNotNone(job, "ci.yml has no pre-commit job")
        body = job["body"]
        steps = {m["id"]: m["step"] for m in re.finditer(
            r"(?ms)^      - (?:name: [^\n]*\n\s+)?id: (?P<id>[\w-]+)\n(?P<step>.*?)(?=^      - |\Z)",
            body)}
        resolving = [i for i, s in steps.items() if "scripts/dispatch_link_base.py" in s]
        self.assertTrue(resolving, f"no step of the pre-commit job with an id runs"
                                   f" dispatch_link_base.py; steps with ids: {sorted(steps)}")
        self.assertIn("workflow_dispatch", steps[resolving[0]])
        link_base = re.search(r"\n\s+LINK_BASE: (?P<value>[^\n]*)", body)
        self.assertIsNotNone(link_base, "the pre-commit job sets no LINK_BASE")
        self.assertIn(f"steps.{resolving[0]}.outputs.", link_base["value"])


if __name__ == "__main__":
    unittest.main()
