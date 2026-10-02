"""§FS-distribution.4.5 — `stamp`, the step just before the rotation: each entry
under `docs/changelog/unreleased/` (§FS-distribution.4.12) that does not already
end in its number gains the number of the pull request whose commit added it, at
its end and nowhere else; anything else is warned about once and left, and the
release never fails for it. A number it would write into more than one entry is
written into none of them, whichever commits added them; a write-up
(§FS-distribution.4.6), whose unnumbered entries all resolve to its own number, is
the usual case.

The history half runs against a real throwaway repository, because which commit
added an entry — through an edit, a change of category, and a slug used a second
time — is the thing under test; the commit-to-pull-request half is the `resolve`
seam, so no case reaches the network. Every call is made the way the release
workflow makes it: from the repository root, on the default `docs/changelog.md`."""

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import patch

from changelog_gate_fixture import ENTRIES, ENTRY_README, REPO_ROOT, GitFixture

SCRIPT_PATH = REPO_ROOT / "scripts" / "prepare_changelog_release.py"
SPEC = importlib.util.spec_from_file_location("prepare_changelog_release", SCRIPT_PATH)
prepare_changelog_release = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(prepare_changelog_release)

CHANGELOG = "# Changelog\n\n## Unreleased\n\nA pointer.\n\n## 2. [0.2.0] — 2026-05-17\n\n- Released. (PR #4)\n\n## 3. Older releases\n"


@contextlib.contextmanager
def inside(directory: Path):
    previous = Path.cwd()
    os.chdir(directory)
    try:
        yield
    finally:
        os.chdir(previous)


