"""§AR-ci.3.4.1 — every build of the Node acceptance shares one persistent target.

The acceptance itself takes minutes, so nothing here builds. The source package and
the CLI oracle are asked for in a child whose `checked` only records the commands
it is handed; the other build sites are read from their source, where each one
names its `--target-dir`. The wrapper and the helper are asked, with the variable
unset, where they put the shared target, and the CI cache must restore it.
"""

import ast
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
NODE_HARNESS = REPO / "tests" / "bindings" / "node"
CI = REPO / ".github" / "workflows" / "ci.yml"
# The build sites outside `support.py`, which the child below exercises instead.
BUILD_SITES = (NODE_HARNESS / "test_package.py", NODE_HARNESS / "test_parity.py",
               REPO / "tests" / "e2e" / "test_node_binding.py")
NOT_LOCAL = ("GRUND_NODE_TARGET", "CI", "GITHUB_ACTIONS", "GITHUB_ENV")

RECORD = """
import json, shutil, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import support

calls = []

class Done:
    def __init__(self, stdout=""):
        self.stdout, self.stderr, self.returncode = stdout, "", 0

def record(argv, cwd=None, env=None):
    argv = [str(arg) for arg in argv]
    calls.append(argv)
    if argv[:2] == ["node", "--version"]:
        return Done("v22.0.0\\n")
    if argv[:3] == ["npm", "pack", "--json"]:
        return Done('[{"filename": "grund-cli-0.0.0.tgz"}]')
    return Done()

support.checked = record
shutil.which = lambda name, *rest, **options: name
stage, target, tarball = support.source_package()
first, second = support.checkout_cli(), support.checkout_cli()
print(json.dumps({"calls": calls, "target": str(target), "cli": [str(first), str(second)]}))
"""

DEFAULTS = """
import json, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
sys.path.insert(0, sys.argv[2])
import run_python_gate, support
helper = getattr(support, "node_target", None)
print(json.dumps({"wrapper": run_python_gate.inputs().get("GRUND_NODE_TARGET"),
                  "helper": str(helper()) if helper else None}))
"""


def child(script, *args, cwd, **given):
    environment = {key: value for key, value in os.environ.items() if key not in NOT_LOCAL}
    environment.update(given)
    result = subprocess.run([sys.executable, "-c", script, *map(str, args)], cwd=cwd,
                            env=environment, capture_output=True, text=True)
    return result


def within(path, root):
    try:
        Path(path).resolve().relative_to(Path(root).resolve())
        return True
    except ValueError:
        return False


class NodeSharedTargetTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.temp = Path(temp.name)

    def test_the_source_package_and_the_cli_build_under_the_given_target(self):
        shared = self.temp / "shared"
        result = child(RECORD, NODE_HARNESS, cwd=self.temp, GRUND_NODE_TARGET=str(shared))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        recorded = json.loads(result.stdout.splitlines()[-1])
        named = [argv[argv.index("--target-dir") + 1]
                 for argv in recorded["calls"] if "--target-dir" in argv]
        self.assertTrue(named, recorded["calls"])
        outside = [path for path in named if not within(path, shared)]
        self.assertEqual([], outside,
                         f"a build names a --target-dir outside GRUND_NODE_TARGET={shared}")
        self.assertTrue(within(recorded["target"], shared), recorded["target"])
        builds = [argv for argv in recorded["calls"]
                  if "cargo" in argv and "build" in argv and "grund" in argv]
        self.assertEqual(1, len(builds), "the CLI is built once per run")
        self.assertEqual(1, len(set(recorded["cli"])), recorded["cli"])
        self.assertTrue(within(recorded["cli"][0], shared), recorded["cli"][0])

    def test_every_other_build_site_names_the_shared_target(self):
        allowed = ("self.r.target", "self.rehearsal.target")
        stray = []
        for path in BUILD_SITES:
            for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
                if not isinstance(node, ast.List):
                    continue
                values = [ast.literal_eval(item) if isinstance(item, ast.Constant) else None
                          for item in node.elts]
                site = f"{path.relative_to(REPO).as_posix()}:{node.lineno}"
                if "cargo" in values and "build" in values:
                    stray.append(f"{site}: builds its own CLI instead of support.checkout_cli()")
                if "--target-dir" not in values:
                    continue
                value = node.elts[values.index("--target-dir") + 1]
                shared = (isinstance(value, ast.Call)
                          and ast.unparse(value.func).split(".")[-1] == "node_target")
                if not shared and ast.unparse(value) not in allowed:
                    stray.append(f"{site}: --target-dir {ast.unparse(value)}")
        self.assertEqual([], stray, "a Node acceptance build names its own target directory")

    def test_unset_the_wrapper_and_the_helper_share_one_persistent_target_ci_restores(self):
        built = self.temp / "grund-built-unused"
        released = self.temp / "grund-released"
        released.write_text("#!/bin/sh\necho 'grund 0.16.1'\n", encoding="utf-8")
        released.chmod(0o755)
        if os.name == "nt":
            released = self.temp / "grund-released.cmd"
            released.write_text("@echo grund 0.16.1\n", encoding="utf-8")
        result = child(DEFAULTS, REPO / "scripts", NODE_HARNESS, cwd=self.temp,
                       GRUND_BUILT=str(built), GRUND_RELEASED=str(released),
                       GRUND_BINDINGS_ORACLE=str(built))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        defaults = json.loads(result.stdout.splitlines()[-1])
        wrapper = defaults["wrapper"]
        self.assertIsNotNone(wrapper, "run_python_gate.inputs() prepares no GRUND_NODE_TARGET")
        self.assertTrue(Path(wrapper).is_absolute(), wrapper)
        self.assertTrue(Path(wrapper).is_dir(), "the shared target is prepared and kept")
        self.assertFalse(within(wrapper, REPO), "generated files stay outside the checkout")
        self.assertIsNotNone(defaults["helper"], "tests/bindings/node/support.py has no node_target()")
        self.assertEqual(Path(wrapper).resolve(), Path(defaults["helper"]).resolve())
        home = Path.home().resolve()
        self.assertTrue(within(wrapper, home), wrapper)
        cached = "~/" + Path(wrapper).resolve().relative_to(home).as_posix()
        self.assertIn(f"            {cached}\n", CI.read_text(encoding="utf-8"),
                      "the CI cargo cache does not restore the shared Node target")


if __name__ == "__main__":
    unittest.main()
