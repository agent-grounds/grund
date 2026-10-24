"""§AR-ci.5.5, §AR-ci.5.5.1, §AR-ci.5.5.2 — `bench-store.yml` runs after `CI` completes,
outside every verdict. Its one job admits only a push to `main` in this repository, checks
out the commit the record measured, alone holds `contents: write`, and hands the event to
`scripts/bench_store.py` through `env:` alone. It finds the record by the names `ci.yml`
writes it by, and no workflow, action, hook or other script names the store.

Read as text, the way `test_bench_tier_ci.py` reads `ci.yml` (§AR-ci.13.5). Each check
returns the problems it finds, and runs against the layout asked for and against altered
copies of it, so a check that can no longer fail is caught. `test_bench_store.py` drives
the script."""

import re
import unittest

from test_bench_store import BRANCH, CONCLUSION, SCRIPT
from ci_workflow import header
from test_release_workflow import REPO_ROOT, WORKFLOWS, jobs, squash, steps

CI = WORKFLOWS / "ci.yml"
STORE = WORKFLOWS / "bench-store.yml"
REPOSITORY = "agent-grounds/grund"
HEAD_SHA = "${{ github.event.workflow_run.head_sha }}"
# §AR-ci.5.5.2: the names in ci.yml the store finds the record by.
ARTIFACT, RECORD_FILE, RECORD_JOB = "iai-callgrind-summaries", "first.json", "cargo bench (record)"
# §AR-ci.5.5.3: the store only adds, and the workflow leaves the one push to the script.
REWRITES = re.compile(r"--force|--mirror|--delete|\+refs/|\bgit/refs\b")
PUSHES = re.compile(r"\bgit\s+push\b")

# The layout §AR-ci.5.5 asks for, which every check passes; the tests alter it to fail them.
LAYOUT = """name: Bench store

on:
  workflow_run:
    workflows: [CI]
    types: [completed]

permissions: {}

jobs:
  store:
    if: >-
      github.event.workflow_run.event == 'push' &&
      github.event.workflow_run.head_branch == 'main' &&
      github.event.workflow_run.head_repository.full_name == github.repository
    runs-on: ubuntu-latest
    permissions:
      contents: write
      actions: read
    steps:
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.event.workflow_run.head_sha }}

      - name: Store the record
        env:
          GH_TOKEN: ${{ github.token }}
          RUN_ID: ${{ github.event.workflow_run.id }}
          COMMIT: ${{ github.event.workflow_run.head_sha }}
        run: |
          conclusion=$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$RUN_ID/jobs" --paginate \\
            --jq '.jobs[] | select(.name == "cargo bench (record)") | .conclusion')
          gh run download "$RUN_ID" --name iai-callgrind-summaries --dir record || true
          python3 scripts/bench_store.py --record record/first.json --commit "$COMMIT" \\
            --bench-record-conclusion "$conclusion"
"""


def nested(lines, index, column):
    """The lines under `lines[index]`: each following line indented past `column`."""
    found = []
    for line in lines[index + 1:]:
        if line.strip() and len(line) - len(line.lstrip()) <= column:
            break
        found.append(line)
    return found


def entry(text, key, column=None):
    """The value on the first `key:` line of `text` (at `column`, when given) and the lines
    nested under it, or `(None, [])`."""
    lines = text.splitlines()
    for index, line in enumerate(lines):
        found = re.match(rf"^(\s*(?:- )?){re.escape(key)}:(?:[ \t]+(.*))?$", line)
        if found and (column is None or len(found.group(1)) == column):
            return (found.group(2) or "").strip(), nested(lines, index, len(found.group(1)))
    return None, []


def mapping(text, key, column=None):
    """A `key:` mapping as a dict, written inline (`{a: b}`) or as a block; a scalar such as
    `read-all` is returned as it is, and a missing key as `None`."""
    value, lines = entry(text, key, column)
    if value is None:
        return None
    if value.startswith("{"):
        pairs = [item.split(":", 1) for item in value.strip("{}").split(",") if item.strip()]
        return {name.strip(): item.strip().strip("'\"") for name, item in pairs}
    if value:
        return value
    found = [(len(line) - len(line.lstrip()), re.match(r"^\s*([\w-]+):[ \t]*(.*)$", line))
             for line in lines if line.strip()]
    column = min((indent for indent, _ in found), default=0)
    return {match[1]: match[2].strip().strip("'\"") for indent, match in found
            if match and indent == column}


def scalar(text, key, column):
    """A `key:` scalar, its folded or literal block joined into one line."""
    value, lines = entry(text, key, column)
    if value is not None and value[:1] in (">", "|"):
        return squash("\n".join(lines))
    return value


