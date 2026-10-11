"""§AR-ci.requirements.5 — a green CI step prints a screenful, a red one its failures.

Each suite is run on a suite of its own in a temporary directory: the cargo wrapper of
§AR-ci.requirements.5.2 on a small crate, and `run_python_gate.run_tests` on a small
`unittest` tree. Both must print the shape §AR-ci.requirements.5.1 names — failures in
full, a check-in each 5%, one summary as the last line — and nothing a passing test
wrote. `ci.yml` and the hooks are read as text, as the parity test reads them, to hold
that the CI steps and the `cargo-test` hook go through the wrapper and that every build
and install is quiet (§AR-ci.requirements.5.4).
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
WRAPPER = REPO / "scripts" / "run_cargo_suite.py"
CI = REPO / ".github" / "workflows" / "ci.yml"
PRE_COMMIT = REPO / ".pre-commit-config.yaml"
# The oracle build every job that runs the Python gate shares (§AR-ci.13.1).
ORACLE = REPO / "scripts" / "prepare_binding_oracle.py"
WRAPPER_COMMAND = "python scripts/run_cargo_suite.py "

ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
CHECK_IN = re.compile(r"^(\d+)/(\d+) tests, (\d+) failed, (\d+(?:\.\d+)?)s elapsed, slowest: (.+)$")
SLOWEST = re.compile(r"(\S+) (\d+(?:\.\d+)?)s")
SUMMARY = re.compile(r"^(\d+) tests: (\d+) passed, (\d+) failed, (\d+) ignored, (\d+(?:\.\d+)?)s$")
LIBTEST_OK = re.compile(r"^test .* \.\.\. ok$")
CARGO_PROGRESS = re.compile(r"^\s*(Compiling|Finished|Running|Doc-tests|Downloading|Downloaded)\b")
DOTS = re.compile(r"^[.sxuEF]+$")
GREEN_BUDGET = 30  # the ticket's "about twenty lines per suite", with slack
PASSING = 40

PYTHON_CHILD = """
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate
sys.exit(run_python_gate.run_tests(sys.argv[2], dict(os.environ)))
"""


def lines_of(result):
    text = ANSI.sub("", result.stdout + result.stderr)
    return [line.rstrip() for line in text.splitlines() if line.strip()]


class SuiteShape:
    """The assertions §AR-ci.requirements.5.1 makes of any suite's output."""

    def assert_shape(self, lines, total, failed, ignored=0):
        output = "\n".join(lines)
        check_ins = [CHECK_IN.match(line) for line in lines]
        check_ins = [match for match in check_ins if match]
        self.assertGreaterEqual(len(check_ins), 2, f"no periodic check-in:\n{output}")
        self.assertLessEqual(len(check_ins), 20, f"a check-in more often than each 5%:\n{output}")
        done = [int(match.group(1)) for match in check_ins]
        self.assertEqual(sorted(done), done, output)
        for match in check_ins:
            self.assertEqual(total, int(match.group(2)), f"the total is the whole suite:\n{output}")
        last = check_ins[-1]
        self.assertEqual(total, int(last.group(1)), f"the last check-in is {total}/{total}:\n{output}")
        self.assertEqual(failed, int(last.group(3)), output)
        slowest = SLOWEST.findall(last.group(5))
        self.assertTrue(1 <= len(slowest) <= 3, output)
        self.assertIn("sleeps_one_second", slowest[0][0], f"the slowest test leads:\n{output}")
        self.assertGreaterEqual(float(slowest[0][1]), 0.9, output)
        summary = SUMMARY.match(lines[-1])
        self.assertIsNotNone(summary, f"the last line is not the summary:\n{output}")
        self.assertEqual((total, total - failed - ignored, failed, ignored),
                         tuple(int(summary.group(n)) for n in (1, 2, 3, 4)), output)

    def assert_quiet(self, lines):
        output = "\n".join(lines)
        self.assertLessEqual(len(lines), GREEN_BUDGET, f"a green suite printed:\n{output}")
        for line in lines:
            self.assertIsNone(LIBTEST_OK.match(line), f"a line per passing test:\n{output}")
            self.assertIsNone(CARGO_PROGRESS.match(line), f"build progress:\n{output}")
            self.assertIsNone(DOTS.match(line), f"a unittest dot stream:\n{output}")
            self.assertNotIn("NOISE-", line, f"output of a passing test:\n{output}")


