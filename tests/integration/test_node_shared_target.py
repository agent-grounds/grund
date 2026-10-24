"""§AR-ci.3.4.1 — every build of the Node acceptance shares one persistent target root.

§AR-ci.3.4.1.1 — it pays for the release profile once, and each build prints one line.

§AR-ci.3.4.1.2 — under that root each source checkout builds into a target of its own.

The acceptance itself takes minutes, so nothing here builds. The source package and
the CLI oracle are asked for in a child whose `checked` only records the commands
it is handed; the other build sites are read from their source, where each one
names its `--target-dir`. The wrapper and the helper are asked, with the variable
unset, where they put the shared target; a CI cache step with a key of its own must
restore it, because a key another step shares hits exactly and never saves. A copy of
the acceptance wrapper runs under the gate beside a stand-in `run.py`, as CI's
`node acceptance` step runs the real one, so its build lines are seen on a green log.
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

from ci_workflow import ci_text

REPO = Path(__file__).resolve().parents[2]
NODE_HARNESS = REPO / "tests" / "bindings" / "node"
CI = REPO / ".github" / "workflows" / "ci.yml"
MACOS = CI.with_name("macos-suites.yml")
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
print(json.dumps({"calls": calls, "incremental": envs, "target": str(target),
                  "again": str(support.node_target()), "cli": [str(first), str(second)]}))
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

# The gate reaches the acceptance only through this wrapper, which captures run.py's output.
FORWARD = """
import sys, unittest
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import test_node_binding_acceptance as wrapper

class Done:
    stdout, returncode = "noise out\\n", 0
    stderr = "Compiling grund\\nnode build: dev /t 5.0s\\ntest_x ... node build: release /t 49.2s\\ntest_x ... ok\\n"

wrapper.subprocess.run = lambda *args, **options: Done()
tests = unittest.TestSuite([wrapper.NodeAcceptanceWorkflowTests(
    "test_node_acceptance_is_in_the_complete_local_gate")])
sys.exit(not unittest.TextTestRunner().run(tests).wasSuccessful())
"""

# CI's `node acceptance` step: the gate over the acceptance alone (§AR-ci.3.4.3).
GATE = """
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate
sys.exit(run_python_gate.run_tests(sys.argv[2], dict(os.environ), "only-node-acceptance"))
"""

# Stands in for tests/bindings/node/run.py beside a copy of the wrapper, which runs it.
RUN = """
import sys
print("noise out")
sys.stderr.write("Compiling grund\\nnode build: dev /t 5.0s\\n"
                 "test_x ... node build: release /t 49.2s\\ntest_x ... ok\\n")
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


