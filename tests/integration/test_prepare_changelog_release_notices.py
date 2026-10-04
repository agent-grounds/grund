"""§FS-distribution.4.6.4 — compatibility notices come from the decisions: `prepare`
publishes, under `### Compatibility notices`, the `release-note` section of every
decision record that gained one since the previous tag, matched by decision ID,
ordered by it, with links rebased to `docs/`; a notice that shipped already is not
published again however it was edited, and a malformed one is refused
(§FS-distribution.4.6.3).

Each case runs the script in a throwaway repository against the stub forge of
`release_forge_fixture.py`; the list it sits under is
`test_prepare_changelog_release_list.py`."""

import unittest

from release_forge_fixture import ForgeRepository

VERSION, DATE = "0.3.0", "2026-10-05"
CHANGELOG = """# Changelog

## 2. [0.2.0] — 2026-05-17

- [One](https://github.com/agent-grounds/grund/pull/4) (PR #4)

## 3. Older releases
"""
FUNCTIONAL, ARCHITECTURAL = "docs/decisions/functional", "docs/decisions/architectural"


def decision(identifier: str, note: str | None) -> str:
    """A decision record, with `note` as its `release-note` section's text where given."""
    record = f"# {identifier}: a decision\n\n**Status:** Accepted\n\n## 1. Decision\n\nDecided.\n"
    return record if note is None else f"{record}\n## release-note: Release note\n\n{note}"


class NoticeTests(ForgeRepository, unittest.TestCase):
    def _tagged(self, files: dict[str, str]) -> None:
        self._repository({"docs/changelog.md": CHANGELOG, **files})

    def _release(self, files: dict[str, str | None]) -> None:
        self._merged(20, "The change", ("The change", files))

    def _notices(self) -> str:
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        section = self._read().split(f"## 2. [{VERSION}] — {DATE}\n\n", 1)[1].split("\n## 3. Older releases", 1)[0]
        parts = section.split("### Compatibility notices\n\n", 1)
        return parts[1] if len(parts) == 2 else ""

    def test_a_notice_added_since_the_tag_is_published(self) -> None:
        self._tagged({f"{FUNCTIONAL}/DF-quiet.md": decision("DF-quiet", None)})
        self._release({f"{FUNCTIONAL}/DF-quiet.md": decision("DF-quiet", "- **A verdict moved.** Who this breaks: CI.\n")})
        self.assertEqual("- **A verdict moved.** Who this breaks: CI.\n", self._notices())

    def test_a_new_decision_s_notice_is_published(self) -> None:
        self._tagged({})
        self._release({f"{ARCHITECTURAL}/DA-new.md": decision("DA-new", "- **New.** Who this breaks: nobody.\n")})
        self.assertEqual("- **New.** Who this breaks: nobody.\n", self._notices())

    def test_a_notice_present_at_the_tag_is_not_published(self) -> None:
        self._tagged({f"{FUNCTIONAL}/DF-old.md": decision("DF-old", "- **Shipped.** Who this breaks: CI.\n")})
        self._release({"src/a.rs": "fn a() {}\n"})
        self.assertEqual("", self._notices())
        self.assertNotIn("### Compatibility notices", self._read())

    def test_a_notice_edited_after_its_release_is_not_published_again(self) -> None:
        self._tagged({f"{FUNCTIONAL}/DF-old.md": decision("DF-old", "- **Shipped.** Who this breaks: CI.\n")})
        self._release({f"{FUNCTIONAL}/DF-old.md": decision("DF-old", "- **Shipped, reworded.** Who this breaks: CI.\n")})
        self.assertEqual("", self._notices())

    def test_a_record_moved_since_the_tag_is_matched_by_its_id(self) -> None:
        self._tagged({f"{FUNCTIONAL}/DF-old.md": decision("DF-old", "- **Shipped.** Who this breaks: CI.\n")})
        self._release(
            {
                f"{FUNCTIONAL}/DF-old.md": None,
                f"{FUNCTIONAL}/moved/DF-old-renamed.md": decision("DF-old", "- **Shipped.** Who this breaks: CI.\n"),
            }
        )
        self.assertEqual("", self._notices())

    def test_notices_are_ordered_by_decision_id(self) -> None:
        self._tagged({})
        self._release(
            {
                f"{FUNCTIONAL}/DF-b.md": decision("DF-b", "- **B.**\n"),
                f"{ARCHITECTURAL}/DA-z.md": decision("DA-z", "- **Z.**\n"),
                f"{FUNCTIONAL}/DF-a.md": decision("DF-a", "- **A.**\n"),
            }
        )
        self.assertEqual("- **Z.**\n- **A.**\n- **B.**\n", self._notices())

    def test_links_are_rebased_from_the_record_to_docs(self) -> None:
        note = (
            "- **Moved.** See [\u00a7DF-a](#df-a-a-decision), [\u00a7REQ-x.1](../../requirements/REQ-x.md#1-y),\n"
            "  [the DA](../architectural/DA-q.md#q), [code](../../../crates/x.rs), [abs](/README.md) and\n"
            "  [a url](https://example.com/a).\n"
        )
        rebased = (
            "- **Moved.** See [\u00a7DF-a](decisions/functional/DF-a.md#df-a-a-decision), [\u00a7REQ-x.1](requirements/REQ-x.md#1-y),\n"
            "  [the DA](decisions/architectural/DA-q.md#q), [code](../crates/x.rs), [abs](/README.md) and\n"
            "  [a url](https://example.com/a).\n"
        )
        self._tagged({})
        self._release({f"{FUNCTIONAL}/DF-a.md": decision("DF-a", note)})
        self.assertEqual(rebased, self._notices())

    def test_a_malformed_notice_is_refused(self) -> None:
        malformed = {
            "two bullets": "- One.\n- Two.\n",
            "no bullet": "Just a paragraph.\n",
            "a paragraph beside the bullet": "- One.\n\nAnd a paragraph.\n",
        }
        for case, note in malformed.items():
            with self.subTest(case=case):
                self._tagged({})
                self._release({f"{FUNCTIONAL}/DF-bad.md": decision("DF-bad", note)})
                self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), "DF-bad")


if __name__ == "__main__":
    unittest.main()
