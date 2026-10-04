"""§FS-distribution.4.5 — the release step that moves the changelog archives the
former inline latest release one-per-file, with its links rewritten for the
directory it moves into.

Each case runs the script the way the release workflow does, in a throwaway
repository tagged at the previous release, against the stub forge of
`release_forge_fixture.py`, with one merged pull request to release. The section
the release writes is `test_prepare_changelog_release_list.py`, and its notices
`test_prepare_changelog_release_notices.py`."""

import unittest

from release_forge_fixture import ForgeRepository

VERSION, DATE = "0.3.0", "2026-10-02"

PREVIOUS_LATEST = """## 2. [0.2.0] — 2026-05-17

Workspace and agent-entrypoint release. The main user-visible change is workspace aliases.

### Added

- [FS-workspace](functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace): validate aliases.

## 3. Older releases

- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release and baseline CLI surface.
"""
BASE = f"# Changelog\n\nIntro.\n\n{PREVIOUS_LATEST}"


class ArchiveTests(ForgeRepository, unittest.TestCase):
    """The former latest release, archived one-per-file as it always was."""

    def _released(self, changelog: str = BASE) -> None:
        self._repository({"docs/changelog.md": changelog})
        self._merged(10, "A fine change", ("A fine change", {"src/fine.rs": "fn fine() {}\n"}))
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))

    def test_prepare_archives_the_previous_latest_and_links_it(self) -> None:
        self._released()
        self.assertEqual(
            """# 0.2.0 — 2026-05-17

Workspace and agent-entrypoint release. The main user-visible change is workspace aliases.

### Added

- [FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace): validate aliases.

""",
            self._read("docs/changelog/0.2.0.md"),
        )

    def test_archived_links_that_climb_above_docs_gain_another_level(self) -> None:
        """A destination is rewritten by how far the file moved, not by its shape.

        `docs/changelog.md` sits in `docs/`, so a link to a repository-root path is
        written `../crates/...`. Archiving moves the body one directory deeper, to
        `docs/changelog/<version>.md`, where that same destination has to climb twice.
        Leaving an already-climbing link alone is how v0.10.0 shipped a link resolving
        to `docs/crates/...` and turned the tree's own link check red.
        """
        climbing = (
            "## 2. [0.2.0] — 2026-05-17\n\nPrevious release.\n\n### Added\n\n"
            "- [AR-checker.2.12](../crates/grund-core/src/checker.rs): an inline declaration.\n"
            "- [FS-workspace](functional-spec/FS-workspace.md#fs-workspace): a sibling under docs.\n"
            "- [an anchor](#goal-x) and [an absolute one](/README.md) and [a url](https://example.com/a).\n\n"
            "## 3. Older releases\n"
        )
        self._released(BASE.replace(PREVIOUS_LATEST, climbing))
        archived = self._read("docs/changelog/0.2.0.md")
        # Climbed once against `docs/`, so it climbs twice from `docs/changelog/`.
        self.assertIn("](../../crates/grund-core/src/checker.rs)", archived)
        # A sibling under `docs/` gains exactly one level.
        self.assertIn("](../functional-spec/FS-workspace.md#fs-workspace)", archived)
        # An anchor, an absolute path, and a URL mean the same thing at any depth.
        self.assertIn("](#goal-x)", archived)
        self.assertIn("](/README.md)", archived)
        self.assertIn("](https://example.com/a)", archived)


if __name__ == "__main__":
    unittest.main()