class CargoSuiteOutputTests(SuiteShape, unittest.TestCase):
    """§AR-ci.requirements.5.2: `scripts/run_cargo_suite.py` on a crate of its own."""

    def crate(self, failing=False, broken=False):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "Cargo.toml").write_text(textwrap.dedent("""
            [package]
            name = "ci_output_fixture"
            version = "0.0.0"
            edition = "2021"
            publish = false

            [workspace]
            """), encoding="utf-8")
        tests = "".join(f"    #[test]\n    fn passes_{n}() {{ println!(\"NOISE-{n}\"); }}\n"
                        for n in range(PASSING))
        tests += ("    #[test]\n    fn sleeps_one_second() {\n"
                  "        std::thread::sleep(std::time::Duration::from_secs(1));\n    }\n")
        if failing:
            tests += ("    #[test]\n    fn fails_on_purpose() {\n"
                      "        println!(\"FAIL-STDOUT-MARK\");\n"
                      "        panic!(\"FAIL-PANIC-MARK\");\n    }\n")
        if broken:
            tests += "    fn does_not_compile() -> u32 { \"BUILD-ERROR-MARK\" }\n"
        (root / "src").mkdir()
        (root / "src" / "lib.rs").write_text(
            "#[cfg(test)]\nmod tests {\n" + tests + "}\n", encoding="utf-8")
        return root

    def run_suite(self, crate):
        self.assertTrue(WRAPPER.is_file(), f"{WRAPPER.relative_to(REPO)} does not exist")
        environment = dict(os.environ, CARGO_TARGET_DIR=str(crate / "target"))
        return subprocess.run(
            [sys.executable, str(WRAPPER), "--manifest-path", str(crate / "Cargo.toml")],
            cwd=REPO, env=environment, capture_output=True, text=True, timeout=600)

    def test_a_green_suite_prints_check_ins_and_a_summary_only(self):
        result = self.run_suite(self.crate())
        lines = lines_of(result)
        self.assertEqual(0, result.returncode, "\n".join(lines))
        self.assert_quiet(lines)
        self.assert_shape(lines, PASSING + 1, failed=0)

    def test_a_red_suite_prints_the_failure_in_full_and_fails(self):
        result = self.run_suite(self.crate(failing=True))
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertEqual(101, result.returncode, f"cargo test's own status:\n{output}")
        for mark in ("fails_on_purpose", "FAIL-STDOUT-MARK", "FAIL-PANIC-MARK"):
            self.assertIn(mark, output)
        self.assertFalse(any(LIBTEST_OK.match(line) for line in lines), output)
        self.assertFalse(any("NOISE-" in line for line in lines), output)
        self.assert_shape(lines, PASSING + 2, failed=1)

    def test_a_broken_build_prints_the_compile_error_and_fails(self):
        result = self.run_suite(self.crate(broken=True))
        output = "\n".join(lines_of(result))
        self.assertNotEqual(0, result.returncode, output)
        self.assertIn("BUILD-ERROR-MARK", output, "the compile error is printed in full")
        self.assertIn("error[E0308]", output)