def listed(text, key, column):
    value, lines = entry(text, key, column)
    if value:
        return [item.strip().strip("'\"") for item in value.strip("[]").split(",") if item.strip()]
    return [line.strip()[2:].strip().strip("'\"") for line in lines if line.strip().startswith("- ")]


def run_text(step):
    value, lines = entry(step, "run")
    return value if value and value[:1] not in "|>" else "\n".join(lines)


def store_job(text):
    found = jobs(text)
    return next(iter(found.values())) if len(found) == 1 else None


def run_event(event, branch, repository=REPOSITORY, conclusion="success"):
    """The `github.event.workflow_run` of a completed `CI` run."""
    return {"event": event, "head_branch": branch, "conclusion": conclusion, "name": "CI",
            "status": "completed", "head_sha": "a" * 40, "id": 1,
            "head_repository": {"full_name": repository}, "repository": {"full_name": REPOSITORY}}


def context(path, run):
    if path == "github.repository":
        return REPOSITORY
    if path in ("github.event_name", "github.ref"):  # a `workflow_run` job runs on the default branch
        return {"github.event_name": "workflow_run", "github.ref": "refs/heads/main"}[path]
    value = run if path.startswith("github.event.workflow_run.") else None
    for part in path.split(".")[3:]:
        if not isinstance(value, dict) or part not in value:
            raise ValueError(f"cannot read `{path}`")
        value = value[part]
    if value is None or isinstance(value, dict):
        raise ValueError(f"cannot read `{path}`")
    return value


def admits(condition, run):
    """Whether a job's `if:` holds for `run`. It reads comparisons of the event's fields,
    `github.repository` and literals, `!`, `&&`, `||` and parentheses, and no more."""
    operand = r"'[^']*'|[A-Za-z_][\w.-]*"
    def value(token):
        return token[1:-1] if token.startswith("'") else context(token, run)
    def compare(found):
        equal = str(value(found[1])).lower() == str(value(found[3])).lower()  # as GitHub compares
        return f" {equal == (found[2] == '==')} "
    text = (condition or "").replace("${{", "").replace("}}", "")
    text = re.sub(rf"({operand})\s*(==|!=)\s*({operand})", compare, text)
    text = text.replace("&&", " and ").replace("||", " or ").replace("!", " not ")
    if not text.strip() or re.sub(r"\b(?:True|False|and|or|not)\b|[()\s]", "", text):
        raise ValueError(f"cannot read `if: {condition}`")
    return eval(text, {"__builtins__": {}})  # only booleans, `and`, `or`, `not` and parentheses


ADMITTED = {"a push to main": run_event("push", "main"),
            "a push to main whose CI run failed": run_event("push", "main", conclusion="failure")}
REFUSED = {"a pull request": run_event("pull_request", "fix/issue-1"),
           "a pull request from a branch named main": run_event("pull_request", "main"),
           "a merge-queue group": run_event("merge_group", "gh-readonly-queue/main/pr-1-" + "b" * 40),
           "a release candidate's dispatch": run_event("workflow_dispatch", "release-candidate/0.19.0"),
           "a dispatch on main": run_event("workflow_dispatch", "main"),
           "a push to another branch": run_event("push", "release-candidate/0.19.0"),
           "a push to main from another repository": run_event("push", "main", "someone/grund")}


def trigger_problems(text):
    """§AR-ci.5.5: on `workflow_run` of `CI` alone, once it has completed."""
    problems = []
    triggers = list(mapping(text, "on", 0) or {})
    if triggers != ["workflow_run"]:
        problems.append(f"the store runs on {triggers}, not on `workflow_run` alone")
    trigger = "\n".join(entry(text, "workflow_run", 2)[1])
    if listed(trigger, "workflows", 4) != ["CI"]:
        problems.append("the store does not follow the workflow named `CI` alone")
    if listed(trigger, "types", 4) != ["completed"]:
        problems.append("the store does not wait for `CI` to complete")
    return problems


def guard_problems(text):
    """§AR-ci.5.5.1: the one job's own `if:` admits a push to `main` in this repository."""
    body = store_job(text)
    if body is None:
        return [f"the store has {len(jobs(text))} jobs, not one"]
    condition = scalar(header(body), "if", 4)
    if not condition:
        return ["the store job has no `if:`, so every run of CI reaches its write token"]
    problems = []
    for cases, wanted in ((ADMITTED, True), (REFUSED, False)):
        for case, run in cases.items():
            try:
                if admits(condition, run) != wanted:
                    problems.append(f"the guard {'refuses' if wanted else 'admits'} {case}")
            except ValueError as error:
                return [str(error)]
    return problems


