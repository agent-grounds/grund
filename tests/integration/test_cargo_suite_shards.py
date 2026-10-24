"""§AR-ci.13.7 — `scripts/run_cargo_suite.py --shard <k>/<n>` runs one shard of a suite.

The wrapper runs on a workspace of its own in a temporary directory: two member crates,
each with a library and an integration test, whose four targets hold 1, 2, 4 and 8 tests.
Every sum of a set of them is distinct, so shards whose totals are each above zero and add
up to the whole suite's total hold every target exactly once: none dropped, none run twice.
A shard that selects nothing and a malformed `--shard` are refused with their own statuses.
The deal itself is checked in-process on listings of cargo's `Running` lines: the binaries
`cargo_shards.WEIGHTS` names, and light ones it does not.
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
sys.path.insert(0, str(REPO / "scripts"))
import cargo_shards  # noqa: E402
# The summary line of §AR-ci.requirements.5.1: `<total> tests: <passed> passed, …`.
SUMMARY = re.compile(r"(?m)^(\d+) tests: (\d+) passed, (\d+) failed")
# The four targets and their tests: a library and an integration test in each member.
TARGETS = {("alpha", "lib"): 1, ("alpha", "two"): 2, ("beta", "lib"): 4, ("beta", "eight"): 8}


def tests(count, prefix):
    return "".join(f"#[test]\nfn {prefix}_{n}() {{}}\n" for n in range(count))


def listed(name, suffix=""):
    """The `Running` line cargo prints for the test target `name`, its executable hashed."""
    executable = Path("target", "debug", "deps", f"{name}-0123456789abcdef{suffix}")
    return f"Running tests/{name}.rs ({executable})"


def placed(shards):
    """Each target's shard, counted from 1."""
    return {cargo_shards.target(key): index for index, keys in enumerate(shards, 1)
            for key in keys}


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


class CargoShardDealTests(unittest.TestCase):
    """§AR-ci.13.7: the deal weighs the slow binaries, so they share no shard."""

    def listing(self, suffix=""):
        names = [*cargo_shards.WEIGHTS, *(f"light_{n}" for n in range(30))]
        return [listed(name, suffix) for name in names] + ["Doc-tests grund_core"]

    def test_every_binary_lands_in_exactly_one_shard_on_every_call(self):
        for suffix in ("", ".exe"):
            keys = self.listing(suffix)
            for count in (1, 2, 3, 4, 5):
                with self.subTest(suffix=suffix, shards=count):
                    shards = cargo_shards.deal(keys, count)
                    self.assertEqual(sorted(keys), sorted(key for keys_ in shards for key in keys_))
                    self.assertEqual(shards, cargo_shards.deal(list(reversed(keys)), count))

    def test_the_slowest_binaries_share_no_shard_and_leave_shard_one(self):
        heaviest = sorted(cargo_shards.WEIGHTS, key=cargo_shards.WEIGHTS.get, reverse=True)
        for suffix in ("", ".exe"):
            with self.subTest(suffix=suffix):
                where = placed(cargo_shards.deal(self.listing(suffix), 4))
                apart = ("command_agreement", "scan_determinism", "grund_core")
                self.assertEqual(3, len({where[name] for name in apart}), where)
                # Shard 1 runs the leg's other Rust checks first (§AR-ci.13.1).
                self.assertNotIn(1, {where[name] for name in heaviest[:3]}, where)
                self.assertEqual(3, len({where[name] for name in heaviest[:3]}), where)

    def test_the_light_binaries_are_dealt_round_the_shards(self):
        shards = cargo_shards.deal(self.listing(), 4)
        light = [sum(key not in cargo_shards.WEIGHTS for key in map(cargo_shards.target, keys))
                 for keys in shards]
        self.assertLessEqual(max(light) - min(light), 1, light)


if __name__ == "__main__":
    unittest.main()
