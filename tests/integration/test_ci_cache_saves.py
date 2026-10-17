"""§AR-ci.13.6 — every cache is restored on every event and saved only by a push to main,
or by the scheduled run of a workflow on main (§AR-ci.14.1).

Read as text, the way `test_ci_language_jobs.py` reads `ci.yml`: the CI Python
has no YAML parser. Every workflow under `.github/workflows/` is held to it, and
each check returns the problems it finds, so the same check runs against fixture
workflows that show it can still fail."""

import re
import unittest

from test_release_workflow import WORKFLOWS, jobs, squash, step_name, steps

MAIN_ONLY = re.compile(
    r"github\.event_name\s*==\s*'(push|schedule)'\s*&&\s*github\.ref\s*==\s*'refs/heads/main'"
    r"|github\.ref\s*==\s*'refs/heads/main'\s*&&\s*github\.event_name\s*==\s*'(push|schedule)'"
)
# A path a cargo build writes: the registry, the git checkouts or a target directory.
CARGO_PATH = re.compile(r"\.cargo/|target")


def cache_action(step):
    """`combined`, `restore` or `save` for a step using `actions/cache`, else None."""
    found = re.search(r"(?m)^\s*(?:- )?uses:\s*actions/cache(/restore|/save)?@", step)
    if not found:
        return None
    return found.group(1)[1:] if found.group(1) else "combined"


def condition(text):
    found = re.search(r"(?m)^\s*(?:- )?if:\s*(.+)$", text)
    return squash(found.group(1)) if found else ""


def job_condition(body):
    found = re.search(r"(?m)^    if:\s*(.+)$", body.split("\n    steps:", 1)[0])
    return squash(found.group(1)) if found else ""


def with_value(step, key):
    """A `with:` input of a step, a block scalar folded to its lines."""
    lines = step.splitlines()
    for index, line in enumerate(lines):
        found = re.match(rf"(\s*){key}:\s*(.*)$", line)
        if not found:
            continue
        value = found.group(2).strip()
        if value and value[0] not in "|>":
            return value
        indent = len(found.group(1))
        block = []
        for rest in lines[index + 1:]:
            if rest.strip() and len(rest) - len(rest.lstrip()) <= indent:
                break
            if rest.strip():
                block.append(rest.strip())
        return "\n".join(block)
    return ""


def step_id(step):
    found = re.search(r"(?m)^\s*(?:- )?id:\s*(\S+)", step)
    return found.group(1) if found else None


def save_problems(text):
    """No combined step; a save is guarded to a push or a schedule on main; a restore is not guarded."""
    problems = []
    for job, body in jobs(text).items():
        guard = job_condition(body)
        for step in steps(body):
            action = cache_action(step)
            where = f"{job}: {step_name(step) or 'a cache step'!r}"
            if action == "combined":
                problems.append(f"{where} is a combined actions/cache step, which saves on "
                                "every event its workflow runs on")
            elif action == "save" and not MAIN_ONLY.search(f"{condition(step)} {guard}"):
                problems.append(f"{where} saves without the push- or schedule-on-main guard")
            elif action == "restore" and re.search(r"github\.(event_name|ref)", condition(step)):
                problems.append(f"{where} restores on some events only")
    return problems


def pairing_problems(text):
    """Each save follows a restore of the same path and key in its job."""
    problems = []
    for job, body in jobs(text).items():
        restored = []
        for step in steps(body):
            action = cache_action(step)
            if action == "restore":
                restored.append(step)
            elif action == "save":
                path, key = with_value(step, "path"), with_value(step, "key")
                if not any(with_value(r, "path") == path and (
                        with_value(r, "key") == key
                        or key == f"${{{{ steps.{step_id(r)}.outputs.cache-primary-key }}}}")
                        for r in restored):
                    problems.append(f"{job}: {step_name(step)!r} saves a path and key "
                                    "no earlier step of its job restores")
    return problems


