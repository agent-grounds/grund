"""§AR-benchmarks.1.2, §AR-benchmarks.1.5, §AR-ci.5.2 — one declarative scenario catalog,
`crates/grund-cli/benches/scenarios.json`, defines the instruction benchmarks: the harness
builds its group from it, every row carries a relative limit and an absolute `Ir` ceiling,
and its `pgo` subset is the one training list `scripts/distribution/train.py` reads.

Static reads and one `train.py describe` run: no Valgrind, so it runs in pre-commit. The
counts themselves come from the bench job, where a ceiling crossed fails `cargo bench`
(§AR-benchmarks.5.2).
"""

import json
import math
import re
import subprocess
import sys
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
CATALOG = REPO / "crates/grund-cli/benches/scenarios.json"
HARNESS = REPO / "crates/grund-cli/benches/instructions.rs"
PGO_BUILD = REPO / "scripts/pgo-build.sh"
TRAIN = REPO / "scripts/distribution/train.py"
CI = REPO / ".github/workflows/ci.yml"
AR_BENCHMARKS = REPO / "docs/architecture/AR-benchmarks.md"
BOUND = re.compile(r"The same-binary bound is \*\*([0-9.]+)%\*\* `Ir`")

# The ten scenarios as the hand-written harness ran them at cf706c4530: name, the
# arguments before the fixture path (`{id}` the fixture's subject), the fixture, the
# relative limit. Equal invocations on equal fixtures are what keep the counts equal.
TODAY = (
    ("check", ["check"], "canonical", 12.5),
    ("check_large_10k", ["check"], "large", 5.0),
    ("check_citations", ["check"], "citations", 12.5),
    ("list", ["list"], "canonical", 12.5),
    ("show_brief", ["show", "{id}", "--brief"], "canonical", 12.5),
    ("show", ["show", "{id}"], "canonical", 12.5),
    ("show_full", ["show", "{id}", "--full"], "canonical", 12.5),
    ("refs", ["refs", "{id}"], "canonical", 12.5),
    ("cover", ["cover"], "canonical", 12.5),
    ("fmt_check", ["fmt", "--check"], "canonical", 12.5),
)
FIXTURES = {
    "canonical": ("target/bench-fixtures/canonical-repo", 1_500, False),
    "large": ("target/bench-fixtures/large-conformant-repo", 10_000, False),
    "citations": ("target/bench-fixtures/canonical-citations-repo", 1_500, True),
}
FIXTURE_ID = "FS-00001-feature-00001"
PGO_SUBSET = ("check", "list", "show_brief", "show", "show_full", "refs", "cover", "fmt_check")

# §FS-distribution-candidate.7.3: the `training` evidence `pgo-build.sh` recorded for the
# `grund` executable at cf706c4530, and the two API workloads `train.py describe` printed.
# `training_sha256` and the profile key hang on these bytes.
GRUND_TRAINING = ('["grund check", "grund list", "grund show FS-check --brief", '
                  '"grund show FS-check", "grund show FS-check --full", '
                  '"grund refs GOAL-fast-feedback", "grund cover", "grund fmt --check"]')
API = ("check(root)", "list(root)", "show FS-check --brief", "show FS-check",
       "show FS-check --full", "refs GOAL-fast-feedback", "cover(root)", "fmt preview")


def catalog():
    return json.loads(CATALOG.read_text(encoding="utf-8"))


def rows():
    return catalog()["scenarios"]


def describe(product):
    result = subprocess.run([sys.executable, "-I", str(TRAIN), "describe", product],
                            capture_output=True, text=True, encoding="utf-8", check=False)
    return result.returncode, result.stdout.rstrip("\n"), result.stderr


