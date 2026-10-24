"""§AR-ci.13 — each operating system's suites run as parallel jobs, judged by one
required `cargo test (<os>)` aggregate per operating system.

Read as text by `ci_workflow.py`, with the leg's jobs in place of its calls and every
matrix expanded. Each check returns the problems it finds, so the same check runs against
`ci.yml` and against fixture workflows that show it can still fail (§AR-ci.13.5).

The benchmark jobs' pinned environment and their one-harness comparison are read the same
way, in `test_ci_bench_jobs.py`."""

import itertools
import re
import unittest

from ci_workflow import (CALL, CI, LEG, aggregate_fails, ci_text, holds, instances, shard_count,
                         with_called_workflows)
from test_release_workflow import squash

# §AR-ci.13.1: macOS is no leg of `ci.yml`; its suites run once a day (§AR-ci.14).
SYSTEMS = ("ubuntu-latest", "windows-latest")
RUNNER_OS = {"ubuntu-latest": "Linux", "macos-latest": "macOS", "windows-latest": "Windows"}
GATE = "python scripts/run_python_gate.py"
# §AR-ci.requirements.5.2: the cargo suites go through the quiet wrapper; the hook runs it whole.
CARGO_SUITE = ("python scripts/run_cargo_suite.py --workspace --all-targets --locked"
               " --features grund/test-workspace-load-count")
# §AR-ci.13.7: CI runs it as `--shard <k>/<n>`, one `n` per leg and at least two.
SHARD = re.compile(r"python scripts/run_cargo_suite\.py --shard (\d+)/(\d+) ")


def shard_command(index, count):
    return CARGO_SUITE.replace(".py ", f".py --shard {index}/{count} ", 1)


SHARDS = shard_count(LEG.read_text(encoding="utf-8"))
SUITES = {
    **{f"rust {k}/{SHARDS}": shard_command(k, SHARDS) for k in range(1, SHARDS + 1)},
    # §AR-ci.3.4.3: the four selections of the Python gate.
    "python": f"{GATE} --rest",
    "node": f"{GATE} --only-node-acceptance",
    "python acceptance": f"{GATE} --only-python-acceptance",
    "slow": f"{GATE} --only-slow",
}
# §AR-ci.13.1: the one pair of suites a job may run, the slow tests after the acceptance.
PAIRED = {"python acceptance", "slow"}
RUST_SUITES = tuple(key for key in SUITES if key.startswith("rust "))
# The rest of the Rust job (§AR-ci.13.1); §AR-ci.4's self-check is inline or extracted.
RUST_CHECKS = {
    "fmt": "cargo fmt --all -- --check",
    "native watch contract": ("python scripts/run_cargo_suite.py -p grund --features test-watch"
                              " --test check_watch_observer --locked"),
    "build": "cargo build --quiet --workspace --all-targets --locked",
    "self-check": '"check", ".", "--format", "json", "--full"',
}
PRE_COMMIT = "pre-commit run --all-files"
# §AR-ci.5: the build job, the shard matrix that measures, and the record job.
BENCH_BUILD, BENCH_SHARD, BENCH_RECORD = "bench-build", "bench", "bench-record"
# §AR-ci.5.2: the shard job's comparing step, which the harness's `Ir` limits fail.
BENCH = ("cargo bench -p grund --features bench --locked --quiet --bench instructions"
         " -- --baseline=main --save-summary=json")
# §AR-ci.5: the bench job's image, named by its digest.
BENCH_IMAGE = re.compile(r"(?m)^\s+(?:container|image):\s*rust:[\w.-]+-bookworm@sha256:([0-9a-f]{64})\s*$")
BENCH_ENV = {"LC_ALL": "C.UTF-8", "TZ": "UTC", "RAYON_NUM_THREADS": "4",
             "CARGO_INCREMENTAL": "0", "RUSTFLAGS": ""}
# §AR-ci.12, §AR-ci.11: the events `ci.yml` runs on.
EVENTS = ("pull_request", "push", "workflow_dispatch", "merge_group")


