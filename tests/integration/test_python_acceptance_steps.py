"""§AR-bindings.6.2.2 — every step of the Python acceptance fails with its output, and
its one registry fetch is retried only for a network failure.

The harness runs in a child, so no `support` leaks into this run. Its helpers are made to
fail on a command that says something; the fetch runs a real pip against a local index
that drops every connection or answers 404, so nothing here reaches the registry, and a
`sitecustomize` beside pip counts how many times the harness ran it.
"""

import ast
import http.server
import json
import os
import socket
import subprocess
import sys
import tempfile
import threading
import unittest
import venv
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BINDINGS = REPO / "tests" / "bindings"
PYTHON_HARNESS = BINDINGS / "python"
SCRATCH = Path.home() / "ag/tmp"
OUT, ERR = "step-stdout-896", "step-stderr-896"
# The command builds what it says, so a failure that shows only the command shows neither.
FAILING = ("import sys; print('step-' + 'stdout-896'); "
           "sys.stderr.write('step-' + 'stderr-896\\n'); sys.exit(3)")
PIP_SAID = "No matching distribution found for build"

HELPER = """
import json, os, sys, traceback
sys.dont_write_bytecode = True
sys.path[:0] = [sys.argv[1], sys.argv[2]]
helper, failing, scratch = sys.argv[3], sys.argv[4], sys.argv[5]
try:
    import support, test_build
    if helper == "checked":
        support.checked([sys.executable, "-c", failing], cwd=scratch)
    elif helper == "run":
        test_build.run([sys.executable, "-c", failing], scratch)
    else:
        support.rust_call("check", scratch)
    print(json.dumps({"error": None}))
except BaseException as error:
    print(json.dumps({"error": "".join(traceback.format_exception(error))}))
"""

INSTALL = """
import json, sys, time, traceback
sys.dont_write_bytecode = True
sys.path[:0] = [sys.argv[1], sys.argv[2]]
python, cwd, env = sys.argv[3], sys.argv[4], json.loads(sys.argv[5])
pauses = []
time.sleep = pauses.append
try:
    import test_build
    test_build.install_build(python, cwd, env)
    error = None
except BaseException as caught:
    error = "".join(traceback.format_exception(caught))
print(json.dumps({"error": error, "pauses": pauses}))
"""

# Each run of the venv's interpreter adds a line to the file the environment names.
COUNTER = ("import os\n"
           "if os.environ.get('GRUND_896_INVOCATIONS'):\n"
           "    open(os.environ['GRUND_896_INVOCATIONS'], 'a').write('pip\\n')\n")


def child(script, *args, env=None):
    result = subprocess.run([sys.executable, "-c", script, str(PYTHON_HARNESS), str(BINDINGS),
                             *map(str, args)], capture_output=True, text=True, env=env)
    assert result.returncode == 0, result.stdout + result.stderr
    return json.loads(result.stdout.splitlines()[-1])


class DroppingIndex:
    """An index that accepts every connection and closes it at once: a network failure."""

    def __enter__(self):
        self.server = socket.socket()
        self.server.bind(("127.0.0.1", 0))
        self.server.listen(16)
        threading.Thread(target=self.serve, daemon=True).start()
        return f"http://127.0.0.1:{self.server.getsockname()[1]}/simple/"

    def serve(self):
        while True:
            try:
                connection, _ = self.server.accept()
            except OSError:
                return
            connection.close()

    def __exit__(self, *exc):
        self.server.close()


class EmptyIndex:
    """An index that answers, with no such package: a failure no retry can cure."""

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_error(404)

        def log_message(self, *args):
            pass

    def __enter__(self):
        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), self.Handler)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        return f"http://127.0.0.1:{self.server.server_address[1]}/simple/"

    def __exit__(self, *exc):
        self.server.shutdown()
        self.server.server_close()


