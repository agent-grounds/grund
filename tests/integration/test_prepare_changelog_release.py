"""§FS-distribution.4.5 — the release step that moves the changelog: the entries
under `docs/changelog/unreleased/` (§FS-distribution.4.12) become the inline
latest release, grouped by category and oldest-landed first, and are deleted;
the previous latest is archived one-per-file; `preview` shows the body without
writing it; and the release notes are read back from the section
(§FS-distribution.4.7).

Each case runs the script the way the release workflow does, in a throwaway
repository whose commits carry fixed dates, because which entry landed first is
a fact of the history and not of the files. `stamp`, the step just before the
rotation, is `test_prepare_changelog_release_stamp.py`."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess

from changelog_gate_fixture import ENTRIES, ENTRY_README, POINTER, REPO_ROOT, GitFixture, environment

SCRIPT_PATH = REPO_ROOT / "scripts" / "prepare_changelog_release.py"
VERSION, DATE = "0.3.0", "2026-10-02"

PREVIOUS_LATEST = """## 2. [0.2.0] — 2026-05-17

Workspace and agent-entrypoint release. The main user-visible change is workspace aliases.

### Added

- [FS-workspace](functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace): validate aliases.

## 3. Older releases

- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release and baseline CLI surface.
"""
BASE = f"# Changelog\n\nIntro.\n\n## Unreleased\n\n{POINTER}\n\n{PREVIOUS_LATEST}"

# Landed in this order, one commit per row; a row of two landed in one commit.
# `zz-kept-landing` is filed as a fix and later moved to `added`, edited in the
# same commit: it keeps the place its first landing gave it.
LANDINGS = (
    {"zz-oldest.added.md": "- The oldest addition.\n"},
    {"parser-crash.fixed.md": "- A parser crash.\n"},
    {"zz-kept-landing.fixed.md": "- Kept its landing, filed as a fix.\n"},
    {"mm-middle.added.md": "- The middle addition.\n", "old-flag.removed.md": "- An old flag is gone.\n"},
    {"wording.changed.md": "- A wording change\n  that wraps onto a second line.\n"},
    {"ab-newest-too.added.md": "- The newest addition's twin, landed in the same commit.\n", "aa-newest.added.md": "- The newest addition.\n"},
    {"zz-kept-landing.fixed.md": None, "zz-kept-landing.added.md": "- Kept its landing: first filed as a fix, now an addition.\n"},
)
BODY = """### Added

- The oldest addition.
- Kept its landing: first filed as a fix, now an addition.
- The middle addition.
- The newest addition.
- The newest addition's twin, landed in the same commit.

### Changed

- A wording change
  that wraps onto a second line.

### Removed

- An old flag is gone.

### Fixed

- A parser crash.
"""
RELEASED = f"""# Changelog

Intro.

## Unreleased

{POINTER}

## 2. [{VERSION}] — {DATE}

{BODY}
## 3. Older releases

