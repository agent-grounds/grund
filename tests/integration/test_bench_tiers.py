"""§AR-benchmarks.1.2.1, §AR-benchmarks.1.2.2 — the scenario catalog holds a pull-request
tier of about 40 rows, split into six to eight shards, and a nightly tier that brings it to
100-200 rows. §AR-benchmarks.1.6.1, §AR-benchmarks.1.7, §AR-benchmarks.1.8 — the waivers
that deferred to #699 are scenarios, every designated mode but `node` names scenarios, the
language server is measured with its PGO session, and a pair of rows straddles the
parallel-scan threshold.

Static reads, the fixture generator and `train.py`: no Valgrind, so it runs in pre-commit.
"""

import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from test_bench_surface import designated_modes, help_flags, surface


REPO = Path(__file__).resolve().parents[2]
CATALOG = REPO / "crates/grund-cli/benches/scenarios.json"
HARNESS = REPO / "crates/grund-cli/benches/instructions.rs"
SHARDS = REPO / "scripts/bench_shards.py"
TRAIN = REPO / "scripts/distribution/train.py"
GENERATOR = REPO / "scripts/generate_large_benchmark_fixture.py"
TREE = REPO / "crates/grund-core/src/scanner/tree.rs"

TIERS = {"pr", "nightly"}
FRONTENDS = {"lsp", "python"}
# The rows the catalog held before the tiers (§AR-benchmarks.1.2.1): all `pr`, unchanged.
TEN = ("check", "check_large_10k", "check_citations", "list", "show_brief", "show",
       "show_full", "refs", "cover", "fmt_check")
# The inventory items whose waiver said "#699 is to add a scenario" at 7d2fec2f5f: each is
# now measured (§AR-benchmarks.1.6.1). A flag is (command, flag); a command or mode is a name.
DEFERRED_FLAGS = (
    ("check", "--format"), ("check", "--full"), ("check", "--require-grounding"),
    ("check", "--rule"), ("check", "--suggestions"),
    ("show", "--all"), ("show", "--batch"), ("show", "--section"), ("show", "--toc"),
    ("list", "--format"), ("list", "--selector"), ("list", "--size"), ("list", "--unused"),
    ("refs", "--descendants"), ("cover", "--lines"),
    ("fmt", "--cross-refs"), ("fmt", "--marker"),
)
DEFERRED_COMMANDS = ("id", "complete")
# §AR-benchmarks.1.6.1: the modes a scenario must measure; `node` alone stays waived.
MEASURED_MODES = ("parallel-scan", "sequential-scan", "workspace", "lsp", "python", "rules",
                  "named-sections", "values", "external-facts")
# §AR-benchmarks.1.7, §FS-distribution-candidate.7.3: the language server's training
# evidence at 7d2fec2f5f, which the shared session must leave byte for byte.
LSP_TRAINING = (
    '["grund-lsp initialize", "grund-lsp initialized", '
    '"grund-lsp textDocument/didOpen README.md", '
    '"grund-lsp textDocument/didOpen docs/functional-spec/FS-check.md", '
    '"grund-lsp textDocument/didOpen crates/grund-core/src/lib.rs", '
    '"grund-lsp textDocument/hover on the first citation of each document, answered after '
    'its diagnostics", "grund-lsp shutdown", "grund-lsp exit"]')
PLAIN_FIXTURE = {"dir", "files", "citations", "subject"}


def catalog():
    return json.loads(CATALOG.read_text(encoding="utf-8"))


def rows():
    return catalog()["scenarios"]


def tier(name):
    return [row for row in rows() if row.get("tier") == name]


def threshold():
    found = re.search(r"const PARALLEL_SCAN_MIN_FILES: usize = (\d+);",
                      TREE.read_text(encoding="utf-8"))
    assert found is not None, "tree.rs no longer declares PARALLEL_SCAN_MIN_FILES"
    return int(found.group(1))


def shards_module():
    if not SHARDS.is_file():
        raise AssertionError(f"no {SHARDS.relative_to(REPO)}: §AR-benchmarks.1.2.2 derives the "
                             "shards there")
    spec = importlib.util.spec_from_file_location("bench_shards", SHARDS)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def generate(root, files, citations=False):
    args = [sys.executable, "-I", str(GENERATOR), "--root", str(root), "--files", str(files)]
    subprocess.run(args + (["--citations"] if citations else []), check=True,
                   capture_output=True, text=True)


