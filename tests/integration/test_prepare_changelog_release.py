"""§FS-distribution.4 — the release step that moves the changelog: the
`## Unreleased` section becomes the inline latest release, the previous latest is
archived one-per-file, and the release notes are read back from it.

`StampTests` covers the step immediately before that rotation
(§FS-distribution.4.5): every `## Unreleased` bullet is blamed to the commits
that wrote it and each of those to its pull request, so the number the
contributor could not have known at push time is written at release time. The
blame half runs against a real throwaway repository rather than a stand-in,
because the line ranges are the thing under test; the commit-to-pull-request
half is an injected seam, so no case here reaches the network."""

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import patch


SCRIPT_PATH = Path(__file__).resolve().parents[2] / "scripts" / "prepare_changelog_release.py"
SPEC = importlib.util.spec_from_file_location("prepare_changelog_release", SCRIPT_PATH)
prepare_changelog_release = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(prepare_changelog_release)


SAMPLE_CHANGELOG = """# Changelog

Intro.

## Unreleased

### Fixed

- [§FS-distribution.4](functional-spec/FS-distribution.md#4-release-process): rotate release notes automatically.

## 2. [0.2.0] — 2026-05-17

Workspace and agent-entrypoint release. The main user-visible change is workspace aliases.

### Added

- [§FS-workspace](functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace): validate aliases.

## 3. Older releases

- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release and baseline CLI surface.
"""


class PrepareChangelogReleaseTests(unittest.TestCase):
    def write_changelog(self, text: str = SAMPLE_CHANGELOG) -> Path:
        root = Path(self.tempdir.name)
        changelog = root / "docs" / "changelog.md"
        changelog.parent.mkdir(parents=True)
        changelog.write_text(text, encoding="utf-8")
        return changelog

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    # §FS-distribution.4.5 — rotation moves curated `## Unreleased` bullets into
    # the inline release section, archives the former latest under
    # docs/changelog/<version>.md, and adds its link to the older-release index.
    def test_prepare_promotes_unreleased_and_archives_previous_latest(self) -> None:
        changelog = self.write_changelog()

        prepare_changelog_release.prepare_release(changelog, "0.2.1", "2026-05-18")

        updated = changelog.read_text(encoding="utf-8")
        self.assertIn("## Unreleased\n\n## 2. [0.2.1] — 2026-05-18", updated)
        self.assertIn("rotate release notes automatically.", updated)
        self.assertIn(
            "- [0.2.0](changelog/0.2.0.md) — 2026-05-17: Workspace and agent-entrypoint release.",
            updated,
        )
        self.assertIn(
            "- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release and baseline CLI surface.",
            updated,
        )

        archived = changelog.parent / "changelog" / "0.2.0.md"
        self.assertEqual(
            archived.read_text(encoding="utf-8"),
            """# 0.2.0 — 2026-05-17

Workspace and agent-entrypoint release. The main user-visible change is workspace aliases.

### Added

- [§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace): validate aliases.

""",
        )

    def test_archived_links_that_climb_above_docs_gain_another_level(self) -> None:
        """A destination is rewritten by how far the file moved, not by its shape.

        `docs/changelog.md` sits in `docs/`, so a link to a repository-root path is
        written `../crates/...`. Archiving moves the body one directory deeper, to
        `docs/changelog/<version>.md`, where that same destination has to climb twice.
        Leaving an already-climbing link alone is how v0.10.0 shipped a link resolving
        to `docs/crates/...` and turned the tree's own link check red.
        """
        changelog = self.write_changelog(
            """# Changelog

## Unreleased

### Added

- New thing.

## 2. [0.2.0] — 2026-05-17

Previous release.

### Added

- [§AR-checker.2.12](../crates/grund-core/src/checker.rs): an inline declaration.
- [§FS-workspace](functional-spec/FS-workspace.md#fs-workspace): a sibling under docs.
- [an anchor](#goal-x) and [an absolute one](/README.md) and [a url](https://example.com/a).

## 3. Older releases
"""
        )

        prepare_changelog_release.prepare_release(changelog, "0.3.0", "2026-05-18")
        archived = (changelog.parent / "changelog" / "0.2.0.md").read_text(encoding="utf-8")

        # Climbed once against `docs/`, so it climbs twice from `docs/changelog/`.
        self.assertIn("](../../crates/grund-core/src/checker.rs)", archived)
        # A sibling under `docs/` gains exactly one level.
        self.assertIn("](../functional-spec/FS-workspace.md#fs-workspace)", archived)
        # An anchor, an absolute path, and a URL mean the same thing at any depth.
        self.assertIn("](#goal-x)", archived)
        self.assertIn("](/README.md)", archived)
        self.assertIn("](https://example.com/a)", archived)

    def test_prepare_fails_when_unreleased_has_no_bullets(self) -> None:
        changelog = self.write_changelog(
            """# Changelog

## Unreleased

## 2. [0.2.0] — 2026-05-17

Previous release.

## 3. Older releases
"""
        )

        with self.assertRaisesRegex(prepare_changelog_release.ChangelogError, "no bullet entries"):
            prepare_changelog_release.prepare_release(changelog, "0.2.1", "2026-05-18")

    # §FS-distribution.4.7 — the changelog is the source of the GitHub release
    # notes: the requested vX.Y.Z section is extracted from docs/changelog.md and
    # handed over as the release body; the older-release index is not part of it.
    def test_extract_notes_writes_inline_release_body(self) -> None:
        changelog = self.write_changelog()
        output = changelog.parent / "release-notes.md"

        prepare_changelog_release.extract_notes(changelog, "0.2.0", output)

        notes = output.read_text(encoding="utf-8")
        self.assertIn("Workspace and agent-entrypoint release.", notes)
        self.assertIn("### Added", notes)
        self.assertNotIn("Older releases", notes)


