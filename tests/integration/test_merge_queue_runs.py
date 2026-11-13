"""§AR-ci.12 — a merge queue lands every pull request, so CI runs on its groups.

The queue tests each group under the checks a pull request must pass and lands
exactly the commit it tested, which is only true while `ci.yml` answers the
`merge_group` event. On a group the commit-message job scans what the group puts
on `main` (§AR-ci.8.2), and the three benchmark jobs hold its head to the absolute
ceilings, once per shard, and record nothing (§AR-ci.5.1). Read as text by
`ci_workflow.py`, with the leg's jobs in place of its calls (§AR-ci.13.5); each `if:` is
read for every event `ci.yml` runs on. A group the queue dropped has its run cancelled by
the sweep job (§AR-ci.12.2), whose script `test_cancel_dropped_groups.py` runs."""

import re
import unittest

from ci_workflow import (CALL, CI, aggregate_fails, commands, field, header, holds, needs,
                         step_if, with_called_workflows)
from test_ci_language_jobs import CHECK_IF, EVENTS, check_if_problems, job_env
from test_release_workflow import jobs, squash, step_name, steps

# §AR-ci.5: the build job, the shard matrix and the record job.
BENCH_JOBS = ("bench-build", "bench", "bench-record")
# §AR-ci.5.1: the head's run, on its own binary, with no base and no baseline.
HEAD_RUN = ("cargo bench -p grund --features bench --locked --quiet --bench instructions"
            " -- --save-summary=json")
# §AR-ci.12.1: the group a pull request's run shares, the one any other run has to itself, and
# the one condition under which a run cancels.
PULL_REQUEST_KEY = "github.event.pull_request.number"
OWN_RUN_KEY = "github.run_id"
GROUP = f"ci-${{{{ {PULL_REQUEST_KEY} || {OWN_RUN_KEY} }}}}"
CANCELS = "${{ github.event_name == 'pull_request' }}"
# §AR-ci.12.2: the sweep, the one job that may cancel runs, and how it is called.
SWEEP = "cancel-dropped-groups"
SWEEP_CALL = 'python3 scripts/cancel_dropped_groups.py "$GITHUB_REPOSITORY" "$GITHUB_RUN_ID"'


def runs(job_body, step, event):
    """Whether `step` runs on `event`: its job's `if:` and its own both hold."""
    return holds(field(header(job_body), "if"), event) and holds(step_if(step), event)


def measures_head(step):
    """A step that runs the harness on the head's binary, not the base's (§AR-ci.5.1)."""
    return "GRUND_BENCH_BIN" not in step and any(
        command.startswith("cargo bench") for command in commands(step))


def is_required_check(body):
    """Whether a job carries a required `cargo test (<os>)` check (§AR-ci.13.2)."""
    return re.search(r"(?m)^    name:\s*cargo test \(", body) is not None


def judged_jobs(text, called=None):
    """Each job a required `cargo test (<os>)` check judges, by id: the check, each job it
    needs and, for a need that calls the leg, every `<call>/<job>` the call runs (§AR-ci.13.2).
    `called` maps the leg's `uses:` path to its text, as `with_called_workflows` takes it."""
    written, read = jobs(text), jobs(with_called_workflows(text, called))
    judged = {}
    for job, body in written.items():
        if not is_required_check(body):
            continue
        judged[job] = body
        for need in needs(header(body)):
            judged[need] = written.get(need)
            judged.update((found, inlined) for found, inlined in read.items()
                          if found.startswith(f"{need}/"))
    return judged


def every_event_problems(text, called=None):
    """The jobs a required check judges that some event skips (§AR-ci.12, §AR-ci.13.2). A leg's
    job is one: a skipped job does not fail the call it runs in, so the check would pass it. A
    check takes the `if:` of §AR-ci.12.1 and no other; a job it judges, `always()` or none."""
    judged = judged_jobs(text, called)
    problems = [] if judged else ["no job carries a required `cargo test (<os>)` check"]
    for job, body in sorted(judged.items()):
        if body is None:
            problems.append(f"a required check needs `{job}`, which is no job")
            continue
        condition = field(header(body), "if")
        if is_required_check(body):
            problems += check_if_problems(job, condition, needs(header(body)))
        elif condition and condition.replace("${{", "").replace("}}", "").strip() != "always()":
            problems.append(f"`{job}` runs only `if: {condition}`; its required check must "
                            "report on a queue group too")
    return problems