class PythonGateOutputTests(SuiteShape, unittest.TestCase):
    """§AR-ci.requirements.5.1: `run_python_gate.run_tests` on a tree of its own."""

    def tree(self, failing=False):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        passing = "".join(f"    def test_passes_{n}(self):\n        pass\n" for n in range(PASSING - 3))
        (root / "test_noise.py").write_text(textwrap.dedent("""
            import subprocess, sys, time, unittest
            class Noise(unittest.TestCase):
                def test_sleeps_one_second(self):
                    time.sleep(1)
                def test_prints(self):
                    print("NOISE-STDOUT")
                    print("NOISE-STDERR", file=sys.stderr)
                def test_child_process_writes(self):
                    subprocess.run([sys.executable, "-c",
                                    "import sys; print('NOISE-CHILD-OUT'); "
                                    "sys.stderr.write('NOISE-CHILD-ERR\\\\n')"], check=True)
                def test_skips(self):
                    self.skipTest("on purpose")
            """) + passing, encoding="utf-8")
        if failing:
            (root / "test_fails.py").write_text(textwrap.dedent("""
                import unittest
                class Fails(unittest.TestCase):
                    def test_fails_on_purpose(self):
                        print("FAIL-STDOUT-MARK")
                        self.fail("FAIL-ASSERT-MARK")
                """), encoding="utf-8")
        return root

    def run_gate(self, start):
        return subprocess.run([sys.executable, "-c", PYTHON_CHILD, str(REPO / "scripts"), str(start)],
                              cwd=REPO, env=dict(os.environ), capture_output=True, text=True,
                              timeout=600)

    def test_a_green_gate_prints_check_ins_and_a_summary_only(self):
        result = self.run_gate(self.tree())
        lines = lines_of(result)
        self.assertEqual(0, result.returncode, "\n".join(lines))
        # The slowest-ten table of §AR-ci.3.4.2 stays: eleven lines the budget allows for.
        self.assert_quiet([line for line in lines if not re.match(r"^\s*\d+\.\d+s \S+$", line)])
        self.assert_shape(lines, PASSING + 1, failed=0, ignored=1)  # a skip counts as ignored

    def test_a_red_gate_prints_the_failure_in_full_and_fails(self):
        result = self.run_gate(self.tree(failing=True))
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertEqual(1, result.returncode, output)
        for mark in ("test_fails_on_purpose", "FAIL-STDOUT-MARK", "FAIL-ASSERT-MARK"):
            self.assertIn(mark, output)
        self.assertFalse(any("NOISE-" in line for line in lines), output)
        self.assert_shape(lines, PASSING + 2, failed=1, ignored=1)


def ci_steps():
    """`(name, run)` for each step of `ci.yml` whose `run:` is one line."""
    steps, name = [], None
    for raw in CI.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line.startswith("- name:"):
            name = line[len("- name:"):].strip()
        elif line.startswith("run:") and line != "run: |" and name is not None:
            steps.append((name, line[len("run:"):].strip()))
    return steps


def hook_entry(hook_id):
    lines = PRE_COMMIT.read_text(encoding="utf-8").splitlines()
    start = next(i for i, line in enumerate(lines) if line.strip() == f"- id: {hook_id}")
    entry = next(line for line in lines[start:] if line.strip().startswith("entry:"))
    return entry.strip()[len("entry:"):].strip()


class CiStepsGoThroughTheQuietFormsTests(unittest.TestCase):
    """§AR-ci.requirements.5.2 and §AR-ci.requirements.5.4, read from the files CI runs."""

    def test_the_cargo_suite_steps_and_the_hook_call_the_wrapper(self):
        runs = dict(ci_steps())
        for step in ("native watch contract", "cargo test"):
            with self.subTest(step=step):
                self.assertTrue(runs[step].startswith(WRAPPER_COMMAND), runs[step])
        self.assertTrue(hook_entry("cargo-test").startswith(WRAPPER_COMMAND), hook_entry("cargo-test"))
        self.assertIn(hook_entry("cargo-test"), runs.values(), "the hook and CI spell one command")

    def test_no_ci_step_runs_cargo_test_bare(self):
        text = CI.read_text(encoding="utf-8")
        self.assertEqual([], re.findall(r"^\s*run: cargo test\b.*$", text, re.M))

    def test_every_build_and_install_is_quiet(self):
        text = CI.read_text(encoding="utf-8")
        shell = re.findall(r"^\s*(?:run:\s*)?((?:python -m )?(?:cargo|pip) (?:build|run|install)\b.*)$",
                           text, re.M)
        listed = re.findall(r"\[\s*(\"cargo\",[^\]]*?\"(?:build|run)\"[^\]]*)",
                            text + ORACLE.read_text(encoding="utf-8"), re.S)
        self.assertGreaterEqual(len(shell), 4, shell)
        self.assertGreaterEqual(len(listed), 3, listed)
        for command in shell:
            with self.subTest(command=command):
                self.assertRegex(command, r"(^|\s)(--quiet|-q)(\s|$)")
        for command in listed:
            with self.subTest(command=" ".join(command.split())):
                self.assertIn('"--quiet"', command)
        self.assertIn("--quiet", hook_entry("cargo-build").split())


if __name__ == "__main__":
    unittest.main()
