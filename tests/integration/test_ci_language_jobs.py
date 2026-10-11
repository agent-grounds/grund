"""§AR-ci.13 — each operating system's suites run as parallel jobs, judged by one
required `cargo test (<os>)` aggregate per operating system.

Read as text, the way `test_release_workflow.py` reads the workflows: the CI
Python has no YAML parser. A job's `matrix.os` list is expanded into one
instance per operating system, and each check returns the problems it finds, so
the same check runs against `ci.yml` and against fixture workflows that show it
can still fail (§AR-ci.13.5)."""

import re
import unittest

from test_release_workflow import WORKFLOWS, jobs, squash, steps

CI = WORKFLOWS / "ci.yml"
SYSTEMS = ("ubuntu-latest", "macos-latest", "windows-latest")
# §AR-ci.13.1: macOS may keep its suites in one job.
SPLIT = ("ubuntu-latest", "windows-latest")
RUNNER_OS = {"ubuntu-latest": "Linux", "macos-latest": "macOS", "windows-latest": "Windows"}
GATE = "python scripts/run_python_gate.py"
SUITES = {
    # §AR-ci.requirements.5.2: the cargo suites go through the quiet wrapper.
    "rust": ("python scripts/run_cargo_suite.py --workspace --all-targets --locked"
             " --features grund/test-workspace-load-count"),
    "python": f"{GATE} --without-node-acceptance",
    "node": f"{GATE} --only-node-acceptance",
}
# The rest of the Rust job (§AR-ci.13.1); the self-check is the Python heredoc of §AR-ci.4.
RUST_CHECKS = {
    "fmt": "cargo fmt --all -- --check",
    "native watch contract": ("python scripts/run_cargo_suite.py -p grund --features test-watch"
                              " --test check_watch_observer --locked"),
    "build": "cargo build --quiet --workspace --all-targets --locked",
    "self-check": '"check", ".", "--format", "json", "--full"',
}
PRE_COMMIT = "pre-commit run --all-files"
AGGREGATE = re.compile(r"cargo test \((.+)\)")


def header(body):
    return body.split("\n    steps:", 1)[0]


def field(head, key):
    found = re.search(rf"(?m)^    {key}:[ \t]*(.*)$", head)
    return found.group(1).strip() if found else None


def needs(head):
    value = field(head, "needs")
    if value is None:
        return []
    if value.startswith("["):
        return [item.strip() for item in value.strip("[]").split(",") if item.strip()]
    if value:
        return [value]
    block = head.split("\n    needs:", 1)[1]
    return re.findall(r"(?m)^      - ([A-Za-z0-9_-]+)\s*$", block.split("\n    ", 1)[0] + "\n")


def matrix_os(head):
    listed = re.search(r"(?m)^        os:\s*\[(.*)\]", head)
    return [item.strip() for item in listed.group(1).split(",")] if listed else []


def commands(step):
    """The commands of a step's `run:`, one per line of a block scalar."""
    lines = step.splitlines()
    for index, line in enumerate(lines):
        found = re.match(r"\s*(?:- )?run:\s*(.*)$", line)
        if found:
            value = found.group(1).strip()
            if value and value[0] not in "|>":
                return [value]
            return [rest.strip() for rest in lines[index + 1:] if rest.strip()]
    return []


def step_runs_on(step, system):
    """False only for a step whose `if:` compares `matrix.os` against another system."""
    condition = re.search(r"(?m)^\s*(?:- )?if:\s*(.+)$", step)
    if not condition:
        return True
    text = condition.group(1)
    equal = re.search(r"matrix\.os\s*==\s*'([^']+)'", text)
    unequal = re.search(r"matrix\.os\s*!=\s*'([^']+)'", text)
    if equal:
        return equal.group(1) == system
    if unequal:
        return unequal.group(1) != system
    return True