if __name__ == "__main__":
    unittest.main()


STAMPABLE = """# Changelog

## Unreleased

### Changed

- A bullet whose author left a placeholder. (PR #TBD)
- A bullet whose author left nothing.

## 2. [0.2.0] — 2026-05-17

### Added

- A released bullet, already stamped. (PR #4)

## 3. Older releases

- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release.
"""


class StampTests(unittest.TestCase):
    """§FS-distribution.4.5 — `stamp` writes the numbers it can resolve, warns
    about the rest once each, and never fails the release."""

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.root = Path(self.tempdir.name)

    def changelog(self, text: str = STAMPABLE) -> Path:
        path = self.root / "docs" / "changelog.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def stamp(self, changelog: Path, commits, resolve) -> str:
        prepare_changelog_release.stamp_release_numbers(
            changelog, resolve=resolve, blame=lambda _path, start, end: commits(start, end)
        )
        return changelog.read_text(encoding="utf-8")

    def test_a_placeholder_is_replaced_in_place_and_a_bare_bullet_gains_the_number(self) -> None:
        changelog = self.changelog()
        stamped = self.stamp(changelog, lambda start, end: ["a" * 40], lambda _commit: {12})
        self.assertIn("- A bullet whose author left a placeholder. (PR #12)", stamped)
        self.assertIn("- A bullet whose author left nothing. (PR #12)", stamped)
        self.assertNotIn("PR #TBD", stamped)

    def test_a_placeholder_on_a_bullets_first_line_is_replaced_there(self) -> None:
        # An author who wraps a bullet leaves `PR #TBD` on its first line as often
        # as on its last; writing only into the last line appends the number to a
        # continuation and ships the placeholder into the archive.
        changelog = self.changelog(
            STAMPABLE.replace(
                "- A bullet whose author left a placeholder. (PR #TBD)",
                "- A bullet whose author left a placeholder. (PR #TBD)\n  and a continuation line saying more.",
            )
        )
        stamped = self.stamp(changelog, lambda start, end: ["a" * 40], lambda _commit: {412})
        self.assertIn("- A bullet whose author left a placeholder. (PR #412)\n", stamped)
        self.assertIn("  and a continuation line saying more.\n", stamped)
        self.assertNotIn("PR #TBD", stamped)

    def test_a_run_that_stamps_nothing_leaves_the_file_byte_identical(self) -> None:
        # `stamp` runs on every release whether or not it resolves anything, and a
        # release in which nothing resolves is exactly today's release.
        path = self.root / "docs" / "changelog.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        before = STAMPABLE.replace("\n", "\r\n").encode("utf-8")
        path.write_bytes(before)
        with patch("sys.stderr"):
            self.stamp(path, lambda start, end: ["a" * 40, "b" * 40], lambda commit: {12} if commit[0] == "a" else {13})
        self.assertEqual(before, path.read_bytes())

    def test_a_released_section_is_never_touched(self) -> None:
        changelog = self.changelog()
        stamped = self.stamp(changelog, lambda start, end: ["a" * 40], lambda _commit: {12})
        self.assertIn("- A released bullet, already stamped. (PR #4)", stamped)
        self.assertIn("- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release.", stamped)

    def test_two_commits_that_resolve_to_one_pull_request_stamp_it(self) -> None:
        changelog = self.changelog()
        stamped = self.stamp(changelog, lambda start, end: ["a" * 40, "b" * 40], lambda _commit: {12})
        self.assertIn("(PR #12)", stamped)

    def test_two_pull_requests_leave_the_bullet_alone_with_one_warning(self) -> None:
        changelog = self.changelog()
        with patch("sys.stderr") as stderr:
            stamped = self.stamp(
                changelog,
                lambda start, end: ["a" * 40, "b" * 40],
                lambda commit: {12} if commit.startswith("a") else {13},
            )
        self.assertIn("- A bullet whose author left a placeholder. (PR #TBD)", stamped)
        warnings = [call.args[0] for call in stderr.write.call_args_list if "warning:" in str(call.args[0])]
        self.assertEqual(2, len(warnings), warnings)

    def test_a_bullet_that_already_names_its_pull_request_is_left_alone(self) -> None:
        changelog = self.changelog(STAMPABLE.replace("left nothing.", "left nothing. (PR #9)"))
        stamped = self.stamp(changelog, lambda start, end: ["a" * 40], lambda _commit: {12})
        self.assertIn("- A bullet whose author left nothing. (PR #9)", stamped)

    def test_nothing_resolving_still_exits_zero(self) -> None:
        changelog = self.changelog()
        def refuse(_commit):
            raise prepare_changelog_release.ChangelogError("gh could not resolve it")

        with patch.object(prepare_changelog_release, "pull_requests_for_commit", refuse), patch.object(
            prepare_changelog_release, "_blame_commits", lambda _path, start, end: ["a" * 40]
        ), patch("sys.stderr"):
            self.assertEqual(0, prepare_changelog_release.main(["--changelog", str(changelog), "stamp"]))
        self.assertEqual(STAMPABLE, changelog.read_text(encoding="utf-8"))

    def test_prepare_still_fails_on_an_empty_unreleased(self) -> None:
        changelog = self.changelog(STAMPABLE.replace(
            "- A bullet whose author left a placeholder. (PR #TBD)\n- A bullet whose author left nothing.",
            "*Nothing yet.*",
        ))
        with self.assertRaisesRegex(prepare_changelog_release.ChangelogError, "no bullet entries"):
            prepare_changelog_release.prepare_release(changelog, "0.2.1", "2026-05-18")

    def test_the_resolver_asks_github_for_the_commit_s_pull_requests(self) -> None:
        with patch.object(
            prepare_changelog_release.subprocess,
            "run",
            return_value=CompletedProcess(args=[], returncode=0, stdout="12\n", stderr=""),
        ) as run:
            self.assertEqual({12}, prepare_changelog_release.pull_requests_for_commit("a" * 40))
        self.assertEqual(
            ["gh", "api", f"/repos/{{owner}}/{{repo}}/commits/{'a' * 40}/pulls", "--jq", ".[].number"],
            run.call_args.args[0],
        )