def checkout_problems(text):
    """§AR-ci.5.5.1: the commit the record measured, not the default branch's tip."""
    body = store_job(text) or ""
    checkouts = [step for step in steps(body) if re.search(r"uses:\s*actions/checkout@", step)]
    if len(checkouts) != 1:
        return [f"the store checks out {len(checkouts)} times, not once"]
    ref = (mapping(checkouts[0], "with") or {}).get("ref")
    return [] if ref == HEAD_SHA else [f"the store checks out `{ref}`, not `{HEAD_SHA}`"]


def permission_problems(text):
    """§AR-ci.5.5.1: the workflow grants nothing; the job alone writes contents."""
    problems = []
    if mapping(text, "permissions", 0) != {}:
        problems.append("the workflow does not set `permissions: {}`")
    granted = mapping(header(store_job(text) or ""), "permissions", 4)
    if granted != {"contents": "write", "actions": "read"}:
        problems.append(f"the store job grants {granted}, not contents: write and actions: read")
    return problems


def input_problems(text):
    """§AR-ci.5.5.1, §AR-ci.5.5.2: the event reaches the script through `env:` alone, and the
    script is handed the head commit and the record job's conclusion."""
    body = store_job(text) or ""
    problems = [f"a `run:` splices `${{{{ }}}}` into a script: {squash(run_text(step))[:60]}"
                for step in steps(body) if "${{" in run_text(step)]
    calls = [step for step in steps(body) if "scripts/bench_store.py" in run_text(step)]
    if len(calls) != 1:
        return problems + [f"{len(calls)} steps run scripts/bench_store.py, not one"]
    env = dict(mapping(header(body), "env", 4) or {}, **(mapping(calls[0], "env") or {}))
    heads = {name for name, value in env.items() if squash(value) == HEAD_SHA}
    commit = re.search(r"--commit[ =]\"?\$\{?(\w+)", run_text(calls[0]))
    if not commit or commit.group(1) not in heads:
        problems.append(f"bench_store.py is not given `--commit` from an `env:` of {HEAD_SHA}")
    if CONCLUSION not in run_text(calls[0]):
        problems.append(f"bench_store.py is not given the record job's `{CONCLUSION}`")
    return problems


def rewrite_problems(text):
    """§AR-ci.5.5.3: nothing in the workflow pushes, forces, deletes or creates a ref."""
    found = sorted({hit.group(0) for pattern in (REWRITES, PUSHES) for hit in pattern.finditer(text)})
    return [f"the store workflow runs `{hit}`" for hit in found]


def name_problems(text, ci):
    """§AR-ci.5.5.2: the store names the artifact, the file and the job ci.yml writes them by."""
    problems = [f"the store does not name `{name}`" for name in (ARTIFACT, RECORD_FILE, RECORD_JOB)
                if name not in text]
    record = jobs(ci).get("bench-record", "")
    if (scalar(header(record), "name", 4) or "").strip("'\"") != RECORD_JOB:
        problems.append(f"ci.yml's bench-record job is no longer named `{RECORD_JOB}`")
    if not re.search(rf"(?m)^\s+name:\s*{ARTIFACT}\s*$", record) or "merge first" not in record:
        problems.append(f"ci.yml's bench-record job no longer uploads {RECORD_FILE} as {ARTIFACT}")
    return problems


def reader_problems(files):
    """§AR-ci.5.5, §REQ-runs-offline: no workflow, action, hook or script but the store's own
    names the branch or runs the store, so no gate can read it."""
    return [f"{path} names `{name}`" for path, text in sorted(files.items())
            for name in (BRANCH, "bench_store") if name in text]


def readers():
    paths = [*WORKFLOWS.glob("*.y*ml"), *(WORKFLOWS.parent / "actions").glob("*/action.y*ml"),
             REPO_ROOT / ".pre-commit-config.yaml", *(REPO_ROOT / "scripts").rglob("*.py"),
             *(REPO_ROOT / "scripts").rglob("*.sh")]
    return {path.relative_to(REPO_ROOT).as_posix(): path.read_text(encoding="utf-8")
            for path in paths if path not in (STORE, SCRIPT)}


CHECKS = (trigger_problems, guard_problems, checkout_problems, permission_problems,
          input_problems, rewrite_problems, lambda text: name_problems(text, CI.read_text("utf-8")))
GUARD = "github.event.workflow_run.head_repository.full_name == github.repository"