def leg_commands(system):
    wanted = dict(SUITES, **RUST_CHECKS)
    if system == "ubuntu-latest":
        wanted["pre-commit"] = PRE_COMMIT
    return wanted


def command_count(instance, command):
    if command.startswith('"'):
        count = instance["text"].count(command)
        helper = "python scripts/check_ci_self.py"
        if helper in instance["commands"]:
            source = (CI.parents[2] / "scripts/check_ci_self.py").read_text(encoding="utf-8")
            if command in source and "timeout=30" in source:
                count += instance["commands"].count(helper)
        return count
    return instance["commands"].count(command)


def runs(instance, command):
    return command_count(instance, command) > 0


def leg_jobs(found, system):
    """The non-aggregate jobs on `system` that run a command of its leg (§AR-ci.13.2)."""
    wanted = leg_commands(system).values()
    return [i for i in found if not i["aggregate"] and i["system"] == system
            and any(runs(i, command) for command in wanted)]


def bench_jobs(found):
    """The jobs that run the bench comparison, and the jobs they need or that need them —
    the build and record jobs of §AR-ci.5; the Ubuntu aggregate judges them (§AR-ci.13.2)."""
    jobs_ = [i for i in found if not i["aggregate"]]
    names = {i["job"] for i in jobs_ if BENCH in i["commands"]}
    while True:
        linked = {need for i in jobs_ if i["job"] in names for need in i["needs"]}
        linked |= {i["job"] for i in jobs_ if names & set(i["needs"])}
        if linked <= names:
            return [i for i in jobs_ if i["job"] in names]
        names |= linked


def required_check_problems(text):
    """§AR-ci.13.2: one aggregate per system, needing exactly that system's leg jobs."""
    found = instances(text)
    problems = []
    aggregates = [i for i in found if i["aggregate"]]
    if sorted(i["system"] for i in aggregates) != sorted(SYSTEMS):
        problems.append(f"the required checks are {sorted(i['name'] for i in aggregates)}, "
                        f"not one `cargo test (<os>)` for each of {list(SYSTEMS)}")
    for aggregate in aggregates:
        name, system = aggregate["name"], aggregate["system"]
        ran = [key for key, command in leg_commands(system).items() if runs(aggregate, command)]
        if ran:
            problems.append(f"{name!r} runs {ran} itself instead of judging the jobs that do")
        if not aggregate["needs"]:
            problems.append(f"{name!r} aggregates no jobs (no `needs:`)")
            continue
        judged = leg_jobs(found, system) + (bench_jobs(found) if system == "ubuntu-latest" else [])
        expected = sorted({i["need"] for i in judged})
        if sorted(aggregate["needs"]) != expected:
            problems.append(f"{name!r} needs {sorted(aggregate['needs'])}, not the jobs that run "
                            f"its leg on {system}: {expected}")
        for need in aggregate["needs"]:
            elsewhere = sorted({i["runs_on"] for i in found if i["need"] == need} - {system})
            if elsewhere:
                problems.append(f"{name!r} needs {need!r}, which also runs on {elsewhere}, so its "
                                "result is not that of one system")
        if aggregate["runs_on"] != "ubuntu-latest":
            problems.append(f"{name!r} runs on {aggregate['runs_on']}, not ubuntu-latest")
    return problems


def guard_problems(text):
    """§AR-ci.13.2: the aggregate runs `always()` and fails on any need that did not succeed."""
    problems = []
    for aggregate in (i for i in instances(text) if i["aggregate"]):
        name = aggregate["name"]
        condition = (aggregate["if"] or "").replace("${{", "").replace("}}", "").strip()
        if condition != "always()":
            problems.append(f"{name!r} has `if: {aggregate['if']}`, not `always()`, so a failed "
                            "need skips it and a skipped required check passes")
        flat = squash(aggregate["text"])
        named = all(f"contains(needs.*.result, '{result}')" in flat
                    for result in ("failure", "cancelled", "skipped"))
        compared = bool(aggregate["needs"]) and "'success'" in flat and all(
            f"needs.{need}.result" in flat for need in aggregate["needs"])
        if not (named or compared):
            problems.append(f"{name!r} has no step that fails unless every need succeeded")
    return problems


