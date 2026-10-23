"""A stand-in for `gh` that answers a release helper's forge calls from a state
directory and plays grund's merge queue (§FS-distribution.4.4.1.1, §AR-ci.11).

It is a script, not a test: `release_queue_fixture.py` puts a wrapper named `gh`
on `PATH` that runs it. `GH_STUB_STATE` names the state directory and
`GH_STUB_ORIGIN` the bare repository that stands for agent-grounds/grund.

Every dispatched run is recorded with the commit its branch held, the `main` it
was dispatched over and its round: the ordinal of that `main` among the
candidate runs' (the publisher's dispatch is no round). A run is hidden from the
first `run list` after its dispatch, as a real dispatch returns before its run
exists, and takes two looks to pass: in progress, then success. `scenario.json`
says what else happens, per round:

- `move`: on the first look at a run of the round (or, with `move_at` set to
  `dispatch`, when the round's third run is dispatched), `main` fast-forwards to
  this commit, the way a merge-queue group lands. `lander` names a second commit
  landed one second later from the background, while the helper settles.
- `tag`: a tag name created on the moved `main` at the same moment.
- `fail`: the workflow file whose runs in the round conclude `failure` on their
  first look.
- `slow`: no run ever completes.
- `finish_before_cancel`: a cancel finds its run already passed, and fails the
  way `gh` does on a completed run.

A call this stub does not model is logged to `unmodelled` and exits 3."""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

STATE = Path(os.environ["GH_STUB_STATE"])
ORIGIN = os.environ["GH_STUB_ORIGIN"]
NAMES = {"ci.yml": "CI", "release.yml": "Release"}
FILES = {name: file for file, name in NAMES.items()}
REPO = {"full_name": "agent-grounds/grund", "html_url": "https://github.com/agent-grounds/grund"}


def git(*args, check=False):
    return subprocess.run(["git", "--git-dir", ORIGIN, *args], capture_output=True, text=True,
                          check=check).stdout.strip()


def tip(ref):
    return git("rev-parse", "--verify", "--quiet", ref) or None


def read(name, default):
    path = STATE / name
    return json.loads(path.read_text()) if path.exists() else default


def write(name, value):
    (STATE / name).write_text(json.dumps(value, indent=1))


def unmodelled(args):
    with open(STATE / "unmodelled", "a") as log:
        log.write(json.dumps(args) + "\n")
    sys.exit(f"gh stub: `gh {' '.join(args)}` is not modelled")


def options(args):
    """--name value / --name=value pairs, every -f value in order, and the rest."""
    found, fields, rest, i = {}, [], [], 0
    while i < len(args):
        arg = args[i]
        if arg in ("-f", "--field", "-F", "--raw-field"):
            fields.append(args[i + 1])
            i += 2
        elif arg.startswith("--") and "=" in arg:
            key, value = arg[2:].split("=", 1)
            found[key] = value
            i += 1
        elif arg.startswith("--") and i + 1 < len(args) and not args[i + 1].startswith("-"):
            found[arg[2:]] = args[i + 1]
            i += 2
        elif arg in ("-H", "--header", "-X", "--method"):
            found[arg.lstrip("-")] = args[i + 1]
            i += 2
        else:
            rest.append(arg)
            i += 1
    return found, fields, rest


def land(scenario, round_):
    """The queue lands this round's commit, and the tag and later commit with it."""
    landed = read("landed", [])
    if round_ in [entry["round"] for entry in landed]:
        return
    commit = scenario.get("move", {}).get(str(round_))
    if commit is None:
        return
    before = tip("refs/heads/main")
    git("update-ref", "refs/heads/main", commit, check=True)
    landed.append({"round": round_, "from": before, "to": commit})
    write("landed", landed)
    tag = scenario.get("tag", {}).get(str(round_))
    if tag:
        git("tag", tag, commit, check=True)
    later = scenario.get("lander", {}).get(str(round_))
    if later:
        subprocess.Popen(["sh", "-c", f'sleep 1; git --git-dir "{ORIGIN}" update-ref refs/heads/main {later}'],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)


def view(run):
    return {"databaseId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
            "workflowName": NAMES.get(run["workflow"], run["workflow"]), "headBranch": run["branch"],
            "headSha": run["sha"], "event": run["event"],
            "url": f"https://github.invalid/agent-grounds/grund/actions/runs/{run['id']}"}


def emit(rows, keys):
    keys = keys.split(",") if keys else None
    print(json.dumps([{k: row[k] for k in keys} if keys else row for row in rows]))


def workflow_run(args, runs, scenario):
    found, fields, rest = options(args[3:])
    if rest or set(found) - {"ref"}:
        unmodelled(args)
    workflow = FILES.get(args[2], args[2])
    branch = found.get("ref", "main")
    sha = tip(f"refs/heads/{branch}")
    if sha is None:
        sys.exit(f"could not find any workflows on ref {branch}")
    values = dict(f.split("=", 1) for f in fields)
    main = tip("refs/heads/main")
    candidate = values.get("publish_crates") != "true"
    bases = list(dict.fromkeys(r["main"] for r in runs if r["round"]))
    round_ = (bases.index(main) + 1 if main in bases else len(bases) + 1) if candidate else 0
    runs.append({"id": 1000 + len(runs), "workflow": workflow, "branch": branch, "sha": sha,
                 "fields": values, "event": "workflow_dispatch", "status": "queued", "conclusion": None,
                 "main": main, "round": round_, "looks": 0, "hidden": 1})
    write("runs.json", runs)
    in_round = [r for r in runs if r["round"] == round_ and round_]
    if round_ and scenario.get("move_at") == "dispatch" and len(in_round) == 3:
        land(scenario, round_)