def incremental_problems(text):
    """A job caching a cargo path runs with `CARGO_INCREMENTAL: 0`."""
    def sets_it(head):
        return re.search(r"(?m)^\s+CARGO_INCREMENTAL:\s*[\"']?0[\"']?\s*$", head)

    workflow = text.split("\njobs:", 1)[0]
    problems = []
    for job, body in jobs(text).items():
        cached = [s for s in steps(body)
                  if cache_action(s) and CARGO_PATH.search(with_value(s, "path"))]
        if cached and not (sets_it(workflow) or sets_it(body.split("\n    steps:", 1)[0])):
            problems.append(f"{job} caches a cargo path without CARGO_INCREMENTAL: 0")
    return problems


CHECKS = (save_problems, pairing_problems, incremental_problems)
GUARD = "github.event_name == 'push' && github.ref == 'refs/heads/main'"
PATH = "          path: |\n            ~/.cargo/registry\n            target\n"
KEY = "          key: ${{ runner.os }}-cargo-${{ github.job }}-${{ hashFiles('Cargo.toml', '**/Cargo.lock') }}\n"


def fixture(combined=False, guard=GUARD, restore_guard=None, incremental=True,
            save_key=KEY, job_guard=False):
    """A workflow cached as §AR-ci.13.6 asks, with one knob per way it can go wrong."""
    env = "env:\n  CARGO_INCREMENTAL: 0\n" if incremental else "env:\n  CARGO_TERM_COLOR: always\n"
    head = f"    if: {guard}\n" if job_guard else ""
    if combined:
        cache = f"      - name: Cache cargo\n        uses: actions/cache@v5\n        with:\n{PATH}{KEY}"
        save = ""
    else:
        cache = ("      - name: Restore cargo\n        id: cargo\n"
                 + (f"        if: {restore_guard}\n" if restore_guard else "")
                 + f"        uses: actions/cache/restore@v5\n        with:\n{PATH}{KEY}")
        save = ("      - name: Save cargo\n"
                + (f"        if: {guard}\n" if guard and not job_guard else "")
                + f"        uses: actions/cache/save@v5\n        with:\n{PATH}{save_key}")
    return (f"on:\n  push:\n  pull_request:\n{env}jobs:\n  build:\n    runs-on: ubuntu-latest\n{head}"
            f"    steps:\n{cache}      - run: cargo test --locked\n{save}")


class CacheSavesTests(unittest.TestCase):
    """§AR-ci.13.6 over every workflow the repository has."""

    maxDiff = None

    def test_every_workflow_restores_everywhere_and_saves_only_from_main(self):
        for workflow in sorted(WORKFLOWS.glob("*.yml")):
            text = workflow.read_text(encoding="utf-8")
            for check in CHECKS:
                with self.subTest(workflow=workflow.name, check=check.__name__):
                    self.assertEqual([], check(text))


class CacheSavesCheckTests(unittest.TestCase):
    """Each check still fails on the layout it exists to refuse."""

    def test_the_layout_asked_for_passes_every_check(self):
        primary = "          key: ${{ steps.cargo.outputs.cache-primary-key }}\n"
        scheduled = "github.event_name == 'schedule' && github.ref == 'refs/heads/main'"
        for text in (fixture(), fixture(job_guard=True), fixture(save_key=primary),
                     fixture(guard=scheduled)):
            for check in CHECKS:
                with self.subTest(check=check.__name__):
                    self.assertEqual([], check(text))

    def test_a_combined_cache_step_is_caught(self):
        self.assertTrue(save_problems(fixture(combined=True)))

    def test_an_unguarded_or_half_guarded_save_is_caught(self):
        for guard in (None, "github.ref == 'refs/heads/main'", "github.event_name == 'push'",
                      "github.event_name == 'schedule'",
                      "github.event_name == 'pull_request' && github.ref == 'refs/heads/main'"):
            with self.subTest(guard=guard):
                self.assertTrue(save_problems(fixture(guard=guard)))

    def test_a_restore_limited_to_some_events_is_caught(self):
        self.assertTrue(save_problems(fixture(restore_guard="github.event_name == 'push'")))

    def test_a_save_no_restore_matches_is_caught(self):
        self.assertTrue(pairing_problems(fixture(save_key="          key: other-key\n")))

    def test_caching_cargo_with_incremental_state_is_caught(self):
        self.assertTrue(incremental_problems(fixture(incremental=False)))


if __name__ == "__main__":
    unittest.main()
