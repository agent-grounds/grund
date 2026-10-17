"""§AR-benchmarks.7.1, §AR-benchmarks.7.1.1 — a run record carries the `tier` that ran it,
outside the series key, and `bench_record.py merge` folds the shards' partial records into
one record per revision, merging `fixtures` as a bundle and refusing partials that do not
belong together."""

import json
import tempfile
import unittest
from pathlib import Path

from test_bench_record import CHANGED, IDENTITY, load, run


def partial(module, ir, commit="1" * 40, binary="2" * 64, identity=None, refused=(),
            tier="pr"):
    return module.build_record(commit=commit, binary_sha256=binary,
                               identity=dict(IDENTITY if identity is None else identity),
                               ir=ir, refused=list(refused), tier=tier)


class TheRecordCarriesItsTier(unittest.TestCase):
    """§AR-benchmarks.7.1"""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    # Red until #780 bumps RECORD_VERSION to 3; implement removes this marker.
    @unittest.expectedFailure
    def test_the_format_is_version_three(self):
        # §AR-benchmarks.7.1: 3 since the measured process runs under the runtime pins.
        self.assertEqual(3, self.module.RECORD_VERSION)

    def test_a_record_carries_its_tier(self):
        for tier in ("pr", "nightly"):
            with self.subTest(tier=tier):
                self.assertEqual(tier, partial(self.module, {"check": 1}, tier=tier)["tier"])

    def test_the_tier_is_outside_the_series_key(self):
        self.assertNotIn("tier", self.module.IDENTITY_FIELDS)
        self.assertEqual(partial(self.module, {"check": 1}, tier="pr")["series"],
                         partial(self.module, {"check": 1}, tier="nightly")["series"])


class MergeFoldsTheShards(unittest.TestCase):
    """§AR-benchmarks.7.1.1"""

    @classmethod
    def setUpClass(cls):
        cls.module = load()

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def merge(self, *records):
        paths = []
        for index, record in enumerate(records):
            path = self.dir / f"part-{index}.json"
            path.write_text(json.dumps(record), encoding="utf-8")
            paths.append(path)
        out = self.dir / "merged.json"
        return run("merge", "--out", out, *paths), out

    def test_partials_of_one_revision_merge_into_one_record(self):
        result, out = self.merge(partial(self.module, {"check": 10, "list": 20}, refused=["id"]),
                                 partial(self.module, {"show": 30}, refused=["refs"]))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        merged = json.loads(out.read_text(encoding="utf-8"))
        self.assertEqual({"check": 10, "list": 20, "show": 30}, merged["ir"])
        self.assertEqual(["id", "refs"], sorted(merged["refused"]))
        self.assertEqual("pr", merged["tier"])
        self.assertEqual(partial(self.module, {"check": 10})["series"], merged["series"])

    def test_partials_of_two_series_are_refused_naming_the_field(self):
        for name in ("image", "threads", "fixtures", "rustc"):
            with self.subTest(field=name):
                drifted = dict(IDENTITY, **{name: CHANGED[name]})
                result, _ = self.merge(partial(self.module, {"check": 10}),
                                       partial(self.module, {"show": 30}, identity=drifted))
                self.assertNotEqual(0, result.returncode, "a drifted pin was merged")
                self.assertIn(name, result.stdout + result.stderr)

    def test_partials_of_two_binaries_or_tiers_are_refused(self):
        for other in ({"commit": "3" * 40}, {"binary": "4" * 64}, {"tier": "nightly"}):
            with self.subTest(differs=sorted(other)[0]):
                result, _ = self.merge(partial(self.module, {"check": 10}),
                                       partial(self.module, {"show": 30}, **other))
                self.assertNotEqual(0, result.returncode, result.stdout)

    def bundle(self, *names):
        return dict(IDENTITY, fixtures={name: {"schema_version": 1, "digest": name[0] * 64}
                                        for name in names})

    def merged_series(self, tier, *shards):
        result, out = self.merge(*(partial(self.module, {f"{tier}-{index}": 1}, tier=tier,
                                           identity=self.bundle(*names))
                                   for index, names in enumerate(shards)))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return json.loads(out.read_text(encoding="utf-8"))["series"]

    def test_each_tier_is_a_series_of_its_own(self):
        # The bundle is the fixtures of the rows a tier ran, so two pull-request runs share a
        # key, and a nightly run, whose rows generate more fixtures, gets another one.
        pr = self.merged_series("pr", ["canonical-repo"], ["large-repo"])
        self.assertEqual(pr, self.merged_series("pr", ["large-repo"], ["canonical-repo"]))
        nightly = self.merged_series("nightly", ["canonical-repo", "small-repo"],
                                     ["large-repo"])
        self.assertNotEqual(pr, nightly)

    def test_a_fixture_two_partials_digest_differently_is_refused(self):
        drifted = dict(IDENTITY, fixtures={"canonical-repo": {"schema_version": 1,
                                                              "digest": "e" * 64}})
        result, _ = self.merge(partial(self.module, {"check": 10}),
                               partial(self.module, {"show": 30}, identity=drifted))
        self.assertNotEqual(0, result.returncode, "a fixture digested two ways was merged")
        self.assertIn("canonical-repo", result.stdout + result.stderr)

    def test_a_scenario_in_two_partials_is_refused(self):
        result, _ = self.merge(partial(self.module, {"check": 10}),
                               partial(self.module, {"check": 11}))
        self.assertNotEqual(0, result.returncode, "one scenario was measured by two shards")
        self.assertIn("check", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
