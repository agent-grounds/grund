"""§AR-benchmarks.7 — `scripts/bench_record.py` normalizes a benchmark run into one
run record, keys it to a series, and refuses to compare records of two series.

§AR-benchmarks.5.1 — a scenario the base binary refuses is reported `no base, not
compared` rather than failing the comparison; the probe that finds it is driven
here with a fake base binary, since the regular matrix has no Valgrind."""

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "bench_record.py"


def load():
    if not SCRIPT.is_file():
        raise AssertionError(f"{SCRIPT.name} does not exist: §AR-benchmarks.7 has no record writer")
    spec = importlib.util.spec_from_file_location("bench_record", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


# Every identity field §AR-benchmarks.7.1 names, with a value a pinned CI run would give.
IDENTITY = {
    "record_version": 1,
    "environment": "ci",
    "image": "sha256:" + "a" * 64,
    "rustc": "rustc 1.90.0 (1159e78c4 2025-09-14)",
    "valgrind": "valgrind-3.19.0",
    "runner": "0.16.1",
    "build_flags": {"RUSTFLAGS": "", "CARGO_INCREMENTAL": "0", "profile": "bench"},
    "locale": "C.UTF-8",
    "timezone": "UTC",
    "threads": 4,
    "fixtures": {"canonical-repo": {"schema_version": 1, "digest": "b" * 64}},
}
# A different value for each identity field, which must move the series key.
CHANGED = {
    "record_version": 2,
    "environment": "local",
    "image": "sha256:" + "c" * 64,
    "rustc": "rustc 1.91.0 (f8297e351 2025-10-28)",
    "valgrind": "valgrind-3.20.0",
    "runner": "0.16.2",
    "build_flags": {"RUSTFLAGS": "-C target-cpu=native", "CARGO_INCREMENTAL": "0",
                    "profile": "bench"},
    "locale": "en_US.UTF-8",
    "timezone": "Europe/Belgrade",
    "threads": 8,
    "fixtures": {"canonical-repo": {"schema_version": 1, "digest": "d" * 64}},
}


def record(module, commit="1" * 40, binary="2" * 64, identity=None, ir=None, refused=()):
    return module.build_record(
        commit=commit,
        binary_sha256=binary,
        identity=dict(IDENTITY if identity is None else identity),
        ir={"check": 764_400_000, "cover": 558_000_000} if ir is None else ir,
        refused=list(refused),
    )


def run(*args):
    return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)],
                          capture_output=True, text=True, check=False)


class RecordTests(unittest.TestCase):
    """§AR-benchmarks.7.1, §AR-benchmarks.7.2"""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    def test_the_identity_fields_are_the_ones_the_spec_names(self):
        self.assertEqual(set(IDENTITY), set(self.module.IDENTITY_FIELDS))

    def test_a_record_has_every_identity_field_and_its_counts(self):
        built = record(self.module, refused=["show"])
        for name in IDENTITY:
            with self.subTest(field=name):
                self.assertEqual(IDENTITY[name], built[name])
        self.assertEqual("1" * 40, built["commit"])
        self.assertEqual("2" * 64, built["binary_sha256"])
        self.assertEqual(764_400_000, built["ir"]["check"])
        self.assertEqual(["show"], built["refused"])
        self.assertRegex(built["series"], r"^[0-9a-f]{64}$")

    def test_a_record_missing_an_identity_field_is_refused(self):
        for name in IDENTITY:
            with self.subTest(field=name):
                partial = {key: value for key, value in IDENTITY.items() if key != name}
                with self.assertRaisesRegex(ValueError, name):
                    record(self.module, identity=partial)

    def test_every_identity_field_moves_the_series_key(self):
        base = self.module.series_key(record(self.module))
        for name, value in CHANGED.items():
            with self.subTest(field=name):
                changed = dict(IDENTITY, **{name: value})
                self.assertNotEqual(base, self.module.series_key(record(self.module, identity=changed)))

    def test_the_commit_binary_and_counts_do_not_move_the_series_key(self):
        base = self.module.series_key(record(self.module))
        other = record(self.module, commit="3" * 40, binary="4" * 64,
                       ir={"check": 1, "cover": 2}, refused=["cover"])
        self.assertEqual(base, self.module.series_key(other))


