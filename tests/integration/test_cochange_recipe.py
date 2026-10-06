"""Collect black-box recipe cases in the normal gate (§FS-cochange-recipe.examples).

The scenario modules live in the e2e home: these are Git/process acceptance tests.
EvidenceTests pins comparison/obligation/waiver vocabulary (§FS-cochange-recipe.terms)
and both related edit classes for one target (§FS-cochange-recipe.evidence).
HistoryTests pins commit-local trailer scopes (§FS-cochange-recipe.waivers)
and same-tree entry-point results (§FS-cochange-recipe.exit).
RefusalTests pins policy validation (§FS-cochange-recipe.inputs) and exact
snapshot/query refusals (§FS-cochange-recipe.snapshots).
SnapshotEdgeTests pins deterministic JSON refusals (§FS-cochange-recipe.output).
"""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "e2e"))
from cochange_evidence import EvidenceTests
from cochange_history import HistoryTests
from cochange_refusals import RefusalTests, WorkspaceTests
from cochange_compatibility import CompatibilityTests
from cochange_snapshot_edges import SnapshotEdgeTests


if __name__ == "__main__":
    unittest.main()
