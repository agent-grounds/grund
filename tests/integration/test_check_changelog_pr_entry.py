"""§AR-ci.7 — the changelog gate, exercised the way its two halves run it: a
branch must add a bullet to the `## Unreleased` section of `docs/changelog.md`
and may not put another pull request's number on it (§FS-distribution.4.6), and
the script reads the number and the base the way the event supplies them.

The end-to-end class at the bottom is the gate's own parity proof, and the only
place §AR-ci.1.1's "the same verdict on the same tree" is actually checked: it
reads the pre-push invocation out of `.pre-commit-config.yaml` and the CI
invocation out of `.github/workflows/ci.yml`, runs both against one fixture
repository, and asserts they agree. It reads the two invocations rather than
spelling them so that it measures the arrangement this repository ships rather
than the one it happened to have when the test was written.

`ChangelogGateRulesTests` above it drives one half at a time over the same
fixture shape, so R1 (a bullet the merge base does not hold) and R2 (a counted
bullet may not carry another pull request's number) each have a case of their
own, including the two conditions that skip the check."""

import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import unittest
from subprocess import CompletedProcess
from unittest.mock import patch
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "check_changelog_pr_entry.py"
PRE_COMMIT_CONFIG = REPO_ROOT / ".pre-commit-config.yaml"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
HOOK_ID = "changelog-pr-entry"
SCRIPT_NAME = SCRIPT_PATH.name
EXPRESSION = re.compile(r"\$\{\{[^}]*\}\}")
HOOK_START = re.compile(r"- id:\s*(\S+)$")

SPEC = importlib.util.spec_from_file_location("check_changelog_pr_entry", SCRIPT_PATH)
check_changelog_pr_entry = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(check_changelog_pr_entry)

FIXTURE_PR = 999
EARLIER_BULLET = "- An earlier pull request's bullet. (PR #1)"
NEW_BULLET = "- A bullet this branch adds, with no number yet."
CHANGELOG = (
    "# Changelog\n\n## Unreleased\n\n### Changed\n\n{bullets}\n\n"
    "## 1. [0.1.0] — 2026-01-01\n\n- The previous release.\n"
)


class ChangelogEventTests(unittest.TestCase):
    """What the `pull_request` event supplies: the number R2 judges against, and
    the base commit R1 takes its merge base from."""

    def write_event(self, payload: dict) -> Path:
        event_path = Path(self.tempdir.name) / "event.json"
        event_path.write_text(json.dumps(payload), encoding="utf-8")
        return event_path

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)

    def test_reads_pull_request_number_from_event_file(self) -> None:
        event_path = self.write_event({"pull_request": {"number": 15, "base": {"sha": "abc123"}}})
        self.assertEqual(check_changelog_pr_entry.pr_number_from_event(event_path), 15)

    def test_reads_base_sha_from_event_file(self) -> None:
        event_path = self.write_event({"pull_request": {"number": 15, "base": {"sha": "abc123"}}})
        self.assertEqual(check_changelog_pr_entry.base_sha_from_event(event_path), "abc123")

    def test_non_pull_request_event_supplies_neither(self) -> None:
        event_path = self.write_event({"ref": "refs/heads/main"})
        self.assertIsNone(check_changelog_pr_entry.pr_number_from_event(event_path))
        self.assertIsNone(check_changelog_pr_entry.base_sha_from_event(event_path))

    def test_rejects_an_invalid_number_in_the_event(self) -> None:
        event_path = self.write_event({"pull_request": {"number": "fifteen"}})
        with self.assertRaises(check_changelog_pr_entry.ChangelogPrError):
            check_changelog_pr_entry.pr_number_from_event(event_path)

    def test_pushed_ref_is_the_head_the_stage_names(self) -> None:
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": "deadbeef"}, clear=False):
            self.assertEqual(check_changelog_pr_entry.head_ref_for_pre_push(), "deadbeef")

    def test_a_branch_deletion_has_no_head_and_nothing_to_require(self) -> None:
        # pre-commit hands the all-zero sha when the push deletes the ref: there
        # is no head to read a changelog from, and nothing is being added.
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": "0" * 40}, clear=False):
            self.assertIsNone(check_changelog_pr_entry.head_ref_for_pre_push())

    def test_an_unset_pushed_ref_falls_back_to_the_checkout(self) -> None:
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": ""}, clear=False):
            self.assertEqual(check_changelog_pr_entry.head_ref_for_pre_push(), "HEAD")


