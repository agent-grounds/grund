"""§DISC-core-concerns.10 and §DISC-core-concerns.9.9 — the 1.0 plan says who
decides each open choice and when each deprecated surface goes, and says it
against the releases that actually shipped.

Every numbered item of §DISC-core-concerns.10.1 and §DISC-core-concerns.10.2
carries either `Open — owner #N`, naming the ticket that rules on it, or
`Settled by`, naming what settled it. Every row of the deprecation ledger in
§DISC-core-concerns.9.9 removes its surface at least one minor after its notice
(§REQ-backwards-compatibility.2, §DISC-core-concerns.9.7), and no row places a
planned notice in a release at or below the last one shipped. The last shipped
release is read from the repository — its `v*` tags, and the released headings
of `docs/changelog.md` for a clone fetched without tags — never written here.

No architecture point describes this repository's planning documents, so this
test cites the discussion points it holds them to instead. The two tests that
read the documents landed under `@unittest.expectedFailure`, one commit before
the reconciliation, because the pre-commit hook runs this suite and a plainly
failing test could not be committed without `--no-verify`. An expected failure
that passes fails the suite, so the commit that reconciles the plan takes the
decorators off.
"""

import re
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DISCUSSION = (
    REPO_ROOT / "docs" / "discussions" / "proposals" / "2026-09-30-core-concerns.md"
)
CHANGELOG = REPO_ROOT / "docs" / "changelog.md"

HEADING = re.compile(r"^(#{2,3}) (\d+(?:\.\d+)?)[. ]", re.MULTILINE)
ITEM = re.compile(r"^(\d+)\. ", re.MULTILINE)
OWNED = re.compile(r"Open — owner #\d+|Settled by")
VERSION = re.compile(r"`?v?(\d+)\.(\d+)\.(\d+)`?")
LEDGER_HEADER = ("surface", "notice", "removal", "status")
STATUSES = ("planned", "shipped")
TAG = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
RELEASED = re.compile(
    r"^(?:## \d+\. |- )\[(\d+)\.(\d+)\.(\d+)\]", re.MULTILINE
)


def section(text, number):
    """The body of section `number`, up to the next heading of any depth."""
    for match in HEADING.finditer(text):
        if match.group(2) == number:
            rest = text[match.end():]
            following = HEADING.search(rest)
            return rest[: following.start()] if following else rest
    return None


def items(body):
    """The numbered list items of a section body, each with its number."""
    starts = list(ITEM.finditer(body))
    found = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(body)
        found.append((match.group(1), body[match.start():end]))
    return found


def version(cell):
    match = VERSION.fullmatch(cell.strip())
    return tuple(int(part) for part in match.groups()) if match else None


def ledger_rows(body):
    """The rows of the table whose header is Surface | Notice | Removal | Status."""
    rows, inside = [], False
    for line in body.splitlines():
        if not line.startswith("|"):
            if inside:
                break
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if not inside:
            inside = tuple(cell.lower() for cell in cells) == LEDGER_HEADER
            continue
        if all(set(cell) <= set("-: ") for cell in cells):
            continue
        rows.append(cells)
    return rows


def row_problems(cells, last_shipped):
    """What is wrong with one ledger row, as sentences; empty when nothing is."""
    if len(cells) != len(LEDGER_HEADER):
        return [f"{cells}: a row has {len(LEDGER_HEADER)} cells"]
    surface, notice_cell, removal_cell, status = cells
    notice, removal = version(notice_cell), version(removal_cell)
    problems = []
    if notice is None or removal is None:
        problems.append(f"{surface}: notice and removal must be release versions")
    elif removal[:2] <= notice[:2]:
        problems.append(
            f"{surface}: removal {removal_cell} is not a minor after notice {notice_cell}"
        )
    if status not in STATUSES:
        problems.append(f"{surface}: status {status!r} is not one of {STATUSES}")
    elif notice is not None and notice <= last_shipped and status != "shipped":
        problems.append(
            f"{surface}: notice {notice_cell} is at or below the last shipped release "
            f"{'.'.join(map(str, last_shipped))} but the row is not marked shipped"
        )
    return problems


def last_shipped_release():
    """The newest release the repository records: its tags, then its changelog."""
    found = [
        tuple(int(part) for part in match.groups())
        for match in RELEASED.finditer(CHANGELOG.read_text(encoding="utf-8"))
    ]
    try:
        tags = subprocess.run(
            ["git", "tag", "--list", "v*"],
            cwd=REPO_ROOT, capture_output=True, text=True, check=True,
        ).stdout.split()
    except (OSError, subprocess.CalledProcessError):
        tags = []
    found += [
        tuple(int(part) for part in match.groups())
        for tag in tags if (match := TAG.match(tag))
    ]
    return max(found)


class CoreConcernsLedgerTest(unittest.TestCase):
    def setUp(self):
        self.text = DISCUSSION.read_text(encoding="utf-8")

    def test_every_open_choice_names_its_owner_or_what_settled_it(self):
        problems = []
        for number in ("10.1", "10.2"):
            body = section(self.text, number)
            self.assertIsNotNone(body, f"DISC-core-concerns.{number} is missing")
            listed = items(body)
            self.assertTrue(listed, f"DISC-core-concerns.{number} lists no choice")
            problems += [
                f"{number}({item}) carries neither `Open — owner #N` nor `Settled by`"
                for item, text in listed if not OWNED.search(text)
            ]
        self.assertEqual([], problems)

    def test_every_ledger_row_removes_after_its_notice_and_after_what_shipped(self):
        body = section(self.text, "9.9")
        self.assertIsNotNone(body, "§DISC-core-concerns.9.9 is missing")
        rows = ledger_rows(body)
        self.assertTrue(rows, "§DISC-core-concerns.9.9 carries no ledger row")
        last = last_shipped_release()
        self.assertEqual([], [p for cells in rows for p in row_problems(cells, last)])

    def test_the_last_shipped_release_is_read_from_the_repository(self):
        self.assertGreaterEqual(last_shipped_release(), (0, 16, 1))

    def test_a_row_is_held_to_both_bounds(self):
        last = (0, 16, 1)
        self.assertEqual([], row_problems(["`A`", "`0.17.0`", "`0.19.0`", "planned"], last))
        self.assertEqual([], row_problems(["`A`", "`0.16.0`", "`0.17.0`", "shipped"], last))
        self.assertTrue(row_problems(["`A`", "`0.17.0`", "`0.17.3`", "planned"], last))
        self.assertTrue(row_problems(["`A`", "`0.16.0`", "`0.19.0`", "planned"], last))
        self.assertTrue(row_problems(["`A`", "`0.16.1`", "`0.19.0`", "planned"], last))
        self.assertTrue(row_problems(["`A`", "none", "`0.19.0`", "planned"], last))
        self.assertTrue(row_problems(["`A`", "`0.17.0`", "`0.19.0`", "maybe"], last))

    def test_the_ledger_table_is_found_by_its_header(self):
        body = (
            "Lead.\n\n| Surface | Notice | Removal | Status |\n|---|---|---|---|\n"
            "| `A` | `0.17.0` | `0.19.0` | planned |\n\nAfter.\n| x | y |\n"
        )
        self.assertEqual([["`A`", "`0.17.0`", "`0.19.0`", "planned"]], ledger_rows(body))
        self.assertEqual([], ledger_rows("| Release | What it carries |\n|---|---|\n| a | b |\n"))


if __name__ == "__main__":
    unittest.main()
