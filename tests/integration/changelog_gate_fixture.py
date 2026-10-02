"""The fixture the changelog release's tests share: the entry directory
(§FS-distribution.4.12) and the pointer `## Unreleased` keeps, a child
environment, and a throwaway repository's git, for the release that numbers,
collects and deletes the entries (§FS-distribution.4.5).

It is not named `test_*`, so `unittest discover` imports it only from the modules
that use it and never collects it as a module of its own."""

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

ENTRIES = "docs/changelog/unreleased"
POINTER = "Pending changes are one file each under [changelog/unreleased/](changelog/unreleased/README.md)."
ENTRY_README = "# Unreleased changelog entries\n\nThe format, which is not an entry.\n"


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