def bench_guard_problems(text):
    """§AR-ci.13.2: on every event the Ubuntu aggregate accepts a bench job's success alone."""
    found = instances(text)
    benches = {i["job"] for i in bench_jobs(found)}
    problems = []
    for aggregate in (i for i in found if i["aggregate"] and i["system"] == "ubuntu-latest"):
        for job in sorted(benches & set(aggregate["needs"])):
            for event, result in itertools.product(
                    EVENTS, ("success", "failure", "cancelled", "skipped")):
                results = dict(dict.fromkeys(aggregate["needs"], "success"), **{job: result})
                if aggregate_fails(aggregate["steps"], event, results) != (result != "success"):
                    verdict = "fails" if result == "success" else "passes"
                    problems.append(f"{aggregate['name']!r} {verdict} on {event} when {job!r} "
                                    f"ends {result}")
    return problems


def split_problems(text):
    """§AR-ci.13.1: no job runs two suites, a shard counting as one and the slow tests after
    the Python acceptance excepted; every leg command runs once; the Rust checks run in a
    shard job; the whole cargo suite runs nowhere."""
    found = instances(text)
    problems = []
    for instance in found:
        ran = [suite for suite, command in SUITES.items() if runs(instance, command)]
        ran += ["python gate"] if GATE in instance["commands"] else []
        if len(ran) > 1 and set(ran) != PAIRED:
            problems.append(f"{instance['name']!r} on {instance['system']} runs {ran} one after "
                            "another in one job")
        if GATE in instance["commands"]:
            problems.append(f"{instance['name']!r} runs the whole Python gate, the acceptances "
                            "included, instead of one selection")
        if CARGO_SUITE in instance["commands"]:
            problems.append(f"{instance['name']!r} on {instance['system']} runs the whole cargo "
                            "suite instead of one shard (§AR-ci.13.7)")
    for system in SYSTEMS:
        on = [i for i in found if i["system"] == system]
        for key, command in leg_commands(system).items():
            count = sum(command_count(i, command) for i in on)
            if count != 1:
                problems.append(f"{system} runs {key} {count} times, not once")
        rust = [i for i in on if any(runs(i, SUITES[key]) for key in RUST_SUITES)]
        for key, command in RUST_CHECKS.items():
            if not any(runs(i, command) for i in rust):
                problems.append(f"{system} runs {key} in no job that runs a cargo shard")
    return problems


def saves(step):
    """Whether a step writes a cache: `actions/cache/save`, or the combined `actions/cache`."""
    return bool(re.search(r"(?m)^\s*(?:- )?uses:\s*actions/cache(?:/save)?@", step))


def cache_problems(text):
    """§AR-ci.13.4: no two jobs on one system save under one cache key, and of a job's
    instances, its shards, one alone saves each of its keys."""
    owners = {}
    for instance in instances(text):
        for step in instance["steps"]:
            if not saves(step):
                continue
            key = re.search(r"(?m)^\s*key:\s*(.+)$", step)
            if not key:
                continue
            resolved = (key.group(1).strip()
                        .replace("${{ github.job }}", instance["job"])
                        .replace("${{ matrix.os }}", instance["runs_on"])
                        .replace("${{ runner.os }}", RUNNER_OS.get(instance["runs_on"], instance["runs_on"])))
            owners.setdefault(resolved, []).append((instance["job"], instance["shard"]))
    problems = []
    for key, saving in sorted(owners.items()):
        jobs_ = sorted({job for job, _ in saving})
        if len(jobs_) > 1:
            problems.append(f"jobs {jobs_} share the cache key {key}")
        elif len(saving) > 1:
            problems.append(f"shards {sorted(shard for _, shard in saving)} of {jobs_[0]} all save "
                            f"the cache key {key}, and only the first save is kept")
    return problems