def run_list(args, runs):
    found, _fields, rest = options(args[2:])
    if rest or set(found) - {"workflow", "branch", "event", "limit", "json", "status"}:
        unmodelled(args)
    workflow = FILES.get(found.get("workflow"), found.get("workflow"))
    rows = []
    for run in reversed(runs):
        if workflow not in (None, run["workflow"]) or found.get("branch") not in (None, run["branch"]):
            continue
        if found.get("event") not in (None, run["event"]):
            continue
        if run["hidden"]:
            run["hidden"] -= 1
            continue
        rows.append(view(run))
    write("runs.json", runs)
    if workflow == "ci.yml" and found.get("branch") == "main" and found.get("event") in (None, "push"):
        # A push to main runs CI there, and every main tip reads as green.
        rows.insert(0, view({"id": 9000, "workflow": "ci.yml", "branch": "main", "event": "push",
                             "sha": tip("refs/heads/main"), "status": "completed", "conclusion": "success"}))
    emit(rows[: int(found.get("limit", 20))], found.get("json"))


def run_view(args, runs, scenario):
    found, fields, rest = options(args[3:])
    if rest or fields or set(found) - {"json"}:
        unmodelled(args)
    run = next((r for r in runs if str(r["id"]) == args[2]), None)
    if run is None:
        sys.exit(f"could not find any run with id {args[2]}")
    if run["round"]:
        land(scenario, run["round"])
    if run["status"] != "completed" and not scenario.get("slow"):
        run["looks"] += 1
        if scenario.get("fail", {}).get(str(run["round"])) == run["workflow"]:
            run["status"], run["conclusion"] = "completed", "failure"
        elif run["looks"] >= 2:
            run["status"], run["conclusion"] = "completed", "success"
        else:
            run["status"] = "in_progress"
    elif run["status"] != "completed":
        run["status"] = "in_progress"
    write("runs.json", runs)
    row = view(run)
    keys = found.get("json")
    print(json.dumps({k: row[k] for k in keys.split(",")} if keys else row))


def run_cancel(args, runs, scenario):
    if len(args) != 3:
        unmodelled(args)
    run = next((r for r in runs if str(r["id"]) == args[2]), None)
    if run is None:
        sys.exit(f"could not find any run with id {args[2]}")
    if scenario.get("finish_before_cancel") and run["status"] != "completed":
        run["status"], run["conclusion"] = "completed", "success"
    if run["status"] == "completed":
        write("runs.json", runs)
        sys.exit("Cannot cancel a workflow run that is completed (HTTP 409)")
    run["status"], run["conclusion"], run["cancelled"] = "completed", "cancelled", True
    write("runs.json", runs)
    print(f"✓ Request to cancel workflow {run['id']} submitted.")


def api(args):
    found, _fields, rest = options(args[1:])
    rest = [arg for arg in rest if arg != "--paginate"]
    if len(rest) != 1 or found.get("X", found.get("method", "GET")).upper() != "GET":
        unmodelled(args)
    path = rest[0].lstrip("/")
    if re.fullmatch(r"repos/[^/]+/[^/]+", path):
        print(json.dumps(REPO))
        return
    commit = re.fullmatch(r"repos/[^/]+/[^/]+/commits/([0-9a-f]{7,40})/pulls", path)
    if commit is None:
        unmodelled(args)
    sha = git("rev-parse", "--verify", "--quiet", commit[1] + "^{commit}")
    subject = git("log", "-1", "--format=%s", sha) if sha else ""
    number = re.search(r" \(#(\d+)\)$", subject)
    if not sha or number is None:
        print("[]")  # a direct push, as the release and -dev commits are
        return
    print(json.dumps([{"number": int(number[1]), "title": subject[: number.start()],
                       "html_url": f"{REPO['html_url']}/pull/{number[1]}", "merged_at": "2026-10-23T12:00:00Z",
                       "base": {"ref": "main", "repo": dict(REPO)}, "merge_commit_sha": sha,
                       "state": "closed"}]))


def main(args):
    with open(STATE / "calls", "a") as log:
        log.write(json.dumps(args) + "\n")
    runs, scenario = read("runs.json", []), read("scenario.json", {})
    if args[:2] == ["workflow", "run"] and len(args) > 2:
        workflow_run(args, runs, scenario)
    elif args[:2] == ["run", "list"]:
        run_list(args, runs)
    elif args[:2] == ["run", "view"] and len(args) > 2:
        run_view(args, runs, scenario)
    elif args[:2] == ["run", "cancel"]:
        run_cancel(args, runs, scenario)
    elif args[:1] == ["api"]:
        api(args)
    else:
        unmodelled(args)


if __name__ == "__main__":
    main(sys.argv[1:])
