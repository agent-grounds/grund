"""§AR-goal-measurement.1 — the `out-of-scope-` family of §FS-check.3.14 is
written down in three places that must agree: the match of `tag_out_of_scope`
that mints the codes, the sentence of §FS-check.3.14.5 that enumerates them, and
the rows of the §FS-errors.5.5 catalog that document them. This spans the
implementation and two specification pages, which is why it is an integration
test rather than a unit one.

It exists because the drift of agent-grounds/grund#348 began as prose that
enumerated what the match no longer matched: a fifth arm was added and the
sentence listing four was left alone, with nothing to notice. Correcting the
sentence without this test would leave the next arm free to start the same drift
again. The failure prints the sets, so what to add and where is the message."""

import re
import unittest
from pathlib import Path

from test_check_codes_are_section_handles import _catalog_rows


REPO_ROOT = Path(__file__).resolve().parents[2]
REFERENCE_SCOPE = REPO_ROOT / "crates" / "grund-core" / "src" / "checker" / "reference_scope.rs"
CHECK_PAGE = REPO_ROOT / "docs" / "functional-spec" / "FS-check.md"

PREFIX = "out-of-scope-"
# `"dangling" => "out-of-scope-dangling",` — the minted code, not the one matched.
MATCH_ARM = re.compile(r'=>\s*"(' + PREFIX + r'[a-z][a-z0-9-]*)"')
TAG_FN = "pub(crate) fn tag_out_of_scope("
COMPOUND_CODE_HEADING = re.compile(r"^#### 3\.14\.5 ")
HEADING = re.compile(r"^#{1,6}\s")
# A backticked code in the prose. The bare prefix `out-of-scope-` names the
# scheme rather than a member, and has no code after the final hyphen to match.
PROSE_CODE = re.compile(r"`(" + PREFIX + r"[a-z][a-z0-9-]*)`")


def _minted_codes():
    """Every tier code the `tag_out_of_scope` match mints."""
    source = REFERENCE_SCOPE.read_text(encoding="utf-8")
    start = source.index(TAG_FN)
    return set(MATCH_ARM.findall(source[start:]))


def _prose_codes():
    """Every tier code the sentence of §FS-check.3.14.5 names."""
    lines = CHECK_PAGE.read_text(encoding="utf-8").splitlines()
    start = next(index for index, line in enumerate(lines) if COMPOUND_CODE_HEADING.match(line))
    body = []
    for line in lines[start + 1 :]:
        if HEADING.match(line):
            break
        body.append(line)
    return set(PROSE_CODE.findall("\n".join(body)))


def _catalog_codes():
    """Every tier code carrying a row in the catalog of §FS-errors.5.5."""
    return {code for code in _catalog_rows() if code.startswith(PREFIX)}


class OutOfScopeCodeFamilyTests(unittest.TestCase):
    maxDiff = None  # the failure is the difference between two lists of codes

    def test_the_prose_names_every_code_the_match_mints(self):
        self.assertEqual(
            _minted_codes(),
            _prose_codes(),
            "§FS-check.3.14.5's sentence and tag_out_of_scope's match arms disagree",
        )

    def test_the_catalog_has_a_row_for_every_code_the_match_mints(self):
        self.assertEqual(
            _minted_codes(),
            _catalog_codes(),
            "the §FS-errors.5.5 catalog and tag_out_of_scope's match arms disagree",
        )

    def test_the_catalog_and_the_prose_name_the_same_codes(self):
        self.assertEqual(
            _prose_codes(),
            _catalog_codes(),
            "§FS-check.3.14.5's sentence and the §FS-errors.5.5 catalog disagree",
        )

    def test_the_family_is_read_from_all_three_places(self):
        """A parser that silently found nothing would make the three comparisons
        above agree on the empty set."""
        for name, codes in (
            ("tag_out_of_scope", _minted_codes()),
            ("FS-check.3.14.5", _prose_codes()),
            ("the FS-errors.5.5 catalog", _catalog_codes()),
        ):
            self.assertNotEqual(set(), codes, f"read no {PREFIX}* code out of {name}")


if __name__ == "__main__":
    unittest.main()
