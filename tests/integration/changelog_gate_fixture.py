"""The fixture the changelog gate's tests share (§AR-ci.7): a throwaway repository
whose base and head commits each hold a whole entry directory
(§FS-distribution.4.12), and the two ways the gate is run against it.

It is not named `test_*`, so `unittest discover` imports it only from the modules
that use it and never collects it as a module of its own."""

import os
import subprocess
import sys
import tempfile
from pathlib import Path
from subprocess import CompletedProcess

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "check_changelog_pr_entry.py"

FIXTURE_PR = 999
BRANCH = "fix/the-change"
ENTRIES = "docs/changelog/unreleased"
REFUSED = "gains no entry"
POINTER = "Pending changes are one file each under [changelog/unreleased/](changelog/unreleased/README.md)."
CHANGELOG = "# Changelog\n\n## Unreleased\n\n{unreleased}\n\n## 1. [0.1.0] — 2026-01-01\n\n- The previous release.\n"
ENTRY_README = "# Unreleased changelog entries\n\nThe format, which is not an entry.\n"
EARLIER = {"earlier-change.fixed.md": "- An earlier pull request's entry. (PR #1)\n"}
NEW = {"the-change.added.md": "- An entry this branch adds, with no number yet.\n"}


def environment(extra: dict[str, str], path_prefix: Path | None = None) -> dict[str, str]:
    """A child environment with no ambient CI or pre-commit state: the suite runs
    inside pull-request CI itself, where a leaked `GITHUB_EVENT_PATH` would hand
    the local half a number no push ever has."""
    child = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("GITHUB_") and not key.startswith("PRE_COMMIT_")
    }
    child.update(extra)
    if path_prefix is not None:
        child["PATH"] = f"{path_prefix}{os.pathsep}{child['PATH']}"
    return child


def pre_push_environment(base: str, head: str, branch: str, binaries: Path | None) -> dict[str, str]:
    """What `pre-commit` hands the `pre-push` stage: the local ref is a full ref name."""
    return environment(
        {
            "PRE_COMMIT_REMOTE_NAME": "origin",
            "PRE_COMMIT_REMOTE_BRANCH": "refs/heads/main",
            "PRE_COMMIT_LOCAL_BRANCH": branch,
            "PRE_COMMIT_FROM_REF": base,
            "PRE_COMMIT_TO_REF": head,
        },
        path_prefix=binaries,
    )


class GitFixture:
    """A throwaway repository: a base commit, one head commit on top of it, and
    `origin/main` pointing at the base. Each side is the whole of the entry
    directory as that commit holds it. No network, no `gh`, no remote — the gate
    reads local git alone (§AR-ci.7). Mixed into a `unittest.TestCase`."""

    def _git(self, repo: Path, *arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(repo), *arguments],
            check=True,
            capture_output=True,
            text=True,
            env=environment({"GIT_CONFIG_GLOBAL": str(repo / ".gitconfig-absent")}),
        )
        return result.stdout.strip()

    def _commit(self, repo: Path, message: str) -> str:
        self._git(repo, "add", "-A")
        identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid")
        self._git(repo, *identity, "-c", "commit.gpgsign=false", "commit", "-m", message)
        return self._git(repo, "rev-parse", "HEAD")

    def _write(self, repo: Path, entries: dict[str, str], unreleased: str) -> None:
        """Lay `docs/changelog.md` and the entry directory down as given, removing
        any entry `entries` does not name. The README is there unless replaced."""
        (repo / "docs").mkdir(parents=True, exist_ok=True)
        (repo / "docs" / "changelog.md").write_bytes(CHANGELOG.format(unreleased=unreleased).encode("utf-8"))
        directory = repo / ENTRIES
        directory.mkdir(parents=True, exist_ok=True)
        for present in directory.iterdir():
            if present.name not in entries and present.name != "README.md":
                present.unlink()
        for name, text in {"README.md": ENTRY_README, **entries}.items():
            (directory / name).write_bytes(text.encode("utf-8"))

    def _fixture(
        self,
        root: Path,
        head: dict[str, str],
        base: dict[str, str] = EARLIER,
        base_unreleased: str = POINTER,
        head_unreleased: str = POINTER,
    ) -> tuple[Path, str, str]:
        repo = root / "repo"
        self._git(root, "init", "-b", "main", str(repo))
        self._write(repo, base, base_unreleased)
        (repo / "README.md").write_text("The base.\n", encoding="utf-8")
        base_sha = self._commit(repo, "The base commit")
        self._git(repo, "update-ref", "refs/remotes/origin/main", base_sha)
        self._write(repo, head, head_unreleased)
        (repo / "README.md").write_text("The change.\n", encoding="utf-8")
        return repo, base_sha, self._commit(repo, "The change under test")

    def _recording_gh(self, root: Path) -> tuple[Path, Path]:
        """A `gh` that records being run: the gate may not need it on either side."""
        binaries = root / "bin"
        binaries.mkdir()
        marker = root / "gh-was-run"
        (binaries / "gh").write_text(f'#!/bin/sh\ntouch "{marker}"\nexit 1\n', encoding="utf-8")
        (binaries / "gh").chmod(0o755)
        (binaries / "gh.bat").write_text(f'@type nul > "{marker}"\r\n@exit /b 1\r\n', encoding="utf-8")
        return binaries, marker

    def _run_pre_push(self, repo: Path, base: str, head: str, branch: str, binaries: Path | None = None) -> CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--pre-push"],
            cwd=repo,
            capture_output=True,
            text=True,
            env=pre_push_environment(base, head, branch, binaries),
        )

    def _run_ci(self, repo: Path, base: str, head: str, *extra: str) -> CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--base-sha", base, "--head-sha", head, "--pr-number", str(FIXTURE_PR), *extra],
            cwd=repo,
            capture_output=True,
            text=True,
            env=environment({}),
        )

    def _pre_push(self, head: dict[str, str], base: dict[str, str] = EARLIER, branch: str = f"refs/heads/{BRANCH}") -> CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base_sha, head_sha = self._fixture(root, head, base)
            binaries, marker = self._recording_gh(root)
            result = self._run_pre_push(repo, base_sha, head_sha, branch, binaries)
            self.gh_was_run = marker.exists()
            return result

    def _ci(self, head: dict[str, str], base: dict[str, str] = EARLIER, base_sha: str | None = None, **unreleased: str) -> CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            repo, fixture_base, head_sha = self._fixture(Path(tmp), head, base, **unreleased)
            return self._run_ci(repo, base_sha or fixture_base, head_sha)

    def _output(self, result: CompletedProcess) -> str:
        return result.stdout + result.stderr

    def assertPassed(self, result: CompletedProcess) -> None:
        self.assertEqual(0, result.returncode, self._output(result))

    def assertRefused(self, result: CompletedProcess, *needles: str) -> None:
        self.assertEqual(1, result.returncode, self._output(result))
        for needle in needles:
            self.assertIn(needle, self._output(result))
