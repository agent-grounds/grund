"""§AR-ci.3.4.3 — the Python gate runs all but the binding acceptances, or one of them.

The cut is checked on the real integration suite in-process, by discovery alone,
so neither acceptance is ever run here: the three selections must partition what
discovery finds, the Node one must find exactly `test_node_binding_acceptance`, and
the Python one exactly `test_python_binding_acceptance`.
The plumbing is checked in a child on suites of its own in a temporary directory,
and the command line with inputs that would fail if the wrapper reached them, so a
wrong argument must be refused before any build.
"""

import os
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HOME = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO / "scripts"))
import run_python_gate  # noqa: E402

NODE, PYTHON = "test_node_binding_acceptance", "test_python_binding_acceptance"
WITHOUT, ONLY, ONLY_PYTHON = "without-acceptance", "only-node-acceptance", "only-python-acceptance"
CHILD = """
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate
selection = sys.argv[3] or None
sys.exit(run_python_gate.run_tests(sys.argv[2], dict(os.environ), selection))
"""
# The summary line of §AR-ci.requirements.5.1: `<total> tests: <passed> passed, …`.
RAN = re.compile(r"(?m)^(\d+) tests: ")


def ids(suite):
    found = []
    for item in suite:
        found.extend(ids(item) if isinstance(item, unittest.TestSuite) else [item.id()])
    return found


class PythonGateSelectionTests(unittest.TestCase):
    def select(self):
        select = getattr(run_python_gate, "select", None)
        self.assertIsNotNone(select, "scripts/run_python_gate.py has no select(suite, selection)")
        return select

    def test_the_three_selections_partition_the_integration_suite(self):
        select = self.select()
        discovered = unittest.TestLoader().discover(str(HOME), "test_*.py", top_level_dir=str(HOME))
        everything = ids(discovered)
        without, only = ids(select(discovered, WITHOUT)), ids(select(discovered, ONLY))
        only_python = ids(select(discovered, ONLY_PYTHON))
        self.assertEqual(sorted(everything), sorted(ids(select(discovered, None))))
        self.assertEqual(set(), set(without) & set(only))
        self.assertEqual(set(), set(without) & set(only_python))
        self.assertEqual(set(), set(only) & set(only_python))
        self.assertEqual(sorted(everything), sorted(without + only + only_python))
        self.assertTrue(only, "the Node selection finds no test")
        self.assertEqual({NODE}, {test.split(".")[0] for test in only})
        self.assertTrue(only_python, "the Python acceptance selection finds no test")
        self.assertEqual({PYTHON}, {test.split(".")[0] for test in only_python})

    def suite(self, node_source):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "test_other.py").write_text(textwrap.dedent("""
            import unittest
            class Other(unittest.TestCase):
                def test_passes(self):
                    pass
        """), encoding="utf-8")
        if node_source is not None:
            (root / f"{NODE}.py").write_text(textwrap.dedent(node_source), encoding="utf-8")
            (root / f"{PYTHON}.py").write_text(textwrap.dedent(node_source), encoding="utf-8")
        return root

    def run_selected(self, root, selection):
        result = subprocess.run([sys.executable, "-c", CHILD, str(REPO / "scripts"), str(root),
                                 selection or ""], cwd=root, capture_output=True, text=True)
        ran = RAN.search(result.stdout)
        return result.returncode, int(ran.group(1)) if ran else None, result.stdout + result.stderr

    def test_run_tests_hands_the_selection_to_its_discovery(self):
        root = self.suite("""
            import unittest
            class Node(unittest.TestCase):
                def test_acceptance(self):
                    pass
        """)
        for selection, expected in ((None, 3), (WITHOUT, 1), (ONLY, 1), (ONLY_PYTHON, 1)):
            with self.subTest(selection=selection):
                code, ran, output = self.run_selected(root, selection)
                self.assertEqual((0, expected), (code, ran), output)

    def test_an_acceptance_that_fails_to_import_stays_in_its_selection(self):
        root = self.suite("import a_module_that_does_not_exist_here\n")
        for selection in (ONLY, ONLY_PYTHON):
            with self.subTest(selection=selection):
                code, ran, output = self.run_selected(root, selection)
                self.assertEqual((1, 1), (code, ran), output)
        code, ran, output = self.run_selected(root, WITHOUT)
        self.assertEqual((0, 1), (code, ran), output)

    def test_an_acceptance_selection_that_finds_nothing_exits_5(self):
        for selection in (ONLY, ONLY_PYTHON):
            with self.subTest(selection=selection):
                code, _, output = self.run_selected(self.suite(None), selection)
                self.assertEqual(5, code, output)

    def test_a_wrong_argument_is_refused_before_any_input_is_prepared(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        missing = Path(temp.name) / "missing"
        # Inputs that are all given, and a released binary that is not there: reaching
        # inputs() fails with 1, so only a refusal before it exits 2.
        environment = dict(os.environ, GRUND_BUILT=str(missing), GRUND_RELEASED=str(missing),
                           GRUND_BINDINGS_ORACLE=str(missing),
                           GRUND_NODE_TARGET=str(Path(temp.name) / "node-target"))
        # §AR-ci.3.4.3: the old two-way selection is no selection any more.
        for argv in (["--bogus"], [f"--{WITHOUT}", f"--{ONLY}"], [WITHOUT],
                     ["--without-node-acceptance"], [f"--{ONLY}", f"--{ONLY_PYTHON}"]):
            with self.subTest(argv=argv):
                result = subprocess.run([sys.executable, str(REPO / "scripts/run_python_gate.py"), *argv],
                                        cwd=REPO, env=environment, capture_output=True, text=True,
                                        timeout=60)
                self.assertEqual(2, result.returncode, result.stdout + result.stderr)
                self.assertIn("usage", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()