def concurrency_problems(text):
    """What keeps a workflow's own `concurrency:` from §AR-ci.12.1: one group per pull request,
    a group of its own for every other run, and only a pull request's run cancelling."""
    block = re.search(r"(?m)^concurrency:[ \t]*\n((?:  .*\n)+)", text)
    if not block:
        return ["no workflow-level `concurrency:` block: a replaced pull-request run keeps its "
                "runners"]
    def value(key):
        found = re.search(rf"(?m)^  {key}:[ \t]*(.*?)[ \t]*$", block.group(1))
        return found.group(1) if found else None
    group, cancel = value("group"), value("cancel-in-progress")
    expression = re.fullmatch(r"[\w-]*\$\{\{([^}]*)\}\}", group or "")
    keys = [key.strip() for key in expression.group(1).split("||")] if expression else []
    problems = []
    if keys[:1] != [PULL_REQUEST_KEY]:
        problems.append(f"`group: {group}` is not one group per pull request: use "
                        f"`{PULL_REQUEST_KEY}`, since two forks can share a branch name")
    if keys[1:] != [OWN_RUN_KEY]:
        problems.append(f"`group: {group}` gives another event's run no group of its own: fall "
                        f"back to `{OWN_RUN_KEY}`, since a newer run displaces a waiting one")
    if cancel != CANCELS:
        problems.append(f"`cancel-in-progress: {cancel}` must be `{CANCELS}`: only a pull "
                        "request's run is replaced")
    return problems


def job_permissions(head):
    """A job's own `permissions:` block, scope to access; `None` when it declares none."""
    block = re.search(r"(?m)^    permissions:[ \t]*\n((?:      .*\n?)+)", head + "\n")
    if not block:
        return None
    return dict(re.findall(r"(?m)^      ([\w-]+):[ \t]*([\w-]+)", block.group(1)))


def sweep_problems(text):
    """What keeps `ci.yml`'s sweep from §AR-ci.12.2: a merge_group-only job with no needs, a
    short bound and the one write permission it uses, calling the script on its own run."""
    found = jobs(text)
    if SWEEP not in found:
        return [f"no `{SWEEP}` job: a dropped group's run holds its runners to the end"]
    head = header(found[SWEEP])
    problems = []
    for event in EVENTS:
        if holds(field(head, "if"), event) != (event == "merge_group"):
            problems.append(f"`{SWEEP}` must run on merge_group alone, and its `if:` "
                            f"{'skips' if event == 'merge_group' else 'runs on'} {event}")
    if needs(head):
        problems.append(f"`{SWEEP}` needs {needs(head)}, so it waits behind the starved jobs")
    timeout = field(head, "timeout-minutes")
    if not (timeout and timeout.isdigit() and int(timeout) <= 5):
        problems.append(f"`{SWEEP}` has `timeout-minutes: {timeout}`, not a bound of five")
    if job_permissions(head) != {"actions": "write", "contents": "read"}:
        problems.append(f"`{SWEEP}` has permissions {job_permissions(head)}, not "
                        "`actions: write` and `contents: read` alone")
    if not any(SWEEP_CALL in squash(step) for step in steps(found[SWEEP])):
        problems.append(f"`{SWEEP}` does not run `{SWEEP_CALL}`")
    if "GH_TOKEN" not in found[SWEEP]:
        problems.append(f"`{SWEEP}` gives `gh` no GH_TOKEN")
    for job, body in found.items():
        if job != SWEEP and (job_permissions(header(body)) or {}).get("actions") == "write":
            problems.append(f"`{job}` takes `actions: write`, which only the sweep may")
        if SWEEP in needs(header(body)):
            problems.append(f"`{job}` needs the sweep, so a forge error would reach a verdict")
    return problems