def _hook_entry() -> str:
    """The `entry:` line of the `changelog-pr-entry` hook, read as text — the CI
    Python has no YAML parser, so §AR-ci.1.2's parity test reads these two files
    line-shaped and this one does the same."""
    current = None
    for raw in PRE_COMMIT_CONFIG.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        started = HOOK_START.match(line)
        if started:
            current = started.group(1)
            continue
        if current == HOOK_ID and line.startswith("entry:"):
            return line[len("entry:") :].strip()
    raise AssertionError(f"{PRE_COMMIT_CONFIG} declares no `entry:` for the {HOOK_ID} hook")


def _ci_run() -> str:
    """The workflow's own invocation of the gate, wherever it sits — a step of
    the test matrix or a job of its own."""
    runs = [
        line.strip()[len("run:") :].strip()
        for line in CI_WORKFLOW.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("run:")
    ]
    named = [run for run in runs if SCRIPT_NAME in run]
    if len(named) != 1:
        raise AssertionError(f"{CI_WORKFLOW} has {len(named)} steps running {SCRIPT_NAME}, expected 1")
    return named[0]


def _argv(command: str, expressions: dict[str, str]) -> list[str]:
    def expand(match: re.Match[str]) -> str:
        key = match.group(0)[3:-2].strip()
        if key not in expressions:
            raise AssertionError(f"the workflow reads `{key}`, which this test supplies no value for")
        return expressions[key]

    tokens = shlex.split(EXPRESSION.sub(expand, command))
    return [
        sys.executable if token == "python" else str(SCRIPT_PATH) if token.endswith(SCRIPT_NAME) else token
        for token in tokens
    ]


def _environment(extra: dict[str, str], path_prefix: Path | None = None) -> dict[str, str]:
    """A child environment with no ambient CI or pre-commit state: the suite runs
    inside pull-request CI itself, where a leaked `GITHUB_EVENT_PATH` would hand
    the local half a number no push ever has."""
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("GITHUB_") and not key.startswith("PRE_COMMIT_")
    }
    environment.update(extra)
    if path_prefix is not None:
        environment["PATH"] = f"{path_prefix}{os.pathsep}{environment['PATH']}"
    return environment