def job_env(head):
    """The job-level `env:` of a bench job header, unquoted."""
    block = re.search(r"(?m)^    env:\s*\n((?:      .*\n?)+)", head + "\n")
    if not block:
        return {}
    pairs = re.findall(r"(?m)^      ([A-Z_]+):[ \t]*(.*)$", block.group(1))
    return {key: value.strip().strip("'\"") for key, value in pairs}


def leg(job_id, system, *runs_, extra=""):
    lines = "".join(f"      - run: {command}\n" for command in runs_)
    return f"  {job_id}:\n    runs-on: {system}\n{extra}    steps:\n{lines}"


def aggregate(system, needed, guard=True, condition="always()", skipped_in_queue=()):
    check = ("      - if: ${{ contains(needs.*.result, 'failure') || contains(needs.*.result, "
             "'cancelled') || contains(needs.*.result, 'skipped') }}\n        run: exit 1\n"
             if guard else "      - run: echo done\n")
    if guard and skipped_in_queue:
        # The exception §AR-ci.13.2 no longer makes: a bench job's skip accepted in the queue.
        others = " || ".join(f"needs.{need}.result != 'success'" for need in needed)
        skips = " || ".join(f"!(needs.{job}.result == 'success' || needs.{job}.result == "
                            "'skipped' && github.event_name == 'merge_group')"
                            for job in skipped_in_queue)
        check = f"      - if: ${{{{ {others} || {skips} }}}}\n        run: exit 1\n"
        needed = needed + list(skipped_in_queue)
    cond = f"    if: {condition}\n" if condition else ""
    return (f"  test-{system.split('-')[0]}:\n    name: cargo test ({system})\n    runs-on: ubuntu-latest\n"
            f"    needs: [{', '.join(needed)}]\n{cond}    steps:\n{check}")


def cache(key, action="cache", condition=None):
    """A cache step: the combined `actions/cache`, or its `restore` or `save` half."""
    use = "cache" if action == "cache" else f"cache/{action}"
    head = f"      - if: {condition}\n        uses:" if condition else "      - uses:"
    return (f"{head} actions/{use}@v5\n        with:\n          path: target\n"
            f"          key: {key}\n")


def fixture(cache_key="${{ runner.os }}-cargo-${{ github.job }}", node_rust_merged=False,
            guard=True, condition="always()", needs_pre_commit=True, needs_bench=True,
            bench_skip_in_queue=False, every_shard_saves=False):
    """A workflow laid out as §AR-ci.13 asks, its jobs written out, with one knob per way it
    can go wrong; the first shard alone saves the cargo cache the others restore."""
    first, *rest = RUST_SUITES
    rust = list(RUST_CHECKS.values())[:3] + [SUITES[first]]
    selfcheck = '      - run: |\n          python - "check", ".", "--format", "json", "--full"\n'
    text = "jobs:\n"
    legs = {}
    for system in SYSTEMS:
        short = system.split("-")[0]
        def add(job, *runs_, restored=False):
            legs.setdefault(system, []).append(job)
            body_cache = cache(cache_key, "restore" if restored else "cache")
            return leg(job, system, *runs_).replace("    steps:\n", "    steps:\n" + body_cache)
        text += add(f"rust-{short}", *rust) + selfcheck
        shared = cache_key.replace("${{ github.job }}", f"rust-{short}")
        for index, key in enumerate(rest, start=2):
            text += add(f"rust-{short}-{index}", SUITES[key]).replace(
                cache(cache_key), cache(shared, "cache" if every_shard_saves else "restore"))
        python_runs = [SUITES["python"]] + ([SUITES[first]] if node_rust_merged else [])
        text += add(f"python-{short}", *python_runs)
        text += add(f"node-{short}", SUITES["node"])
        text += add(f"python-acceptance-{short}", SUITES["python acceptance"], SUITES["slow"])
    text += leg("pre-commit", "ubuntu-latest", PRE_COMMIT)
    ubuntu = legs["ubuntu-latest"] + (["pre-commit"] if needs_pre_commit else [])
    bench = [BENCH_BUILD, BENCH_SHARD, BENCH_RECORD]
    text += leg(BENCH_BUILD, "ubuntu-latest", "cargo bench --no-run")
    text += leg(BENCH_SHARD, "ubuntu-latest", BENCH, extra=f"    needs: [{BENCH_BUILD}]\n")
    text += leg(BENCH_RECORD, "ubuntu-latest", "python3 scripts/bench_record.py merge",
                extra=f"    needs: [{BENCH_SHARD}]\n")
    judged = bench if needs_bench else bench[:1] + bench[2:]
    if bench_skip_in_queue:
        text += aggregate("ubuntu-latest", ubuntu, guard, condition, skipped_in_queue=judged)
    else:
        text += aggregate("ubuntu-latest", ubuntu + judged, guard, condition)
    text += aggregate("windows-latest", legs["windows-latest"], guard, condition)
    return text


