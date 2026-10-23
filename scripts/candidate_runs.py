"""The workflow runs a release helper waits on before `main` moves
(§FS-distribution.4.4.1): the `release.yml` dry run on the release commit, and
CI on it and on the `-dev` advance built over it. `scripts/release_candidates.py`
starts and gives up a round's runs through it (§AR-ci.11).

    candidate_runs.py dispatch <workflow> <branch> <sha> [<field>=<value>...]
        starts <workflow> on <branch>, each field passed as `-f`, and prints the
        id of the run it started on exactly <sha>
    candidate_runs.py watch [--main <sha>] [--deadline <unix-seconds>] <run-id>...
        waits until every run has finished, reading `main` on every poll, and
        exits 0 when every run passed; 1 on a run that did not pass, naming its
        workflow, branch, commit, conclusion and page; 3 when `main` is no longer
        <sha>; 4 when the deadline passes first. Every exit but 0 first cancels
        the runs still going.

A dispatch returns before its run exists, so the run is found by the commit it
was started on, among the runs that were not on the branch before the dispatch:
the publisher is dispatched on the branch and commit the dry run ran on.

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
# The watch's exits (§AR-ci.11).
PASSED, FAILED, MOVED, DEADLINE = 0, 1, 3, 4


def fail(message: str) -> None:
    sys.exit(f"error: {message}")


def gh(*arguments: str, check: bool = True) -> subprocess.CompletedProcess:
    # `which` finds a `gh.cmd` through PATHEXT, where a bare name would not.
    binary = shutil.which("gh")
    if binary is None:
        fail("gh is not on PATH")
    result = subprocess.run([binary, *arguments], capture_output=True, encoding="utf-8", errors="replace")
    if check and result.returncode != 0:
        fail(f"`gh {' '.join(arguments)}` exited {result.returncode}: {result.stderr.strip()}")
    return result


def runs_on(workflow: str, branch: str) -> list[dict]:
    return json.loads(gh("run", "list", "--workflow", workflow, "--branch", branch, "--event",
                         "workflow_dispatch", "--limit", "20", "--json", "databaseId,headSha").stdout)


def dispatch(workflow: str, branch: str, sha: str, fields: list[str]) -> None:
    for field in fields:
        if "=" not in field:
            fail(f"field {field!r} is not <field>=<value>")
    # §AR-ci.11: a run already on the branch is never the one this dispatch starts.
    before = {run["databaseId"] for run in runs_on(workflow, branch)}
    gh("workflow", "run", workflow, "--ref", branch, *[part for field in fields for part in ("-f", field)])
    for attempt in range(FIND_ATTEMPTS):
        started = [run["databaseId"] for run in runs_on(workflow, branch)
                   if run["headSha"] == sha and run["databaseId"] not in before]
        if started:
            print(started[0])
            return
        if attempt + 1 < FIND_ATTEMPTS:
            time.sleep(POLL)
    fail(f"dispatched {workflow} on {branch}, but no run on {sha} appeared after {FIND_ATTEMPTS} looks")


def main_tip() -> str | None:
    """`main` as git reads it, so polling spends none of the token's API budget (§AR-ci.11)."""
    result = subprocess.run(["git", "ls-remote", "origin", "refs/heads/main"], capture_output=True,
                            encoding="utf-8", errors="replace")
    fields = result.stdout.split()
    return fields[0] if result.returncode == 0 and fields else None


def cancel(run_ids: list[str]) -> None:
    """Cancel each run; one the forge reports already finished ends nothing (§AR-ci.11)."""
    for run_id in run_ids:
        result = gh("run", "cancel", run_id, check=False)
        if result.returncode == 0:
            print(f"cancelled run {run_id}", file=sys.stderr)
        elif "completed" in result.stderr or "409" in result.stderr:
            print(f"run {run_id} had already finished", file=sys.stderr)
        else:
            print(f"warning: could not cancel run {run_id}: {result.stderr.strip()}", file=sys.stderr)


def watch(run_ids: list[str], main: str | None, deadline: float | None) -> int:
    pending = list(run_ids)
    while True:
        still, failed = [], None
        for run_id in pending:
            run = json.loads(gh("run", "view", run_id, "--json",
                                "status,conclusion,workflowName,headBranch,headSha,url").stdout)
            name = f"{run['workflowName']} on {run['headBranch']} ({run['headSha'][:12]})"
            if run["status"] != "completed":
                still.append(run_id)
            elif run["conclusion"] != "success":
                failed = failed or (f"{name} ended {run['conclusion'] or 'without a conclusion'}, "
                                    f"so main does not move: {run['url']}")
            else:
                print(f"{name} passed", file=sys.stderr)
        pending = still
        # A failure seen on the same poll as a move is a failure (§FS-distribution.4.4.1.4).
        if failed:
            cancel(pending)
            print(f"error: {failed}", file=sys.stderr)
            return FAILED
        tip = main_tip() if main else None
        if tip and tip != main:
            print(f"main moved from {main} to {tip} while the candidate runs were going", file=sys.stderr)
            cancel(pending)
            return MOVED
        if not pending:
            return PASSED
        if deadline is not None and time.time() >= deadline:
            print(f"the watch's deadline passed with {len(pending)} run(s) still going", file=sys.stderr)
            cancel(pending)
            return DEADLINE
        time.sleep(WATCH)


def watch_arguments(arguments: list[str]) -> tuple[list[str], str | None, float | None]:
    main = deadline = None
    while arguments[:1] in (["--main"], ["--deadline"]) and len(arguments) >= 2:
        if arguments[0] == "--main":
            main = arguments[1]
        else:
            try:
                deadline = float(arguments[1])
            except ValueError:
                fail(f"--deadline {arguments[1]!r} is not a number of seconds")
        arguments = arguments[2:]
    return arguments, main, deadline


def main(arguments: list[str]) -> None:
    if len(arguments) >= 4 and arguments[0] == "dispatch":
        dispatch(arguments[1], arguments[2], arguments[3], arguments[4:])
        return
    if arguments[:1] == ["watch"]:
        run_ids, main_sha, deadline = watch_arguments(arguments[1:])
        if run_ids and not any(run_id.startswith("--") for run_id in run_ids):
            sys.exit(watch(run_ids, main_sha, deadline))
    fail("usage: candidate_runs.py dispatch <workflow> <branch> <sha> [<field>=<value>...]"
         " | watch [--main <sha>] [--deadline <unix-seconds>] <run-id>...")


if __name__ == "__main__":
    main(sys.argv[1:])
