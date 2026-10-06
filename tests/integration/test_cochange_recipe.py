"""Collect black-box recipe cases in the normal gate (§FS-cochange-recipe.examples).

The scenario modules live in the e2e home: these are Git/process acceptance tests.
"""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "e2e"))
from cochange_evidence import EvidenceTests
from cochange_history import HistoryTests
from cochange_refusals import RefusalTests, WorkspaceTests
from cochange_compatibility import CompatibilityTests


if __name__ == "__main__":
    unittest.main()
