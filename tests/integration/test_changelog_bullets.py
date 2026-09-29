"""§AR-ci.7 — the one place the changelog gate and the release stamper agree about
`docs/changelog.md`: where `## Unreleased` is, what a bullet is, and when two
bullets are the same (§FS-distribution.4.6).

The last class is the property the fix rests on rather than a remembered rule:
the defect this module exists to close was two halves of one gate answering one
question two ways, so both scripts must reach these answers through *this*
module and carry no copy of their own."""

import importlib.util
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "scripts"


def _load(name: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


changelog_bullets = _load("changelog_bullets")

SECTION = """# Changelog

## Unreleased

### Added

- A first bullet.
- A second bullet that
  wraps onto a continuation line.
  - and carries a nested bullet of its own

### Changed

- A third bullet, under the next heading. (PR #7)

## 1. [0.1.0] — 2026-01-01

- A released bullet, which is not under Unreleased. (PR #2)
"""


class UnreleasedSectionTests(unittest.TestCase):
    def test_the_section_runs_to_the_next_top_level_heading(self) -> None:
        body = changelog_bullets.unreleased_body(SECTION.splitlines())
        self.assertIn("- A third bullet, under the next heading. (PR #7)", body)
        self.assertNotIn("- A released bullet, which is not under Unreleased. (PR #2)", body)

    def test_a_third_level_heading_does_not_close_the_section(self) -> None:
        self.assertIn("### Changed", changelog_bullets.unreleased_body(SECTION.splitlines()))

    def test_a_changelog_without_the_section_is_refused(self) -> None:
        with self.assertRaises(changelog_bullets.ChangelogFormatError):
            changelog_bullets.unreleased_body(["# Changelog", "", "## 1. [0.1.0] — 2026-01-01"])

    def test_has_bullet_sees_a_bullet_and_an_empty_section(self) -> None:
        self.assertTrue(changelog_bullets.has_bullet(["- something"]))
        self.assertFalse(changelog_bullets.has_bullet(["*Nothing yet.*", ""]))


class BulletTests(unittest.TestCase):
    def setUp(self) -> None:
        self.bullets = changelog_bullets.bullets(SECTION.splitlines())

    def test_a_bullet_is_its_line_plus_its_continuations(self) -> None:
        self.assertEqual(3, len(self.bullets))
        self.assertEqual(
            ["- A second bullet that", "  wraps onto a continuation line.", "  - and carries a nested bullet of its own"],
            list(self.bullets[1].lines),
        )

    def test_the_line_range_is_one_based_and_inclusive(self) -> None:
        # `git blame -L start,end` takes exactly this, which is how the stamper
        # finds the commits that wrote a bullet (§FS-distribution.4.5).
        lines = SECTION.splitlines()
        for bullet in self.bullets:
            self.assertEqual(bullet.lines[0], lines[bullet.start - 1])
            self.assertEqual(bullet.lines[-1], lines[bullet.end - 1])

    def test_a_heading_closes_a_bullet(self) -> None:
        self.assertEqual(["- A third bullet, under the next heading. (PR #7)"], list(self.bullets[2].lines))

    def test_trailing_blank_lines_are_not_part_of_a_bullet(self) -> None:
        found = changelog_bullets.bullets(["## Unreleased", "", "- One.", "", "", ""])
        self.assertEqual(["- One."], list(found[0].lines))


class SamenessTests(unittest.TestCase):
    def same(self, left: str, right: str) -> None:
        self.assertEqual(changelog_bullets.normalise(left), changelog_bullets.normalise(right))

    def test_rewrapping_is_not_a_change(self) -> None:
        self.same("- A bullet that says one thing. (PR #3)", "- A bullet that says\n  one thing. (PR #3)")

    def test_rewording_is_a_change(self) -> None:
        self.assertNotEqual(
            changelog_bullets.normalise("- A bullet that says one thing."),
            changelog_bullets.normalise("- A bullet that says another thing."),
        )

    def test_a_link_destination_is_disregarded(self) -> None:
        # `grund fmt --write` rewrites the anchor when a heading is renamed, and
        # it does that inside bullets nobody on this branch wrote.
        self.same("- [§AR-ci.7](a.md#7-old-title): a change.", "- [§AR-ci.7](a.md#7-the-new-title): a change.")

    def test_a_trailing_number_is_dropped_and_so_is_a_placeholder(self) -> None:
        self.same("- A bullet. (PR #3)", "- A bullet. (PR #TBD)")
        self.same("- A bullet. (PR #3)", "- A bullet.")

    def test_appending_a_second_number_drops_both(self) -> None:
        self.same("- Somebody else's bullet. (PR #3)", "- Somebody else's bullet. (PR #3) (PR #9)")

    def test_a_number_that_is_not_trailing_is_part_of_the_text(self) -> None:
        self.assertNotEqual(
            changelog_bullets.normalise("- A bullet. (PR #3)"),
            changelog_bullets.normalise("- A bullet that reverts PR #9 and says so. (PR #3)"),
        )


class NumberTests(unittest.TestCase):
    def test_every_form_a_bullet_may_name_a_pull_request_in(self) -> None:
        self.assertEqual({3}, set(changelog_bullets.pr_numbers("- A bullet. (PR #3)")))
        self.assertEqual({3}, set(changelog_bullets.pr_numbers("- A bullet. pull request #3")))
        self.assertEqual({3}, set(changelog_bullets.pr_numbers("- A bullet ([x](https://g/o/r/pull/3)).")))

    def test_a_placeholder_is_not_a_number(self) -> None:
        self.assertEqual(set(), set(changelog_bullets.pr_numbers("- A bullet. (PR #TBD)")))

    def test_an_issue_link_is_not_a_pull_request(self) -> None:
        self.assertEqual(set(), set(changelog_bullets.pr_numbers("- Closes [issue #9](https://g/o/r/issues/9).")))


class OneCodePathTests(unittest.TestCase):
    """The gate and the stamper reach these answers here, or the fix is not a fix."""

    def test_both_scripts_hold_the_same_module_object(self) -> None:
        gate = _load("check_changelog_pr_entry")
        stamper = _load("prepare_changelog_release")
        self.assertIs(gate.changelog_bullets, stamper.changelog_bullets)

    def test_neither_script_carries_its_own_answer(self) -> None:
        for name in ("check_changelog_pr_entry", "prepare_changelog_release"):
            with self.subTest(script=name):
                source = (SCRIPTS / f"{name}.py").read_text(encoding="utf-8")
                self.assertNotIn("## Unreleased\\s", source, "it matches the section heading itself")
                self.assertNotIn('startswith("- ")', source, "it decides what a bullet is itself")


if __name__ == "__main__":
    unittest.main()
