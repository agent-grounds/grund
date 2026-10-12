"""§AR-ci.3.4.1 — every build of the Node acceptance shares one persistent target.

§AR-ci.3.4.1.1 — it pays for the release profile once, and each build prints one line.

The acceptance itself takes minutes, so nothing here builds. The source package and
the CLI oracle are asked for in a child whose `checked` only records the commands
it is handed; the other build sites are read from their source, where each one
names its `--target-dir`. The wrapper and the helper are asked, with the variable
unset, where they put the shared target; a CI cache step with a key of its own must
restore it, because a key another step shares hits exactly and never saves.
"""

import ast
import json
import os
import re
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

calls, envs = [], []

class Done:
    def __init__(self, stdout=""):
        self.stdout, self.stderr, self.returncode = stdout, "", 0

def record(argv, cwd=None, env=None):
    argv = [str(arg) for arg in argv]
    calls.append(argv)
    envs.append((env or {}).get("CARGO_INCREMENTAL"))
    if argv[:2] == ["node", "--version"]:
        return Done("v22.0.0\\n")
    if argv[:3] == ["npm", "pack", "--json"]:
        return Done('[{"filename": "grund-cli-0.0.0.tgz"}]')
    return Done()

support.checked = record
shutil.which = lambda name, *rest, **options: name
stage, target, tarball = support.source_package()
first, second = support.checkout_cli(), support.checkout_cli()
print(json.dumps({"calls": calls, "incremental": envs, "target": str(target), "cli": [str(first), str(second)]}))
"""

# Every command goes through `launch.subprocess.run`, so a fake one there sees each build.
ATTRIBUTE = """
import subprocess, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import launch

class Done:
    stdout, stderr, returncode = "", "", 0

launch.subprocess.run = lambda *args, **options: Done()
target, node = sys.argv[2], sys.argv[3]
launch.checked(["node", "--version"])
launch.checked(["node", node + "/build.mjs", "--source-root", ".", "--out-dir", "out",
                "--target-dir", target, "--profile", "dev", "--test-seams"])
launch.checked(["npm", "run", "build:source", "--", "--target-dir", target])
launch.checked(["cargo", "+1.95.0", "build", "-p", "grund", "--locked", "--target-dir", target])
launch.checked(["npm", "pack", "--json"])
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


def profile(argv):
    """The profile a binding build resolves to: `build.mjs` and `build:source` default to release."""
    return argv[argv.index("--profile") + 1] if "--profile" in argv else "release"


def binding_builds(path):
    """Each `checked([...])` of the builder or `build:source` in a file, with its profile.

    A build through `command(...)` is a refusal the test expects or a Cargo probe's, which
    compiles nothing (§AR-ci.3.4.1.1), so only `checked` counts. A site is named by the
    innermost function around it.
    """
    tree = ast.parse(path.read_text(encoding="utf-8"))
    functions = [node for node in ast.walk(tree) if isinstance(node, ast.FunctionDef)]
    builds = []
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and ast.unparse(node.func).split(".")[-1] == "checked"
                and node.args and isinstance(node.args[0], ast.List)):
            continue
        elements = node.args[0].elts
        if not any("build.mjs" in ast.unparse(item) or "build:source" in ast.unparse(item)
                   for item in elements):
            continue
        values = [item.value if isinstance(item, ast.Constant) else ast.unparse(item)
                  for item in elements]
        around = max((scope for scope in functions
                      if scope.lineno <= node.lineno <= scope.end_lineno),
                     key=lambda scope: scope.lineno)
        builds.append((f"{path.relative_to(REPO).as_posix()} {around.name}", profile(values)))
    return builds


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
        incremental = recorded["incremental"][recorded["calls"].index(builds[0])]
        self.assertEqual("0", incremental, "the debug CLI writes no incremental state to cache")
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

    def test_the_acceptance_pays_for_the_release_profile_once(self):
        result = child(RECORD, NODE_HARNESS, cwd=self.temp, GRUND_NODE_TARGET=str(self.temp))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        recorded = json.loads(result.stdout.splitlines()[-1])
        source = [(index, argv) for index, argv in enumerate(recorded["calls"])
                  if "build:source" in argv]
        self.assertEqual(1, len(source), recorded["calls"])
        index, argv = source[0]
        builds = [("tests/bindings/node/support.py source_package", profile(argv))]
        for path in BUILD_SITES:
            builds += binding_builds(path)
        release = [site for site, chosen in builds if chosen == "release"]
        self.assertEqual(
            ["tests/e2e/test_node_binding.py test_documented_native_source_builder"], release,
            f"{len(release)} release builds of the binding in one acceptance run, "
            f"want only the documented native builder's: {builds}")
        others = [(site, chosen) for site, chosen in builds if chosen not in ("release", "dev")]
        self.assertEqual([], others, "a binding build names a profile the test cannot read")
        # A dev build is incremental by default, and the cache archives the target whole.
        self.assertEqual("0", recorded["incremental"][index],
                         "the dev source build writes no incremental state to cache")

    def test_each_build_prints_one_line_naming_its_profile_target_and_seconds(self):
        target = self.temp / "shared"
        result = child(ATTRIBUTE, NODE_HARNESS, target, NODE_HARNESS.parents[2] / "crates" /
                       "grund-node", cwd=self.temp)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        lines = [line for line in result.stderr.splitlines() if line.startswith("node build:")]
        shape = re.compile(r"node build: (\S+) (.+) (\d+(?:\.\d+)?)s")
        parsed = [shape.fullmatch(line) for line in lines]
        self.assertTrue(all(parsed), lines)
        self.assertEqual([("dev", str(target)), ("release", str(target)), ("dev", str(target))],
                         [(match[1], match[2]) for match in parsed],
                         "one line per build, none for anything else the harness runs; "
                         f"stderr was {result.stderr!r}")

    def test_the_ci_key_drops_entries_saved_before_the_dev_builds(self):
        steps = CI.read_text(encoding="utf-8").split("\n      - ")
        keys = [line.strip() for step in steps for line in step.splitlines()
                if "~/ag/tmp/grund-node-target" in step and line.strip().startswith("key:")]
        self.assertEqual(1, len(keys), keys)
        self.assertIn("-node-target-v2-", keys[0],
                      "an entry saved before the dev builds holds none of their dependencies")

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
        steps = CI.read_text(encoding="utf-8").split("\n      - ")
        restoring = [step for step in steps if f"            {cached}\n" in step]
        # §AR-ci.13.4: one such step in each job that runs the Node acceptance, all one key.
        self.assertTrue(restoring, "no CI cache step restores the shared Node target")
        key = {line.strip() for step in restoring for line in step.splitlines()
               if line.strip().startswith("key:")}
        self.assertEqual(1, len(key), restoring)
        others = [line.strip() for step in steps if step not in restoring
                  for line in step.splitlines()
                  if "actions/cache" in step and line.strip().startswith("key:")]
        # An exact hit saves nothing, so a key shared with another cache never saves this one.
        self.assertNotIn(next(iter(key)), others, f"{key} is shared with another cache step")


if __name__ == "__main__":
    unittest.main()
