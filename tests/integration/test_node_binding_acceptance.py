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
# §AR-ci.3.4.1.1: what a build line starts with; run_python_gate.py holds the same marker.
BUILD_LINE = "node build: "


def build_lines(stderr):
    """The acceptance's per-build lines (§AR-ci.3.4.1.1), the only output a green run shows.

    A verbose runner writes `test ... ` before a test's output, so a line may begin
    with that; each build line runs from its marker to the end of the line.
    """
    return "".join(line[line.index(BUILD_LINE):] for line in stderr.splitlines(keepends=True)
                   if BUILD_LINE in line)


class NodeAcceptanceWorkflowTests(unittest.TestCase):
    def test_rehearsals_prepare_each_source_once_and_isolate_consumers(self):
        """§AR-ci.3.4.1.4: actual npm preparation, private bytes and same-version sources."""
        result = subprocess.run([sys.executable, str(Path(__file__).with_name(
            "node_consumer_fixture.py"))], cwd=REPO, capture_output=True, text=True)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_node_acceptance_is_in_the_complete_local_gate(self):
        result = subprocess.run(
            [sys.executable, "tests/bindings/node/run.py"], cwd=REPO,
            capture_output=True, text=True,
        )
        # §AR-ci.3.4.1.1: the capture above hides them, so they go to the gate, which shows a
        # green test's build lines (§AR-ci.requirements.5.1).
        sys.stderr.write(build_lines(result.stderr))
        sys.stderr.flush()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