class TheCatalogHasTwoTiers(unittest.TestCase):
    """§AR-benchmarks.1.2, §AR-benchmarks.1.2.1"""

    def test_every_row_names_a_known_tier(self):
        unknown = [row["name"] for row in rows() if row.get("tier") not in TIERS]
        self.assertEqual([], unknown, f"rows with no `tier` of {sorted(TIERS)}")

    def test_the_ten_rows_before_the_tiers_are_pr_rows(self):
        by_name = {row["name"]: row for row in rows()}
        self.assertEqual({name: "pr" for name in TEN},
                         {name: by_name[name].get("tier") for name in TEN})

    def test_the_pr_tier_holds_about_forty_rows(self):
        self.assertTrue(35 <= len(tier("pr")) <= 45,
                        f"the pr tier holds {len(tier('pr'))} rows, not about 40")

    def test_the_catalog_holds_one_to_two_hundred_rows(self):
        self.assertTrue(100 <= len(rows()) <= 200,
                        f"the catalog holds {len(rows())} rows, not 100-200")

    def test_every_nightly_row_names_its_group(self):
        missing = [row["name"] for row in tier("nightly")
                   if not isinstance(row.get("group"), str) or not row["group"].strip()]
        self.assertEqual([], missing, "nightly rows with no feature `group`")

    def test_a_frontend_is_lsp_or_python(self):
        odd = [row["name"] for row in rows() if row.get("frontend", "lsp") not in FRONTENDS]
        self.assertEqual([], odd)


class ThePrTierIsSharded(unittest.TestCase):
    """§AR-benchmarks.1.2.2: one script derives the shards, and every pr row is in one."""

    def test_the_catalog_names_six_to_eight_shards(self):
        count = catalog().get("shards")
        self.assertIsInstance(count, int, "the catalog names no integer `shards`")
        self.assertTrue(6 <= count <= 8, f"`shards` is {count}, not 6-8")

    def test_every_pr_row_lands_in_exactly_one_shard(self):
        data = catalog()
        assigned = shards_module().assign(data)
        self.assertEqual(data["shards"], len(assigned))
        names = [name for shard in assigned for name in shard]
        self.assertEqual(sorted(row["name"] for row in tier("pr")), sorted(names))
        self.assertEqual(len(names), len(set(names)), "a row is in two shards")
        self.assertNotIn([], assigned, "a shard measures nothing")

    def test_select_prints_one_shards_rows(self):
        data = catalog()
        assigned = shards_module().assign(data)
        for index, expected in enumerate(assigned, start=1):
            with self.subTest(shard=index):
                result = subprocess.run(
                    [sys.executable, "-I", str(SHARDS), "select", f"{index}/{data['shards']}"],
                    capture_output=True, text=True, check=False)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertEqual(expected, result.stdout.split())

    def test_the_harness_selects_by_shard_and_tier(self):
        source = HARNESS.read_text(encoding="utf-8")
        for variable in ("GRUND_BENCH_SHARD", "GRUND_BENCH_TIER", "bench_shards.py"):
            with self.subTest(variable=variable):
                self.assertIn(variable, source)