class StampTests(GitFixture, unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.repo = Path(scratch.name) / "repo"
        self._git(Path(scratch.name), "init", "-b", "main", str(self.repo))
        (self.repo / ENTRIES).mkdir(parents=True)
        (self.repo / "docs" / "changelog.md").write_bytes(CHANGELOG.encode("utf-8"))
        (self.repo / ENTRIES / "README.md").write_bytes(ENTRY_README.encode("utf-8"))
        self._commit(self.repo, "The base")

    def _put(self, entries: dict[str, str | None]) -> None:
        for name, text in entries.items():
            if text is None:
                (self.repo / ENTRIES / name).unlink()
            else:
                (self.repo / ENTRIES / name).write_bytes(text.encode("utf-8"))

    def _land(self, entries: dict[str, str | None]) -> str:
        self._put(entries)
        return self._commit(self.repo, "Land entries")

    def _stamp(self, numbers: dict[str, set[int]]) -> str:
        """Stamp with `numbers` as the commit-to-pull-request answer; what it warned."""
        warned = io.StringIO()
        with inside(self.repo), contextlib.redirect_stderr(warned):
            prepare_changelog_release.stamp_release_numbers(
                Path("docs/changelog.md"), resolve=lambda commit: numbers.get(commit, set())
            )
        return warned.getvalue()

    def _entry(self, name: str) -> str:
        return (self.repo / ENTRIES / name).read_text(encoding="utf-8")

    def _warnings(self, stderr: str) -> list[str]:
        return [line for line in stderr.splitlines() if "warning:" in line]

    # Which number: the pull request whose commit added the entry.
    def test_an_entry_gains_the_number_of_the_pull_request_that_added_it(self) -> None:
        added = self._land({"the-change.added.md": "- An entry.\n"})
        self._stamp({added: {31}})
        self.assertEqual("- An entry. (PR #31)\n", self._entry("the-change.added.md"))
        self.assertEqual(ENTRY_README, self._entry("README.md"))
        self.assertEqual(CHANGELOG.encode("utf-8"), (self.repo / "docs" / "changelog.md").read_bytes())

    def test_a_later_edit_keeps_the_first_pull_request(self) -> None:
        added = self._land({"the-change.added.md": "- First words.\n"})
        edited = self._land({"the-change.added.md": "- Better words.\n"})
        self._stamp({added: {31}, edited: {32}})
        self.assertEqual("- Better words. (PR #31)\n", self._entry("the-change.added.md"))

    def test_a_change_of_category_made_with_an_edit_keeps_the_first_pull_request(self) -> None:
        # Rewritten in the same commit, git sees no rename; the slug is what persists.
        added = self._land({"the-change.fixed.md": "- First words.\n"})
        moved = self._land({"the-change.fixed.md": None, "the-change.added.md": "- Other words, filed as an addition.\n"})
        self._stamp({added: {31}, moved: {32}})
        self.assertEqual("- Other words, filed as an addition. (PR #31)\n", self._entry("the-change.added.md"))

    def test_a_slug_used_again_after_a_release_takes_the_new_pull_request(self) -> None:
        first = self._land({"the-change.fixed.md": "- The first use of the slug.\n"})
        released = self._land({"the-change.fixed.md": None})
        again = self._land({"the-change.fixed.md": "- A new change under an old slug.\n"})
        self._stamp({first: {31}, released: set(), again: {33}})
        self.assertEqual("- A new change under an old slug. (PR #33)\n", self._entry("the-change.fixed.md"))

    # Where the number goes: at the end of the entry, and nowhere else.
    def test_a_trailing_placeholder_is_replaced(self) -> None:
        added = self._land({"the-change.added.md": "- An entry. (PR #TBD)\n"})
        self._stamp({added: {31}})
        self.assertEqual("- An entry. (PR #31)\n", self._entry("the-change.added.md"))

    def test_a_placeholder_in_the_prose_is_left_and_the_number_appended(self) -> None:
        # The 0.15.0 cut wrote `PR #346` into a sentence that told authors to write `PR #TBD`.
        prose = "- Write `(PR #TBD)`, or nothing, and the release fills it in."
        added = self._land({"the-change.changed.md": f"{prose}\n"})
        self._stamp({added: {346}})
        self.assertEqual(f"{prose} (PR #346)\n", self._entry("the-change.changed.md"))

    def test_a_wrapped_entry_is_numbered_on_its_last_line(self) -> None:
        for ending in ("", " (PR #TBD)"):
            with self.subTest(ending=ending):
                added = self._land({"the-change.added.md": f"- A wrapped entry\n  that goes on.{ending}\n"})
                self._stamp({added: {31}})
                self.assertEqual("- A wrapped entry\n  that goes on. (PR #31)\n", self._entry("the-change.added.md"))
                self._land({"the-change.added.md": None})

    def test_an_entry_that_already_ends_in_a_number_is_left(self) -> None:
        added = self._land({"the-change.added.md": "- An entry. (PR #9)\n"})
        self._stamp({added: {31}})
        self.assertEqual("- An entry. (PR #9)\n", self._entry("the-change.added.md"))

    # A write-up: one commit adds every entry, so every one resolves to the write-up's own number.
    WRITE_UP = {
        "fix-issue-401.changed.md": "- Written from issue #401, with no number.\n",
        "fix-issue-397.added.md": "- Names its issue, as a write-up from the issues would. (#397)\n",
        "fix-issue-384.added.md": "- Ends in the placeholder. (PR #TBD)\n",
        "fix-issue-375.added.md": "- Its writer looked up the change's own pull request. (PR #375)\n",
    }

    def assertWarnedOnceEach(self, warnings: list[str], names: list[str], number: str) -> None:
        for name in names:
            with self.subTest(warned=name):
                named = [warning for warning in warnings if name in warning]
                self.assertEqual(1, len(named), warnings)
                self.assertIn(number, named[0])
        self.assertEqual(len(names), len(warnings), warnings)

    def test_a_number_that_would_go_into_several_entries_goes_into_none(self) -> None:
        # Triage's probe on fdb5392180: all three unnumbered entries became `(PR #500)`, unwarned.
        added = self._land(self.WRITE_UP)
        warnings = self._warnings(self._stamp({added: {500}}))
        for name, text in self.WRITE_UP.items():
            with self.subTest(entry=name):
                self.assertEqual(text, self._entry(name))
        unnumbered = ["fix-issue-401.changed.md", "fix-issue-397.added.md", "fix-issue-384.added.md"]
        self.assertWarnedOnceEach(warnings, unnumbered, "#500")

    def test_an_entry_in_its_own_pull_request_is_stamped_beside_a_write_up(self) -> None:
        own = self._land({"the-change.added.md": "- An entry its own pull request added.\n"})
        write_up = self._land({"one.fixed.md": "- One.\n", "two.fixed.md": "- Two. (PR #TBD)\n"})
        warnings = self._warnings(self._stamp({own: {31}, write_up: {500}}))
        self.assertEqual("- An entry its own pull request added. (PR #31)\n", self._entry("the-change.added.md"))
        self.assertEqual("- One.\n", self._entry("one.fixed.md"))
        self.assertEqual("- Two. (PR #TBD)\n", self._entry("two.fixed.md"))
        self.assertWarnedOnceEach(warnings, ["one.fixed.md", "two.fixed.md"], "#500")

    def test_a_write_up_whose_entries_carry_their_own_numbers_is_left(self) -> None:
        numbered = {"one.fixed.md": "- One. (PR #375)\n", "two.added.md": "- Two. (PR #380)\n"}
        added = self._land(numbered)
        self.assertEqual([], self._warnings(self._stamp({added: {500}})))
        for name, text in numbered.items():
            with self.subTest(entry=name):
                self.assertEqual(text, self._entry(name))

    # What is left: warned about once each, and never a failure.
    def test_an_entry_two_pull_requests_resolve_for_is_warned_about_once_and_left(self) -> None:
        self._land({"the-change.added.md": "- An entry.\n"})
        warned = io.StringIO()
        with inside(self.repo), contextlib.redirect_stderr(warned), patch.object(
            prepare_changelog_release, "pull_requests_for_commit", lambda _commit: {12, 13}
        ):
            self.assertEqual(0, prepare_changelog_release.main(["stamp"]))
        self.assertEqual("- An entry.\n", self._entry("the-change.added.md"))
        warnings = self._warnings(warned.getvalue())
        self.assertEqual(1, len(warnings), warned.getvalue())
        self.assertIn("the-change.added.md", warnings[0])

    def test_an_uncommitted_entry_is_warned_about_and_left(self) -> None:
        added = self._land({"the-change.added.md": "- An entry.\n"})
        self._put({"not-yet.fixed.md": "- Not committed.\n"})
        warnings = self._warnings(self._stamp({added: {31}}))
        self.assertEqual("- An entry. (PR #31)\n", self._entry("the-change.added.md"))
        self.assertEqual("- Not committed.\n", self._entry("not-yet.fixed.md"))
        self.assertEqual(1, len(warnings), warnings)
        self.assertIn("not-yet.fixed.md", warnings[0])

    def test_a_release_in_which_nothing_resolves_writes_nothing(self) -> None:
        self._land({"one.added.md": "- One.\n", "two.fixed.md": "- Two. (PR #TBD)\n"})
        self.assertEqual(2, len(self._warnings(self._stamp({}))))
        self.assertEqual("", self._git(self.repo, "status", "--porcelain", "--untracked-files=all"))

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


if __name__ == "__main__":
    unittest.main()