def concurrency_workflow(group=GROUP, cancel=CANCELS):
    """A workflow whose `concurrency:` block is `group` and `cancel`, for
    `concurrency_problems`; `cancel=None` leaves that key out (§AR-ci.12.1)."""
    keys = f"  group: {group}\n" + (f"  cancel-in-progress: {cancel}\n" if cancel else "")
    return ("name: CI\n\non:\n  pull_request:\n\n"
            f"concurrency:\n{keys}\njobs:\n  a:\n    runs-on: ubuntu-latest\n")


def leg_fixture(guarded=None, call_guarded=False, guard="github.event_name != 'merge_group'",
                check_if=CHECK_IF):
    """A `ci.yml` whose Windows check needs its leg's call, and the leg it calls, for
    `every_event_problems`: `guarded` names a leg job, and `call_guarded` the call, that `guard`
    keeps from a run, and `check_if` is the check's own `if:` (§AR-ci.13.5)."""
    skip = f"    if: {guard}\n"
    leg = "jobs:\n" + "".join(
        f"  {job}:\n    runs-on: ${{{{ inputs.os }}}}\n" + (skip if job == guarded else "")
        + "    steps:\n      - run: exit 0\n" for job in ("rust", "python", "python-acceptance"))
    text = ("jobs:\n  leg-windows:\n    name: windows-latest\n" + (skip if call_guarded else "")
            + f"    uses: {CALL}\n    with:\n      os: windows-latest\n"
            "  test-windows:\n    name: cargo test (windows-latest)\n    runs-on: ubuntu-latest\n"
            f"    needs: [leg-windows]\n    if: {check_if}\n    steps:\n      - run: exit 1\n")
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

    def test_only_a_replaced_pull_request_run_is_cancelled(self):
        # §AR-ci.12.1: a replaced run is cancelled, so it stops taking runners from a group.
        self.assertEqual([], concurrency_problems(self.written))

    @unittest.expectedFailure  # red until #1388 lands the sweep; implement removes this
    def test_a_dropped_groups_run_is_cancelled_by_the_sweep(self):
        # §AR-ci.12.2: no concurrency key ends it, so a job of every group's run does.
        self.assertEqual([], sweep_problems(self.written))

    @unittest.expectedFailure  # red until #1388 lands the sweep; implement removes this
    def test_a_required_check_stays_off_a_runner_on_a_cancelled_pull_request_or_group_run(self):
        # §AR-ci.12.1, §AR-ci.12.2: so the run frees as the suites stop; §AR-ci.13.2: elsewhere
        # it judges, a live group's failed need included.
        checks = {field(header(body), "name"): body for body in self.jobs.values()
                  if is_required_check(body)}
        self.assertEqual({"cargo test (ubuntu-latest)", "cargo test (windows-latest)"}, set(checks))
        for name, body in sorted(checks.items()):
            with self.subTest(check=name):
                self.assertEqual([], check_if_problems(name, field(header(body), "if"),
                                                       needs(header(body))))

    @unittest.expectedFailure  # red until #1388 lands the sweep; implement removes this
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

    def test_a_judged_job_on_the_checks_condition_is_caught(self):
        # §AR-ci.12.1: that a cancelled run skips it is a check's exception, not a judged job's.
        problems = every_event_problems(*leg_fixture(guarded="rust", guard=CHECK_IF))
        self.assertEqual(1, len(problems), problems)
        self.assertIn("`leg-windows/rust`", problems[0])

    def test_a_check_off_its_condition_is_caught(self):
        # §AR-ci.12.1: `always()` skips no event, and still holds a replaced run's group.
        for check_if in ("always()", "!cancelled()", "success()",
                         "github.event_name != 'merge_group'"):
            with self.subTest(check_if=check_if):
                self.assertTrue(every_event_problems(*leg_fixture(check_if=check_if)))