class TheCatalogIsTheScenarioList(unittest.TestCase):
    """§AR-benchmarks.1.2: one catalog, and the ten scenarios it starts with are today's."""

    def test_the_catalog_exists_and_is_json(self):
        self.assertTrue(CATALOG.is_file(), f"no scenario catalog at {CATALOG.relative_to(REPO)}")
        data = catalog()
        self.assertIsInstance(data.get("fixtures"), dict, "the catalog names no `fixtures`")
        self.assertIsInstance(data.get("scenarios"), list, "the catalog lists no `scenarios`")

    def test_the_fixtures_are_the_generated_ones(self):
        fixtures = catalog()["fixtures"]
        self.assertEqual(sorted(FIXTURES), sorted(fixtures))
        for name, (directory, files, citations) in FIXTURES.items():
            with self.subTest(fixture=name):
                self.assertEqual(directory, fixtures[name]["dir"])
                self.assertEqual(files, fixtures[name]["files"])
                self.assertEqual(citations, fixtures[name]["citations"])
        self.assertEqual(FIXTURE_ID, fixtures["canonical"]["subject"])

    def test_the_ten_scenarios_keep_their_names_invocations_and_limits(self):
        found = [(row["name"], row["args"], row["fixture"], float(row["limit_pct"]))
                 for row in rows()]
        self.assertEqual(list(TODAY), found[:len(TODAY)],
                         "a scenario of cf706c4530 changed its name, invocation or limit")

    def test_names_are_unique(self):
        names = [row["name"] for row in rows()]
        self.assertEqual(len(names), len(set(names)), "a scenario name repeats")

    def test_fmt_check_normalizes_its_fixture_first(self):
        """§AR-benchmarks.1.3: `fmt --write` outside Callgrind, on the same fixture."""
        by_name = {row["name"]: row for row in rows()}
        self.assertEqual(["fmt", "--write"], by_name["fmt_check"].get("setup"))
        for row in rows():
            if row["name"] != "fmt_check":
                with self.subTest(scenario=row["name"]):
                    self.assertNotIn("setup", row)


class EveryRowHasALimitAndACeiling(unittest.TestCase):
    """§AR-ci.5.2: a relative limit and an absolute `Ir` ceiling, set from a CI count."""

    def test_every_row_has_a_relative_limit_clear_of_jitter(self):
        bound = float(BOUND.search(AR_BENCHMARKS.read_text(encoding="utf-8")).group(1))
        for row in rows():
            with self.subTest(scenario=row["name"]):
                self.assertIsInstance(row.get("limit_pct"), (int, float))
                self.assertGreaterEqual(row["limit_pct"], 5 * bound)

    def test_every_row_has_a_ceiling_set_from_a_ci_count(self):
        for row in rows():
            with self.subTest(scenario=row["name"]):
                ceiling, source = row.get("ceiling_ir"), row.get("ceiling_from")
                self.assertIsInstance(ceiling, int, "no integer `ceiling_ir`")
                self.assertIsInstance(source, dict, "no `ceiling_from`")
                self.assertIsInstance(source.get("ir"), int, "`ceiling_from` names no `ir`")
                self.assertRegex(source.get("commit", ""), r"^[0-9a-f]{40}$",
                                 "`ceiling_from` names no full commit")
                floor = math.ceil(source["ir"] * (1 + row["limit_pct"] / 100))
                self.assertGreaterEqual(ceiling, floor,
                                        "the ceiling sits less than one relative limit above "
                                        "the count it was set from")
                margin = math.ceil(source["ir"] * (1 + 2 * row["limit_pct"] / 100) / 1e6)
                self.assertEqual(margin * 1_000_000, ceiling,
                                 "the ceiling is not two relative limits above the count it "
                                 "was set from, rounded up to a whole million")