def instances(text):
    """One record per job and operating system it runs on, matrices expanded."""
    found = []
    for job_id, body in jobs(text).items():
        head = header(body)
        name = (field(head, "name") or job_id).strip("'\"")
        runs_on = field(head, "runs-on") or ""
        matrix = "matrix.os" in runs_on or "matrix.os" in name
        for system in matrix_os(head) if matrix else [runs_on]:
            label = name.replace("${{ matrix.os }}", system)
            aggregate = AGGREGATE.fullmatch(label)
            job_steps = [step for step in steps(body) if step_runs_on(step, system)]
            found.append({
                "job": job_id, "name": label,
                "runs_on": system if "matrix.os" in runs_on else runs_on,
                # An aggregate judges the system its name carries, wherever it runs.
                "system": aggregate.group(1) if aggregate else system,
                "aggregate": bool(aggregate), "needs": needs(head), "if": field(head, "if"),
                "steps": job_steps,
                "commands": [command for step in job_steps for command in commands(step)],
                "text": "\n".join(job_steps),
            })
    return found


def leg_commands(system):
    wanted = dict(SUITES, **RUST_CHECKS)
    if system == "ubuntu-latest":
        wanted["pre-commit"] = PRE_COMMIT
    return wanted


def runs(instance, command):
    if command.startswith('"'):
        return command in instance["text"]
    return command in instance["commands"]


def leg_jobs(found, system):
    """The non-aggregate jobs on `system` that run a command of its leg (§AR-ci.13.2)."""
    wanted = leg_commands(system).values()
    return [i for i in found if not i["aggregate"] and i["system"] == system
            and any(runs(i, command) for command in wanted)]


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
        expected = sorted({i["job"] for i in leg_jobs(found, system)})
        if sorted(aggregate["needs"]) != expected:
            problems.append(f"{name!r} needs {sorted(aggregate['needs'])}, not the jobs that run "
                            f"its leg on {system}: {expected}")
        for need in aggregate["needs"]:
            elsewhere = sorted({i["runs_on"] for i in found if i["job"] == need} - {system})
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


def split_problems(text):
    """§AR-ci.13.1: on the split systems no job runs two suites; every leg command runs once."""
    found = instances(text)
    problems = []
    for instance in found:
        ran = [suite for suite, command in SUITES.items() if runs(instance, command)]
        ran += ["python gate"] if GATE in instance["commands"] else []
        if instance["system"] in SPLIT and len(ran) > 1:
            problems.append(f"{instance['name']!r} on {instance['system']} runs {ran} one after "
                            "another in one job")
        if GATE in instance["commands"]:
            problems.append(f"{instance['name']!r} runs the whole Python gate, the Node acceptance "
                            "included, instead of one selection")
    for system in SYSTEMS:
        on = [i for i in found if i["system"] == system]
        for key, command in leg_commands(system).items():
            if command.startswith('"'):
                count = sum(i["text"].count(command) for i in on)
            else:
                count = sum(i["commands"].count(command) for i in on)
            if count != 1:
                problems.append(f"{system} runs {key} {count} times, not once")
        rust = [i for i in on if runs(i, SUITES["rust"])]
        for instance in rust:
            missing = [key for key, command in RUST_CHECKS.items() if not runs(instance, command)]
            if missing:
                problems.append(f"{instance['name']!r} runs the Rust suite without {missing}")
    return problems


def cache_problems(text):
    """§AR-ci.13.4: no two jobs on one system share a cache key."""
    owners = {}
    for instance in instances(text):
        for step in instance["steps"]:
            if "actions/cache" not in step:
                continue
            key = re.search(r"(?m)^\s*key:\s*(.+)$", step)
            if not key:
                continue
            resolved = (key.group(1).strip()
                        .replace("${{ github.job }}", instance["job"])
                        .replace("${{ matrix.os }}", instance["runs_on"])
                        .replace("${{ runner.os }}", RUNNER_OS.get(instance["runs_on"], instance["runs_on"])))
            owners.setdefault(resolved, set()).add(instance["job"])
    return [f"jobs {sorted(jobs_)} share the cache key {key}"
            for key, jobs_ in sorted(owners.items()) if len(jobs_) > 1]


def leg(job_id, system, *runs_, extra=""):
    lines = "".join(f"      - run: {command}\n" for command in runs_)
    return f"  {job_id}:\n    runs-on: {system}\n{extra}    steps:\n{lines}"


