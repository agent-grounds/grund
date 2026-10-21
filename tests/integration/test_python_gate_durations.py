"""§AR-ci.3.4.2 — the Python gate ends with its ten slowest tests, not a line per test.

`run_python_gate.run_tests` is run on a suite of its own in a temporary directory:
one test that sleeps and a dozen that do not. The table must exist, name the slow
test first, and stop at ten lines; a failing suite must still fail the run.
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
CHILD = """
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate
run_tests = getattr(run_python_gate, "run_tests", None)
if run_tests is None:
    sys.exit("scripts/run_python_gate.py has no run_tests(start, environment)")
sys.exit(run_tests(sys.argv[2], dict(os.environ)))
"""
ROW = re.compile(r"^\s*(\d+(?:\.\d+)?)s\s+(\S+)\s*$")


class PythonGateDurationsTests(unittest.TestCase):
    def suite(self, failing=False):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "test_slow.py").write_text(textwrap.dedent("""
            import time, unittest
            class Slow(unittest.TestCase):
                def test_sleeps(self):
                    time.sleep(0.5)
            """), encoding="utf-8")
        fast = "".join(f"    def test_fast_{n}(self):\n        pass\n" for n in range(12))
        if failing:
            fast += "    def test_fails(self):\n        self.fail('on purpose')\n"
        (root / "test_fast.py").write_text(
            "import unittest\nclass Fast(unittest.TestCase):\n" + fast, encoding="utf-8")
        return root

    def run_tests(self, start):
        return subprocess.run([sys.executable, "-c", CHILD, str(REPO / "scripts"), str(start)],
                              cwd=REPO, env=dict(os.environ), capture_output=True, text=True)

    def test_a_passing_run_ends_with_its_ten_slowest_tests(self):
        result = self.run_tests(self.suite())
        output = result.stdout + result.stderr
        self.assertEqual(0, result.returncode, output)
        lines = output.splitlines()
        headings = [index for index, line in enumerate(lines) if "Slowest tests" in line]
        self.assertEqual(1, len(headings), output)
        rows = [ROW.match(line) for line in lines[headings[0] + 1:]]
        rows = [row for row in rows if row]
        self.assertTrue(rows, output)
        self.assertLessEqual(len(rows), 10, "the table is the slowest ten, not every test")
        self.assertIn("test_sleeps", rows[0].group(2), output)
        self.assertGreaterEqual(float(rows[0].group(1)), 0.5, output)
        seconds = [float(row.group(1)) for row in rows]
        self.assertEqual(sorted(seconds, reverse=True), seconds, "slowest first")

    def test_a_failing_run_still_fails(self):
        result = self.run_tests(self.suite(failing=True))
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("Slowest tests", result.stdout + result.stderr)

    def test_class_fixture_time_is_counted(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        (Path(temp.name) / "test_fixture.py").write_text(textwrap.dedent("""
            import time, unittest
            class Fixture(unittest.TestCase):
                @classmethod
                def setUpClass(cls):
                    time.sleep(0.5)
                def test_quick(self):
                    pass
            """), encoding="utf-8")
        result = self.run_tests(temp.name)
        output = result.stdout + result.stderr
        self.assertEqual(0, result.returncode, output)
        rows = [row for row in map(ROW.match, output.splitlines()) if row]
        self.assertTrue(rows, output)
        self.assertIn("test_quick", rows[0].group(2), output)
        self.assertGreaterEqual(float(rows[0].group(1)), 0.5,
                                "setUpClass time is missing from the table")

    def test_a_run_with_no_tests_exits_5(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        result = self.run_tests(temp.name)
        self.assertEqual(5, result.returncode, result.stdout + result.stderr)


SLEEPS, LEADER = "test_slow.Slow.test_sleeps", "test_fast.Fast.test_fast_0"
FAST = tuple((f"test_fast.Fast.test_fast_{n}", "0.00") for n in range(1, 9))  # ten rows at most


class SlowestTableTests(unittest.TestCase):
    """§AR-ci.3.4.2 on canned output: the table lists its slowest first, whichever leads.

    `test_a_passing_run_ends_with_its_ten_slowest_tests` is run on a table the gate may
    print: a module fixture is charged to its module's first test, so a fast test can be
    charged more than the sleeper (#912). Each case changes only the rows.
    """

    def failures(self, *rows):
        table = "".join(f"    {seconds}s {name}\n" for name, seconds in rows)
        output = f"Slowest tests\n{table}13 tests: 13 passed, 0 failed, 0 ignored, 1.3s\n"

        class Canned(PythonGateDurationsTests):
            def suite(self, failing=False):
                return None

            def run_tests(self, start):
                return subprocess.CompletedProcess([], 0, stdout=output, stderr="")

        result = unittest.TestResult()
        Canned("test_a_passing_run_ends_with_its_ten_slowest_tests").run(result)
        self.assertEqual([], result.errors, "\n".join(trace for _, trace in result.errors))
        return [trace for _, trace in result.failures]

    @unittest.skipUnless(os.environ.get("GRUND_912_RED"),
                         "red until #912 holds the table to its order rather than to which "
                         "test leads it; implement removes this")
    def test_a_test_charged_more_than_the_sleeper_leads_it(self):
        failures = self.failures((LEADER, "0.80"), (SLEEPS, "0.50"), *FAST)
        if failures:
            self.fail(failures[0])

    def test_a_test_charged_less_than_the_sleeper_may_not_lead_it(self):
        self.assertTrue(self.failures((LEADER, "0.01"), (SLEEPS, "0.50"), *FAST))

    def test_a_table_without_the_sleeper_is_refused(self):
        self.assertTrue(self.failures((LEADER, "0.80"), *FAST))

    def test_a_sleeper_charged_less_than_it_slept_is_refused(self):
        self.assertTrue(self.failures((LEADER, "0.80"), (SLEEPS, "0.30"), *FAST))


if __name__ == "__main__":
    unittest.main()