class TheStoreWorkflow(unittest.TestCase):
    """§AR-ci.5.5: `bench-store.yml` as the tree has it."""

    def check(self, check, *extra):
        self.assertTrue(STORE.is_file(), f"no {STORE.relative_to(REPO_ROOT)}: §AR-ci.5.5 has no store")
        self.assertEqual([], check(STORE.read_text(encoding="utf-8"), *extra))

    def test_it_runs_once_ci_has_completed(self):
        self.check(trigger_problems)

    def test_its_job_admits_only_a_push_to_main_in_this_repository(self):
        self.check(guard_problems)
        self.assertIn(GUARD, squash(STORE.read_text(encoding="utf-8")))

    def test_it_checks_out_the_commit_the_record_measured(self):
        self.check(checkout_problems)

    def test_only_its_job_can_write_and_only_contents(self):
        self.check(permission_problems)

    def test_the_event_reaches_the_script_through_env_alone(self):
        self.check(input_problems)

    def test_nothing_in_it_pushes_forces_or_creates_a_ref(self):
        self.check(rewrite_problems)

    def test_it_finds_the_record_by_the_names_ci_writes_it_by(self):
        self.check(name_problems, CI.read_text(encoding="utf-8"))

    def test_ci_is_still_named_ci_which_the_store_follows(self):
        self.assertEqual("CI", scalar(CI.read_text(encoding="utf-8"), "name", 0))

    def test_no_gate_names_the_store(self):
        self.assertEqual([], reader_problems(readers()))


class TheChecksCanFail(unittest.TestCase):
    """Each check passes the layout §AR-ci.5.5 asks for and fails each way off it."""

    def caught(self, check, *alterations):
        for old, new in alterations:
            with self.subTest(old=old, new=new):
                self.assertIn(old, LAYOUT)
                self.assertTrue(check(LAYOUT.replace(old, new, 1)))

    def test_the_layout_asked_for_passes_every_check(self):
        for check in CHECKS:
            self.assertEqual([], check(LAYOUT))
        self.assertIn(GUARD, squash(LAYOUT))

    def test_another_trigger_is_caught(self):
        self.caught(trigger_problems, ("[CI]", "[CI duration]"), ("[completed]", "[requested]"),
                    ("on:\n", "on:\n  push:\n"))

    def test_a_guard_that_admits_more_is_caught(self):
        self.caught(guard_problems, ("' &&", "' ||"),
                    ("' &&\n      " + GUARD, "'"),
                    ("      github.event.workflow_run.head_branch == 'main' &&\n", ""),
                    ("github.event.workflow_run.event == 'push'", "github.event_name == 'push'"),
                    ("    if: >-\n", "    env: >-\n"))

    def test_checking_out_the_tip_is_caught(self):
        self.caught(checkout_problems, (f"        with:\n          ref: {HEAD_SHA}\n", ""),
                    ("workflow_run.head_sha }}\n\n", "workflow_run.head_branch }}\n\n"))

    def test_a_wider_grant_is_caught(self):
        self.caught(permission_problems, ("permissions: {}\n", "permissions:\n  contents: write\n"),
                    ("      actions: read\n", "      actions: read\n      pull-requests: write\n"),
                    ("      contents: write\n", ""))

    def test_an_event_field_spliced_into_a_script_is_caught(self):
        self.caught(input_problems, ('--commit "$COMMIT"', f'--commit "{HEAD_SHA}"'),
                    (f"COMMIT: {HEAD_SHA}", "COMMIT: ${{ github.sha }}"),
                    (f' \\\n            {CONCLUSION} "$conclusion"', ""))

    def test_a_push_a_force_or_a_ref_creation_is_caught(self):
        push = '"$conclusion"\n'
        self.caught(rewrite_problems, (push, push + "          git push origin HEAD:benchmark-data\n"),
                    (push, push + "          python3 x.py --force\n"),
                    (push, push + "          gh api -X POST repos/o/r/git/refs -f ref=x\n"))

    def test_a_renamed_artifact_file_or_job_is_caught(self):
        ci = CI.read_text(encoding="utf-8")
        self.caught(lambda text: name_problems(text, ci), (ARTIFACT, "iai-callgrind-summary"),
                    (RECORD_FILE, "head.json"), (RECORD_JOB, "bench-record"))
        for old, new in ((f"name: {ARTIFACT}", "name: bench-records"),
                         (f"name: {RECORD_JOB}", "name: cargo bench (merge)")):
            with self.subTest(ci=old):
                self.assertIn(old, ci)
                self.assertTrue(name_problems(LAYOUT, ci.replace(old, new)))

    def test_a_gate_that_reads_the_store_is_caught(self):
        self.assertTrue(reader_problems({"ci.yml": "git fetch origin benchmark-data"}))
        self.assertTrue(reader_problems({"pre-commit": "python3 scripts/bench_store.py"}))

if __name__ == "__main__":
    unittest.main()
