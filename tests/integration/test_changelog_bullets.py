"""§AR-ci.7 — the one place the release's readers agree about the changelog: what
an entry under `docs/changelog/unreleased/` is, by its file name and its shape
(§FS-distribution.4.12), and, for the pointer, where `## Unreleased` is and what
a bullet is (§FS-distribution.4.5).

The README that states the format is held to the same answer: its category list
is the module's, and its first part, the part another repository copies, carries
nothing of this one's.

The last class is the property the module rests on rather than a remembered
rule: the defect it exists to close was two readers answering one question two
ways, so both scripts that read entries must reach these answers through *this*
module and carry no copy of their own."""

import ast
import importlib.util
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "scripts"
ENTRY_README = REPO_ROOT / "docs" / "changelog" / "unreleased" / "README.md"
ANSWERS = (
    "unreleased_range", "unreleased_body", "bullets", "has_bullet", "entry_name", "entry_problem",
)
KEEP_A_CHANGELOG = ("added", "changed", "deprecated", "removed", "fixed", "security")


def _load(name: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


changelog_bullets = _load("changelog_bullets")


def answer(name: str):
    """One of the module's answers, failing the case that asks for one it lacks."""
    found = getattr(changelog_bullets, name, None)
    if found is None:
        raise AssertionError(f"scripts/changelog_bullets.py defines no `{name}`")
    return found

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


class EntryNameTests(unittest.TestCase):
    """`<slug>.<category>.md`: the slug is lowercase letters, digits and hyphens."""

    def test_a_name_is_a_slug_and_a_category(self) -> None:
        entry_name = answer("entry_name")
        self.assertEqual(("fix-issue-379", "changed"), tuple(entry_name("fix-issue-379.changed.md")))
        self.assertEqual(("0-9", "fixed"), tuple(entry_name("0-9.fixed.md")))

    def test_a_name_without_a_category_is_read_with_none(self) -> None:
        self.assertEqual(("untyped-change", None), tuple(answer("entry_name")("untyped-change.md")))

    def test_a_category_is_any_lowercase_word_to_the_format(self) -> None:
        self.assertEqual(("a-note", "note"), tuple(answer("entry_name")("a-note.note.md")))

    def test_what_is_not_an_entry_name(self) -> None:
        for name in (
            "README.md", "notes.txt", "Fix-Thing.fixed.md", "fix_thing.fixed.md",
            "fix.thing.fixed.md", "the-change.Fixed.md", ".md", "the-change.added.markdown",
        ):
            with self.subTest(name=name):
                self.assertIsNone(answer("entry_name")(name))


class EntryShapeTests(unittest.TestCase):
    """What `entry_problem` refuses: grund's categories, and one bullet per file."""

    def test_grund_takes_the_keep_a_changelog_categories_in_their_order(self) -> None:
        self.assertEqual(KEEP_A_CHANGELOG, tuple(answer("CATEGORIES")))

    def test_every_category_grund_takes_is_an_entry(self) -> None:
        for category in KEEP_A_CHANGELOG:
            with self.subTest(category=category):
                self.assertIsNone(answer("entry_problem")(f"the-change.{category}.md", "- One.\n"))

    def test_a_missing_or_unknown_category_is_a_problem_naming_it(self) -> None:
        self.assertIn("category", answer("entry_problem")("untyped-change.md", "- One.\n"))
        self.assertIn("note", answer("entry_problem")("a-note.note.md", "- One.\n"))

    def test_one_bullet_is_an_entry_however_it_is_wrapped_or_ended(self) -> None:
        shapes = {
            "one line": "- One.\n",
            "wrapped": "- One that\n  goes on.\n",
            "crlf": "- One that\r\n  goes on.\r\n",
            "no final newline": "- One.",
            "trailing blank lines": "- One.\n\n\n",
        }
        for shape, text in shapes.items():
            with self.subTest(shape=shape):
                self.assertIsNone(answer("entry_problem")("the-change.fixed.md", text))

    def test_anything_but_one_bullet_is_a_problem(self) -> None:
        shapes = {
            "empty": "",
            "prose": "Just prose, with no bullet.\n",
            "two bullets": "- One.\n- Two.\n",
            "a heading": "### Fixed\n\n- One.\n",
            "indented": "  - Indented under nothing.\n",
        }
        for shape, text in shapes.items():
            with self.subTest(shape=shape):
                self.assertIsNotNone(answer("entry_problem")("the-change.fixed.md", text))

    def test_a_name_that_is_not_an_entry_is_a_problem_naming_it(self) -> None:
        self.assertIn("notes.txt", answer("entry_problem")("notes.txt", "- One.\n"))


class EntryReadmeTests(unittest.TestCase):
    """The README that states the format agrees with the module that reads it."""

    def setUp(self) -> None:
        text = ENTRY_README.read_text(encoding="utf-8")
        self.part_one, _, self.part_two = text.partition("\n## Part two")

    def test_the_readme_lists_the_categories_the_module_takes(self) -> None:
        listing = next(line for line in self.part_two.splitlines() if "is one of" in line)
        self.assertEqual(list(answer("CATEGORIES")), re.findall(r"`([a-z]+)`", listing))

    def test_part_one_carries_no_rule_of_this_repository(self) -> None:
        self.assertTrue(self.part_two, "the README has no part two")
        self.assertNotIn("\u00a7", self.part_one)
        self.assertNotIn("grund", self.part_one.lower())


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

    def test_a_heading_closes_a_bullet(self) -> None:
        self.assertEqual(["- A third bullet, under the next heading. (PR #7)"], list(self.bullets[2].lines))

    def test_trailing_blank_lines_are_not_part_of_a_bullet(self) -> None:
        found = changelog_bullets.bullets(["## Unreleased", "", "- One.", "", "", ""])
        self.assertEqual(["- One."], list(found[0].lines))


class OneCodePathTests(unittest.TestCase):
    """The release helper and the module it reads entries through reach these
    answers here, or the module is not the one place."""

    def test_both_scripts_hold_the_same_module_object(self) -> None:
        reader = _load("changelog_entries")
        stamper = _load("prepare_changelog_release")
        self.assertIs(reader.changelog_bullets, stamper.changelog_bullets)

    def test_each_script_reaches_the_answers_only_through_the_shared_module(self) -> None:
        # The positive property, not the absence of two spellings a copy can
        # evade: every route either script has to one of these answers is a call
        # on `changelog_bullets`, and neither defines one of its own.
        for name in ("changelog_entries", "prepare_changelog_release"):
            with self.subTest(script=name):
                tree = ast.parse((SCRIPTS / f"{name}.py").read_text(encoding="utf-8"))
                defined = {
                    node.name
                    for node in ast.walk(tree)
                    if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in ANSWERS
                }
                self.assertEqual(set(), defined, "it answers one of the shared questions itself")
                through_the_module = 0
                for node in ast.walk(tree):
                    if not isinstance(node, ast.Call):
                        continue
                    if isinstance(node.func, ast.Name) and node.func.id in ANSWERS:
                        self.fail(f"`{node.func.id}` is called as a bare name rather than on the shared module")
                    if isinstance(node.func, ast.Attribute) and node.func.attr in ANSWERS:
                        self.assertEqual(
                            "changelog_bullets",
                            getattr(node.func.value, "id", None),
                            f"`{node.func.attr}` is reached through something other than the shared module",
                        )
                        through_the_module += 1
                self.assertTrue(through_the_module, "it asks the shared module nothing, so this proves nothing")

    def test_each_script_reads_entries_through_the_shared_module(self) -> None:
        for name in ("changelog_entries", "prepare_changelog_release"):
            with self.subTest(script=name):
                tree = ast.parse((SCRIPTS / f"{name}.py").read_text(encoding="utf-8"))
                asked = {
                    node.func.attr
                    for node in ast.walk(tree)
                    if isinstance(node, ast.Call)
                    and isinstance(node.func, ast.Attribute)
                    and getattr(node.func.value, "id", None) == "changelog_bullets"
                }
                self.assertTrue(asked & {"entry_name", "entry_problem"}, f"it asks the module only {sorted(asked)}")


if __name__ == "__main__":
    unittest.main()