class GitFixture:
    """A throwaway repository: a base commit carrying one earlier bullet, one head
    commit on top of it, and `origin/main` pointing at the base. No network, no
    `gh`, no remote — the gate reads local git alone (§AR-ci.7)."""

    def _git(self, repo: Path, *arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(repo), *arguments],
            check=True,
            capture_output=True,
            text=True,
            env=_environment({"GIT_CONFIG_GLOBAL": str(repo / ".gitconfig-absent")}),
        )
        return result.stdout.strip()

    def _commit(self, repo: Path, message: str) -> str:
        self._git(repo, "add", "-A")
        self._git(
            repo,
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            message,
        )
        return self._git(repo, "rev-parse", "HEAD")

    def _fixture(self, root: Path, head_bullets: str, base_bullets: str = EARLIER_BULLET) -> tuple[Path, str, str]:
        """A repository whose base carries one earlier bullet and whose head is
        one commit on top of it, with `origin/main` pointing at the base."""
        repo = root / "repo"
        (repo / "docs").mkdir(parents=True)
        self._git(root, "init", "-b", "main", str(repo))
        changelog = repo / "docs" / "changelog.md"
        changelog.write_text(CHANGELOG.format(bullets=base_bullets), encoding="utf-8")
        (repo / "README.md").write_text("The base.\n", encoding="utf-8")
        base = self._commit(repo, "The base commit")
        self._git(repo, "update-ref", "refs/remotes/origin/main", base)
        changelog.write_text(CHANGELOG.format(bullets=head_bullets), encoding="utf-8")
        (repo / "README.md").write_text("The change.\n", encoding="utf-8")
        head = self._commit(repo, "The change under test")
        return repo, base, head

    def _merge_checkout(self, root: Path, theirs: str, mine: str) -> tuple[Path, str, str]:
        """The tree `actions/checkout` leaves on a `pull_request` event: `refs/pull/N/merge`,
        the head merged with a base tip that moved after the branch point."""
        repo, branch_point, head = self._fixture(root, f"{EARLIER_BULLET}\n{mine}")
        changelog = repo / "docs" / "changelog.md"
        self._git(repo, "checkout", "-q", "-B", "their-main", branch_point)
        changelog.write_text(CHANGELOG.format(bullets=f"{EARLIER_BULLET}\n{theirs}"), encoding="utf-8")
        base = self._commit(repo, "Their change, merged after the branch point")
        self._git(repo, "update-ref", "refs/remotes/origin/main", base)
        self._git(repo, "merge", "--no-commit", "--no-ff", "-s", "ours", head)
        changelog.write_text(CHANGELOG.format(bullets=f"{EARLIER_BULLET}\n{theirs}\n{mine}"), encoding="utf-8")
        self._commit(repo, "Merge the head into the base tip")
        return repo, base, head

    def _no_pull_request_yet(self, root: Path) -> Path:
        """A `gh` that answers the way it answers on a branch with no pull
        request, so the local half meets the condition without the network."""
        binaries = root / "bin"
        binaries.mkdir()
        shell = binaries / "gh"
        shell.write_text('#!/bin/sh\necho \'no pull requests found for branch\' >&2\nexit 1\n', encoding="utf-8")
        shell.chmod(0o755)
        (binaries / "gh.bat").write_text("@echo no pull requests found 1>&2\r\n@exit /b 1\r\n", encoding="utf-8")
        return binaries