class RequiredCheckConditionTests(unittest.TestCase):
    """§AR-ci.12.1: the table passes the point's `if:` and fails each way off it."""

    def test_the_points_condition_passes(self):
        self.assertEqual([], check_if_problems("c", CHECK_IF, ["a"]))

    def test_always_is_caught_for_holding_the_group(self):
        for condition in ("always()", "${{ always() }}"):
            with self.subTest(condition=condition):
                # §AR-ci.12.1, §AR-ci.12.2: a replaced run's and a dropped group's.
                problems = check_if_problems("c", condition, ["a"])
                self.assertEqual(2, len(problems), problems)
                self.assertIn("not hold on a cancelled pull_request run", problems[0])
                self.assertIn("not hold on a cancelled merge_group run", problems[1])

    def test_plain_not_cancelled_is_caught_for_cancelling_a_push_or_dispatch(self):
        # §AR-ci.12.1, §AR-ci.5.5.3: a push's and a dispatch's checks still run, whatever happens.
        problems = check_if_problems("c", "!cancelled()", ["a"])
        self.assertEqual(2, len(problems), problems)
        for event, problem in zip(("push", "workflow_dispatch"), problems):
            self.assertIn(f"hold on a cancelled {event} run", problem)

    def test_the_pull_request_only_condition_is_caught_for_holding_a_dropped_group(self):
        # §AR-ci.12.2: under it a cancelled group's checks would each wait for a runner.
        problems = check_if_problems(
            "c", "${{ github.event_name != 'pull_request' || !cancelled() }}", ["a"])
        self.assertEqual(1, len(problems), problems)
        self.assertIn("not hold on a cancelled merge_group run", problems[0])

    def test_success_is_caught(self):
        problems = check_if_problems("c", "success()", ["a"])
        self.assertEqual(1, len(problems), problems)
        self.assertIn("does not read as", problems[0])

    def test_no_condition_is_caught(self):
        problems = check_if_problems("c", None, ["a"])
        self.assertEqual(1, len(problems), problems)
        self.assertIn("no `if:`", problems[0])

    def test_a_condition_off_the_table_is_caught(self):
        # §AR-ci.13.2: a check skipped over a failed need reads as passed.
        for condition in (
                "${{ github.event_name != 'pull_request' || !contains(needs.*.result, 'failure') }}",
                "${{ github.event_name != 'pull_request' || !cancelled() && "
                "!contains(needs.*.result, 'failure') }}",
                "${{ github.event_name != 'pull_request' || needs.a.result == 'success' }}",
                "${{ github.event_name != 'pull_request' && !cancelled() }}",
                "${{ github.event_name != 'merge_group' || !cancelled() }}",
                "${{ github.event_name != 'pull_request' }}",
                "${{ github.event_name != 'pull_request' || !cancelled() || needs.b.result == 'x' }}"):
            with self.subTest(condition=condition):
                self.assertTrue(check_if_problems("c", condition, ["a"]))


def sweep_workflow(**changes):
    """A `ci.yml` with the sweep of §AR-ci.12.2 and one required check, for `sweep_problems`:
    each keyword replaces one line of the sweep's header, `None` drops it, `permissions_block`
    replaces its permissions and `extra` adds a job."""
    extra = changes.pop("extra", "")
    permissions = changes.pop("permissions_block",
                              "    permissions:\n      actions: write\n      contents: read\n")
    lines = {"if": "github.event_name == 'merge_group'", "runs-on": "ubuntu-latest",
             "timeout-minutes": "5", "needs": None, **changes}
    head = "".join(f"    {key}: {value}\n" for key, value in lines.items() if value is not None)
    return ("name: CI\n\non:\n  merge_group:\n\njobs:\n"
            f"  {SWEEP}:\n{head}{permissions}    steps:\n"
            f"      - env:\n          GH_TOKEN: ${{{{ github.token }}}}\n"
            f"        run: {SWEEP_CALL}\n" + extra)