def called_fixture(gated=True, saving=(2,), needs_leg=True):
    """The layout `ci.yml` has: the leg written once and called per system, its shards a
    matrix (§AR-ci.13.7), read as the layout test reads it, with one knob per way it can go
    wrong: a shard-1 check that every shard runs, the shards that save, an unjudged call."""
    def step(command, condition=None):
        head = f"      - if: {condition}\n        run: " if condition else "      - run: "
        return f"{head}{command}\n"
    when = "matrix.shard == 1" if gated else None
    key = "${{ runner.os }}-cargo-rust"
    shards = ", ".join(str(k) for k in range(1, SHARDS + 1))
    leg_text = ("jobs:\n  rust:\n    name: rust ${{ matrix.shard }}/" f"{SHARDS}\n"
                "    runs-on: ${{ inputs.os }}\n    strategy:\n      matrix:\n"
                f"        shard: [{shards}]\n    steps:\n" + cache(key, "restore")
                + "".join(step(command, when) for command in list(RUST_CHECKS.values())[:3])
                + step('|\n          python - "check", ".", "--format", "json", "--full"', when)
                + step(CARGO_SUITE.replace(".py ", f".py --shard ${{{{ matrix.shard }}}}/{SHARDS} ", 1))
                + "".join(cache(key, "save", f"matrix.shard == {k}") for k in saving))
    for job, suites in (("python", ["python"]), ("node", ["node"]),
                        ("python-acceptance", ["python acceptance", "slow"])):
        leg_text += (f"  {job}:\n    runs-on: ${{{{ inputs.os }}}}\n    steps:\n"
                     + cache(f"${{{{ runner.os }}}}-cargo-{job}")
                     + "".join(step(SUITES[suite]) for suite in suites))
    text = "jobs:\n"
    for system in SYSTEMS:
        text += (f"  leg-{system.split('-')[0]}:\n    name: {system}\n    uses: {CALL}\n"
                 f"    with:\n      os: {system}\n")
    bench = [BENCH_BUILD, BENCH_SHARD, BENCH_RECORD]
    text += leg("pre-commit", "ubuntu-latest", PRE_COMMIT)
    text += leg(BENCH_BUILD, "ubuntu-latest", "cargo bench --no-run")
    text += leg(BENCH_SHARD, "ubuntu-latest", BENCH, extra=f"    needs: [{BENCH_BUILD}]\n")
    text += leg(BENCH_RECORD, "ubuntu-latest", "python3 scripts/bench_record.py merge",
                extra=f"    needs: [{BENCH_SHARD}]\n")
    text += aggregate("ubuntu-latest", ["leg-ubuntu", "pre-commit", *bench])
    text += aggregate("windows-latest", ["leg-windows" if needs_leg else "pre-commit"])
    return with_called_workflows(text, {CALL: leg_text})