class ChangelogGateRulesTests(GitFixture, unittest.TestCase):
    """R1 and R2 of §FS-distribution.4.6, one half at a time: what counts as a new
    or changed bullet, what a number in one may be, and the two skips."""

    def _recording_gh(self, root: Path) -> tuple[Path, Path]:
        """A `gh` that records being run: the gate may not need it on either side."""
        binaries = root / "bin"
        binaries.mkdir()
        marker = root / "gh-was-run"
        (binaries / "gh").write_text(f'#!/bin/sh\ntouch "{marker}"\nexit 1\n', encoding="utf-8")
        (binaries / "gh").chmod(0o755)
        (binaries / "gh.bat").write_text(f'@type nul > "{marker}"\r\n@exit /b 1\r\n', encoding="utf-8")
        return binaries, marker

    def _pre_push(self, head_bullets: str, base_bullets: str = EARLIER_BULLET) -> CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head = self._fixture(root, head_bullets, base_bullets)
            binaries, self.gh_marker = self._recording_gh(root)
            self.gh_ran = lambda: self.gh_marker.exists()
            return subprocess.run(
                [sys.executable, str(SCRIPT_PATH), "--pre-push"],
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment(
                    {
                        "PRE_COMMIT_REMOTE_NAME": "origin",
                        "PRE_COMMIT_LOCAL_BRANCH": "fix/the-change",
                        "PRE_COMMIT_FROM_REF": base,
                        "PRE_COMMIT_TO_REF": head,
                    },
                    path_prefix=binaries,
                ),
            )

    def _ci(self, head_bullets: str, base_bullets: str = EARLIER_BULLET, base: str | None = None) -> CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, fixture_base, head = self._fixture(root, head_bullets, base_bullets)
            return subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT_PATH),
                    "--base-sha",
                    base or fixture_base,
                    "--head-sha",
                    head,
                    "--pr-number",
                    str(FIXTURE_PR),
                ],
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )

    def _output(self, result: CompletedProcess) -> str:
        return result.stdout + result.stderr

    # R1 — a bullet the merge base does not already hold, and no number needed.
    def test_a_new_bullet_needs_no_number_and_no_pull_request(self) -> None:
        result = self._pre_push(f"{EARLIER_BULLET}\n{NEW_BULLET}")
        self.assertEqual(0, result.returncode, self._output(result))
        self.assertFalse(self.gh_ran(), "the gate ran `gh`; both halves read local git alone")

    def test_a_rewrap_is_not_a_change(self) -> None:
        rewrapped = "- An earlier pull request's\n  bullet. (PR #1)"
        result = self._pre_push(rewrapped)
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("SKIP=changelog-pr-entry git push", self._output(result))

    def test_a_formatter_anchor_rewrite_is_neither_a_change_nor_an_r2_trip(self) -> None:
        # `grund fmt --write` moves the anchor inside somebody else's bullet when
        # a heading is renamed. Reading that as this branch's edit would refuse a
        # push for a change the formatter made — and its `PR #305` is not ours.
        theirs = "- [§AR-ci.7](architecture/AR-ci.md#7-{anchor}): their change. (PR #305)"
        result = self._ci(theirs.format(anchor="the-new-title"), base_bullets=theirs.format(anchor="old-title"))
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("no new or changed bullet", self._output(result))
        self.assertNotIn("305", self._output(result))

    def test_a_bullet_the_base_gained_after_the_branch_point_is_not_this_branchs(self) -> None:
        # Driven as `ci.yml` drives it, on the merge tree the event leaves. Read
        # the changelog from the checkout rather than from the head commit and
        # every bullet the base gained since the branch point counts as this
        # branch's, so R2 refuses on a number nobody here could have written.
        theirs = "- Somebody else's freshly merged bullet. (PR #340)"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head = self._merge_checkout(root, theirs, NEW_BULLET)
            result = subprocess.run(
                _argv(
                    _ci_run(),
                    {
                        "github.event.pull_request.number": str(FIXTURE_PR),
                        "github.event.pull_request.base.sha": base,
                        "github.event.pull_request.head.sha": head,
                        "github.event.pull_request.base.ref": "main",
                    },
                ),
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )
        self.assertEqual(0, result.returncode, self._output(result))
        self.assertNotIn("340", self._output(result))

    def test_an_absolute_changelog_path_reaches_the_same_verdict_as_a_relative_one(self) -> None:
        # `git show <commit>:/abs/path` is refused, and reading that refusal as
        # "the file was not there" made every bullet count as added.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head = self._fixture(root, EARLIER_BULLET)
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT_PATH),
                    "--base-sha",
                    base,
                    "--head-sha",
                    head,
                    "--pr-number",
                    str(FIXTURE_PR),
                    "--changelog",
                    str(repo / "docs" / "changelog.md"),
                ],
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("no new or changed bullet", self._output(result))
        self.assertNotIn("PR #1", self._output(result))

    def test_an_uncommitted_bullet_is_not_a_pushed_one(self) -> None:
        # A push carries commits: the head side is read from the head commit, so
        # an edit still in the working tree is not what this branch adds.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head = self._fixture(root, EARLIER_BULLET)
            (repo / "docs" / "changelog.md").write_text(
                CHANGELOG.format(bullets=f"{EARLIER_BULLET}\n{NEW_BULLET}"), encoding="utf-8"
            )
            result = subprocess.run(
                [sys.executable, str(SCRIPT_PATH), "--base-sha", base, "--head-sha", head],
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("no new or changed bullet", self._output(result))

    def test_appending_your_number_to_somebody_elses_bullet_is_not_a_change(self) -> None:
        result = self._ci(f"{EARLIER_BULLET} (PR #{FIXTURE_PR})")
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("no new or changed bullet", self._output(result))

    # R2 — a counted bullet may not carry some other pull request's number.
    def test_a_counted_bullet_carrying_another_pull_requests_number_is_refused(self) -> None:
        result = self._ci(f"{EARLIER_BULLET}\n{NEW_BULLET[:-1]} (PR #305)")
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertIn("PR #305", self._output(result))
        self.assertIn(f"PR #{FIXTURE_PR}", self._output(result))

    def test_a_tbd_placeholder_is_not_a_number(self) -> None:
        result = self._ci(f"{EARLIER_BULLET}\n{NEW_BULLET[:-1]} (PR #TBD)")
        self.assertEqual(0, result.returncode, self._output(result))

    # The two skips, both by condition.
    def test_a_head_the_base_already_contains_skips(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, _head = self._fixture(root, EARLIER_BULLET)
            result = subprocess.run(
                [sys.executable, str(SCRIPT_PATH), "--base-sha", base, "--head-sha", base],
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )
        self.assertEqual(0, result.returncode, self._output(result))
        self.assertIn("already contains this head", self._output(result))

    def test_an_unresolvable_base_degrades_to_one_bullet_and_warns(self) -> None:
        result = self._ci(EARLIER_BULLET, base="0" * 40)
        self.assertEqual(0, result.returncode, self._output(result))
        self.assertIn("no base ref resolves", self._output(result))

    def test_an_unresolvable_base_still_refuses_an_empty_unreleased(self) -> None:
        result = self._ci("*Nothing yet.*", base="0" * 40)
        self.assertEqual(1, result.returncode, self._output(result))


class ChangelogGateParityTests(GitFixture, unittest.TestCase):
    """Both halves of one gate, on one tree, from the repository's own configuration."""

    def _both_halves(self, head_bullets: str) -> tuple[CompletedProcess, CompletedProcess]:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head = self._fixture(root, head_bullets)
            local = subprocess.run(
                _argv(_hook_entry(), {}),
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment(
                    {
                        "PRE_COMMIT_REMOTE_NAME": "origin",
                        "PRE_COMMIT_REMOTE_BRANCH": "refs/heads/main",
                        "PRE_COMMIT_LOCAL_BRANCH": "fix/the-change",
                        "PRE_COMMIT_FROM_REF": base,
                        "PRE_COMMIT_TO_REF": head,
                    },
                    path_prefix=self._no_pull_request_yet(root),
                ),
            )
            remote = subprocess.run(
                _argv(
                    _ci_run(),
                    {
                        "github.event.pull_request.number": str(FIXTURE_PR),
                        "github.event.pull_request.base.sha": base,
                        "github.event.pull_request.head.sha": head,
                        "github.event.pull_request.base.ref": "main",
                    },
                ),
                cwd=repo,
                capture_output=True,
                text=True,
                env=_environment({}),
            )
            return local, remote

    def _report(self, half: str, result: CompletedProcess) -> str:
        return f"{half}: exit {result.returncode}\n{(result.stdout + result.stderr).strip()}"

    def _verdicts(self, local: CompletedProcess, remote: CompletedProcess) -> None:
        self.assertEqual(
            local.returncode,
            remote.returncode,
            "the two halves of one gate disagree on one tree\n"
            f"{self._report('pre-push', local)}\n{self._report('pull-request CI', remote)}",
        )

    def test_both_halves_refuse_a_branch_that_added_no_bullet(self) -> None:
        local, remote = self._both_halves(EARLIER_BULLET)
        self._verdicts(local, remote)
        self.assertEqual(1, local.returncode, self._report("pre-push", local))
        for half, result in (("pre-push", local), ("pull-request CI", remote)):
            with self.subTest(half=half):
                refusal = result.stdout + result.stderr
                self.assertIn("docs/changelog.md", refusal)
                self.assertIn("## Unreleased", refusal)
        self.assertIn("SKIP=changelog-pr-entry", local.stdout + local.stderr)

    def test_both_halves_accept_a_new_bullet_that_names_no_number(self) -> None:
        local, remote = self._both_halves(f"{EARLIER_BULLET}\n{NEW_BULLET}")
        self._verdicts(local, remote)
        self.assertEqual(0, local.returncode, self._report("pre-push", local))


if __name__ == "__main__":
    unittest.main()