- [0.2.0](changelog/0.2.0.md) — 2026-05-17: Workspace and agent-entrypoint release.
- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release and baseline CLI surface.
"""


class ReleaseRepository(GitFixture):
    """A repository whose base commit holds `BASE` and the entry README, and
    whose every later commit lands one day after the one before it."""

    def _repository(self, changelog: str = BASE) -> Path:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.repo = Path(scratch.name) / "repo"
        self._git(Path(scratch.name), "init", "-b", "main", str(self.repo))
        self.day = 0
        self._land({"docs/changelog.md": changelog, f"{ENTRIES}/README.md": ENTRY_README}, "The base")
        return self.repo

    def _land(self, files: dict[str, str | None], message: str = "Land entries", entries: bool = False) -> str:
        """Write each file, or delete it where the text is `None`, and commit the result."""
        for relative, text in files.items():
            path = self.repo / (f"{ENTRIES}/{relative}" if entries else relative)
            if text is None:
                path.unlink()
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(text.encode("utf-8"))
        self.day += 1
        when = f"2026-09-{self.day:02d}T12:00:00+00:00"
        identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false")
        dated = {"GIT_AUTHOR_DATE": when, "GIT_COMMITTER_DATE": when, "GIT_CONFIG_GLOBAL": str(self.repo / ".absent")}
        for arguments in (("add", "-A"), (*identity, "commit", "-q", "-m", message)):
            subprocess.run(["git", "-C", str(self.repo), *arguments], check=True, capture_output=True, env=environment(dated))
        return self._git(self.repo, "rev-parse", "HEAD")

    def _script(self, *arguments: str) -> CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), *arguments],
            cwd=self.repo,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            env=environment({"PYTHONUTF8": "1"}),
        )

    def _read(self, relative: str = "docs/changelog.md") -> str:
        return (self.repo / relative).read_text(encoding="utf-8")

    def _entries(self) -> list[str]:
        return sorted(path.name for path in (self.repo / ENTRIES).iterdir())

    def assertSucceeded(self, result: CompletedProcess) -> None:
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


class RotationTests(ReleaseRepository, unittest.TestCase):
    """`prepare`, `preview` and `notes` over the same landed entries."""

    def _landed(self) -> None:
        self._repository()
        for files in LANDINGS:
            self._land(files, entries=True)

    def test_prepare_writes_the_entries_by_category_oldest_landed_first(self) -> None:
        self._landed()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(RELEASED, self._read())

    def test_prepare_deletes_the_entries_and_keeps_the_readme(self) -> None:
        self._landed()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(["README.md"], self._entries())
        self.assertEqual(ENTRY_README, self._read(f"{ENTRIES}/README.md"))

    def test_preview_prints_the_body_prepare_writes_and_writes_nothing(self) -> None:
        self._landed()
        preview = self._script("preview")
        self.assertSucceeded(preview)
        self.assertEqual("", self._git(self.repo, "status", "--porcelain"))
        self.assertEqual(BODY.strip("\n"), preview.stdout.strip("\n"))

    def test_the_release_notes_are_the_body_prepare_wrote(self) -> None:
        self._landed()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        notes = self.repo.parent / "notes.md"
        self.assertSucceeded(self._script("notes", VERSION, "--output", str(notes)))
        self.assertEqual(BODY.strip("\n"), notes.read_text(encoding="utf-8").strip("\n"))

    def test_links_are_rebased_from_the_entry_directory_to_docs(self) -> None:
        self._repository()
        written = "- [FS-x](../../functional-spec/FS-x.md#a-b), [root](../../../crates/x.rs), [older](../0.15.0.md), [format](README.md), [here](#anchor), [abs](/README.md), [url](https://example.com/a).\n"
        rebased = "- [FS-x](functional-spec/FS-x.md#a-b), [root](../crates/x.rs), [older](changelog/0.15.0.md), [format](changelog/unreleased/README.md), [here](#anchor), [abs](/README.md), [url](https://example.com/a).\n"
        self._land({"links.changed.md": written}, entries=True)
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertIn(f"## 2. [{VERSION}] — {DATE}\n\n### Changed\n\n{rebased}\n## 3. Older releases", self._read())


class RefusalTests(ReleaseRepository, unittest.TestCase):
    """What `prepare` refuses rather than invent or drop, writing nothing."""

    def assertRefusedUntouched(self, result: CompletedProcess, *needles: str) -> None:
        self.assertEqual(1, result.returncode, result.stdout + result.stderr)
        for needle in needles:
            self.assertIn(needle, result.stderr)
        self.assertEqual("", self._git(self.repo, "status", "--porcelain", "--untracked-files=all"))

    def test_an_empty_directory_is_refused(self) -> None:
        self._repository()
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), "docs/changelog/unreleased/")

    def test_a_file_that_is_not_a_well_formed_entry_is_refused(self) -> None:
        for name, text in {"a-note.note.md": "- A bullet.\n", "two-bullets.fixed.md": "- One.\n- Two.\n"}.items():
            with self.subTest(name=name):
                self._repository()
                self._land({"fine.added.md": "- A fine entry.\n", name: text}, entries=True)
                self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), name)

    def test_a_bullet_under_the_pointer_is_refused(self) -> None:
        stray = "- A bullet somebody wrote under the pointer."
        self._repository(BASE.replace(f"{POINTER}\n", f"{POINTER}\n\n### Fixed\n\n{stray}\n"))
        self._land({"fine.added.md": "- A fine entry.\n"}, entries=True)
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), "## Unreleased", stray)


class ArchiveTests(ReleaseRepository, unittest.TestCase):
    """The former latest release, archived one-per-file as before the entries moved."""

    def test_prepare_archives_the_previous_latest_and_links_it(self) -> None:
        self._repository()
        self._land({"fine.added.md": "- A fine entry.\n"}, entries=True)
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
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
        self._repository(BASE.replace(PREVIOUS_LATEST, climbing))
        self._land({"fine.added.md": "- A fine entry.\n"}, entries=True)
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
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
