"""§FS-distribution.4.12 — why a pending change is one file of its own, proved the
way agent-grounds/grund#379 measured the cost: two pull requests cut from one
base each add an entry, and they must apply in either order, each pass the gate
(§AR-ci.7) against the base it was cut from and again once rebased onto the
other, and both reach the release `prepare` writes (§FS-distribution.4.5).

It ports `grund-379-changelog-conflict.sh`, which exits 1 on the `main` this
change was cut from: every rebase conflicted in `docs/changelog.md`. The
repository is seeded with this tree's own changelog and entry README, so what
it proves is the arrangement the repository ships rather than one written for
the test. Two scenarios run, as in the script: `empty`, a base fresh from a
release, and `seeded`, a base that already holds one merged entry."""

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess

from changelog_gate_fixture import ENTRIES, REPO_ROOT, SCRIPT_PATH, GitFixture, environment

PREPARE = REPO_ROOT / "scripts" / "prepare_changelog_release.py"
SEEDED_FROM = ("docs/changelog.md", f"{ENTRIES}/README.md")
SCENARIOS = ("empty", "seeded")
VERSION = "99.0.0"
IDENTITY = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false")


def entry_text(name: str) -> str:
    return f"- Reproducer entry {name}. (PR #TBD)\n"


class TwoPullRequestsTests(GitFixture, unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        self.repo = root / "repo"
        self._git(root, "init", "-b", "main", str(self.repo))
        for relative in SEEDED_FROM:
            (self.repo / relative).parent.mkdir(parents=True, exist_ok=True)
            (self.repo / relative).write_bytes((REPO_ROOT / relative).read_bytes())
        self._git(self.repo, "update-ref", "refs/remotes/origin/main", self._commit(self.repo, "The base"))

    def _try(self, *arguments: str) -> CompletedProcess:
        """Git without `check`: a rebase that conflicts is an answer, not an error."""
        return subprocess.run(
            ["git", "-C", str(self.repo), *IDENTITY, "-c", "rerere.enabled=false", *arguments],
            capture_output=True,
            text=True,
            env=environment({"GIT_CONFIG_GLOBAL": str(self.repo / ".gitconfig-absent")}),
        )

    def _add_entry(self, name: str) -> str:
        (self.repo / ENTRIES / f"{name}.added.md").write_bytes(entry_text(name).encode("utf-8"))
        return self._commit(self.repo, f"Add changelog entry {name}")

    def _scenario(self, scenario: str) -> tuple[str, str, str]:
        """The base, and the tips of two branches cut from it that each add one entry.
        Whatever an earlier scenario left in the working tree is discarded first."""
        self._git(self.repo, "reset", "-q", "--hard")
        self._git(self.repo, "clean", "-qfd")
        self._git(self.repo, "checkout", "-q", "-B", f"{scenario}-base", "main")
        base = self._add_entry("already-merged") if scenario == "seeded" else self._git(self.repo, "rev-parse", "HEAD")
        tips = []
        for side in ("a", "b"):
            self._git(self.repo, "checkout", "-q", "-B", f"{scenario}-{side}", base)
            tips.append(self._add_entry(f"{scenario}-{side}"))
        return base, tips[0], tips[1]

    def _rebase(self, tip: str, onto: str) -> str:
        """Rebase `tip` onto `onto` on a trial branch; the rebased tip, or a failure naming the conflict."""
        self._git(self.repo, "checkout", "-q", "-B", f"trial-{tip[:12]}-onto-{onto[:12]}", tip)
        result = self._try("rebase", "--quiet", onto)
        if result.returncode != 0:
            conflicted = self._try("diff", "--name-only", "--diff-filter=U").stdout.split()
            self._try("rebase", "--abort")
            self.fail(f"rebasing {tip[:12]} onto {onto[:12]} conflicts in {conflicted}: {result.stderr}")
        return self._git(self.repo, "rev-parse", "HEAD")

    def _gate(self, base: str, head: str, number: int) -> CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--base-sha", base, "--head-sha", head, "--pr-number", str(number)],
            cwd=self.repo,
            capture_output=True,
            text=True,
            env=environment({}),
        )

    def test_two_entries_apply_in_either_order(self) -> None:
        for scenario in SCENARIOS:
            base, a, b = self._scenario(scenario)
            for order, (tip, onto) in {"b onto a": (b, a), "a onto b": (a, b)}.items():
                with self.subTest(scenario=scenario, order=order):
                    self._rebase(tip, onto)

    def test_each_pull_request_passes_the_gate_before_and_after_the_other_lands(self) -> None:
        for scenario in SCENARIOS:
            base, a, b = self._scenario(scenario)
            runs = {
                "a against its base": (base, a, 101),
                "b against its base": (base, b, 102),
                "b rebased onto a": (a, self._rebase(b, a), 102),
            }
            for run, (against, head, number) in runs.items():
                with self.subTest(scenario=scenario, run=run):
                    result = self._gate(against, head, number)
                    self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_the_release_collects_both_entries_and_empties_the_directory(self) -> None:
        for scenario in SCENARIOS:
            with self.subTest(scenario=scenario):
                base, a, b = self._scenario(scenario)
                self._git(self.repo, "checkout", "-q", self._rebase(b, a))
                result = subprocess.run(
                    [sys.executable, str(PREPARE), "prepare", VERSION, "--date", "2026-10-02"],
                    cwd=self.repo,
                    capture_output=True,
                    text=True,
                    env=environment({}),
                )
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                changelog = (self.repo / "docs" / "changelog.md").read_text(encoding="utf-8")
                release = re.search(rf"^## \d+\. \[{re.escape(VERSION)}\][^\n]*\n(.*?)(?=^## )", changelog, re.M | re.S)
                self.assertIsNotNone(release, f"no `[{VERSION}]` section in:\n{changelog[:2000]}")
                landed = ["already-merged"] * (scenario == "seeded") + [f"{scenario}-a", f"{scenario}-b"]
                for name in landed:
                    self.assertIn(entry_text(name).strip(), release.group(1))
                self.assertEqual(["README.md"], sorted(path.name for path in (self.repo / ENTRIES).iterdir()))


if __name__ == "__main__":
    unittest.main()
