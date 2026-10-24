"""§AR-ci.requirements.5 — a green CI step prints a screenful, a red one its failures.

Each suite is run on a suite of its own in a temporary directory: the cargo wrapper of
§AR-ci.requirements.5.2 on a small crate, and `run_python_gate.run_tests` on a small
`unittest` tree. Both must print the shape §AR-ci.requirements.5.1 names — failures in
full, a check-in each 5%, one summary as the last line — and nothing a passing test
wrote but the lines the point keeps: the `gate time:` lines, and the `node build:` lines
of §AR-ci.3.4.1.1 that only the Python gate has, which it shows once each. `ci.yml`
and the hooks are read as text, as the parity test reads them, to hold that the CI
steps and the `cargo-test` hook go through the wrapper and that every build and install
is quiet (§AR-ci.requirements.5.4).
"""

import os
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

from test_ci_language_jobs import SHARDS

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
TABLE_ROW = re.compile(r"^\s*\d+\.\d+s \S+$")  # a row of §AR-ci.3.4.2's slowest-ten table
# The lines §AR-ci.requirements.5.1 keeps that the `building` tree writes: the build lines
# of §AR-ci.3.4.1.1 in-process, from a child process, from a skipped test and from the
# last module's teardown, and a `gate time:` line from a child process.
KEPT = ("node build: dev /in-process 5.0s", "node build: release /child 49.2s",
        "node build: dev /skipped 1.0s", "node build: dev /teardown 2.0s",
        "gate time: fixture phase 1.5s")

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
        # §AR-ci.requirements.5.1 holds the order, not the leader, which a stall can move (#912).
        seconds = [float(listed) for _, listed in slowest]
        self.assertEqual(sorted(seconds, reverse=True), seconds, f"slowest first:\n{output}")
        sleeper = [float(listed) for name, listed in slowest if "sleeps_one_second" in name]
        self.assertTrue(sleeper, f"the one-second sleeper is listed:\n{output}")
        self.assertGreaterEqual(sleeper[0], 0.9, f"the sleeper is charged its second:\n{output}")
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

    def crate(self, failing=False, broken=False, chatty=False, crashing=False, timed=False):
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
        if chatty:
            # Written past libtest's capture, as a child process writes: a partial line, and
            # the alternate-screen exit a terminal program leaves behind.
            tests += "".join(
                f"    #[test]\n    fn writes_past_capture_{n}() {{\n"
                "        use std::io::Write;\n"
                f"        let text: &[u8] = {text};\n"
                "        std::io::stdout().write_all(text).unwrap();\n"
                "        std::io::stdout().flush().unwrap();\n    }\n"
                for n, text in enumerate((r'b"NOISE-PARTIAL"', r'b"\x1b[?1049l"',
                                          r'b"NOISE-LINE\n"')))
        if timed:
            # What the command agreement's hosts write past libtest's capture, beside noise.
            tests += ("    #[test]\n    fn writes_its_gate_time() {\n"
                      "        use std::io::Write;\n"
                      "        std::io::stderr().write_all(b\"NOISE-TIMED\\n\").unwrap();\n"
                      "        std::io::stderr().write_all(b\"gate time: fixture phase 1.5s\\n\")"
                      ".unwrap();\n    }\n")
        if crashing:
            (root / "tests").mkdir()
            (root / "tests" / "crash.rs").write_text(
                "fn main() {\n    if std::env::args().any(|arg| arg == \"--list\") { return; }\n"
                "    std::process::abort();\n}\n", encoding="utf-8")
            with (root / "Cargo.toml").open("a", encoding="utf-8") as manifest:
                manifest.write('\n[[test]]\nname = "crash"\npath = "tests/crash.rs"\nharness = false\n')
        (root / "src").mkdir()
        (root / "src" / "lib.rs").write_text(
            "#[cfg(test)]\nmod tests {\n" + tests + "}\n", encoding="utf-8")
        return root

    def run_suite(self, crate, *args):
        self.assertTrue(WRAPPER.is_file(), f"{WRAPPER.relative_to(REPO)} does not exist")
        environment = dict(os.environ, CARGO_TARGET_DIR=str(crate / "target"))
        return subprocess.run(
            [sys.executable, str(WRAPPER), "--manifest-path", str(crate / "Cargo.toml"), *args],
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

    def test_output_past_libtest_capture_loses_no_test(self):
        # A partial line or an escape sequence runs into libtest's own line; on one thread
        # it splits `test <name> ... ` from its verdict.
        crate = self.crate(chatty=True)
        for args in ((), ("--", "--test-threads=1")):
            with self.subTest(args=args):
                result = self.run_suite(crate, *args)
                lines = lines_of(result)
                self.assertEqual(0, result.returncode, "\n".join(lines))
                self.assert_quiet(lines)
                self.assert_shape(lines, PASSING + 4, failed=0)

    def test_a_green_suite_passes_on_its_gate_time_lines_and_nothing_else(self):
        # The build is a phase of its own, and a line a test's child wrote to say where the
        # gate's time went survives a green run when nothing else it wrote does.
        result = self.run_suite(self.crate(timed=True))
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertEqual(0, result.returncode, output)
        self.assert_quiet(lines)
        self.assert_shape(lines, PASSING + 2, failed=0)
        timed = [line for line in lines if line.startswith("gate time: ")]
        self.assertRegex(timed[0], r"^gate time: build \d+\.\ds$", output)
        self.assertEqual(["gate time: fixture phase 1.5s"], timed[1:], output)

    def test_a_crashed_binary_is_an_error_the_summary_counts(self):
        result = self.run_suite(self.crate(crashing=True), "--no-fail-fast")
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertNotEqual(0, result.returncode, output)
        self.assertRegex(lines[-1], rf"^{PASSING + 1} tests: {PASSING + 1} passed, 0 failed, "
                                    r"0 ignored, 1 error, \d+(?:\.\d+)?s$", output)

    def test_a_broken_build_prints_the_compile_error_and_fails(self):
        result = self.run_suite(self.crate(broken=True))
        output = "\n".join(lines_of(result))
        self.assertNotEqual(0, result.returncode, output)
        self.assertIn("BUILD-ERROR-MARK", output, "the compile error is printed in full")
        self.assertIn("error[E0308]", output)


class PythonGateOutputTests(SuiteShape, unittest.TestCase):
    """§AR-ci.requirements.5.1: `run_python_gate.run_tests` on a tree of its own."""

    def tree(self, failing=False, building=False):
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
        if building:
            # Sorts after test_noise.py, so its teardown runs after the last test.
            (root / "test_writes_build_lines.py").write_text(textwrap.dedent("""
                import subprocess, sys, unittest
                def tearDownModule():
                    print("NOISE-TEARDOWN")
                    sys.stderr.write("node build: dev /teardown 2.0s\\n")
                class Builds(unittest.TestCase):
                    def test_writes_a_build_line(self):
                        print("NOISE-BUILD-OUT")
                        sys.stderr.write("node build: dev /in-process 5.0s\\n")
                        print("NOISE-BUILD-ERR", file=sys.stderr)
                    def test_child_process_writes_a_build_line(self):
                        line = "node build: release /child 49.2s"
                        subprocess.run([sys.executable, "-c",
                                        "import sys; print('NOISE-BUILD-CHILD'); "
                                        "sys.stderr.write(sys.argv[1] + '\\\\n')", line],
                                       check=True)
                    def test_child_process_writes_a_gate_time_line(self):
                        line = "gate time: fixture phase 1.5s"
                        subprocess.run([sys.executable, "-c",
                                        "import sys; print('NOISE-TIMED-OUT'); "
                                        "sys.stderr.write('NOISE-TIMED-ERR\\\\n' + sys.argv[1] "
                                        "+ '\\\\nNOISE-TIMED-END\\\\n')", line], check=True)
                    def test_skips_after_a_build_line(self):
                        sys.stderr.write("node build: dev /skipped 1.0s\\n")
                        self.skipTest("on purpose")
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
        self.assert_quiet([line for line in lines if not TABLE_ROW.match(line)])
        self.assert_shape(lines, PASSING + 1, failed=0, ignored=1)  # a skip counts as ignored

    def test_a_green_gate_shows_each_build_and_gate_time_line_once_and_nothing_else(self):
        # §AR-ci.requirements.5.1 keeps a green test's build lines of §AR-ci.3.4.1.1 and its
        # `gate time:` lines.
        result = self.run_gate(self.tree(building=True))
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertEqual(0, result.returncode, output)
        for kept in KEPT:
            self.assertEqual(1, lines.count(kept),
                             f"a green gate shows {kept!r} once, on a line of its own:\n{output}")
        self.assert_quiet([line for line in lines if not TABLE_ROW.match(line)])
        self.assert_shape(lines, PASSING + 5, failed=0, ignored=2)

    def test_a_red_gate_prints_the_failure_in_full_and_fails(self):
        result = self.run_gate(self.tree(failing=True))
        lines = lines_of(result)
        output = "\n".join(lines)
        self.assertEqual(1, result.returncode, output)
        for mark in ("test_fails_on_purpose", "FAIL-STDOUT-MARK", "FAIL-ASSERT-MARK"):
            self.assertIn(mark, output)
        self.assertFalse(any("NOISE-" in line for line in lines), output)
        self.assert_shape(lines, PASSING + 2, failed=1, ignored=1)


FAILS, SLEEPER, CHILD = ("test_fails.Fails.test_fails_on_purpose",
                         "test_noise.Noise.test_sleeps_one_second",
                         "test_noise.Noise.test_child_process_writes")


class SlowestListTests(SuiteShape, unittest.TestCase):
    """§AR-ci.requirements.5.1 on canned lines: the last check-in lists its slowest first.

    The lines are the red tree's last two check-ins and its summary as the Windows job of
    CI run 39638987585 printed them (#912), where a runner stall charged the failing test
    1.14s, ahead of the one-second sleeper. Each case changes only the last slowest list.
    """

    def shape(self, *slowest):
        listed = ", ".join(f"{name} {seconds}s" for name, seconds in slowest)
        lines = [f"40/42 tests, 1 failed, 1.2s elapsed, slowest: {FAILS} 1.14s, {CHILD} 0.02s, "
                 "test_noise.Noise.test_passes_0 0.00s",
                 f"42/42 tests, 1 failed, 2.2s elapsed, slowest: {listed}",
                 "42 tests: 40 passed, 1 failed, 1 ignored, 2.2s"]
        self.assert_shape(lines, PASSING + 2, failed=1, ignored=1)  # the red tree's call

    def test_a_test_charged_more_than_the_sleeper_leads_it(self):
        self.shape((FAILS, "1.14"), (SLEEPER, "1.00"), (CHILD, "0.02"))

    def test_a_test_charged_less_than_the_sleeper_may_not_lead_it(self):
        with self.assertRaises(AssertionError):
            self.shape((FAILS, "0.14"), (SLEEPER, "1.00"), (CHILD, "0.02"))

    def test_a_list_out_of_order_is_refused_though_the_sleeper_leads_it(self):
        with self.assertRaises(AssertionError):
            self.shape((SLEEPER, "1.00"), (CHILD, "0.02"), (FAILS, "1.14"))

    def test_a_list_without_the_sleeper_is_refused(self):
        with self.assertRaises(AssertionError):
            self.shape((FAILS, "1.14"), (CHILD, "0.02"))

    def test_a_sleeper_charged_less_than_it_slept_is_refused(self):
        with self.assertRaises(AssertionError):
            self.shape((FAILS, "1.14"), (SLEEPER, "0.50"), (CHILD, "0.02"))


def ci_steps():
    """`(name, run)` for each step of `ci.yml` whose `run:` is one line."""
    steps, name = [], None
    for raw in CI.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line.startswith("- name:"):
            name = line[len("- name:"):].strip()
        elif line.startswith("- "):  # an unnamed step, such as §AR-ci.13.6's prune
            name = None
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
            for name, run in ci_steps():
                if name == step:
                    self.assertTrue(run.startswith(WRAPPER_COMMAND), run)
        self.assertTrue(hook_entry("cargo-test").startswith(WRAPPER_COMMAND), hook_entry("cargo-test"))
        # §AR-ci.13.7: CI runs the hook's command as its shards, `--shard <k>/<n>` before it.
        everything = [run for _, run in ci_steps()]
        for index in range(1, SHARDS + 1):
            sharded = hook_entry("cargo-test").replace(
                WRAPPER_COMMAND, f"{WRAPPER_COMMAND}--shard {index}/{SHARDS} ", 1)
            self.assertIn(sharded, everything, "the hook and CI spell one command")

    def test_no_ci_step_runs_cargo_test_bare(self):
        text = CI.read_text(encoding="utf-8")
        self.assertEqual([], re.findall(r"^\s*run: cargo test\b.*$", text, re.M))

    def test_every_build_and_install_is_quiet(self):
        text = CI.read_text(encoding="utf-8")
        shell = re.findall(r"^\s*(?:run:\s*)?((?:python -m )?(?:cargo|pip) (?:build|run|install|bench)\b.*)$",
                           text, re.M)
        listed = re.findall(r"\[\s*(\"cargo\",[^\]]*?\"(?:build|run)\"[^\]]*)",
                            text + ORACLE.read_text(encoding="utf-8")
                            + (REPO / "scripts/check_ci_self.py").read_text(encoding="utf-8"), re.S)
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