def aggregate(system, needed, guard=True, condition="always()"):
    check = ("      - if: ${{ contains(needs.*.result, 'failure') || contains(needs.*.result, "
             "'cancelled') || contains(needs.*.result, 'skipped') }}\n        run: exit 1\n"
             if guard else "      - run: echo done\n")
    cond = f"    if: {condition}\n" if condition else ""
    return (f"  test-{system.split('-')[0]}:\n    name: cargo test ({system})\n    runs-on: ubuntu-latest\n"
            f"    needs: [{', '.join(needed)}]\n{cond}    steps:\n{check}")


def cache(key):
    return ("      - uses: actions/cache@v5\n        with:\n          path: target\n"
            f"          key: {key}\n")


def fixture(cache_key="${{ runner.os }}-cargo-${{ github.job }}", node_rust_merged=False,
            guard=True, condition="always()", needs_pre_commit=True):
    """A workflow laid out as §AR-ci.13 asks, with one knob per way it can go wrong."""
    rust = list(RUST_CHECKS.values())[:3] + [SUITES["rust"]]
    selfcheck = '      - run: |\n          python - "check", ".", "--format", "json", "--full"\n'
    text = "jobs:\n"
    for system in SPLIT:
        short = system.split("-")[0]
        body_cache = cache(cache_key)
        text += leg(f"rust-{short}", system, *rust).replace("    steps:\n", "    steps:\n" + body_cache) + selfcheck
        python_runs = [SUITES["python"]] + ([SUITES["rust"]] if node_rust_merged else [])
        text += leg(f"python-{short}", system, *python_runs).replace("    steps:\n", "    steps:\n" + body_cache)
        text += leg(f"node-{short}", system, SUITES["node"]).replace("    steps:\n", "    steps:\n" + body_cache)
    text += leg("pre-commit", "ubuntu-latest", PRE_COMMIT)
    text += leg("macos", "macos-latest", *rust, SUITES["python"], SUITES["node"]) + selfcheck
    ubuntu = ["rust-ubuntu", "python-ubuntu", "node-ubuntu"] + (["pre-commit"] if needs_pre_commit else [])
    text += aggregate("ubuntu-latest", ubuntu, guard, condition)
    text += aggregate("windows-latest", ["rust-windows", "python-windows", "node-windows"], guard, condition)
    text += aggregate("macos-latest", ["macos"], guard, condition)
    return text


class LanguageJobsTests(unittest.TestCase):
    """§AR-ci.13 on `ci.yml`."""

    @classmethod
    def setUpClass(cls):
        cls.text = CI.read_text(encoding="utf-8")

    def test_each_required_check_aggregates_its_systems_jobs(self):
        # §AR-ci.13.2
        self.assertEqual([], required_check_problems(self.text))

    def test_each_required_check_fails_unless_every_need_succeeded(self):
        # §AR-ci.13.2
        self.assertEqual([], guard_problems(self.text))

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
        for check in (required_check_problems, guard_problems, split_problems, cache_problems):
            with self.subTest(check=check.__name__):
                self.assertEqual([], check(text))

    def test_a_shared_cache_key_is_caught(self):
        self.assertTrue(cache_problems(fixture(cache_key="${{ runner.os }}-cargo-x")))

    def test_two_suites_in_one_split_job_are_caught(self):
        self.assertTrue(split_problems(fixture(node_rust_merged=True)))

    def test_an_aggregate_without_always_or_without_a_guard_is_caught(self):
        self.assertTrue(guard_problems(fixture(condition=None)))
        self.assertTrue(guard_problems(fixture(guard=False)))

    def test_an_unjudged_pre_commit_job_is_caught(self):
        self.assertTrue(required_check_problems(fixture(needs_pre_commit=False)))

    def test_an_aggregate_over_matrix_jobs_is_caught(self):
        text = ("jobs:\n  rust:\n    runs-on: ${{ matrix.os }}\n    strategy:\n      matrix:\n"
                f"        os: [{', '.join(SYSTEMS)}]\n    steps:\n      - run: {SUITES['rust']}\n"
                "  test:\n    name: cargo test (${{ matrix.os }})\n    runs-on: ubuntu-latest\n"
                f"    strategy:\n      matrix:\n        os: [{', '.join(SYSTEMS)}]\n"
                "    needs: [rust]\n    if: always()\n    steps:\n      - run: exit 0\n")
        self.assertTrue(any("also runs on" in problem for problem in required_check_problems(text)))


if __name__ == "__main__":
    unittest.main()
