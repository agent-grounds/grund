"""§AR-bindings.6.2 — the Python gate runs the Python binding's acceptance.

The module installs this checkout's `crates/grund-py` into a virtual environment of
its own and runs `tests/bindings/python/run.py` with it, against the gate's
same-source oracle (§AR-bindings.6, §FS-distribution.3.0.3, §FS-distribution.3.3).
A build or install that fails fails the test; nothing here skips.
"""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag/tmp"
# §AR-bindings.6: the crate a local-source install comes from, and the suite it must pass.
SOURCE = REPO / "crates" / "grund-py"
ACCEPTANCE = REPO / "tests" / "bindings" / "python" / "run.py"


def interpreter(environment):
    return Path(environment) / ("Scripts/python.exe" if os.name == "nt" else "bin/python")


class PythonAcceptanceWorkflowTests(unittest.TestCase):
    def test_python_acceptance_is_in_the_complete_local_gate(self):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="grund-python-acceptance-", dir=SCRATCH) as temp:
            environment = Path(temp) / "venv"
            subprocess.run([sys.executable, "-m", "venv", str(environment)], check=True)
            python = interpreter(environment)
            # §AR-bindings.6.2: one install per run, built at the dev profile - the same
            # source as a release, without its LTO, which costs the gate about a minute.
            build = dict(os.environ, MATURIN_PEP517_ARGS="--profile dev")
            install = subprocess.run([str(python), "-m", "pip", "install", "--quiet",
                                      str(SOURCE)], cwd=REPO, env=build,
                                     capture_output=True, text=True)
            self.assertEqual(0, install.returncode, install.stdout + install.stderr)
            # §FS-distribution.3.0.3: the corpus writes and pipes UTF-8, on Windows too.
            env = dict(os.environ, PYTHONUTF8="1")
            result = subprocess.run([str(python), str(ACCEPTANCE)], cwd=REPO, env=env,
                                    capture_output=True, text=True, encoding="utf-8",
                                    errors="replace")
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
