"""§AR-ci.7 — what the changelog gate reads inside the entries a branch adds or
changes (§FS-distribution.4.6): that each is a well-formed entry of the format
§FS-distribution.4.12 adopts, that a number written into one is the pull
request's own, and that an entry moved out of the merge base's `## Unreleased`
is not one this branch wrote.

Each case runs the CI half on the fixture `changelog_gate_fixture.py` lays down;
`test_check_changelog_pr_entry.py` holds which commits count, the skips, the
refusal's wording, and the parity between the two halves."""

import unittest

from changelog_gate_fixture import EARLIER, FIXTURE_PR, NEW, REFUSED, GitFixture

# Two of the three bullets 0.15.0 left under `## Unreleased`: the migrating branch's
# merge base holds them there, and the branch moves them into entry files.
MOVED_BASE = (
    "### Removed\n\n"
    "- [FS-cli.1](functional-spec/FS-cli.md#1-the-default-subcommand): bare `grund` is an error. (PR #331)\n\n"
    "### Fixed\n\n"
    "- [AR-goal-measurement.3](architecture/AR-goal-measurement.md#3-requirement-meters): the meter reads the inline release. (PR #383)"
)
MOVED = {
    "bare-grund-is-an-error.removed.md": "- [FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand): bare `grund` is an error. (PR #331)\n",
    "fix-release-record-reads-latest-release.fixed.md": "- [AR-goal-measurement.3](../../architecture/AR-goal-measurement.md#3-requirement-meters): the meter reads the inline release. (PR #383)\n",
}


class ChangelogEntryContentsTests(GitFixture, unittest.TestCase):
    """§FS-distribution.4.6 on what an added or changed entry holds."""

    # What is refused, for every file the branch adds or changes.
    def test_a_file_that_is_not_an_entry_is_refused(self) -> None:
        for name in ("notes.txt", "Fix-Thing.fixed.md", "fix_thing.fixed.md", "fix.thing.fixed.md"):
            with self.subTest(name=name):
                self.assertRefused(self._ci({**EARLIER, **NEW, name: "- A bullet.\n"}), name)

    def test_an_entry_without_a_category_is_refused(self) -> None:
        self.assertRefused(self._ci({**EARLIER, **NEW, "untyped-change.md": "- A bullet.\n"}), "untyped-change.md", "category")

    def test_an_entry_in_a_category_grund_does_not_take_is_refused(self) -> None:
        self.assertRefused(self._ci({**EARLIER, **NEW, "a-note.note.md": "- A bullet.\n"}), "a-note.note.md", "note")

    def test_an_entry_that_is_not_one_bullet_is_refused(self) -> None:
        shapes = {
            "empty.fixed.md": "",
            "prose.fixed.md": "Just prose, with no bullet.\n",
            "two-bullets.fixed.md": "- One.\n- Two.\n",
            "a-heading.fixed.md": "### Fixed\n\n- One.\n",
            "indented.fixed.md": "  - Indented under nothing.\n",
        }
        for name, text in shapes.items():
            with self.subTest(name=name):
                self.assertRefused(self._ci({**EARLIER, **NEW, name: text}), name)

    def test_a_slug_two_files_share_is_refused(self) -> None:
        held = {**EARLIER, "twice-used.fixed.md": "- One.\n"}
        cases = {
            "both added": (EARLIER, {**EARLIER, **NEW, "twice-used.added.md": "- One.\n", "twice-used.fixed.md": "- Two.\n"}),
            "one held already": (held, {**held, **NEW, "twice-used.added.md": "- Two.\n"}),
        }
        for case, (base, head) in cases.items():
            with self.subTest(case=case):
                self.assertRefused(self._ci(head, base), "twice-used")

    def test_a_malformed_entry_this_branch_did_not_touch_is_not_examined(self) -> None:
        held = {**EARLIER, "older-note.note.md": "Prose somebody else left.\n"}
        self.assertPassed(self._ci({**held, **NEW}, held))

    # The number: optional, and if written, the pull request's own.
    def test_an_entry_carrying_another_pull_requests_number_is_refused(self) -> None:
        result = self._ci({**EARLIER, "the-change.added.md": "- An entry. (PR #305)\n"})
        self.assertRefused(result, "PR #305", f"PR #{FIXTURE_PR}")

    def test_an_entry_carrying_its_own_number_passes(self) -> None:
        self.assertPassed(self._ci({**EARLIER, "the-change.added.md": f"- An entry. (PR #{FIXTURE_PR})\n"}))

    def test_a_tbd_placeholder_is_not_a_number(self) -> None:
        self.assertPassed(self._ci({**EARLIER, "the-change.added.md": "- An entry. (PR #TBD)\n"}))

    def test_a_number_already_in_an_entry_at_the_merge_base_is_not_checked(self) -> None:
        # `grund fmt --write` moves the anchor inside somebody else's entry when a
        # heading is renamed; their `PR #305` was there before this branch was.
        theirs = "- [AR-ci.7](../../architecture/AR-ci.md#7-{anchor}): their change. (PR #305)\n"
        base = {"their-change.fixed.md": theirs.format(anchor="old-title")}
        result = self._ci({"their-change.fixed.md": theirs.format(anchor="the-new-title"), **NEW}, base)
        self.assertPassed(result)
        self.assertNotIn("PR #305", self._output(result))

    def test_a_number_written_into_an_existing_entry_is_checked(self) -> None:
        base = {"older-change.fixed.md": "- An older entry.\n"}
        result = self._ci({"older-change.fixed.md": "- An older entry. (PR #305)\n", **NEW}, base)
        self.assertRefused(result, "PR #305")

    # The move rule: bullets the merge base's `## Unreleased` held are moved, not written.
    def test_a_branch_whose_only_entries_were_moved_is_refused(self) -> None:
        result = self._ci(MOVED, {}, base_unreleased=MOVED_BASE)
        self.assertRefused(result, REFUSED)
        self.assertNotIn("PR #331", self._output(result))
        self.assertNotIn("PR #383", self._output(result))

    def test_a_moved_entry_beside_one_of_your_own_passes(self) -> None:
        own = {"fix-issue-379.changed.md": "- Pending changelog entries are one file each.\n"}
        result = self._ci({**MOVED, **own}, {}, base_unreleased=MOVED_BASE)
        self.assertPassed(result)
        self.assertNotIn("PR #331", self._output(result))
        self.assertNotIn("PR #383", self._output(result))