class CompareTests(unittest.TestCase):
    """§AR-benchmarks.7.2: records of two series are never compared."""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    def write(self, directory, name, value):
        path = Path(directory) / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def test_one_series_compares(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = self.write(tmp, "base.json", record(self.module))
            head = self.write(tmp, "head.json", record(self.module, commit="3" * 40, binary="4" * 64))
            result = run("compare", base, head)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_two_series_are_refused_naming_the_field(self):
        for name in ("fixtures", "rustc", "image", "threads"):
            with self.subTest(field=name), tempfile.TemporaryDirectory() as tmp:
                base = self.write(tmp, "base.json", record(self.module))
                changed = dict(IDENTITY, **{name: CHANGED[name]})
                head = self.write(tmp, "head.json", record(self.module, identity=changed))
                result = run("compare", base, head)
                self.assertNotEqual(0, result.returncode, "two series were compared")
                self.assertIn(name, result.stdout + result.stderr)


class JitterTests(unittest.TestCase):
    """§AR-benchmarks.7.3: two runs of one binary differ by at most the same-binary bound."""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    def check(self, first_ir, second_ir):
        with tempfile.TemporaryDirectory() as tmp:
            first = Path(tmp) / "first.json"
            second = Path(tmp) / "second.json"
            first.write_text(json.dumps(record(self.module, ir=first_ir)), encoding="utf-8")
            second.write_text(json.dumps(record(self.module, ir=second_ir)), encoding="utf-8")
            return run("jitter", first, second)

    def test_the_bound_is_one_percent(self):
        self.assertEqual(1.0, self.module.JITTER_BOUND_PCT)

    def test_counts_within_the_bound_pass(self):
        result = self.check({"check": 1_000_000, "cover": 2_000_000},
                            {"check": 1_005_000, "cover": 1_990_000})
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_a_count_past_the_bound_fails_naming_the_scenario(self):
        result = self.check({"check": 1_000_000, "cover": 2_000_000},
                            {"check": 1_000_000, "cover": 2_050_000})
        self.assertNotEqual(0, result.returncode)
        self.assertIn("cover", result.stdout + result.stderr)


@unittest.skipIf(os.name == "nt", "the fake base binary is a POSIX script; the bench job is Linux")
class BaseRefusesAScenarioTests(unittest.TestCase):
    """§AR-benchmarks.5.1: a base binary that refuses one scenario's invocation."""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)
        # The base binary knows every subcommand but `cover`, as an older grund
        # would not know a subcommand a pull request adds.
        self.base_bin = self.dir / "grund-base"
        self.base_bin.write_text(
            f"#!{sys.executable}\nimport sys\n"
            "if sys.argv[1:2] == ['cover']:\n"
            "    sys.stderr.write(\"error: unrecognized subcommand 'cover'\\n\")\n"
            "    sys.exit(2)\n",
            encoding="utf-8")
        self.base_bin.chmod(0o755)
        self.refused = self.dir / "refused.txt"

    def tearDown(self):
        self.tmp.cleanup()

    def probe(self, scenario, *args):
        return run("probe", "--bin", self.base_bin, "--scenario", scenario,
                   "--refused", self.refused, "--", *args)

    def test_the_probe_passes_a_scenario_the_base_runs(self):
        result = self.probe("check", "check", self.dir)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual([], self.module.read_refused(self.refused))

    def test_the_probe_records_a_scenario_the_base_refuses(self):
        self.probe("check", "check", self.dir)
        result = self.probe("cover", "cover", self.dir)
        self.assertNotEqual(0, result.returncode, "the harness would measure a refused scenario")
        self.assertEqual(["cover"], self.module.read_refused(self.refused))

    def test_a_refused_scenario_is_reported_and_does_not_fail_the_comparison(self):
        self.probe("check", "check", self.dir)
        self.probe("cover", "cover", self.dir)
        refused = self.module.read_refused(self.refused)
        base = record(self.module, ir={"check": 764_400_000}, refused=refused)
        head = record(self.module, commit="3" * 40, binary="4" * 64,
                      ir={"check": 764_500_000, "cover": 558_000_000})
        (self.dir / "base.json").write_text(json.dumps(base), encoding="utf-8")
        (self.dir / "head.json").write_text(json.dumps(head), encoding="utf-8")
        result = run("compare", self.dir / "base.json", self.dir / "head.json")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertTrue(any("cover" in line and "no base, not compared" in line for line in lines),
                        f"`cover` is not reported as not compared:\n{result.stdout}")
        self.assertTrue(any("check" in line and "no base" not in line for line in lines),
                        f"`check`, which both binaries ran, is not compared:\n{result.stdout}")


if __name__ == "__main__":
    unittest.main()
