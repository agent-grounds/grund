"""§AR-ci.3.4 — the Python gate prepares its own inputs, the binding oracle among them.

`scripts/run_python_gate.py` is asked for its environment the way a contributor's
checkout asks: no GRUND_BINDINGS_ORACLE and no CI. The oracle it hands its discovery
run must be the same-source one the Node parity tests compare against
(§FS-distribution.3.0.3), built from this checkout's HEAD. The variable is removed
explicitly, because under the gate itself the wrapper has already set it.

GRUND_BUILT and GRUND_RELEASED point at stand-ins in a temporary directory: the
wrapper only resolves the first and asks the second for its version, and neither is
what this test is about. The oracle build lands wherever the wrapper puts it, so a
gate run that already built it pays nothing more here.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
# Each of these would make the child something other than a local run.
NOT_LOCAL = ("GRUND_BINDINGS_ORACLE", "CI", "GITHUB_ACTIONS", "GITHUB_ENV")
CHILD = """
import json, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate
print(json.dumps(run_python_gate.inputs().get("GRUND_BINDINGS_ORACLE")))
"""


class PythonGateInputsTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.temp = Path(temp.name)
        self.released = self.stand_in("grund-released", "grund 0.16.1")

    def stand_in(self, name, line):
        """An executable printing `line`: a shebang means nothing on Windows, a .cmd does."""
        if os.name == "nt":
            path = self.temp / (name + ".cmd")
            path.write_text(f"@echo {line}\n", encoding="utf-8")
        else:
            path = self.temp / name
            path.write_text(f"#!/bin/sh\necho '{line}'\n", encoding="utf-8")
            path.chmod(0o755)
        return path

    def oracle(self, **given):
        """The GRUND_BINDINGS_ORACLE the wrapper returns to a local run in the temp dir."""
        environment = {key: value for key, value in os.environ.items() if key not in NOT_LOCAL}
        environment.update(GRUND_BUILT=str(self.temp / "grund-built-unused"),
                           GRUND_RELEASED=str(self.released), **given)
        result = subprocess.run([sys.executable, "-c", CHILD, str(REPO / "scripts")],
                                cwd=self.temp, env=environment, capture_output=True, text=True)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return json.loads(result.stdout.splitlines()[-1])

    def test_an_unset_oracle_is_built_from_this_checkouts_head(self):
        oracle = self.oracle()
        self.assertIsNotNone(oracle, "run_python_gate.inputs() returned no GRUND_BINDINGS_ORACLE, "
                             "so the Node parity test fails its prerequisite on a local run")
        self.assertTrue(Path(oracle).is_absolute(), oracle)
        self.assertTrue(Path(oracle).is_file(), oracle)
        reply = subprocess.run([oracle, "--metadata"], capture_output=True, text=True)
        self.assertEqual(0, reply.returncode, reply.stdout + reply.stderr)
        metadata = json.loads(reply.stdout)
        head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True,
                              text=True, check=True).stdout.strip()
        self.assertEqual(head, metadata["sourceSha"])
        self.assertEqual(1, metadata["protocolVersion"])

    def test_a_given_oracle_comes_back_as_its_absolute_path(self):
        given = self.stand_in("grund-binding-oracle", "{}")
        self.assertEqual(str(given.resolve()), self.oracle(GRUND_BINDINGS_ORACLE=given.name))


if __name__ == "__main__":
    unittest.main()
