"""§AR-bindings.6.2 — the Python gate runs the Python binding's acceptance.

The module installs this checkout's `crates/grund-py` into a virtual environment of
its own and runs `tests/bindings/python/run.py` with it, against the gate's
same-source oracle (§AR-bindings.6, §FS-distribution.3.0.3, §FS-distribution.3.3).
Every build of the run shares this checkout's target under `GRUND_PYTHON_TARGET`
(§AR-ci.3.4.5).
A build or install that fails fails the test; nothing here skips.
"""

import importlib.util
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


def bindings_support():
    """`tests/bindings/support.py`, loaded under a name of its own so no `support` leaks here."""
    spec = importlib.util.spec_from_file_location("grund_bindings_support",
                                                  REPO / "tests" / "bindings" / "support.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def interpreter(environment):
    return Path(environment) / ("Scripts/python.exe" if os.name == "nt" else "bin/python")


class PythonAcceptanceWorkflowTests(unittest.TestCase):
    def test_python_acceptance_is_in_the_complete_local_gate(self):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="grund-python-acceptance-", dir=SCRATCH) as temp:
            environment = Path(temp) / "venv"
            subprocess.run([sys.executable, "-m", "venv", str(environment)], check=True)
            python = interpreter(environment)
            support = bindings_support()
            # §AR-ci.3.4.5: this install and run.py's own share this checkout's kept target,
            # which run.py derives again from the root it is handed.
            target = support.python_target()
            # §AR-bindings.6.2: every build of the run, this install and run.py's own, is at
            # the dev profile - the same source as a release, without its LTO.
            build = dict(os.environ, MATURIN_PEP517_ARGS="--profile dev",
                         GRUND_PYTHON_TARGET=str(support.python_root()),
                         CARGO_TARGET_DIR=str(target))
            try:
                install = subprocess.run([str(python), "-m", "pip", "install", "--quiet",
                                          str(SOURCE)], cwd=REPO, env=build,
                                         capture_output=True, text=True)
                self.assertEqual(0, install.returncode, install.stdout + install.stderr)
                # §FS-distribution.3.0.3: the corpus writes and pipes UTF-8, on Windows too.
                env = dict(build, PYTHONUTF8="1")
                result = subprocess.run([str(python), str(ACCEPTANCE)], cwd=REPO, env=env,
                                        capture_output=True, text=True, encoding="utf-8",
                                        errors="replace")
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            finally:
                # Only for the cache's size: the clean checkout and the sdist compile the
                # workspace crates from fresh paths on every run (§AR-ci.3.4.5).
                support.prune_python_target(target)


if __name__ == "__main__":
    unittest.main()
