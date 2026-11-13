"""The sweep of §AR-ci.12.2: cancel every `ci.yml` run of a merge-queue group the queue has
dropped, so it stops taking runners from the groups that can still land.

    cancel_dropped_groups.py <owner/repo> <own-run-id>

lists the `ci.yml` runs on `merge_group` and the queue's live `gh-readonly-queue` branches,
and cancels each run that is a `merge_group` run, has not completed, is not this run, and
whose group is no longer live, printing the run's id and branch. A group is live only while
its branch exists *and* points at the commit the run tests: the queue names a branch after
its pull request and the commit the group is built on, so a pull request queued again on the
same commit reuses a dropped group's name, while that group's run is still going.

The sweep reaches no verdict, so it exits 0 on any forge error, naming the call that failed.
When either list fails it cancels nothing, because a branch list it could not read would
make every group look dropped; when one cancellation fails it goes on to the next."""

import json
import shutil
import subprocess
import sys

# More than the default 20, so an orphan behind the newest runs is still found.
LIMIT = "100"
PREFIX = "refs/heads/"


def gh(*arguments: str) -> tuple[bool, str]:
    """Run `gh`, returning whether it succeeded and its output, or its error on a failure."""
    # `which` finds a `gh.cmd` through PATHEXT, where a bare name would not.
    binary = shutil.which("gh")
    if binary is None:
        return False, "gh is not on PATH"
    result = subprocess.run([binary, *arguments], capture_output=True, encoding="utf-8",
                            errors="replace")
    if result.returncode != 0:
        return False, f"exited {result.returncode}: {result.stderr.strip()}"
    return True, result.stdout


def listed(*arguments: str):
    """The JSON a listing call prints, or `None` after naming the call that failed."""
    ok, output = gh(*arguments)
    if ok:
        try:
            return json.loads(output)
        except ValueError as error:
            output = f"printed no JSON: {error}"
    print(f"cancel-dropped-groups: `gh {' '.join(arguments)}` {output}; cancelling nothing")
    return None


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: cancel_dropped_groups.py <owner/repo> <own-run-id>", file=sys.stderr)
        return 2
    repository, own = argv
    runs = listed("run", "list", "--repo", repository, "--workflow", "ci.yml", "--event",
                  "merge_group", "--limit", LIMIT, "--json",
                  "databaseId,headBranch,headSha,status,event")
    if runs is None:
        return 0
    refs = listed("api", f"repos/{repository}/git/matching-refs/heads/gh-readonly-queue/")
    if refs is None:
        return 0
    # Each live branch and the commit it points at, the forge's word on which group it holds.
    live = {ref["ref"].removeprefix(PREFIX): ref.get("object", {}).get("sha") for ref in refs}
    for run in runs:
        run_id = str(run.get("databaseId"))
        branch = run.get("headBranch")
        head = run.get("headSha")
        if (run.get("event") != "merge_group" or run.get("status") == "completed"
                or run_id == own or (head and live.get(branch) == head)):
            continue
        ok, output = gh("run", "cancel", run_id, "--repo", repository)
        if ok:
            print(f"cancel-dropped-groups: cancelled run {run_id} of dropped group {branch}")
        else:
            print(f"cancel-dropped-groups: `gh run cancel {run_id}` for {branch} {output}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