class ABaseRefusalLeavesTheBundle(unittest.TestCase):
    """§AR-benchmarks.7.1.1, §AR-benchmarks.5.1: a shard's bundle is its selected rows'
    fixtures, refused or not, so its base and head stay one series whatever the base refuses."""

    @classmethod
    def setUpClass(cls):
        import test_bench_record
        cls.record_module = test_bench_record
        cls.module = test_bench_record.load()

    def test_the_harness_ensures_every_selected_fixture_before_a_probe(self):
        source = HARNESS.read_text(encoding="utf-8")
        body = source[source.index("fn commands_group"):]
        self.assertIn("ensure_fixture(", body)
        self.assertLess(body.index("ensure_fixture("), body.index("accepts("),
                        "a fixture is generated only once its row's probe accepts it")

    def records(self, shard, bundle_of):
        """A base record that refused every frontend row and a head that measured them all,
        each digesting the fixtures `bundle_of` names for the rows it is given."""
        by_name = {row["name"]: row for row in rows()}
        refused = [name for name in shard if by_name[name].get("frontend")]
        ran = [name for name in shard if name not in refused]

        def build(names, measured, commit, binary, refusals):
            identity = dict(self.record_module.IDENTITY)
            identity["fixtures"] = {
                name: {"schema_version": 1, "digest": name.encode().hex()[:64].ljust(64, "0")}
                for name in sorted({by_name[row]["fixture"] for row in bundle_of(names)})}
            return self.module.build_record(commit=commit, binary_sha256=binary,
                                            identity=identity, refused=refusals,
                                            ir={name: 1_000_000 for name in measured})

        base = build(shard, ran, "1" * 40, "2" * 64, refused)
        head = build(shard, shard, "3" * 40, "4" * 64, [])
        return refused, ran, base, head

    def compare(self, base, head):
        with tempfile.TemporaryDirectory() as tmp:
            paths = []
            for name, record in (("base", base), ("head", head)):
                path = Path(tmp) / f"{name}.json"
                path.write_text(json.dumps(record), encoding="utf-8")
                paths.append(path)
            return subprocess.run([sys.executable, "-I", str(self.record_module.SCRIPT),
                                   "compare", *map(str, paths)],
                                  capture_output=True, text=True, check=False)

    def test_every_shard_compares_whatever_the_base_refuses(self):
        # Each real shard, and one built so a refused row is alone on its fixture: R2-01's case.
        shards = shards_module().assign(catalog())
        by_name = {row["name"]: row for row in rows()}
        lsp = by_name["lsp"]
        other = next(row["name"] for row in tier("pr")
                     if not row.get("frontend") and row["fixture"] != lsp["fixture"])
        shards.append([other, "lsp"])
        for index, shard in enumerate(shards, start=1):
            refused, ran, base, head = self.records(shard, lambda names: names)
            with self.subTest(shard=index):
                result = self.compare(base, head)
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                self.assertEqual(refused, base["refused"])
                for name in refused:
                    self.assertTrue(any(name in line and "no base, not compared" in line
                                        for line in result.stdout.splitlines()),
                                    f"`{name}` is not reported as refused:\n{result.stdout}")
        # The bundle of the accepted rows alone is another series, which compare refuses.
        _, ran, _, head = self.records(shards[-1], lambda names: names)
        _, _, narrow, _ = self.records(shards[-1], lambda names: [n for n in names if n in ran])
        self.assertNotEqual(0, self.compare(narrow, head).returncode)

class TheScanPairStraddlesTheThreshold(unittest.TestCase):
    """§AR-benchmarks.1.8: one row below the parallel threshold, one at it."""

    def counted(self, mode):
        """The scanned-file count of each pr `check` row the inventory names under `mode`."""
        fixtures, by_name = catalog()["fixtures"], {row["name"]: row for row in rows()}
        names = surface()["modes"].get(mode, {}).get("scenarios", [])
        counts = {}
        with tempfile.TemporaryDirectory() as tmp:
            for name in names:
                row = by_name.get(name)
                if row is None or row.get("tier") != "pr" or row["args"][:1] != ["check"]:
                    continue
                fixture = fixtures[row["fixture"]]
                self.assertLessEqual(set(fixture), PLAIN_FIXTURE,
                                     f"{name}'s fixture is not a plain generated one")
                root = Path(tmp) / name
                generate(root, fixture["files"], fixture["citations"])
                counts[name] = sum(1 for _ in (root / "docs").rglob("*.md"))
        return counts

    def test_a_pr_row_scans_one_file_fewer_than_the_threshold(self):
        counts = self.counted("sequential-scan")
        self.assertIn(threshold() - 1, counts.values(),
                      f"no pr `check` row under `sequential-scan` scans {threshold() - 1} files: "
                      f"{counts}")

    def test_a_pr_row_scans_exactly_the_threshold(self):
        counts = self.counted("parallel-scan")
        self.assertIn(threshold(), counts.values(),
                      f"no pr `check` row under `parallel-scan` scans {threshold()} files: "
                      f"{counts}")