def checkout(root):
    """A second source checkout at `root`: the harness's own modules, and the builder files
    the source package asks for, which the recorder never runs."""
    harness = root / "tests" / "bindings" / "node"
    harness.mkdir(parents=True)
    for name in ("support.py", "launch.py"):
        (harness / name).write_bytes((NODE_HARNESS / name).read_bytes())
    # The shared harness derives the target for both bindings (§AR-ci.3.4.1.2).
    (harness.parent / "support.py").write_bytes((NODE_HARNESS.parent / "support.py").read_bytes())
    node = root / "crates" / "grund-node"
    node.mkdir(parents=True)
    for name in ("build.mjs", "stage.mjs"):
        (node / name).write_text("", encoding="utf-8")
    return harness


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

    def test_each_source_checkout_builds_into_a_target_of_its_own(self):
        shared = self.temp / "shared"
        runs = {}
        for name, harness in (("this", NODE_HARNESS),
                              ("a", checkout(self.temp / "a")), ("b", checkout(self.temp / "b"))):
            result = child(RECORD, harness, cwd=self.temp, GRUND_NODE_TARGET=str(shared))
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            runs[name] = json.loads(result.stdout.splitlines()[-1])
        result = child(RECORD, self.temp / "a" / "tests" / "bindings" / "node", cwd=self.temp,
                       GRUND_NODE_TARGET=str(shared))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        rerun = json.loads(result.stdout.splitlines()[-1])
        targets = {name: Path(run["target"]).resolve() for name, run in runs.items()}
        for name, run in runs.items():
            self.assertTrue(within(targets[name], shared), (name, targets[name]))
            self.assertEqual(targets[name], Path(run["again"]).resolve(),
                             f"checkout {name} gets another target on a second call")
        self.assertEqual(targets["a"], Path(rerun["target"]).resolve(),
                         "checkout a gets another target on a second run")
        names = sorted(targets)
        for index, one in enumerate(names):
            for other in names[index + 1:]:
                self.assertFalse(
                    within(targets[one], targets[other]) or within(targets[other], targets[one]),
                    f"checkouts {one} and {other} share a target under GRUND_NODE_TARGET: "
                    f"{targets[one]} and {targets[other]}")
        for name, run in runs.items():
            named = [argv[argv.index("--target-dir") + 1]
                     for argv in run["calls"] if "--target-dir" in argv]
            self.assertTrue(named, run["calls"])
            stray = [path for path in named + run["cli"] if not within(path, targets[name])]
            self.assertEqual([], stray,
                             f"checkout {name} builds outside its own target {targets[name]}")

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

    def test_a_green_acceptance_shows_its_build_lines_and_nothing_else_it_captured(self):
        result = child(FORWARD, REPO / "tests" / "integration", cwd=self.temp)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        output = result.stdout + result.stderr
        self.assertEqual(["node build: dev /t 5.0s", "node build: release /t 49.2s"],
                         [line for line in output.splitlines() if line.startswith("node build:")],
                         f"the wrapper hides the build lines on success; output was {output!r}")
        for noise in ("Compiling grund", "test_x ... ok", "noise out"):
            self.assertNotIn(noise, output)

    def test_a_green_acceptance_under_the_gate_shows_its_build_lines_and_nothing_else(self):
        # §AR-ci.requirements.5.1: the real acceptance method runs the stand-in run.py.
        # Other constructor regressions need their own fixtures, outside this output test.
        tests = self.temp / "repo" / "tests" / "integration"
        harness = self.temp / "repo" / "tests" / "bindings" / "node"
        for directory in (tests, harness):
            directory.mkdir(parents=True)
        wrapper = "test_node_binding_acceptance.py"
        collection = ('\ndef load_tests(loader, suite, pattern):\n'
                      '    return unittest.TestSuite([NodeAcceptanceWorkflowTests('
                      '"test_node_acceptance_is_in_the_complete_local_gate")])\n')
        (tests / wrapper).write_bytes((REPO / "tests" / "integration" / wrapper).read_bytes()
                                     + collection.encode())
        (harness / "run.py").write_text(RUN, encoding="utf-8")
        result = child(GATE, REPO / "scripts", tests, cwd=self.temp)
        output = result.stdout + result.stderr
        self.assertEqual(0, result.returncode, output)
        self.assertRegex(output, r"(?m)^1 tests: 1 passed, 0 failed, 0 ignored, ")
        self.assertEqual(["node build: dev /t 5.0s", "node build: release /t 49.2s"],
                         [line for line in output.splitlines() if line.startswith("node build:")],
                         f"the gate hides a green acceptance's build lines; output was {output!r}")
        for noise in ("Compiling grund", "test_x ... ok", "noise out"):
            self.assertNotIn(noise, output)

    def test_the_ci_key_drops_entries_saved_before_the_per_checkout_targets(self):
        # The macOS suites left ci.yml for their own workflow (§AR-ci.14) and cache the same target.
        for workflow, text in ((CI, ci_text()), (MACOS, MACOS.read_text(encoding="utf-8"))):
            steps = text.split("\n      - ")
            # The root's own path line: the compiler's directory beneath it has a cache of its own.
            keys = [line.strip() for step in steps for line in step.splitlines()
                    if "            ~/ag/tmp/grund-node-target\n" in step
                    and line.strip().startswith("key:")]
            self.assertTrue(keys, f"no job in {workflow.name} caches the Node target")
            for key in keys:
                self.assertIn("-node-target-v3-", key,
                              f"{workflow.name}: an entry saved before the per-checkout targets "
                              "holds none of them")

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
        # §AR-ci.3.4.1.2: the wrapper prepares the root, the helper's target lies under it.
        self.assertTrue(within(defaults["helper"], wrapper), (wrapper, defaults["helper"]))
        home = Path.home().resolve()
        self.assertTrue(within(wrapper, home), wrapper)
        cached = "~/" + Path(wrapper).resolve().relative_to(home).as_posix()
        steps = ci_text().split("\n      - ")
        restoring = [step for step in steps if f"            {cached}\n" in step]
        # §AR-ci.13.4: the restore and save steps of each job running the Node acceptance, one key.
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
