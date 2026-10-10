"""The workflow runs a release helper waits on before `main` moves
(§FS-distribution.4.4.1): the `release.yml` dry run on the release commit, and
CI on it and on the `-dev` advance built over it.

    candidate_runs.py dispatch <workflow> <branch> <sha> [<field>=<value>...]
        starts <workflow> on <branch>, each field passed as `-f`, and prints the
        id of the run it started on exactly <sha>
    candidate_runs.py watch <run-id>...
        waits until every run has finished, and fails on the first that did not
        pass, naming its workflow, branch, commit, conclusion and page

A dispatch returns before its run exists, so the run is found by the commit it
was started on. A helper's candidate branches carry its own run id and nobody
else pushes them, so the run on that commit is the one just started.

Exits 0 when done, 1 naming the run or the `gh` call that failed."""

import json
import os
import shutil
import subprocess
import sys
import time

# Seconds between looks; a test sets 0 so a case does not wait for a server.
POLL = float(os.environ.get("CANDIDATE_RUNS_POLL", "5"))
WATCH = float(os.environ.get("CANDIDATE_RUNS_WATCH", "30"))
# How many looks a dispatched run gets to appear: five minutes at the default poll.
FIND_ATTEMPTS = 60


def fail(message: str) -> None:
    sys.exit(f"error: {message}")


def gh(*arguments: str) -> str:
    # `which` finds a `gh.cmd` through PATHEXT, where a bare name would not.
    binary = shutil.which("gh")
    if binary is None:
        fail("gh is not on PATH")
    result = subprocess.run([binary, *arguments], capture_output=True, encoding="utf-8", errors="replace")
    if result.returncode != 0:
        fail(f"`gh {' '.join(arguments)}` exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout


def dispatch(workflow: str, branch: str, sha: str, fields: list[str]) -> None:
    for field in fields:
        if "=" not in field:
            fail(f"field {field!r} is not <field>=<value>")
    gh("workflow", "run", workflow, "--ref", branch, *[part for field in fields for part in ("-f", field)])
    for attempt in range(FIND_ATTEMPTS):
        runs = json.loads(gh("run", "list", "--workflow", workflow, "--branch", branch,
                             "--event", "workflow_dispatch", "--limit", "20", "--json", "databaseId,headSha"))
        started = [run["databaseId"] for run in runs if run["headSha"] == sha]
        if started:
            print(started[0])
            return
        if attempt + 1 < FIND_ATTEMPTS:
            time.sleep(POLL)
    fail(f"dispatched {workflow} on {branch}, but no run on {sha} appeared after {FIND_ATTEMPTS} looks")


def watch(run_ids: list[str]) -> None:
    pending = list(run_ids)
    while pending:
        still = []
        for run_id in pending:
            run = json.loads(gh("run", "view", run_id, "--json",
                                "status,conclusion,workflowName,headBranch,headSha,url"))
            if run["status"] != "completed":
                still.append(run_id)
            elif run["conclusion"] != "success":
                fail(f"{run['workflowName']} on {run['headBranch']} ({run['headSha'][:12]}) ended "
                     f"{run['conclusion'] or 'without a conclusion'}, so main does not move: {run['url']}")
            else:
                print(f"{run['workflowName']} on {run['headBranch']} ({run['headSha'][:12]}) passed",
                      file=sys.stderr)
        pending = still
        if pending:
            time.sleep(WATCH)


def main(arguments: list[str]) -> None:
    if len(arguments) >= 4 and arguments[0] == "dispatch":
        dispatch(arguments[1], arguments[2], arguments[3], arguments[4:])
    elif len(arguments) >= 2 and arguments[0] == "watch":
        watch(arguments[1:])
    else:
        fail("usage: candidate_runs.py dispatch <workflow> <branch> <sha> [<field>=<value>...]"
             " | watch <run-id>...")


if __name__ == "__main__":
    main(sys.argv[1:])
