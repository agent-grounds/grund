"""§AR-ci.13.7 — `scripts/run_cargo_suite.py --shard <k>/<n>` runs one shard of a suite.

The wrapper runs on a workspace of its own in a temporary directory: two member crates,
each with a library and an integration test, whose four targets hold 1, 2, 4 and 8 tests.
Every sum of a set of them is distinct, so shards whose totals are each above zero and add
up to the whole suite's total hold every target exactly once: none dropped, none run twice.
A shard that selects nothing and a malformed `--shard` are refused with their own statuses.
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
# The summary line of §AR-ci.requirements.5.1: `<total> tests: <passed> passed, …`.
SUMMARY = re.compile(r"(?m)^(\d+) tests: (\d+) passed, (\d+) failed")
# The four targets and their tests: a library and an integration test in each member.
TARGETS = {("alpha", "lib"): 1, ("alpha", "two"): 2, ("beta", "lib"): 4, ("beta", "eight"): 8}


def tests(count, prefix):
    return "".join(f"#[test]\nfn {prefix}_{n}() {{}}\n" for n in range(count))


class CargoSuiteShardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temp.name)
        (cls.root / "Cargo.toml").write_text(textwrap.dedent("""
            [workspace]
            members = ["alpha", "beta"]
            resolver = "2"
            """), encoding="utf-8")
        for member in ("alpha", "beta"):
            crate = cls.root / member
            (crate / "src").mkdir(parents=True)
            (crate / "tests").mkdir()
            (crate / "Cargo.toml").write_text(textwrap.dedent(f"""
                [package]
                name = "{member}"
                version = "0.0.0"
                edition = "2021"
                publish = false
                """), encoding="utf-8")
            for (owner, target), count in TARGETS.items():
                if owner != member:
                    continue
                if target == "lib":
                    body = "#[cfg(test)]\nmod tests {\n" + tests(count, "unit") + "}\n"
                    (crate / "src" / "lib.rs").write_text(body, encoding="utf-8")
                else:
                    (crate / "tests" / f"{target}.rs").write_text(tests(count, target),
                                                                  encoding="utf-8")

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def run_suite(self, *args, env=None):
        environment = dict(os.environ, CARGO_TARGET_DIR=str(self.root / "target"), **(env or {}))
        return subprocess.run(
            [sys.executable, str(WRAPPER), *args, "--manifest-path", str(self.root / "Cargo.toml"),
             "--workspace", "--all-targets"],
            cwd=REPO, env=environment, capture_output=True, text=True, timeout=600)

    def total(self, *args):
        result = self.run_suite(*args)
        output = result.stdout + result.stderr
        self.assertEqual(0, result.returncode, output)
        found = SUMMARY.findall(output)
        self.assertTrue(found, f"no summary line:\n{output}")
        total, passed, failed = map(int, found[-1])
        self.assertEqual((total, 0), (passed, failed), output)
        return total

    def test_the_shards_together_run_every_target_once(self):
        whole = self.total()
        self.assertEqual(sum(TARGETS.values()), whole)
        for count in (1, 2, 3, 4):
            with self.subTest(shards=count):
                totals = [self.total("--shard", f"{index}/{count}")
                          for index in range(1, count + 1)]
                self.assertTrue(all(totals), f"a shard of {count} ran nothing: {totals}")
                self.assertEqual(whole, sum(totals), f"the {count} shards ran {totals}")

    def test_the_cut_is_the_same_on_every_run(self):
        first = [self.total("--shard", f"{index}/2") for index in (1, 2)]
        self.assertEqual(first, [self.total("--shard", f"{index}/2") for index in (1, 2)])

    def test_a_shard_that_selects_no_target_exits_5(self):
        # Four targets dealt five ways leave at least one shard with none.
        statuses = [self.run_suite("--shard", f"{index}/5").returncode for index in range(1, 6)]
        self.assertIn(5, statuses)
        self.assertEqual(set(), set(statuses) - {0, 5})

    def test_a_malformed_shard_is_refused_before_cargo_runs(self):
        # No cargo on the PATH: reaching it fails otherwise than with the usage status.
        empty = tempfile.TemporaryDirectory()
        self.addCleanup(empty.cleanup)
        for shard in ("0/2", "3/2", "1/0", "2", "a/b", "-1/2"):
            with self.subTest(shard=shard):
                result = self.run_suite("--shard", shard, env={"PATH": empty.name})
                self.assertEqual(2, result.returncode, result.stdout + result.stderr)
                self.assertIn("usage", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()