class StepOutputTests(unittest.TestCase):
    def assert_output_shown(self, outcome):
        self.assertIsNotNone(outcome["error"], "a failing command must fail the step")
        for said in (OUT, ERR):
            self.assertIn(said, outcome["error"], "the failure must carry what the command said")

    def test_checked_fails_with_the_commands_stdout_and_stderr(self):
        with tempfile.TemporaryDirectory(prefix="grund-896-", dir=SCRATCH) as temp:
            self.assert_output_shown(child(HELPER, "checked", FAILING, temp))

    def test_build_run_fails_with_the_commands_stdout_and_stderr(self):
        with tempfile.TemporaryDirectory(prefix="grund-896-", dir=SCRATCH) as temp:
            self.assert_output_shown(child(HELPER, "run", FAILING, temp))

    def test_rust_call_fails_with_the_oracles_stdout_and_stderr(self):
        with tempfile.TemporaryDirectory(prefix="grund-896-", dir=SCRATCH) as temp:
            if os.name == "nt":
                oracle = Path(temp) / "oracle.cmd"
                oracle.write_text(f"@echo {OUT}\r\n@echo {ERR} 1>&2\r\n@exit /b 3\r\n")
            else:
                oracle = Path(temp) / "oracle"
                oracle.write_text(f"#!/bin/sh\necho {OUT}\necho {ERR} >&2\nexit 3\n")
                oracle.chmod(0o755)
            env = dict(os.environ, GRUND_BINDINGS_ORACLE=str(oracle))
            self.assert_output_shown(child(HELPER, "rust_call", FAILING, temp, env=env))

    def test_no_captured_command_checks_its_exit_without_its_output(self):
        # §AR-bindings.6.2.2: `check=True` raises a CalledProcessError that drops the output.
        silent = []
        for path in [BINDINGS / "support.py", *sorted(PYTHON_HARNESS.glob("*.py"))]:
            for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
                if not isinstance(node, ast.Call):
                    continue
                flags = {k.arg: k.value.value for k in node.keywords
                         if isinstance(k.value, ast.Constant)}
                if flags.get("check") is True and flags.get("capture_output") is True:
                    silent.append(f"{path.relative_to(REPO)}:{node.lineno}")
        self.assertEqual([], silent, "route these through support.checked")


class BuildFetchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        cls.temp = tempfile.TemporaryDirectory(prefix="grund-896-pip-", dir=SCRATCH)
        root = Path(cls.temp.name)
        venv.EnvBuilder(with_pip=True).create(root / "venv")
        cls.python = root / "venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        (root / "counter").mkdir()
        (root / "counter" / "sitecustomize.py").write_text(COUNTER)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def install(self, index):
        root = Path(self.temp.name)
        log = root / f"invocations-{self._testMethodName}"
        keep = {k: v for k, v in os.environ.items()
                if not k.startswith("PIP_") and k not in ("PYTHONPATH", "PYTHONHOME")}
        env = dict(keep, PIP_INDEX_URL=index, PIP_RETRIES="1", PIP_TIMEOUT="5",
                   PIP_CONFIG_FILE=os.devnull, PIP_DISABLE_PIP_VERSION_CHECK="1",
                   PIP_NO_CACHE_DIR="1", PIP_NO_INPUT="1",
                   PYTHONPATH=str(root / "counter"), GRUND_896_INVOCATIONS=str(log))
        outcome = child(INSTALL, self.python, root, json.dumps(env))
        runs = len(log.read_text().splitlines()) if log.exists() else 0
        return outcome, runs

    def test_a_dropped_connection_is_retried_three_times_then_fails_with_pips_output(self):
        with DroppingIndex() as index:
            outcome, runs = self.install(index)
        self.assertEqual(3, runs, outcome["error"])
        self.assertEqual(2, len(outcome["pauses"]), "a short pause between attempts")
        self.assertIsNotNone(outcome["error"], "a third network failure fails, never skips")
        self.assertIn(PIP_SAID, outcome["error"])

    def test_a_failure_no_retry_can_cure_fails_at_once_with_pips_output(self):
        with EmptyIndex() as index:
            outcome, runs = self.install(index)
        self.assertEqual(1, runs, outcome["error"])
        self.assertEqual([], outcome["pauses"])
        self.assertIsNotNone(outcome["error"])
        self.assertIn(PIP_SAID, outcome["error"])


if __name__ == "__main__":
    unittest.main()
