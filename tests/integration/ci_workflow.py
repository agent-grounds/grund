"""How the layout tests read a workflow as text: the CI Python has no YAML parser (§AR-ci.13.5).

A job that calls the leg workflow `ci-leg.yml` is read as the jobs it calls, `<caller>/<job>`
each, with its `inputs.os` put in; a job's `matrix.os` list is expanded into one instance per
operating system and its `matrix.shard` list into one per shard, each with only the steps its
`if:` allows. `test_ci_language_jobs.py` holds the layout to §AR-ci.13 with it.
"""

import itertools
import re

from test_release_workflow import WORKFLOWS, jobs, steps

CI = WORKFLOWS / "ci.yml"
# §AR-ci.13: the leg, written once and called by `ci.yml` per system.
LEG = WORKFLOWS / "ci-leg.yml"
CALL = "./.github/workflows/ci-leg.yml"
# §AR-ci.13.2: a required check's name, and the system it judges.
AGGREGATE = re.compile(r"cargo test \((.+)\)")
# The `n` of a shard command as written, its `k` a matrix's `${{ matrix.shard }}` or a number.
COUNT = re.compile(r"python scripts/run_cargo_suite\.py --shard (?:\d+|\$\{\{ matrix\.shard \}\})/(\d+) ")


def shard_count(text):
    """The `n` the workflow's shards name, or two where it names none (§AR-ci.13.7)."""
    return max([int(count) for count in COUNT.findall(text)] or [2])


def with_called_workflows(text, called=None):
    """`text` with each job calling the leg workflow replaced by the jobs it calls.

    Each called job is `<caller>/<job>`, named `<caller's name> / <its name>` as the forge
    names it, with the caller's `with: os:` put in for `inputs.os` and its own id for
    `github.job`. `called` maps the `uses:` path to the leg's text; unset, it is read."""
    found = jobs(text)
    out = []
    for job_id, body in found.items():
        head = header(body)
        if field(head, "uses") != CALL:
            out.append(f"  {job_id}:\n{body}")
            continue
        leg = called[CALL] if called is not None else LEG.read_text(encoding="utf-8")
        system = re.search(r"(?m)^      os:\s*(\S+)\s*$", head).group(1)
        caller = (field(head, "name") or job_id).strip("'\"")
        for called_id, called_body in jobs(leg).items():
            name = (field(header(called_body), "name") or called_id).strip("'\"")
            called_body = re.sub(r"(?m)^    name:.*$", lambda _: f"    name: {caller} / {name}",
                                 called_body, count=1)
            if field(header(called_body), "name") is None:
                called_body = f"    name: {caller} / {name}\n{called_body}"
            called_body = (called_body.replace("${{ inputs.os }}", system)
                           .replace("inputs.os", f"'{system}'")
                           .replace("${{ github.job }}", called_id))
            out.append(f"  {job_id}/{called_id}:\n{called_body}")
    return re.split(r"(?m)^jobs:\n", text, maxsplit=1)[0] + "jobs:\n" + "\n".join(out) + "\n"


def ci_text():
    """`ci.yml` as the layout test reads it, the leg's jobs in place of its calls."""
    return with_called_workflows(CI.read_text(encoding="utf-8"))


def with_local_actions(text):
    """A job's steps with each local composite action's steps in place of its `uses:`, so a
    pin the shard takes from `.github/actions/` is read as the job's own (§AR-ci.5)."""
    def inline(found):
        action = (WORKFLOWS.parent / "actions" / found.group(1) / "action.yml").read_text(
            encoding="utf-8")
        return re.sub(r"(?m)^(?=.)", "  ", action.split("\n  steps:\n", 1)[1])
    return re.sub(r"(?m)^      - uses: \./\.github/actions/([\w-]+)\n(?:        .*\n)*", inline, text)


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


def matrix_shards(head):
    """A job's `matrix.shard` list, or `[None]` for a job with none (§AR-ci.13.7)."""
    listed = re.search(r"(?m)^        shard:\s*\[(.*)\]", head)
    return [int(item) for item in listed.group(1).split(",")] if listed else [None]


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