class StampBlameTests(unittest.TestCase):
    """The line ranges, against a real repository: a bullet is blamed where its
    author left it, and an uncommitted line leaves its bullet alone."""

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.repo = Path(self.tempdir.name) / "repo"
        (self.repo / "docs").mkdir(parents=True)
        self.git("init", "-b", "main", ".")
        self.changelog = self.repo / "docs" / "changelog.md"

    def git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.repo), *arguments], check=True, capture_output=True, text=True
        ).stdout.strip()

    def commit(self, text: str, message: str) -> str:
        self.changelog.write_text(text, encoding="utf-8")
        self.git("add", "-A")
        self.git(
            "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "-m", message,
        )
        return self.git("rev-parse", "HEAD")

    def test_each_bullet_resolves_through_the_commit_that_wrote_it(self) -> None:
        first = self.commit(
            STAMPABLE.replace("- A bullet whose author left nothing.\n", ""), "the first bullet"
        )
        second = self.commit(STAMPABLE, "the second bullet")
        prepare_changelog_release.stamp_release_numbers(
            self.changelog, resolve=lambda commit: {31} if commit == first else {32}
        )
        stamped = self.changelog.read_text(encoding="utf-8")
        self.assertIn("- A bullet whose author left a placeholder. (PR #31)", stamped)
        self.assertIn("- A bullet whose author left nothing. (PR #32)", stamped)
        self.assertNotEqual(first, second)

    def test_an_uncommitted_line_leaves_its_bullet_alone(self) -> None:
        self.commit(STAMPABLE, "the bullets")
        self.changelog.write_text(
            STAMPABLE.replace("left nothing.", "left nothing, and has since edited it."), encoding="utf-8"
        )
        with patch("sys.stderr"):
            prepare_changelog_release.stamp_release_numbers(self.changelog, resolve=lambda _commit: {31})
        stamped = self.changelog.read_text(encoding="utf-8")
        self.assertIn("- A bullet whose author left nothing, and has since edited it.\n", stamped)
        self.assertIn("- A bullet whose author left a placeholder. (PR #31)", stamped)
