"""§DISC-core-concerns.10.2 — grund's own repository states the shape of its
FS, AR and decision records as checked fields, not as prose convention.

Item 2 of that ledger settles the decision records: every `DF` and `DA` record
opens with a `status` field (§FS-fields.identity.name) holding `Accepted` or
`Superseded`, a scalar whose value is its heading's display name
(§FS-fields.content.one-of), and no record keeps the declaration-level
`**Status:**` prose line it replaced. `grund check` holds the field once
`grund.toml` declares it, but it cannot see that last case: a prose line in the
lead is legal beside the field, so this test is what retires it. A `**Status:**`
line under a numbered chapter is the status of a sub-decision, not of the
record, and stays — `DF-require-grounding` carries one in its chapter 4. The
conversion keeps each record's meaning: a record superseded by another is
`Superseded`, and `DF-configured-title-metadata` keeps its qualification as the
prose of its status chapter.

Item 1 settles the Terms chapter: §RULE-terms stays beside an `FS.terms` field
that holds the chapter first and titled Terms, and the AR pages take the same
shape behind a placement chapter. The config test reads those fields, and the
status fields at `must`, through the version-2 tables of `grund.toml`.

The tests that read the records and the config landed under
`@unittest.expectedFailure`, one commit before the adoption, because the
pre-commit hook runs this suite; the commits that adopt v2 and convert the
records take the decorators off.
"""

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONFIG = REPO_ROOT / "grund.toml"
HOMES = (
    REPO_ROOT / "docs" / "decisions" / "functional",
    REPO_ROOT / "docs" / "decisions" / "architectural",
)
ADMITTED = ("Accepted", "Superseded")
QUALIFIED = {
    "DF-configured-title-metadata": (
        "By issue 243's checkpoint-10 supervisory ruling; show ordering "
        "corrected by the checkpoint-18 operator ruling."
    ),
}

FENCE = re.compile(r"^(`{3,}|~{3,})")
CHAPTER = re.compile(r"^## (\S+?)(?::)? (.*)$")
PROSE_STATUS = re.compile(r"^\*\*Status:\*\*")
SUPERSEDED_BY = re.compile(r"^\*\*Superseded by:\*\*")


def unfenced_lines(text):
    """The lines of a Markdown text outside fenced code blocks."""
    lines, fence = [], None
    for line in text.splitlines():
        opened = FENCE.match(line)
        if fence is None and opened:
            fence = opened.group(1)
        elif fence is not None and line.startswith(fence):
            fence = None
        elif fence is None:
            lines.append(line)
    return lines


def shape(text):
    """The record's lead lines, and its first chapter as (name, title, body)."""
    lines = unfenced_lines(text)
    starts = [i for i, line in enumerate(lines) if CHAPTER.match(line)]
    if not starts:
        return lines[1:], None
    first = starts[0]
    name, title = CHAPTER.match(lines[first]).groups()
    end = starts[1] if len(starts) > 1 else len(lines)
    body = " ".join(line.strip() for line in lines[first + 1:end] if line.strip())
    return lines[1:first], (name, title.strip(), body)


def problems(record, text):
    """What is wrong with one record's status, as sentences; empty when nothing is."""
    lead, first = shape(text)
    found = []
    if any(PROSE_STATUS.match(line) for line in lead):
        found.append(f"{record}: the lead still carries a `**Status:**` prose line")
    if first is None or first[0] != "status":
        found.append(f"{record}: the first chapter is not `## status:`")
        return found
    _, value, body = first
    if value not in ADMITTED:
        found.append(f"{record}: status {value!r} is not one of {ADMITTED}")
    superseded = any(SUPERSEDED_BY.match(line) for line in lead)
    if superseded and value != "Superseded":
        found.append(f"{record}: names what supersedes it but its status is {value!r}")
    if record in QUALIFIED and body != QUALIFIED[record]:
        found.append(f"{record}: the status chapter lost its qualification: {body!r}")
    return found


def records():
    return sorted(
        path for home in HOMES for path in home.glob("D[FA]-*.md")
    )


class DecisionStatusFieldTest(unittest.TestCase):
    def test_every_record_opens_with_an_admitted_status_field(self):
        found = [
            problem
            for path in records()
            for problem in problems(path.stem, path.read_text(encoding="utf-8"))
        ]
        self.assertEqual([], found)

    @unittest.expectedFailure
    def test_the_config_declares_the_shapes_at_must(self):
        config = tomllib.loads(CONFIG.read_text(encoding="utf-8"))
        self.assertEqual(2, config.get("grund_config_version"))
        kinds = config.get("schema", {}).get("kinds", {})
        fields = {kind: kinds.get(kind, {}).get("fields", {}) for kind in kinds}
        self.assertEqual(
            {"title": "Terms", "presence": "must", "position": "first"},
            fields.get("FS", {}).get("terms"),
        )
        self.assertEqual(
            {"presence": "must", "position": "first"},
            fields.get("AR", {}).get("placement"),
        )
        self.assertEqual(
            {"title": "Terms", "presence": "must", "after": "placement"},
            fields.get("AR", {}).get("terms"),
        )
        for kind in ("DF", "DA"):
            self.assertEqual(
                {"one_of": list(ADMITTED), "presence": "must", "position": "first"},
                fields.get(kind, {}).get("status"),
                kind,
            )

    def test_the_records_exist(self):
        self.assertGreater(len(records()), 100)

    def test_a_record_is_held_to_its_status_shape(self):
        good = "# DF-x: X\n\n**Date:** 2026-10-15\n\n## status: Accepted\n\n## 1. Context\n"
        self.assertEqual([], problems("DF-x", good))
        sub = good + "\n## 4. Part\n\n**Status:** Accepted\n"
        self.assertEqual([], problems("DF-x", sub))
        fenced = "# DF-x: X\n\n```\n## 1. Not a chapter\n```\n\n## status: Superseded\n"
        self.assertEqual([], problems("DF-x", fenced))
        self.assertTrue(problems("DF-x", "# DF-x: X\n\n**Status:** Accepted\n\n## 1. Context\n"))
        self.assertTrue(problems("DF-x", good.replace("Accepted", "Maybe")))
        self.assertTrue(problems("DF-x", "# DF-x: X\n\n## 1. Context\n\n## status: Accepted\n"))
        self.assertTrue(problems(
            "DF-x", "# DF-x: X\n\n**Status:** Accepted\n\n## status: Accepted\n"
        ))
        replaced = "# DF-x: X\n\n**Superseded by:** DF-y\n\n## status: Accepted\n"
        self.assertTrue(problems("DF-x", replaced))
        self.assertTrue(problems("DF-configured-title-metadata", good))


if __name__ == "__main__":
    unittest.main()
