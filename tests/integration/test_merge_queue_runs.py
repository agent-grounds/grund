"""§AR-ci.12 — a merge queue lands every pull request, so CI runs on its groups.

The queue tests each group under the checks a pull request must pass and lands
exactly the commit it tested, which is only true while `ci.yml` answers the
`merge_group` event. On a group the commit-message job scans what the group puts
on `main` (§AR-ci.8.2), and the three benchmark jobs hold its head to the absolute
ceilings, once per shard, and record nothing (§AR-ci.5.1). Read as text by
`ci_workflow.py`, with the leg's jobs in place of its calls (§AR-ci.13.5); each `if:` is
read for every event `ci.yml` runs on."""

import re
import unittest

from ci_workflow import (CALL, CI, aggregate_fails, commands, field, header, holds, needs,
                         step_if, with_called_workflows)
from test_ci_language_jobs import EVENTS, job_env
from test_release_workflow import jobs, squash, step_name, steps

# §AR-ci.5: the build job, the shard matrix and the record job.
BENCH_JOBS = ("bench-build", "bench", "bench-record")
# §AR-ci.5.1: the head's run, on its own binary, with no base and no baseline.
HEAD_RUN = ("cargo bench -p grund --features bench --locked --quiet --bench instructions"
            " -- --save-summary=json")


def runs(job_body, step, event):
    """Whether `step` runs on `event`: its job's `if:` and its own both hold."""
    return holds(field(header(job_body), "if"), event) and holds(step_if(step), event)


def measures_head(step):
    """A step that runs the harness on the head's binary, not the base's (§AR-ci.5.1)."""
    return "GRUND_BENCH_BIN" not in step and any(
        command.startswith("cargo bench") for command in commands(step))


def judged_jobs(text, called=None):
    """Each job a required `cargo test (<os>)` check judges, by id: the check, each job it
    needs and, for a need that calls the leg, every `<call>/<job>` the call runs (§AR-ci.13.2).
    `called` maps the leg's `uses:` path to its text, as `with_called_workflows` takes it."""
    written, read = jobs(text), jobs(with_called_workflows(text, called))
    judged = {}
    for job, body in written.items():
        if not re.search(r"(?m)^    name:\s*cargo test \(", body):
            continue
        judged[job] = body
        for need in needs(header(body)):
            judged[need] = written.get(need)
            judged.update((found, inlined) for found, inlined in read.items()
                          if found.startswith(f"{need}/"))
    return judged


def every_event_problems(text, called=None):
    """The jobs a required check judges that some event skips (§AR-ci.12, §AR-ci.13.2). A leg's
    job is one: a skipped job does not fail the call it runs in, so the check would pass it."""
    judged = judged_jobs(text, called)
    problems = [] if judged else ["no job carries a required `cargo test (<os>)` check"]
    for job, body in sorted(judged.items()):
        if body is None:
            problems.append(f"a required check needs `{job}`, which is no job")
            continue
        condition = field(header(body), "if")
        if condition and condition.replace("${{", "").replace("}}", "").strip() != "always()":
            problems.append(f"`{job}` runs only `if: {condition}`; its required check must "
                            "report on a queue group too")
    return problems


def leg_fixture(guarded=None, call_guarded=False):
    """A `ci.yml` whose Windows check needs its leg's call, and the leg it calls, for
    `every_event_problems`: `guarded` names a leg job, and `call_guarded` the call, that a
    merge-queue group skips (§AR-ci.13.5)."""
    skip = "    if: github.event_name != 'merge_group'\n"
    leg = "jobs:\n" + "".join(
        f"  {job}:\n    runs-on: ${{{{ inputs.os }}}}\n" + (skip if job == guarded else "")
        + "    steps:\n      - run: exit 0\n" for job in ("rust", "python", "python-acceptance"))
    text = ("jobs:\n  leg-windows:\n    name: windows-latest\n" + (skip if call_guarded else "")
            + f"    uses: {CALL}\n    with:\n      os: windows-latest\n"
            "  test-windows:\n    name: cargo test (windows-latest)\n    runs-on: ubuntu-latest\n"
            "    needs: [leg-windows]\n    if: always()\n    steps:\n      - run: exit 1\n")
    return text, {CALL: leg}


class MergeQueueRunTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.written = CI.read_text(encoding="utf-8")
        # §AR-ci.13.5: the leg's jobs in place of its calls, as every reader of `ci.yml` takes it.
        cls.text = with_called_workflows(cls.written)
        cls.jobs = jobs(cls.text)

    def test_ci_runs_on_merge_queue_groups(self):
        triggers = self.text.split("\non:\n", 1)[1].split("\nenv:", 1)[0]
        self.assertRegex(triggers, r"(?m)^  merge_group:", "ci.yml does not run on merge-queue groups")

    def test_the_required_checks_and_the_jobs_they_judge_run_on_every_event(self):
        # §AR-ci.13.2: a required check is an aggregate, so the jobs it needs must run here too.
        self.assertEqual([], every_event_problems(self.written))
        aggregate = self.jobs["test-ubuntu"]
        succeeded = dict.fromkeys(needs(header(aggregate)), "success")
        for job in sorted(set(BENCH_JOBS) & set(judged_jobs(self.written))):
            with self.subTest(job=job):
                # §AR-ci.13.2, §AR-ci.12: a group runs them too, so only their success passes.
                self.assertTrue(aggregate_fails(steps(aggregate), "merge_group",
                                                dict(succeeded, **{job: "skipped"})),
                                "cargo test (ubuntu-latest) accepts the group's skip")

    def test_the_benchmark_jobs_run_on_merge_queue_groups(self):
        # §AR-ci.12: no job-level guard keeps a group out of the build, the shards or the record.
        for job in BENCH_JOBS:
            for event in EVENTS:
                with self.subTest(job=job, event=event):
                    self.assertTrue(holds(field(header(self.jobs[job]), "if"), event),
                                    f"the {job} job does not run on {event}")

    def test_a_groups_shard_runs_the_head_once_against_the_ceilings(self):
        # §AR-ci.5.1: one run of the head, no base, no baseline, no record; its exit is the gate.
        shard = self.jobs["bench"]
        running = [step for step in steps(shard) if runs(shard, step, "merge_group")
                   and any(command.startswith("cargo bench") for command in commands(step))]
        self.assertEqual(1, len(running), f"a group's shard runs {len(running)} `cargo bench` "
                         "steps, not one")
        (step,) = running
        self.assertEqual(1, squash(step).count("cargo bench "), "a group's shard measures twice")
        self.assertIn(HEAD_RUN, commands(step))
        for absent in ("GRUND_BENCH_BIN", "--baseline", "--save-baseline", "bench_record.py record",
                       "GRUND_BENCH_CEILINGS", "GRUND_BENCH_TIER", "set +e", "|| true",
                       "continue-on-error"):
            with self.subTest(absent=absent):
                self.assertNotIn(absent, step, f"a group's shard step names `{absent}`")
        # §AR-ci.5.2: the job's env holds every run, the group's too, to the ceilings.
        self.assertEqual("1", job_env(header(shard)).get("GRUND_BENCH_CEILINGS"))
        self.assertIsNone(field(header(shard), "continue-on-error"), "the shards swallow a failure")

    def test_a_group_writes_no_record(self):
        # §AR-ci.12, §AR-benchmarks.7: each step that records, merges or uploads a merged
        # record excludes a group itself, so no job-level guard is what keeps it out.
        recording = [("bench", step) for step in steps(self.jobs["bench"])
                     if "bench_record.py record" in step]
        merging = [("bench-record", step) for step in steps(self.jobs["bench-record"])
                   if "bench_record.py merge" in step or "actions/upload-artifact" in step]
        self.assertTrue(recording and merging, "no step records, merges or uploads a record")
        for job, step in recording + merging:
            with self.subTest(job=job, step=step_name(step) or squash(step)[:60]):
                self.assertFalse(holds(step_if(step), "merge_group"),
                                 f"{job} records on a merge-queue group")

    def test_every_event_measures_the_head_in_exactly_one_step(self):
        # §AR-ci.5.1, §AR-ci.12: a pull request also runs the base's baseline, which is not the
        # head; no event may pass with every step that measures the head skipped.
        shard = self.jobs["bench"]
        for event in EVENTS:
            with self.subTest(event=event):
                measuring = [step_name(step) or squash(step)[:60] for step in steps(shard)
                             if runs(shard, step, event) and measures_head(step)]
                self.assertEqual(1, len(measuring), f"on {event} the head is measured by "
                                 f"{measuring}, not by one step")

    def test_the_commit_message_range_covers_the_group(self):
        (scan,) = [step for step in steps(self.jobs["commit-messages"])
                   if step_name(step) == "scan commit messages"]
        flat = squash(scan)
        self.assertIn("github.event_name == 'merge_group' && format('{0}..{1}', "
                      "github.event.merge_group.base_sha, github.event.merge_group.head_sha)", flat)
        self.assertIn('--range "$RANGE"', flat)


class EveryEventCheckTests(unittest.TestCase):
    """§AR-ci.13.5: the every-event check passes a leg every event runs, and fails each way
    a group could skip one of its jobs."""

    def test_a_leg_every_event_runs_passes(self):
        self.assertEqual([], every_event_problems(*leg_fixture()))

    def test_a_leg_job_a_group_skips_is_caught(self):
        problems = every_event_problems(*leg_fixture(guarded="python-acceptance"))
        self.assertEqual(1, len(problems), problems)
        self.assertIn("`leg-windows/python-acceptance`", problems[0])

    def test_a_leg_call_a_group_skips_is_caught(self):
        problems = every_event_problems(*leg_fixture(call_guarded=True))
        self.assertEqual(1, len(problems), problems)
        self.assertIn("`leg-windows`", problems[0])


if __name__ == "__main__":
    unittest.main()
