"""§AR-ci.5, §AR-ci.5.4, §AR-ci.12, §AR-ci.13.2 — one build job hands the binaries to a
shard matrix of the catalog's `shards` legs, a record job merges their partial records,
and `cargo test (ubuntu-latest)` needs all three and accepts only their success, on a
merge-queue group as on every other event. A scheduled workflow runs the nightly tier,
and the pre-release checks call it.

Read as text, the way `test_ci_language_jobs.py` reads `ci.yml` (§AR-ci.13.5)."""

import json
import re
import unittest

from ci_workflow import header, holds, needs, with_local_actions
from test_ci_language_jobs import BENCH_ENV, BENCH_IMAGE, bench_guard_problems, job_env
from test_release_workflow import REPO_ROOT, WORKFLOWS, jobs, squash

CI = WORKFLOWS / "ci.yml"
NIGHTLY = WORKFLOWS / "bench-nightly.yml"
PRE_RELEASE = WORKFLOWS / "pre-release-checks.yml"
CATALOG = REPO_ROOT / "crates/grund-cli/benches/scenarios.json"
BUILD, SHARD, RECORD = "bench-build", "bench", "bench-record"


def shard_count():
    return json.loads(CATALOG.read_text(encoding="utf-8")).get("shards")


class TheBenchmarkJobsAreBuildShardsAndRecord(unittest.TestCase):
    """§AR-ci.5: the three jobs and what each does."""

    @classmethod
    def setUpClass(cls):
        cls.jobs = jobs(with_local_actions(CI.read_text(encoding="utf-8")))

    def job(self, name):
        self.assertIn(name, self.jobs, f"ci.yml has no `{name}` job")
        return self.jobs[name]

    def test_the_shard_job_is_a_matrix_of_the_catalogs_shards(self):
        head = header(self.job(SHARD))
        listed = re.search(r"(?m)^        shard:\s*\[(.*)\]", head)
        self.assertIsNotNone(listed, "the bench job has no `matrix.shard` list")
        found = [int(item) for item in listed.group(1).split(",")]
        self.assertEqual(list(range(1, (shard_count() or 0) + 1)), found)
        self.assertIn("GRUND_BENCH_SHARD", self.job(SHARD))

    def test_the_shards_take_the_build_jobs_binaries(self):
        self.assertEqual([BUILD], needs(header(self.job(SHARD))))
        self.assertIn("actions/upload-artifact", self.job(BUILD))
        self.assertIn("actions/download-artifact", self.job(SHARD))
        self.assertNotIn("cargo build", self.job(SHARD), "a shard builds a binary itself")
        for binary in ("grund-lsp", "pull_request.base.sha"):
            with self.subTest(built=binary):
                self.assertIn(binary, self.job(BUILD))

    def test_the_record_job_merges_the_shards_partials(self):
        self.assertEqual([SHARD], needs(header(self.job(RECORD))))
        flat = squash(self.job(RECORD))
        self.assertIn("bench_record.py merge", flat)
        self.assertIn("bench_record.py compare", flat)

    def test_the_build_job_and_the_shards_share_one_pinned_environment(self):
        build, shard = header(self.job(BUILD)), header(self.job(SHARD))
        digests = [BENCH_IMAGE.search(head) for head in (build, shard)]
        self.assertTrue(all(digests), "a benchmark job names no digest-pinned image")
        self.assertEqual(digests[0].group(1), digests[1].group(1))
        for head in (build, shard):
            env = job_env(head)
            for key, value in BENCH_ENV.items():
                self.assertEqual(value, env.get(key), f"{key} differs between the jobs")

    def test_a_merge_queue_group_runs_all_three(self):
        # §AR-ci.12
        for name in (BUILD, SHARD, RECORD):
            with self.subTest(job=name):
                condition = re.search(r"(?m)^    if:\s*(.+)$", header(self.job(name)))
                self.assertTrue(holds(condition and condition.group(1), "merge_group"),
                                f"{name} does not run on merge-queue groups")

    def test_the_ubuntu_check_needs_all_three_and_accepts_only_their_success(self):
        # §AR-ci.13.2: a failure, a cancellation or a skip of any of them fails it, on every event.
        listed = set(needs(header(self.job("test-ubuntu"))))
        self.assertLessEqual({BUILD, SHARD, RECORD}, listed)
        self.assertEqual([], bench_guard_problems(CI.read_text(encoding="utf-8")))


class TheNightlyTierRunsOnASchedule(unittest.TestCase):
    """§AR-ci.5.4"""

    def test_the_nightly_workflow_exists_and_is_scheduled_dispatched_and_callable(self):
        self.assertTrue(NIGHTLY.is_file(), f"no {NIGHTLY.name}")
        triggers = NIGHTLY.read_text(encoding="utf-8").split("\njobs:", 1)[0]
        for trigger in ("schedule", "workflow_dispatch", "workflow_call"):
            with self.subTest(trigger=trigger):
                self.assertRegex(triggers, rf"(?m)^  {trigger}:")

    def test_it_runs_the_nightly_tier_and_writes_one_merged_record(self):
        text = squash(NIGHTLY.read_text(encoding="utf-8")) if NIGHTLY.is_file() else ""
        self.assertRegex(text, r"GRUND_BENCH_TIER: *nightly")
        self.assertIn("bench_record.py merge", text)
        self.assertRegex(text, r"--tier nightly")
        self.assertIsNotNone(BENCH_IMAGE.search(NIGHTLY.read_text(encoding="utf-8"))
                             if NIGHTLY.is_file() else None,
                             "the nightly tier runs outside the pinned image")

    def test_the_pre_release_checks_call_it(self):
        found = re.search(r"(?m)^    uses: \./\.github/workflows/bench-nightly\.yml\s*$",
                          PRE_RELEASE.read_text(encoding="utf-8"))
        self.assertIsNotNone(found, "pre-release-checks.yml does not run the nightly tier")


if __name__ == "__main__":
    unittest.main()
