"""§DA-one-fact-one-point-one-implementation.consequences — the rulings D1–D20
that the one-answer work (§AR-system.principle) is built from are each recorded
once, in one decision record per step family, and each ruling that moves a
verdict names the compatibility route it ships by. The records it holds are
§DF-one-answer-rulings-run, §DF-one-answer-rulings-parse,
§DF-one-answer-rulings-stubs, §DF-one-answer-rulings-fmt-editor and
§DF-one-answer-rulings-grammar.

The routes are the ones the #576 agora settled (V-A02–V-A26) under the author's
reading: a ruling flips in one release only where an existing requirement
already forbids the old verdict, and its record names that requirement; every
other verdict move takes the warning release. A correction route here is a
*candidate*: the correcting step's own decision record proves the five
conditions and carries the release note. So these records name the correction
route escaped, never as a live citation, which would make the
backward-compatibility meter demand those proofs of them now, and they carry no
`release-note` section, which would publish a notice for a verdict that has not
moved yet (§FS-distribution.4.6.4).

The tests landed under `@unittest.expectedFailure`, one commit before the
records, because the pre-commit hook runs this suite; the commit that wrote the
records took the decorators off.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FUNCTIONAL = REPO_ROOT / "docs" / "decisions" / "functional"
PRINCIPLE_DECISION = (
    REPO_ROOT
    / "docs"
    / "decisions"
    / "architectural"
    / "DA-one-fact-one-point-one-implementation.md"
)
MARKER = "§"
ESCAPE = "<" + MARKER + ">"
ROUTE = "REQ-backwards-compatibility"

# One record per step family of the plan approved on #576, and its rulings.
# G01 and G02 are the token corrections of V-A26, recorded with the grammar.
GROUPS = {
    "DF-one-answer-rulings-run": ["D1", "D5", "D6", "D7", "D14"],
    "DF-one-answer-rulings-parse": [
        "D9", "D17a", "D17b", "D17c", "D17d", "D17e", "D17f",
    ],
    "DF-one-answer-rulings-stubs": ["D2", "D3", "D4", "D8", "D13", "D15", "D16"],
    "DF-one-answer-rulings-fmt-editor": ["D17g"],
    "DF-one-answer-rulings-grammar": ["D10", "D11", "D18", "D20", "G01", "G02"],
}
# Process and structure rulings move nothing; each need only be named, in one
# of the records above or in the principle's own decision record.
NAMED_ANYWHERE = ["D12", "D19"]

# The warning release (route .2): every ruling, or part of a split ruling, that moves
# a verdict no existing requirement forbade. V-A02–V-A08, V-A10, V-A12, V-A16,
# V-A17, V-A18 (the TOC title), V-A20 (the wrapper), V-A21, V-A23.
WARNING = {
    "D1", "D14", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D10", "D16",
    "D17a", "D17b", "D17c", "D17d", "D17e", "D17g", "D18", "D20",
}
# The correction candidates (route .5) and the prior prohibition each rests on; any
# one of the listed points will do. V-A05, V-A08, V-A09, V-A10, V-A18, V-A19,
# V-A20, V-A21, V-A26. D9, D11, D12, D13, D15 and D19 move no verdict on `main`
# (V-A11, V-A13, V-A14, V-C04, V-A15, V-A22) and appear in neither table.
CORRECTION = {
    "D3": {"REQ-no-wrong-citation.1", "REQ-no-wrong-citation.2"},
    "D6": {"REQ-no-wrong-citation.1", "REQ-no-wrong-citation.2"},
    "D7": {"REQ-no-wrong-citation.1", "REQ-no-wrong-citation.2"},
    "D8": {"REQ-no-wrong-citation.1"},
    "D17e": {"REQ-no-wrong-citation.3"},
    "D17f": {"REQ-no-missed-citation.2"},
    "D17g": {"REQ-no-data-loss.2"},
    "D18": {"REQ-no-wrong-citation.1"},
    "G01": {"REQ-no-wrong-citation.1"},
    "G02": {"REQ-no-missed-citation.2"},
}

WARNING_RE = re.compile(re.escape(ROUTE) + r"\.2(?!\.?\d)")
ESCAPED_CORRECTION_RE = re.compile(re.escape(ESCAPE + ROUTE) + r"\.5(?!\.?\d)")
LIVE_CORRECTION_RE = re.compile(
    r"(?<!<)" + re.escape(MARKER + ROUTE) + r"\.5(?!\.?\d)"
)
RANGE_RE = re.compile(r"D17([a-g])\s*[–-]\s*(?:D17)?([a-g])\b")
LIST_ITEM_RE = re.compile(r"^\s*(?:[-*+]|\d+[.)])\s")


def _expand(text):
    """Write `D17a–d` as `D17a D17b D17c D17d`, so a range names each part."""
    return RANGE_RE.sub(
        lambda m: " ".join(
            f"D17{chr(c)}" for c in range(ord(m.group(1)), ord(m.group(2)) + 1)
        ),
        text,
    )


def _names(block, ruling):
    return re.search(rf"(?<![\w-]){re.escape(ruling)}(?!\w)", block) is not None


def _blocks(text):
    """Paragraphs, with each table row and each list item a block of its own,
    so a route stated for one ruling is never read as stated for its neighbour."""
    out = []
    for paragraph in re.split(r"\n\s*\n", text):
        current = []
        for line in paragraph.splitlines():
            if line.lstrip().startswith("|"):
                out.append(line)
                continue
            if LIST_ITEM_RE.match(line) and current:
                out.append("\n".join(current))
                current = []
            current.append(line)
        if current:
            out.append("\n".join(current))
    return out


def _record(record_id):
    path = FUNCTIONAL / f"{record_id}.md"
    if not path.is_file():
        raise AssertionError(f"{record_id}: no decision record at {path}")
    return path.read_text(encoding="utf-8")


class RulingsRecordedTest(unittest.TestCase):
    def test_each_group_has_one_accepted_indexed_record_naming_its_rulings(self):
        index = (FUNCTIONAL / "README.md").read_text(encoding="utf-8")
        gaps = []
        for record_id, rulings in GROUPS.items():
            text = _record(record_id)
            if not text.startswith(f"# {record_id}:"):
                gaps.append(f"{record_id}: the H1 does not declare it")
            if not re.search(r"^\*\*Status:\*\* Accepted\b", text, re.M):
                gaps.append(f"{record_id}: status is not Accepted")
            if f"[{MARKER}{record_id}]({record_id}.md#" not in index:
                gaps.append(f"{record_id}: no line in the functional index")
            if not re.search(re.escape(MARKER) + r"(?:FS|GOAL)-[a-z0-9-]+", text):
                gaps.append(f"{record_id}: cites no FS or GOAL point")
            expanded = _expand(text)
            gaps.extend(
                f"{record_id}: does not name {ruling}"
                for ruling in rulings
                if not _names(expanded, ruling)
            )
        texts = [_expand(_record(r)) for r in GROUPS]
        texts.append(PRINCIPLE_DECISION.read_text(encoding="utf-8"))
        gaps.extend(
            f"{ruling}: named in no ruling record nor the principle's DA"
            for ruling in NAMED_ANYWHERE
            if not any(_names(t, ruling) for t in texts)
        )
        self.assertEqual(gaps, [])

    def test_each_verdict_moving_ruling_names_its_route(self):
        gaps = []
        for record_id, rulings in GROUPS.items():
            blocks = _blocks(_expand(_record(record_id)))
            for ruling in rulings:
                named = [b for b in blocks if _names(b, ruling)]
                if ruling in WARNING and not any(WARNING_RE.search(b) for b in named):
                    gaps.append(f"{record_id}: {ruling} names no warning-release route")
                if ruling not in CORRECTION:
                    continue
                prohibitions = CORRECTION[ruling]
                if not any(
                    ESCAPED_CORRECTION_RE.search(b)
                    and any(
                        re.search(re.escape(MARKER + p) + r"(?!\.?\d)", b)
                        for p in prohibitions
                    )
                    for b in named
                ):
                    gaps.append(
                        f"{record_id}: {ruling} names no correction route resting "
                        f"on {' or '.join(sorted(prohibitions))}"
                    )
        self.assertEqual(gaps, [])

    def test_no_ruling_record_cites_the_correction_route_live(self):
        offenders = [
            record_id
            for record_id in GROUPS
            if LIVE_CORRECTION_RE.search(_record(record_id))
        ]
        self.assertEqual(offenders, [])

    def test_no_ruling_record_carries_a_release_note_or_a_numbered_chapter(self):
        offenders = []
        for record_id in GROUPS:
            text = _record(record_id)
            if re.search(r"^## release-note\b", text, re.M):
                offenders.append(f"{record_id}: carries a release-note section")
            if re.search(r"^## \d", text, re.M):
                offenders.append(f"{record_id}: has a numbered top-level section")
        self.assertEqual(offenders, [])


if __name__ == "__main__":
    unittest.main()
