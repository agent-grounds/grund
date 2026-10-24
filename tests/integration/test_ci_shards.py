"""§AR-ci.13.7 — each leg's cargo suite runs in shards, and the Python acceptance in a job
of its own (§AR-ci.13.1, §AR-ci.3.4.3), so no gate waits on one long job
(§AR-ci.requirements.1).

Read as text with the layout test's reader (§AR-ci.13.5). Each check returns the problems it
finds, and runs against fixture workflows too, so a check that can no longer fail is caught.
"""

import re
import unittest

from test_ci_language_jobs import (CARGO_SUITE, CI, SHARD, SHARDS, SUITES, SYSTEMS, fixture,
                                   instances, required_check_problems, shard_command,
                                   split_problems)

ACCEPTANCE = SUITES["python acceptance"]
WITHOUT = SUITES["python"]


def shard_problems(text):
    """One `n` of at least two per leg, every `k` of 1..`n` in exactly one job, each shard job
    restoring the cargo cache under a key of its own, and the leg's aggregate needing it."""
    found = instances(text)
    problems = []
    for system in SYSTEMS:
        on = [i for i in found if i["system"] == system and not i["aggregate"]]
        shards = [(i["job"], int(k), int(n)) for i in on for command in i["commands"]
                  for k, n in SHARD.findall(command + " ")]
        counts = sorted({n for _, _, n in shards})
        if len(counts) != 1 or counts[0] < 2:
            problems.append(f"{system} runs its cargo suite as shards of {counts or 'none'}, "
                            "not as one `n` of at least two")
            continue
        count = counts[0]
        indices = sorted(k for _, k, _ in shards)
        if indices != list(range(1, count + 1)):
            problems.append(f"{system} runs shards {indices} of {count}, not each of 1..{count} once")
        jobs_ = [job for job, _, _ in shards]
        if len(set(jobs_)) != len(jobs_):
            problems.append(f"{system} runs two shards in one job: {sorted(jobs_)}")
        for job, k, _ in shards:
            if shard_command(k, count) not in next(i for i in on if i["job"] == job)["commands"]:
                problems.append(f"{job!r} runs shard {k}/{count} with other arguments than "
                                f"`{CARGO_SUITE}`")
        for instance in (i for i in on if i["job"] in jobs_):
            keys = [key for step in instance["steps"] if "actions/cache" in step
                    for key in re.findall(r"(?m)^\s*key:\s*(.+)$", step)]
            if not any("-cargo-" in key for key in keys):
                problems.append(f"{instance['job']!r} restores no cargo cache")
        aggregate = [i for i in found if i["aggregate"] and i["system"] == system]
        for instance in aggregate:
            missing = sorted(set(jobs_) - set(instance["needs"]))
            if missing:
                problems.append(f"{instance['name']!r} does not need the shards {missing}")
    return problems


def acceptance_problems(text):
    """§AR-ci.13.1: one Python acceptance job per leg, apart from the Python job."""
    found = [i for i in instances(text) if not i["aggregate"]]
    problems = []
    for system in SYSTEMS:
        on = [i for i in found if i["system"] == system]
        jobs_ = [i["job"] for i in on if ACCEPTANCE in i["commands"]]
        if len(jobs_) != 1:
            problems.append(f"{system} runs the Python acceptance in {jobs_}, not one job")
        shared = [i["job"] for i in on if ACCEPTANCE in i["commands"] and WITHOUT in i["commands"]]
        if shared:
            problems.append(f"{shared} run the Python acceptance in the Python job")
        if not any(WITHOUT in i["commands"] for i in on):
            problems.append(f"{system} has no Python job running `{WITHOUT}`")
    return problems


class ShardTests(unittest.TestCase):
    """§AR-ci.13.7 and the Python acceptance job on `ci.yml`."""

    @classmethod
    def setUpClass(cls):
        cls.text = CI.read_text(encoding="utf-8")

    def test_each_leg_runs_its_cargo_suite_as_shards_its_check_needs(self):
        self.assertEqual([], shard_problems(self.text))

    def test_each_leg_runs_the_python_acceptance_in_a_job_of_its_own(self):
        self.assertEqual([], acceptance_problems(self.text))


class ShardCheckTests(unittest.TestCase):
    """The checks pass the layout §AR-ci.13.7 asks for and fail each way off it."""

    def test_the_layout_asked_for_passes(self):
        self.assertEqual([], shard_problems(fixture()))
        self.assertEqual([], acceptance_problems(fixture()))

    def test_a_whole_cargo_suite_is_caught(self):
        text = fixture().replace(shard_command(1, SHARDS), CARGO_SUITE)
        self.assertTrue(shard_problems(text))
        self.assertTrue(split_problems(text))

    def test_a_missing_or_doubled_shard_is_caught(self):
        last = shard_command(SHARDS, SHARDS)
        self.assertTrue(shard_problems(fixture().replace(last, shard_command(1, SHARDS))))
        self.assertTrue(shard_problems(fixture().replace(last, shard_command(SHARDS, SHARDS + 1))))

    def test_one_shard_is_not_a_split(self):
        text = fixture().replace(shard_command(1, SHARDS), shard_command(1, 1))
        for index in range(2, SHARDS + 1):
            text = text.replace(f"      - run: {shard_command(index, SHARDS)}\n", "")
        self.assertTrue(shard_problems(text))

    def test_an_aggregate_missing_a_shard_is_caught(self):
        text = fixture().replace("rust-windows, rust-windows-2, ", "rust-windows, ")
        self.assertTrue(shard_problems(text))
        self.assertTrue(required_check_problems(text))

    def test_a_python_acceptance_in_the_python_job_is_caught(self):
        text = fixture().replace(f"      - run: {WITHOUT}\n",
                                 f"      - run: {WITHOUT}\n      - run: {ACCEPTANCE}\n", 1)
        self.assertTrue(acceptance_problems(text))
        self.assertTrue(split_problems(text))


if __name__ == "__main__":
    unittest.main()
