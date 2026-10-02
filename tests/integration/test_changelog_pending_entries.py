"""§FS-distribution.4.6, §AR-ci.7 — the tree's own pending entries
(§FS-distribution.4.12) are read the way the release reads them, so a write-up
that is not well formed is refused on its own pull request rather than by the
release run that would have published it.

A guard rather than a pin: it passes on any tree whose entries the release could
publish, and fails on the change that writes one it could not. It asks nothing
of a change that writes no entry."""

import contextlib
import importlib.util
import io
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CHANGELOG = REPO_ROOT / "docs" / "changelog.md"
SPEC = importlib.util.spec_from_file_location(
    "prepare_changelog_release", REPO_ROOT / "scripts" / "prepare_changelog_release.py"
)
prepare_changelog_release = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(prepare_changelog_release)
changelog_bullets = prepare_changelog_release.changelog_bullets
ENTRIES = CHANGELOG.parent / changelog_bullets.ENTRY_DIRECTORY


class PendingEntriesTests(unittest.TestCase):
    def test_unreleased_holds_only_its_pointer(self):
        lines = CHANGELOG.read_text(encoding="utf-8").splitlines(keepends=True)
        self.assertEqual([], [bullet.lines[0] for bullet in changelog_bullets.bullets(lines)])

    def test_every_pending_entry_is_one_the_release_can_publish(self):
        names = sorted(path.name for path in ENTRIES.iterdir() if path.name != changelog_bullets.ENTRY_README)
        if not names:
            self.skipTest("nothing is pending")
        with contextlib.redirect_stderr(io.StringIO()):
            try:
                body = prepare_changelog_release.preview_release(CHANGELOG)
            except prepare_changelog_release.ChangelogError as refused:
                self.fail(f"the release would refuse the pending entries: {refused}")
        # One entry is one bullet, so the section the release would write holds one per file.
        self.assertEqual(len(names), sum(line.startswith("- ") for line in body.splitlines()), names)


if __name__ == "__main__":
    unittest.main()
