"""§FS-distribution.4.6.4.1 — every decision record's `release-note` section is held
to the one bullet the bump publishes on the pull request that writes it, not first
at the next release (§FS-distribution.4.6.3); the Python suite is where the tree is
asked (§AR-ci.7).

The walk reads each record with `release_notices._notice` and judges it with
`_bullet`, the two functions the bump reads a notice through, so this gate and the
bump cannot disagree about a shape. The synthetic trees pin that reading;
`ThisRepositoryTests` runs it over this repository's own records."""

import importlib.util
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "release_notices.py"

_spec = importlib.util.spec_from_file_location("release_notices", SCRIPT)
release_notices = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(release_notices)

FUNCTIONAL, ARCHITECTURAL = "docs/decisions/functional", "docs/decisions/architectural"
PROSE = "This decision now changes a verdict for every project that relied on the old one.\n"


def refused(top: Path) -> list[str]:
    """The bump's refusal of every `release-note` section under `top/docs/decisions`, all of them."""
    refusals = []
    for record in sorted((top / "docs" / "decisions").rglob("*.md")):
        path = record.relative_to(top).as_posix()
        notice = release_notices._notice(path, record.read_text(encoding="utf-8"))
        if notice is None:
            continue
        try:
            release_notices._bullet(notice)
        except release_notices.ChangelogError as refusal:
            refusals.append(str(refusal))
    return refusals


def decision(identifier: str, note: str | None) -> str:
    """A decision record, with `note` as its `release-note` section's text where given."""
    record = f"# {identifier}: a decision\n\n**Status:** Accepted\n\n## 1. Decision\n\nDecided.\n"
    return record if note is None else f"{record}\n## release-note: Release note\n\n{note}"


def message(identifier: str, path: str, problem: str) -> str:
    return f"the release-note section of {identifier} ({path}) is not one well-formed bullet: {problem}"


class ReadingTests(unittest.TestCase):
    def _refused(self, files: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            top = Path(directory)
            for path, text in files.items():
                (top / path).parent.mkdir(parents=True, exist_ok=True)
                (top / path).write_text(text, encoding="utf-8")
            return refused(top)

    def test_a_prose_note_is_refused_naming_its_decision_and_file(self) -> None:
        path = f"{FUNCTIONAL}/DF-prose.md"
        self.assertEqual(
            [message("DF-prose", path, "its first line does not open a bullet with `- `")],
            self._refused({path: decision("DF-prose", PROSE)}),
        )

    def test_one_bullet_with_a_continuation_line_passes(self) -> None:
        note = "- **A verdict moved.** Who this breaks:\n  every CI that relied on the old one.\n"
        self.assertEqual([], self._refused({f"{FUNCTIONAL}/DF-bullet.md": decision("DF-bullet", note)}))

    def test_a_record_without_a_note_is_not_read(self) -> None:
        self.assertEqual([], self._refused({f"{FUNCTIONAL}/DF-quiet.md": decision("DF-quiet", None)}))

    def test_every_malformed_record_is_named_in_one_run(self) -> None:
        prose, split = f"{FUNCTIONAL}/DF-prose.md", f"{ARCHITECTURAL}/nested/DA-split.md"
        two_paragraphs = "- **A verdict moved.** Who this breaks: CI.\n\n  And a second paragraph.\n"
        self.assertEqual(
            [
                message("DA-split", split, "its line 2 is blank, which splits it"),
                message("DF-prose", prose, "its first line does not open a bullet with `- `"),
            ],
            self._refused(
                {
                    prose: decision("DF-prose", PROSE),
                    split: decision("DA-split", two_paragraphs),
                    f"{FUNCTIONAL}/DF-bullet.md": decision("DF-bullet", "- **Fine.** Who this breaks: nobody.\n"),
                }
            ),
        )


class ThisRepositoryTests(unittest.TestCase):
    def test_every_release_note_in_the_tree_is_one_well_formed_bullet(self) -> None:
        refusals = refused(REPO_ROOT)
        if refusals:
            self.fail(
                "the next release bump would refuse these release notes; rewrite each as one `- ` bullet, "
                "continuation lines indented two spaces:\n" + "\n".join(refusals)
            )


if __name__ == "__main__":
    unittest.main()
