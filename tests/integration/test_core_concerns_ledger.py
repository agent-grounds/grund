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
The same rule holds the phases of §DISC-core-concerns.9: a phase whose lead
opens `Not shipped` names no release it is planned for at or below the last one
shipped, because a phase planned for a release that is out has either shipped
or slipped, and its lead says which.

No architecture point describes this repository's planning documents, so this
test cites the discussion points it holds them to instead. The two tests that
read the documents landed under `@unittest.expectedFailure`, one commit before
the reconciliation, because the pre-commit hook runs this suite and a plainly
failing test could not be committed without `--no-verify`. An expected failure
that passes fails the suite, so the commit that reconciles the plan takes the
decorators off. The phase check landed the same way for `agent-grounds/grund#465`.
"""

import re
import subprocess
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))
from deprecation_ledger import (
    LEDGER_HEADER, ledger_rows, planned_notice_due, section, unshipped_phases, version,
)


DISCUSSION = (
    REPO_ROOT / "docs" / "discussions" / "proposals" / "2026-09-30-core-concerns.md"
)
CHANGELOG = REPO_ROOT / "docs" / "changelog.md"

ITEM = re.compile(r"^(\d+)\. ", re.MULTILINE)
OWNED = re.compile(r"Open — owner #\d+|Settled by")
STATUSES = ("planned", "shipped")
TAG = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
RELEASED = re.compile(
    r"^(?:## \d+\. |- )\[(\d+)\.(\d+)\.(\d+)\]", re.MULTILINE
)


def items(body):
    """The numbered list items of a section body, each with its number."""
    starts = list(ITEM.finditer(body))
    found = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(body)
        found.append((match.group(1), body[match.start():end]))
    return found


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
    elif planned_notice_due(notice, status, last_shipped):
        problems.append(
            f"{surface}: notice {notice_cell} is at or below the last shipped release "
            f"{'.'.join(map(str, last_shipped))} but the row is not marked shipped"
        )
    return problems


def phase_problems(text, last_shipped):
    """Each §DISC-core-concerns.9 phase still `Not shipped` for a release that is out."""
    return [
        f"{phase}: not shipped but planned for {planned}, at or below the last shipped "
        f"release {'.'.join(map(str, last_shipped))}"
        for phase, planned in unshipped_phases(text, last_shipped)
    ]


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

    def test_no_phase_is_still_planned_for_a_release_that_shipped(self):
        self.assertIsNotNone(section(self.text, "9"), "§DISC-core-concerns.9 is missing")
        self.assertEqual([], phase_problems(self.text, last_shipped_release()))

    def test_a_phase_is_held_to_what_shipped(self):
        last = (0, 18, 0)

        def problems(lead, title="Phase 1: the engine"):
            text = f"## 9. The releases\n\n### 9.2 {title}\n\n{lead}\n\nMore.\n"
            return phase_problems(text, last)

        self.assertEqual([], problems("Not shipped. Planned for `0.19.0`."))
        self.assertEqual([], problems("Shipped in `0.17.0`."))
        self.assertEqual([], problems("Not shipped. Adoption is planned for `0.18.x`."))
        self.assertEqual([], problems("Not shipped. Planned for `0.17.0`.", "`1.0.0`"))
        self.assertEqual([], problems("Not shipped. Planned for `0.19.0`, beside `0.16.1`'s ramp."))
        self.assertTrue(problems("Not shipped. Planned for `0.17.0`, the notice release."))
        self.assertTrue(problems("Not shipped. Planned for `0.18.0`."))
        self.assertTrue(problems("Not shipped. Adoption is planned for `0.17.x`."))
        self.assertTrue(problems("Not shipped. The removals are for `0.19.0` and `0.18.0`."))

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
