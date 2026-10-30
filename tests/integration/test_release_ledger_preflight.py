"""§FS-distribution.4.2.2.1: refuse what the plan still owes a release before candidates.

Candidate headings exercise the existing §DISC-core-concerns.9.9 validator,
without duplicating its rule. Guard probes restore a planned notice, or a
§DISC-core-concerns.9 phase lead still `Not shipped`, in a copy of the real
plan, so reconciling today's statuses alone cannot satisfy them.

The phase probes are agent-grounds/grund#1116: the guard passed 0.21.0 while the
9.4 lead still planned it, and only candidate CI refused it. The refusing ones
landed under `@unittest.expectedFailure`, one commit before the guard reads the
phase leads, because the pre-commit hook runs this suite; an expected failure
that passes fails the suite, so the commit that teaches the guard takes them off.
"""

import re
import sys
import unittest

import test_core_concerns_ledger as ledger
from release_version_fixture import (
    LEDGER, REPO, copy_tree, release_heading, run, set_version, validate_ledger,
)


def noticed_rows(text):
    """Identify the two #888 surfaces, not a parallel table of deadlines."""
    rows = ledger.ledger_rows(ledger.section(text, "9.9"))
    return [row for row in rows if row[0].startswith(
        ("the 30 integrations root names", "building a promised record"))]


class CandidateLedgerTests(unittest.TestCase):
    """§FS-distribution.4.4.1: the release marker stays on the dev candidate."""

    def check_candidate(self, version):
        root = copy_tree(self)
        rows = noticed_rows((root / LEDGER).read_text(encoding="utf-8"))
        self.assertEqual(2, len(rows), "both reported surfaces must remain in the ledger")
        for row in rows:
            self.assertEqual(["`0.19.0`", "`0.21.0`"], row[1:3],
                             "reconcile status without moving notice or removal dates")
        release_heading(root, "0.19.0")
        result = set_version(root, version)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        result = validate_ledger(root)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_0_19_0_release_candidate_ledger_is_consistent(self):
        self.check_candidate("0.19.0")

    def test_0_19_1_dev_candidate_retains_consistent_released_ledger(self):
        self.check_candidate("0.19.1-dev")


class LedgerPreflightTests(unittest.TestCase):
    """§FS-distribution.4.2.2: use ledger data, independently of message bounds."""

    def guard(self, release, *, notice="0.19.0", status="planned"):
        root = copy_tree(self)
        path = root / LEDGER
        text = path.read_text(encoding="utf-8")
        selected = noticed_rows(text)
        self.assertEqual(2, len(selected))
        lines, changed = [], 0
        for line in text.splitlines(keepends=True):
            if line.startswith("|"):
                cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
                if cells in selected:
                    # Mark the other notice shipped: exactly one deliberate
                    # planned row must account for the guard's refusal.
                    cells[3] = "shipped"
                    if cells[0] == selected[0][0]:
                        cells[1], cells[2], cells[3] = f"`{notice}`", "`0.22.0`", status
                    line = "| " + " | ".join(cells) + " |\n"
                    changed += 1
            lines.append(line)
        self.assertEqual(2, changed)
        path.write_text("".join(lines), encoding="utf-8")
        result = run(root, sys.executable, REPO / "scripts/check_release_ramps.py",
                     release, "--root", root)
        return result

    def check_due(self, release):
        result = self.guard(release)
        self.assertEqual(1, result.returncode, result.stdout + result.stderr)
        self.assertIn(LEDGER, result.stderr)
        self.assertIn("0.19.0", result.stderr)
        self.assertRegex(result.stderr, re.compile("planned|not.*shipped"))

    def test_preflight_refuses_planned_notice_at_requested_release(self):
        self.check_due("0.19.0")

    def test_preflight_refuses_planned_notice_before_requested_release(self):
        self.check_due("0.19.1")

    def test_preflight_accepts_future_planned_notice(self):
        result = self.guard("0.19.0", notice="0.20.0")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_preflight_accepts_shipped_notice(self):
        result = self.guard("0.19.0", status="shipped")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


# §DISC-core-concerns.9.4's lead on main at b8e5dfa210, before #1115 shipped it.
UNSHIPPED_LEAD = (
    "Not shipped in full. Adoption is done and the `0.19.0` removals shipped; "
    "what remains is the removals planned for `0.21.0`."
)
PHASE_HEADING = re.compile(r"^### 9\.4 Phase 3:[^\n]*\n\n", re.MULTILINE)


class PhasePreflightTests(unittest.TestCase):
    """§FS-distribution.4.2.2.1: a phase lead still `Not shipped` for the release refuses it."""

    def guard(self, release, lead=None):
        """Run the guard on a copy of the tree, its 9.4 lead replaced by `lead`."""
        root = copy_tree(self)
        path = root / LEDGER
        text = path.read_text(encoding="utf-8")
        heading = PHASE_HEADING.search(text)
        self.assertIsNotNone(heading, "§DISC-core-concerns.9.4 must stay Phase 3")
        if lead is None:
            self.assertTrue(text[heading.end():].startswith("Shipped in `0.21.0`."),
                            "the real 9.4 lead records Phase 3 as shipped in 0.21.0")
        else:
            end = text.index("\n\n", heading.end())
            path.write_text(text[:heading.end()] + lead + text[end:], encoding="utf-8")
        return run(root, sys.executable, REPO / "scripts/check_release_ramps.py",
                   release, "--root", root)

    def check_refused(self, release, lead=UNSHIPPED_LEAD, planned="0.21.0"):
        result = self.guard(release, lead)
        self.assertEqual(1, result.returncode, result.stdout + result.stderr)
        self.assertIn(LEDGER, result.stderr)
        self.assertIn("9.4", result.stderr)
        self.assertIn(planned, result.stderr)

    def check_accepted(self, release, lead=None):
        result = self.guard(release, lead)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_preflight_refuses_phase_planned_for_requested_release(self):
        self.check_refused("0.21.0")

    def test_preflight_refuses_phase_planned_before_requested_release(self):
        self.check_refused("0.21.1")

    def test_preflight_refuses_phase_series_once_requested_release_is_past_it(self):
        self.check_refused("0.22.0", "Not shipped. Planned for `0.21.x`.", "0.21.x")

    def test_preflight_accepts_phase_planned_after_requested_release(self):
        self.check_accepted("0.20.1", UNSHIPPED_LEAD)

    def test_preflight_accepts_phase_series_while_requested_release_is_in_it(self):
        self.check_accepted("0.21.1", "Not shipped. Planned for `0.21.x`.")

    def test_preflight_accepts_shipped_phase_at_its_release(self):
        self.check_accepted("0.21.0")

    def test_preflight_accepts_phase_planned_for_later_release(self):
        self.check_accepted("0.21.0", UNSHIPPED_LEAD.replace("`0.21.0`", "`0.22.0`"))


if __name__ == "__main__":
    unittest.main()
