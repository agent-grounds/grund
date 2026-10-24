"""§AR-ci.3.4.5 — every build of the Python acceptance shares this checkout's persistent Cargo
target, under a root every checkout shares.

The builds are recorded rather than run: a child process imports the build site with its
`subprocess.run`, or the helper `run`, replaced by a recorder, and reports the
`CARGO_TARGET_DIR` each `pip install` would have built into. The child's `HOME` is a
temporary directory, so the fallback it computes lands there and nothing is written under
the real one. Two checkouts are copies of the shared harness, each asked for its target in a
child. `ci.yml` is read as text, as the layout test reads it (§AR-ci.13.5).
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from ci_workflow import ci_text, instances

REPO = Path(__file__).resolve().parents[2]
SELECTION = "python scripts/run_python_gate.py --only-python-acceptance"
TARGET = "~/ag/tmp/grund-python-target"

# The acceptance module's own install (§AR-bindings.6.2).
MODULE = """
import json, subprocess, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
installs = []

def record(argv, *args, env=None, **kwargs):
    argv = [str(arg) for arg in argv]
    if "pip" in argv and "install" in argv:
        installs.append((env or {}).get("CARGO_TARGET_DIR"))
    return subprocess.CompletedProcess(argv, 0, "", "")

subprocess.run = record
import test_python_binding_acceptance as acceptance
case = acceptance.PythonAcceptanceWorkflowTests("test_python_acceptance_is_in_the_complete_local_gate")
case.test_python_acceptance_is_in_the_complete_local_gate()
print(json.dumps(installs))
"""

# The clean checkout's install in `run.py`'s build test (§FS-distribution.3.3.7), stopped there.
CLEAN_CHECKOUT = """
import json, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
sys.path.insert(0, sys.argv[2])
import test_build

class Installed(Exception):
    pass

def record(arguments, cwd, env=None):
    arguments = [str(arg) for arg in arguments]
    if "pip" in arguments and "install" in arguments:
        print(json.dumps([(env or {}).get("CARGO_TARGET_DIR")]))
        raise Installed()
    return subprocess.CompletedProcess(arguments, 0, "", "")

test_build.run = record
test_build.environment = lambda parent: Path(parent) / "python"
case = test_build.BuildTests("test_clean_checkout_and_unpacked_sdist_install_independently")
try:
    case.test_clean_checkout_and_unpacked_sdist_install_independently()
except Installed:
    pass
"""


# The target `python_target()` returns, asked of the harness at the path given.
TARGET_OF = """
import importlib.util, sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("grund_bindings_support", sys.argv[1])
support = importlib.util.module_from_spec(spec)
spec.loader.exec_module(support)
print(support.python_target())
"""


def recorded(script, *args, **given):
    """The `CARGO_TARGET_DIR` of each install the child reaches, with `given` in its env."""
    home = tempfile.TemporaryDirectory()
    environment = {key: value for key, value in os.environ.items()
                   if key not in ("GRUND_PYTHON_TARGET", "CARGO_TARGET_DIR")}
    environment.update(HOME=home.name, USERPROFILE=home.name, **given)
    result = subprocess.run([sys.executable, "-c", script, *map(str, args)], cwd=REPO,
                            env=environment, capture_output=True, text=True, timeout=120)
    return home, result


def under(path, root):
    return path is not None and Path(path).resolve().is_relative_to(Path(root).resolve())


class PythonSharedTargetTests(unittest.TestCase):
    SITES = {"the acceptance module": (MODULE, REPO / "tests" / "integration"),
             "the clean checkout": (CLEAN_CHECKOUT, REPO / "tests" / "bindings",
                                    REPO / "tests" / "bindings" / "python")}

    def installs(self, site, **given):
        script, *paths = self.SITES[site]
        home, result = recorded(script, *paths, **given)
        self.addCleanup(home.cleanup)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        found = json.loads(result.stdout.strip().splitlines()[-1])
        self.assertTrue(found, f"{site} made no pip install")
        return Path(home.name), found

    def test_each_install_builds_under_the_given_target(self):
        with tempfile.TemporaryDirectory() as shared:
            for site in self.SITES:
                with self.subTest(site=site):
                    _, found = self.installs(site, GRUND_PYTHON_TARGET=shared)
                    self.assertTrue(all(under(target, shared) for target in found),
                                    f"{site} builds into {found}, not under {shared}")

    def test_unset_each_install_builds_under_the_persistent_default(self):
        for site in self.SITES:
            with self.subTest(site=site):
                home, found = self.installs(site)
                default = home / "ag" / "tmp" / "grund-python-target"
                self.assertTrue(all(under(target, default) for target in found),
                                f"{site} builds into {found}, not under {default}")

    def test_two_checkouts_get_disjoint_targets_under_the_shared_root(self):
        # §AR-ci.3.4.1.2: the root is shared, a target is not.
        with tempfile.TemporaryDirectory() as work:
            work = Path(work).resolve()
            root = work / "targets"
            found = {}
            for name in ("one", "two"):
                harness = work / name / "tests" / "bindings" / "support.py"
                harness.parent.mkdir(parents=True)
                shutil.copyfile(REPO / "tests" / "bindings" / "support.py", harness)
                targets = []
                for _ in range(2):
                    home, result = recorded(TARGET_OF, harness, GRUND_PYTHON_TARGET=str(root))
                    self.addCleanup(home.cleanup)
                    self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                    targets.append(Path(result.stdout.strip().splitlines()[-1]))
                self.assertEqual(targets[0], targets[1], f"{name} moves between calls")
                found[name] = targets[0]
                self.assertEqual(root, found[name].parent, f"{name} builds outside the root")
                owner = json.loads((found[name] / "checkout.json").read_text(encoding="utf-8"))
                self.assertEqual(str(work / name), owner["checkout"])
            self.assertNotEqual(found["one"], found["two"], "two checkouts share one target")

    def test_every_python_acceptance_job_restores_the_target_under_a_key_of_its_own(self):
        found = [i for i in instances(ci_text()) if SELECTION in i["commands"]]
        self.assertEqual(["ubuntu-latest", "windows-latest"], sorted(i["system"] for i in found),
                         "no Python acceptance job on each operating system")
        for instance in found:
            with self.subTest(job=instance["job"]):
                restores = [step for step in instance["steps"] if "actions/cache/restore" in step
                            and re.search(rf"(?m)^\s*{re.escape(TARGET)}\s*$", step)]
                self.assertEqual(1, len(restores), f"{instance['job']} restores no {TARGET}")
                key = re.search(r"(?m)^\s*key:\s*(.+)$", restores[0]).group(1)
                self.assertIn("python-target-v2-", key)
                self.assertNotIn("-cargo-", key)


if __name__ == "__main__":
    unittest.main()