class NoWaiverDefersTo699(unittest.TestCase):
    """§AR-benchmarks.1.6.1: the deferred items are measured, and no waiver names an issue."""

    def setUp(self):
        self.surface = surface()
        self.rows = {row["name"]: row for row in rows()}

    def waivers(self):
        for command, entry in self.surface["commands"].items():
            yield f"command {command}", entry.get("waiver")
            for flag, flagged in entry.get("flags", {}).items():
                yield f"flag {command} {flag}", flagged.get("waiver")
        for mode, entry in self.surface["modes"].items():
            yield f"mode {mode}", entry.get("waiver")

    def test_no_waiver_names_an_issue(self):
        named = [item for item, waiver in self.waivers() if waiver and re.search(r"#\d+", waiver)]
        self.assertEqual([], named, "waivers that defer to an issue rather than give a reason")

    def test_every_deferred_flag_is_measured(self):
        unmeasured = []
        for command, flag in DEFERRED_FLAGS:
            carried = any(row["args"][0] == command and flag in row["args"]
                          for row in self.rows.values() if "frontend" not in row)
            entry = self.surface["commands"][command].get("flags", {}).get(flag, {})
            if not carried and "scenarios" not in entry:
                unmeasured.append(f"{command} {flag}")
        self.assertEqual([], unmeasured)

    def test_every_deferred_command_is_measured(self):
        unmeasured = [command for command in DEFERRED_COMMANDS
                      if "scenarios" not in self.surface["commands"][command]]
        self.assertEqual([], unmeasured)

    def test_check_watch_keeps_its_waiver(self):
        self.assertIn("--watch", help_flags("check"))
        self.assertIn("waiver", self.surface["commands"]["check"]["flags"]["--watch"])

    def test_the_feature_modes_are_designated(self):
        self.assertLessEqual(set(MEASURED_MODES) | {"node"}, set(designated_modes()))

    def test_every_mode_but_node_names_scenarios(self):
        waived = [mode for mode in MEASURED_MODES
                  if "scenarios" not in self.surface["modes"].get(mode, {})]
        self.assertEqual([], waived)

    def test_node_is_waived_for_v8s_jitter(self):
        waiver = self.surface["modes"]["node"].get("waiver", "")
        self.assertIn("V8", waiver)
        self.assertIn("repeatable", waiver)


class TheFrontendsAreMeasured(unittest.TestCase):
    """§AR-benchmarks.1.7, §DA-benchmark-frontends"""

    def frontend_rows(self, mode):
        by_name = {row["name"]: row for row in rows()}
        names = surface()["modes"].get(mode, {}).get("scenarios", [])
        return [by_name[name] for name in names if name in by_name]

    def test_the_language_server_is_a_pr_row(self):
        found = [row["name"] for row in self.frontend_rows("lsp")
                 if row.get("frontend") == "lsp" and row.get("tier") == "pr"]
        self.assertTrue(found, "`lsp` names no pr row with `frontend: lsp`")

    def test_the_python_extension_is_a_nightly_row(self):
        found = [row["name"] for row in self.frontend_rows("python")
                 if row.get("frontend") == "python" and row.get("tier") == "nightly"]
        self.assertTrue(found, "`python` names no nightly row with `frontend: python`")

    def test_the_session_is_framed_json_rpc_in_the_training_order(self):
        with tempfile.TemporaryDirectory() as tmp:
            generate(Path(tmp), 3)
            result = subprocess.run([sys.executable, "-I", str(TRAIN), "session", "--root", tmp],
                                    capture_output=True, check=False)
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", "replace"))
        methods, data = [], result.stdout
        while data:
            head, _, rest = data.partition(b"\r\n\r\n")
            length = int(re.search(rb"Content-Length: (\d+)", head).group(1))
            methods.append(json.loads(rest[:length])["method"])
            data = rest[length:]
        # The training client sends each document's hover right after its `didOpen`.
        opened = methods.count("textDocument/didOpen")
        self.assertGreater(opened, 0, "the session opens no document")
        self.assertEqual(["initialize", "initialized"]
                         + ["textDocument/didOpen", "textDocument/hover"] * opened
                         + ["shutdown", "exit"], methods)

    def test_the_training_evidence_keeps_its_bytes(self):
        result = subprocess.run([sys.executable, "-I", str(TRAIN), "describe", "grund-lsp"],
                                capture_output=True, text=True, encoding="utf-8", check=False)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(LSP_TRAINING, result.stdout.rstrip("\n"))


if __name__ == "__main__":
    unittest.main()
