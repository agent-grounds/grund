"""§AR-ci.7 — the pull-request changelog gate, exercised the way CI runs it: a
pull request must be named in the `## Unreleased` section of `docs/changelog.md`
(§FS-distribution.4.6), and the script reads the PR number the way the event
supplies it.

The end-to-end class at the bottom is the gate's own parity proof, and the only
place §AR-ci.1.1's "the same verdict on the same tree" is actually checked: it
reads the pre-push invocation out of `.pre-commit-config.yaml` and the CI
invocation out of `.github/workflows/ci.yml`, runs both against one fixture
repository, and asserts they agree. It reads the two invocations rather than
spelling them so that it measures the arrangement this repository ships rather
than the one it happened to have when the test was written.

Its two cases land under `@unittest.expectedFailure`, one commit before the fix,
because the pre-commit hook runs this suite and a plainly failing test could not
be committed without `--no-verify`, which this repository forbids. They fail
today for the reason agent-grounds/grund#345 reports: on one tree the pre-push
invocation exits 0 and pull-request CI exits 1. The decorator comes off in the
same change that makes them pass, and until then an unexpected success fails the
suite, so neither case can rot into a silent pass."""

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


class CheckChangelogPrEntryTests(unittest.TestCase):
    def write_changelog(self, root: Path, unreleased: str) -> Path:
        changelog = root / "docs" / "changelog.md"
        changelog.parent.mkdir(parents=True)
        changelog.write_text(
            f"# Changelog\n\n## Unreleased\n\n{unreleased}\n\n"
            "## 2. [0.3.0] — 2026-05-18\n\nPrevious release.\n",
            encoding="utf-8",
        )
        return changelog

    def test_accepts_pr_number_in_unreleased_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            changelog = self.write_changelog(
                Path(tmp),
                "### Changed\n\n- §FS-distribution.4.6: add the changelog PR gate. PR #15",
            )
            check_changelog_pr_entry.check_changelog_pr_entry(changelog, 15)

    def test_accepts_pull_url_in_unreleased_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            changelog = self.write_changelog(
                Path(tmp),
                "### Fixed\n\n- §FS-distribution.4.6: fix release notes (https://github.com/agent-grounds/grund/pull/15).",
            )
            check_changelog_pr_entry.check_changelog_pr_entry(changelog, 15)

    def test_rejects_missing_pr_number(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            changelog = self.write_changelog(
                Path(tmp),
                "### Changed\n\n- §FS-distribution.4.6: add the changelog PR gate.",
            )
            with self.assertRaisesRegex(check_changelog_pr_entry.ChangelogPrError, "PR #15"):
                check_changelog_pr_entry.check_changelog_pr_entry(changelog, 15)

    def test_reads_pull_request_number_from_event_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            event_path = root / "event.json"
            event_path.write_text(json.dumps({"pull_request": {"number": 15}}), encoding="utf-8")
            self.assertEqual(check_changelog_pr_entry.pr_number_from_event(event_path), 15)

    def test_non_pull_request_event_skips(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            event_path = root / "event.json"
            event_path.write_text(json.dumps({"ref": "refs/heads/main"}), encoding="utf-8")
            self.assertIsNone(check_changelog_pr_entry.pr_number_from_event(event_path))

    def test_reads_pull_request_number_from_gh_current_branch(self) -> None:
        with patch.object(
            check_changelog_pr_entry.subprocess,
            "run",
            return_value=CompletedProcess(args=[], returncode=0, stdout="18\n", stderr=""),
        ):
            self.assertEqual(check_changelog_pr_entry.pr_number_from_current_branch(), 18)

    def test_missing_gh_current_branch_pr_skips(self) -> None:
        with patch.object(
            check_changelog_pr_entry.subprocess,
            "run",
            return_value=CompletedProcess(args=[], returncode=1, stdout="", stderr="no pull requests found"),
        ):
            self.assertIsNone(check_changelog_pr_entry.pr_number_from_current_branch())


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


class ChangelogGateParityTests(unittest.TestCase):
    """Both halves of one gate, on one tree, from the repository's own configuration."""

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

    def _fixture(self, root: Path, head_bullets: str) -> tuple[Path, str, str]:
        """A repository whose base carries one earlier bullet and whose head is
        one commit on top of it, with `origin/main` pointing at the base."""
        repo = root / "repo"
        (repo / "docs").mkdir(parents=True)
        self._git(root, "init", "-b", "main", str(repo))
        changelog = repo / "docs" / "changelog.md"
        changelog.write_text(CHANGELOG.format(bullets=EARLIER_BULLET), encoding="utf-8")
        (repo / "README.md").write_text("The base.\n", encoding="utf-8")
        base = self._commit(repo, "The base commit")
        self._git(repo, "update-ref", "refs/remotes/origin/main", base)
        changelog.write_text(CHANGELOG.format(bullets=head_bullets), encoding="utf-8")
        (repo / "README.md").write_text("The change.\n", encoding="utf-8")
        head = self._commit(repo, "The change under test")
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

    @unittest.expectedFailure
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

    @unittest.expectedFailure
    def test_both_halves_accept_a_new_bullet_that_names_no_number(self) -> None:
        local, remote = self._both_halves(f"{EARLIER_BULLET}\n{NEW_BULLET}")
        self._verdicts(local, remote)
        self.assertEqual(0, local.returncode, self._report("pre-push", local))


if __name__ == "__main__":
    unittest.main()