class SweepCheckTests(unittest.TestCase):
    """§AR-ci.12.2: the sweep check passes the point's job and fails each way off it."""

    def test_the_points_job_passes(self):
        self.assertEqual([], sweep_problems(sweep_workflow()))

    def test_no_sweep_is_caught(self):
        problems = sweep_problems(sweep_workflow().replace(f"  {SWEEP}:", "  other:"))
        self.assertEqual(1, len(problems), problems)
        self.assertIn(f"no `{SWEEP}` job", problems[0])

    def test_a_sweep_off_merge_group_alone_is_caught(self):
        for condition in (None, "always()", "github.event_name != 'pull_request'",
                          "github.event_name == 'push'"):
            with self.subTest(condition=condition):
                self.assertTrue(sweep_problems(sweep_workflow(**{"if": condition})))

    def test_a_sweep_that_needs_a_job_is_caught(self):
        problems = sweep_problems(sweep_workflow(needs="[leg-ubuntu]"))
        self.assertEqual(1, len(problems), problems)
        self.assertIn("waits behind", problems[0])

    def test_a_sweep_without_a_short_bound_is_caught(self):
        for timeout in (None, "30"):
            with self.subTest(timeout=timeout):
                self.assertTrue(sweep_problems(sweep_workflow(**{"timeout-minutes": timeout})))

    def test_a_sweep_with_other_permissions_is_caught(self):
        for block in ("", "    permissions: write-all\n",
                      "    permissions:\n      actions: write\n      contents: write\n"):
            with self.subTest(block=block):
                self.assertTrue(sweep_problems(sweep_workflow(permissions_block=block)))

    def test_another_job_that_cancels_or_needs_the_sweep_is_caught(self):
        for extra in (f"  b:\n    runs-on: ubuntu-latest\n    needs: [{SWEEP}]\n"
                      "    steps:\n      - run: exit 0\n",
                      "  b:\n    runs-on: ubuntu-latest\n    permissions:\n"
                      "      actions: write\n    steps:\n      - run: exit 0\n"):
            with self.subTest(extra=extra):
                self.assertEqual(1, len(sweep_problems(sweep_workflow(extra=extra))))


class ConcurrencyCheckTests(unittest.TestCase):
    """§AR-ci.12.1: the concurrency check passes the point's block and fails each way a block
    could keep a replaced run, or cancel or displace a run nobody replaced."""

    def test_the_points_block_passes(self):
        self.assertEqual([], concurrency_problems(concurrency_workflow()))

    def test_no_workflow_level_block_is_caught(self):
        absent = concurrency_workflow().replace("concurrency:\n", "# concurrency:\n")
        job_level = ("on:\n  pull_request:\n\njobs:\n  a:\n    runs-on: ubuntu-latest\n"
                     f"    concurrency:\n      group: {GROUP}\n"
                     f"      cancel-in-progress: {CANCELS}\n")
        for label, text in (("absent", absent), ("on a job", job_level)):
            with self.subTest(label):
                problems = concurrency_problems(text)
                self.assertEqual(1, len(problems), problems)
                self.assertIn("no workflow-level `concurrency:` block", problems[0])

    def test_a_cancel_that_is_not_the_pull_requests_alone_is_caught(self):
        for cancel in ("true", "false", None, "${{ github.event_name != 'push' }}"):
            with self.subTest(cancel=cancel):
                problems = concurrency_problems(concurrency_workflow(cancel=cancel))
                self.assertEqual(1, len(problems), problems)
                self.assertIn(f"`cancel-in-progress: {cancel}`", problems[0])

    def test_a_group_not_one_per_pull_request_is_caught(self):
        for group in ("ci-${{ github.head_ref || github.run_id }}",
                      "ci-${{ github.sha || github.run_id }}"):
            with self.subTest(group=group):
                problems = concurrency_problems(concurrency_workflow(group=group))
                self.assertEqual(1, len(problems), problems)
                self.assertIn("is not one group per pull request", problems[0])

    def test_a_group_another_event_shares_is_caught(self):
        # §AR-ci.5.5.3: a shared group holds one waiting run, so a third push cancels the second.
        for group in ("ci-${{ github.event.pull_request.number || github.ref }}",
                      "ci-${{ github.event.pull_request.number }}"):
            with self.subTest(group=group):
                problems = concurrency_problems(concurrency_workflow(group=group))
                self.assertEqual(1, len(problems), problems)
                self.assertIn("no group of its own", problems[0])

    def test_a_group_with_neither_key_is_caught(self):
        for group in ("ci", "ci-${{ github.run_id }}", "ci-${{ github.ref }}",
                      "ci-${{ github.run_id || github.event.pull_request.number }}",
                      f"{GROUP}-${{{{ github.sha }}}}"):
            with self.subTest(group=group):
                problems = concurrency_problems(concurrency_workflow(group=group))
                self.assertEqual(2, len(problems), problems)
                self.assertIn("is not one group per pull request", problems[0])
                self.assertIn("no group of its own", problems[1])


if __name__ == "__main__":
    unittest.main()