def step_if(step):
    found = re.search(r"(?m)^\s*(?:- )?if:\s*(.+)$", step)
    return found.group(1) if found else None


def holds(condition, event, results=None):
    """Whether an `if:` holds on `event` when the needs ended with `results` (§AR-ci.12). It
    reads `github.event_name`, `github.ref` and `needs.<job>.result` against a literal,
    `contains(needs.*.result, …)`, `always()`, `!`, `&&`, `||` and parentheses, and no more."""
    # `ci.yml` runs a push on main alone; every other event runs on a ref of its own (§AR-ci.11).
    values = {"github.event_name": event,
              "github.ref": "refs/heads/main" if event == "push" else f"refs/{event}"}
    def compare(found):
        value = (results or {})[found[2]] if found[2] else values[found[1]]
        return f" {(value == found[4]) == (found[3] == '==')} "
    text = re.sub(r"(github\.event_name|github\.ref|needs\.([\w-]+)\.result)\s*(==|!=)\s*'([^']*)'",
                  compare, (condition or "always()").replace("${{", "").replace("}}", ""))
    text = re.sub(r"contains\(needs\.\*\.result,\s*'(\w+)'\)",
                  lambda found: f" {found[1] in (results or {}).values()} ", text)
    text = text.replace("always()", " True ").replace("&&", " and ").replace("||", " or ")
    text = text.replace("!", " not ")
    if re.sub(r"\b(?:True|False|and|or|not)\b|[()\s]", "", text):
        raise ValueError(f"cannot read `if: {condition}`")
    return eval(text, {"__builtins__": {}})  # only booleans, `and`, `or`, `not` and parentheses


def aggregate_fails(job_steps, event, results):
    """Whether a required check's failing step runs on `event` when its needs ended so."""
    return any(holds(step_if(step), event, results)
               for step in job_steps if "exit 1" in commands(step))


def step_runs_on(step, system, shard=None):
    """False only for a step whose `if:` is a conjunction one of whose terms is false: a
    comparison of `matrix.os` with another system, of `matrix.shard` with another shard, or
    of two literals, as a called leg's `inputs.os` becomes. Anything else may hold."""
    text = (step_if(step) or "").replace("${{", "").replace("}}", "")
    if "||" in text:
        return True
    values = {"matrix.os": f"'{system}'", "matrix.shard": None if shard is None else str(shard)}
    for term in text.split("&&"):
        found = re.fullmatch(r"\s*(matrix\.os|matrix\.shard|'[^']*'|\d+)\s*(==|!=)\s*('[^']*'|\d+)\s*",
                             term)
        if not found:
            continue
        left = values.get(found[1], found[1])
        if left is not None and (left == found[3]) != (found[2] == "=="):
            return False
    return True


def instances(text):
    """One record per job, operating system and shard it runs on, matrices expanded.

    `need` is what an aggregate names to need the job: the job, or the call it came from."""
    found = []
    for job_id, body in jobs(text).items():
        head = header(body)
        name = (field(head, "name") or job_id).strip("'\"")
        runs_on = field(head, "runs-on") or ""
        matrix = "matrix.os" in runs_on or "matrix.os" in name
        for system, shard in itertools.product(matrix_os(head) if matrix else [runs_on],
                                               matrix_shards(head)):
            put = (lambda value, system=system, shard=shard: value
                   .replace("${{ matrix.os }}", system)
                   .replace("${{ matrix.shard }}", str(shard)))
            label = put(name)
            aggregate = AGGREGATE.fullmatch(label)
            job_steps = [put(step) for step in steps(body) if step_runs_on(step, system, shard)]
            found.append({
                "job": job_id, "need": job_id.split("/", 1)[0], "shard": shard, "name": label,
                "runs_on": system if "matrix.os" in runs_on else runs_on,
                # An aggregate judges the system its name carries, wherever it runs.
                "system": aggregate.group(1) if aggregate else system,
                "aggregate": bool(aggregate), "needs": needs(head), "if": field(head, "if"),
                "steps": job_steps,
                "commands": [command for step in job_steps for command in commands(step)],
                "text": "\n".join(job_steps),
            })
    return found
