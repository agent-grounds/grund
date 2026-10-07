"""§AR-bindings.5 — the checked workflow executes the Node consumer contract.

Exact-leaf proof routes, exercised by the suite below:
§FS-distribution.3.0.3: complete-data and separate CLI-byte parity.
§FS-distribution.3.0.3.1: Node field mapping and the frozen CLI wire projection.
§FS-distribution.3.2.1.1: full export inventory and excluded process/editor APIs.
§FS-distribution.3.2.2.1: structured Promise failures and nullable records.
§FS-distribution.3.2.2.2: all operation-specific results.
§FS-distribution.3.2.2.3: scan snapshots and native-only corpus fields.
§FS-distribution.3.2.2.4: closed options and writer validation.
§FS-distribution.3.2.3.1: owned workers, runtime barriers and lease cleanup.
§FS-distribution.3.2.3.2: modules, metadata and lazy structured loading.
§FS-distribution.3.2.3.3: local source/pack/install and bundled fallback.
§FS-distribution.3.2.4: executable acceptance and honest readiness.
"""

import subprocess
import sys
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]


class NodeAcceptanceWorkflowTests(unittest.TestCase):
    def test_node_acceptance_is_in_the_complete_local_gate(self):
        result = subprocess.run(
            [sys.executable, "tests/bindings/node/run.py"], cwd=REPO,
            capture_output=True, text=True,
        )
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
