"""The fixture the changelog release's tests share: a child environment and a
throwaway repository's git, for the release that lists the pull requests merged
since the previous tag (§FS-distribution.4.6) and rotates the changelog
(§FS-distribution.4.5).

It is not named `test_*`, so `unittest discover` imports it only from the modules
that use it and never collects it as a module of its own."""

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def environment(extra: dict[str, str]) -> dict[str, str]:
    """A child environment with no ambient CI or pre-commit state: the suite runs
    inside CI itself, where a leaked `GITHUB_*` variable would reach the script
    under test."""
    child = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("GITHUB_") and not key.startswith("PRE_COMMIT_")
    }
    child.update(extra)
    return child


class GitFixture:
    """Git in a throwaway repository, isolated from the user's configuration. No
    network, no `gh`, no remote. Mixed into a `unittest.TestCase`."""

    def _init_repository(self, repo: Path) -> None:
        """Create a main-branch repository with automatic maintenance disabled.

        §AR-ci.10.4: local policy also covers direct commits and rebases, keeping
        detached repository writers from crossing strict fixture cleanup.
        """
        self._git(repo.parent, "init", "-b", "main", str(repo))
        self._git(repo, "config", "--local", "gc.auto", "0")
        self._git(repo, "config", "--local", "maintenance.auto", "false")

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
