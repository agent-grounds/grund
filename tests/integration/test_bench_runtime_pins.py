"""§AR-benchmarks.3.1 — iai-callgrind clears the environment before Callgrind starts the
measured program, so the harness hands it exactly the runtime pins the shard jobs set,
`RAYON_NUM_THREADS`, `LC_ALL` and `TZ`, and none of the build-only pins of §AR-ci.5.

Read as text, the way `test_bench_tier_ci.py` reads the workflows: the regular matrix has
no Valgrind, so what the harness forwards is read from its source, where it is one list
given to `pass_through_envs`, and held to the shard jobs' `env:`."""

import re
import unittest

from ci_workflow import header
from test_ci_language_jobs import BENCH_ENV, job_env
from test_release_workflow import REPO_ROOT, WORKFLOWS, jobs

HARNESS = REPO_ROOT / "crates/grund-cli/benches/instructions.rs"
SHARD_JOBS = {WORKFLOWS / "ci.yml": "bench", WORKFLOWS / "bench-nightly.yml": "bench"}
# §AR-benchmarks.3.1: what the measured process reads, and what only shapes the build.
RUNTIME_PINS = {"RAYON_NUM_THREADS", "LC_ALL", "TZ"}
BUILD_PINS = {"CARGO_INCREMENTAL", "RUSTFLAGS"}


def function_body(source, name):
    """The body of `fn <name>`, up to the closing brace at column 0."""
    found = re.search(rf"(?ms)^fn {name}\b.*?^\}}", source)
    return found.group(0) if found else ""


def string_list(source, text):
    """The string literals of `text`: an array literal, or the name of a `const` array."""
    text = text.strip()
    if re.fullmatch(r"[A-Z_][A-Z0-9_]*", text):
        const = re.search(rf"(?ms)^\s*(?:pub\s+)?const {text}\s*:[^=]*=\s*(\[.*?\]);", source)
        if not const:
            return None
        text = const.group(1)
    return re.findall(r'"([^"]*)"', text)


def forwarded(source):
    """The variables `limits()` passes through to the measured process, or None when it
    passes none (§AR-benchmarks.3.1)."""
    call = re.search(r"\.pass_through_envs\(\s*([^)]*?)\s*\)", function_body(source, "limits"))
    if not call:
        return None
    names = string_list(source, call.group(1))
    return None if names is None else set(names)


class TheMeasuredProcessRunsUnderTheRuntimePins(unittest.TestCase):
    """§AR-benchmarks.3.1"""

    @classmethod
    def setUpClass(cls):
        cls.source = HARNESS.read_text(encoding="utf-8")
        cls.forwarded = forwarded(cls.source)

    def test_the_spec_names_the_jobs_own_pins(self):
        self.assertEqual(set(BENCH_ENV), RUNTIME_PINS | BUILD_PINS)
        for workflow, name in SHARD_JOBS.items():
            env = job_env(header(jobs(workflow.read_text(encoding="utf-8"))[name]))
            for pin in sorted(RUNTIME_PINS):
                with self.subTest(workflow=workflow.name, pin=pin):
                    self.assertIn(pin, env, f"the {name} job sets no {pin} to forward")

    def test_the_harness_forwards_exactly_the_runtime_pins(self):
        self.assertIsNotNone(
            self.forwarded,
            "limits() in instructions.rs calls no pass_through_envs: iai-callgrind clears the "
            "environment, so the measured grund runs without RAYON_NUM_THREADS, LC_ALL and TZ")
        self.assertEqual(RUNTIME_PINS, self.forwarded)

    def test_no_build_pin_is_forwarded(self):
        self.assertFalse((self.forwarded or set()) & BUILD_PINS)

    def test_the_rest_of_the_environment_stays_cleared(self):
        self.assertIsNone(re.search(r"\.env_clear\(\s*false\s*\)", self.source),
                          "env_clear(false) lets the caller's whole environment into the count")


if __name__ == "__main__":
    unittest.main()