class TheHarnessIsGeneratedFromTheCatalog(unittest.TestCase):
    """§AR-benchmarks.1.2, §AR-ci.5.2: no hand-written list remains in the harness."""

    def setUp(self):
        self.source = HARNESS.read_text(encoding="utf-8")

    def test_the_harness_reads_the_catalog(self):
        self.assertTrue("scenarios.json" in self.source,
                      f"{HARNESS.name} does not read the scenario catalog")

    def test_no_hand_written_benchmark_function_remains(self):
        found = re.findall(r"#\[binary_benchmark[^\]]*\]\s*fn (\w+)", self.source)
        self.assertEqual([], found, "hand-written `#[binary_benchmark]` functions remain")

    def test_no_limit_is_hard_coded_in_the_harness(self):
        found = re.findall(r"const \w*LIMIT\w*\s*:", self.source)
        self.assertEqual([], found, "a limit is a constant of the harness, not of the catalog")

    def test_the_harness_sets_hard_limits_behind_the_ceiling_switch(self):
        self.assertTrue("hard_limits" in self.source, "no scenario gets an absolute `Ir` ceiling")
        self.assertTrue("GRUND_BENCH_CEILINGS" in self.source,
                      "the ceilings are not switched by `GRUND_BENCH_CEILINGS`")

    def test_the_bench_job_switches_the_ceilings_on(self):
        text = CI.read_text(encoding="utf-8")
        job = re.search(r"^  bench:\n(.*?)(?=^  [\w-]+:\n|\Z)", text, re.M | re.S)
        self.assertIsNotNone(job, "ci.yml has no `bench` job")
        self.assertRegex(job.group(1), r"(?m)^ +GRUND_BENCH_CEILINGS: *\S",
                         "the bench job does not set `GRUND_BENCH_CEILINGS`")


class ThePgoSubsetIsTheTrainingList(unittest.TestCase):
    """§AR-benchmarks.1.5, §FS-distribution-candidate.7.3: one list, and the evidence bytes."""

    def test_the_pgo_subset_is_todays_training_list_in_order(self):
        subset = [row["name"] for row in rows() if row.get("pgo")]
        self.assertEqual(list(PGO_SUBSET), subset)
        for row in rows():
            if row["name"] not in PGO_SUBSET:
                with self.subTest(scenario=row["name"]):
                    self.assertIs(False, row.get("pgo"))

    def test_the_subset_renders_todays_training_evidence(self):
        rendered = [" ".join(["grund"] + [arg.replace("{id}", row["pgo"].get("id", ""))
                                          for arg in row["args"]])
                    for row in rows() if row.get("pgo")]
        self.assertEqual(GRUND_TRAINING, json.dumps(rendered))
        self.assertEqual(list(API), [row["pgo"]["api"] for row in rows() if row.get("pgo")])

    def test_train_py_describes_the_grund_training_byte_for_byte(self):
        code, out, err = describe("grund")
        self.assertEqual(0, code, f"`train.py describe grund` failed: {err}")
        self.assertEqual(GRUND_TRAINING, out)

    def test_train_py_describes_the_api_workloads_unchanged(self):
        for product, prefix in (("node-addon", "node "), ("python-extension", "python ")):
            with self.subTest(product=product):
                code, out, err = describe(product)
                self.assertEqual(0, code, err)
                self.assertEqual([prefix + step for step in API], json.loads(out))

    def test_train_py_reads_the_catalog_rather_than_a_list_of_its_own(self):
        source = TRAIN.read_text(encoding="utf-8")
        self.assertTrue("scenarios.json" in source, f"{TRAIN.name} does not read the catalog")
        self.assertIsNone(re.search(r"^API = \[", source, re.M),
                          f"{TRAIN.name} still hand-writes its own `API` list")
        self.assertIsNone(re.search(r"^ *import (tomllib|yaml)\b", source, re.M),
                          "the catalog must read with Python 3.6's standard library")

    def test_pgo_build_runs_no_hand_written_loop(self):
        source = PGO_BUILD.read_text(encoding="utf-8")
        loop = re.findall(r'^\s*"\$grund" (.*?) "\$repo"', source, re.M)
        self.assertEqual([], loop, "pgo-build.sh still hand-writes the training loop")
        self.assertFalse("in sync with crates/grund-cli/benches/instructions.rs" in source,
                         "pgo-build.sh still says it is kept in sync by hand")
        self.assertIsNone(re.search(r"sed -n [^\n]*pgo-build\.sh", source),
                          "pgo-build.sh still parses its own loop for the training evidence")


if __name__ == "__main__":
    unittest.main()