class LanguageJobsTests(unittest.TestCase):
    """§AR-ci.13 on `ci.yml`, its calls of the leg read as the jobs they call."""

    @classmethod
    def setUpClass(cls):
        cls.text = ci_text()

    def test_each_required_check_aggregates_its_systems_jobs(self):
        # §AR-ci.13.2
        self.assertEqual([], required_check_problems(self.text))

    def test_each_required_check_fails_unless_every_need_succeeded(self):
        # §AR-ci.13.2
        self.assertEqual([], guard_problems(self.text))

    def test_the_ubuntu_check_accepts_only_the_bench_jobs_success(self):
        # §AR-ci.13.2, §AR-ci.12, §AR-ci.5.2
        self.assertEqual([], bench_guard_problems(self.text))

    def test_each_suite_runs_once_in_a_job_of_its_own(self):
        # §AR-ci.13.1
        self.assertEqual([], split_problems(self.text))

    def test_no_two_jobs_on_one_system_share_a_cache_key(self):
        # §AR-ci.13.4
        self.assertEqual([], cache_problems(self.text))


class LanguageJobsCheckTests(unittest.TestCase):
    """§AR-ci.13.5: each check passes the layout §AR-ci.13 asks for and fails each way off it."""

    def test_the_layout_asked_for_passes_every_check(self):
        text = fixture()
        for check in (required_check_problems, guard_problems, bench_guard_problems,
                      split_problems, cache_problems):
            with self.subTest(check=check.__name__):
                self.assertEqual([], check(text))

    def test_the_called_leg_passes_every_check(self):
        # §AR-ci.13.5: the leg's calls are read as the jobs they call, its shards one by one.
        text = called_fixture()
        for check in (required_check_problems, guard_problems, bench_guard_problems,
                      split_problems, cache_problems):
            with self.subTest(check=check.__name__):
                self.assertEqual([], check(text))

    def test_a_shard_one_check_every_shard_runs_is_caught(self):
        self.assertTrue(split_problems(called_fixture(gated=False)))

    def test_an_aggregate_not_needing_its_legs_call_is_caught(self):
        self.assertTrue(required_check_problems(called_fixture(needs_leg=False)))

    def test_a_shared_cache_key_is_caught(self):
        self.assertTrue(cache_problems(fixture(cache_key="${{ runner.os }}-cargo-x")))

    def test_shards_saving_one_key_twice_are_caught(self):
        # §AR-ci.13.4: an exact hit saves nothing, so all but the first save are lost.
        self.assertTrue(cache_problems(called_fixture(saving=(1, 2))))
        self.assertTrue(cache_problems(fixture(every_shard_saves=True)))

    def test_two_suites_in_one_split_job_are_caught(self):
        self.assertTrue(split_problems(fixture(node_rust_merged=True)))

    def test_an_aggregate_without_always_or_without_a_guard_is_caught(self):
        self.assertTrue(guard_problems(fixture(condition=None)))
        self.assertTrue(guard_problems(fixture(guard=False)))

    def test_an_unjudged_pre_commit_job_is_caught(self):
        self.assertTrue(required_check_problems(fixture(needs_pre_commit=False)))

    def test_an_unjudged_bench_job_is_caught(self):
        self.assertTrue(required_check_problems(fixture(needs_bench=False)))

    def test_a_bench_skip_accepted_in_the_queue_is_caught(self):
        # §AR-ci.13.2: the exception the Ubuntu check made before #792.
        self.assertTrue(bench_guard_problems(fixture(bench_skip_in_queue=True)))

    def test_a_condition_it_cannot_read_is_refused(self):
        with self.assertRaises(ValueError):
            holds("steps.cargo.outputs.cache-hit != 'true'", "push")

    def test_an_aggregate_over_matrix_jobs_is_caught(self):
        text = ("jobs:\n  rust:\n    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n"
                f"        os: [{', '.join(SYSTEMS)}]\n    steps:\n      - run: {SUITES[RUST_SUITES[0]]}\n"
                "  test:\n    name: cargo test (${{ matrix.os }})\n    runs-on: ubuntu-latest\n"
                f"    strategy:\n      matrix:\n        os: [{', '.join(SYSTEMS)}]\n"
                "    needs: [rust]\n    if: always()\n    steps:\n      - run: exit 0\n")
        self.assertTrue(any("also runs on" in problem for problem in required_check_problems(text)))


if __name__ == "__main__":
    unittest.main()
