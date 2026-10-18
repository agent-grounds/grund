"""The report of §AR-ci.14.3: when the daily macOS run fails, open or edit one issue
naming the commit, the run and each failed step.

    macos_daily.py
        reads this run's jobs from the forge (`GITHUB_RUN_ID`), writes the report and
        opens the macOS failure issue, or edits the open one, labelled `bug` either way,
        by the rule of §AR-ci.requirements.3, which `ci_duration.put_on_issue` implements

Everything but `main` is a function of the forge's JSON, so it is tested offline.
Exits 0 when done, 1 naming the variable or `gh` call that failed."""

import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ci_duration import gh, issue_action, put_on_issue  # noqa: E402  (the open-or-edit rule lives beside this script)

MARKER = "<!-- macos-daily-failure -->"
TITLE = "The daily macOS suites fail on main"
WORKFLOW = "macos.yml"
KIND = "bug"  # §AR-ci.14.3: a suite is red on `main`


def failed_steps(jobs: list) -> list:
    """[(job name, [failed step names])] for each job that failed; the running report job has no conclusion yet."""
    failed = []
    for job in jobs:
        if job.get("conclusion") not in ("failure", "cancelled", "timed_out"):
            continue
        steps = [step["name"] for step in job.get("steps") or [] if step.get("conclusion") == "failure"]
        failed.append((job["name"], steps))
    return failed


def failure_report(jobs: list, repo: str, sha: str, run_url: str) -> str:
    """The macOS failure issue's body: the marker line first, then what failed where."""
    link = f"[AR-ci.14](https://github.com/{repo}/blob/main/docs/architecture/AR-ci.md)"
    lines = [MARKER,
             f"The daily macOS run of `{WORKFLOW}` failed on `main` at "
             f"[`{sha[:12]}`](https://github.com/{repo}/commit/{sha}): {run_url}", ""]
    failed = failed_steps(jobs)
    for name, steps in failed:
        lines.append(f"`{name}` failed" + (" in:" if steps else " before any step failed."))
        lines += [f"- `{step}`" for step in steps]
        lines.append("")
    if not failed:
        lines += ["The forge reported no failed job; the run page says what stopped it.", ""]
    lines.append(f"Each failing scheduled run edits this issue in place ({link}); a green run writes "
                 "nothing, so close it once macOS is green on `main` again.")
    return "\n".join(lines) + "\n"


def main() -> int:
    missing = [name for name in ("GITHUB_RUN_ID", "GITHUB_REPOSITORY", "GITHUB_SHA") if not os.environ.get(name)]
    if missing:
        sys.exit(f"error: {', '.join(missing)} not set; this runs in the macOS workflow's report job")
    repo, run_id = os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_RUN_ID"]
    server = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
    jobs = json.loads(gh("api", f"repos/{repo}/actions/runs/{run_id}/jobs?per_page=100"))["jobs"]
    body = failure_report(jobs, repo, os.environ["GITHUB_SHA"], f"{server}/{repo}/actions/runs/{run_id}")
    action, issue = put_on_issue(MARKER, TITLE, body, KIND)
    print(f"edited macOS failure issue #{issue}" if action == "update" else issue)
    return 0


if __name__ == "__main__":
    sys.exit(main())
