"""The meter of §AR-ci.requirements.1: how long each `ci.yml` gate takes on `main`,
and one drift issue when a gate's average is over the threshold (§AR-ci.requirements.3).

    ci_duration.py [<config>]
        reads the threshold and the window from <config>, `.github/ci-duration.json`
        by default (§AR-ci.requirements.4), averages each gate over the last
        window successful pushes to `main`, and opens or edits the drift issue

Everything but `main` is a function of the forge's JSON, so it is tested offline.
Exits 0 when done, 1 naming the file, key or `gh` call that failed."""

import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = ROOT / ".github" / "ci-duration.json"
WORKFLOW = "ci.yml"
MARKER = "<!-- ci-duration-drift -->"
# How many of a gate's slowest steps the report names.
STEPS_SHOWN = 5


class ConfigError(Exception):
    pass


@dataclass(frozen=True)
class Config:
    threshold_minutes: float
    window: int


@dataclass(frozen=True)
class GateAverage:
    average_minutes: float
    samples: int
    slowest_steps: list  # [(step name, average minutes)], slowest first


def load_config(path) -> Config:
    """§AR-ci.requirements.4: a missing file or key is an error naming it, never a default."""
    path = Path(path)
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ConfigError(f"{path}: no such file") from None
    except json.JSONDecodeError as error:
        raise ConfigError(f"{path}: not JSON: {error}") from None
    for key in ("threshold_minutes", "window"):
        if key not in data:
            raise ConfigError(f"{path}: missing key {key!r}")
    window = int(data["window"])
    # The forge lists at most 100 runs a page; fewer than one measures nothing.
    if not 1 <= window <= 100:
        raise ConfigError(f"{path}: key 'window' is {window}, outside 1..100")
    return Config(threshold_minutes=float(data["threshold_minutes"]), window=window)


def parse_stamp(stamp: str) -> datetime:
    # `fromisoformat` takes a trailing `Z` only from Python 3.11 on.
    return datetime.fromisoformat(stamp.replace("Z", "+00:00"))


def minutes(item: dict):
    """An item's `completed_at` less its `started_at`, or None when it has not both."""
    start, end = item.get("started_at"), item.get("completed_at")
    if not start or not end:
        return None
    return (parse_stamp(end) - parse_stamp(start)).total_seconds() / 60


def gate_averages(runs: list, window: int) -> dict:
    """Each gate's mean over the newest `window` runs it appears in, and its steps' means.

    A gate is a job as the forge names it, so each matrix leg is its own; a skipped
    job is not a sample (§AR-ci.requirements.3)."""
    totals: dict = {}
    for jobs in runs[:window]:
        for job in jobs:
            length = minutes(job)
            if job.get("conclusion") == "skipped" or length is None:
                continue
            gate = totals.setdefault(job["name"], {"sum": 0.0, "samples": 0, "steps": {}})
            gate["sum"] += length
            gate["samples"] += 1
            for step in job.get("steps") or []:
                step_length = minutes(step)
                if step.get("conclusion") == "skipped" or step_length is None:
                    continue
                total, count = gate["steps"].get(step["name"], (0.0, 0))
                gate["steps"][step["name"]] = (total + step_length, count + 1)
    return {
        name: GateAverage(
            average_minutes=gate["sum"] / gate["samples"],
            samples=gate["samples"],
            slowest_steps=sorted(((step, total / count) for step, (total, count) in gate["steps"].items()),
                                 key=lambda pair: pair[1], reverse=True),
        )
        for name, gate in totals.items()
    }


def drift_report(averages: dict, config: Config, repo: str | None = None):
    """The drift issue's body naming every gate over the threshold, or None when none is.

    `repo` (`owner/name`) makes the link to AR-ci absolute; without it the name is plain."""
    over = sorted(((name, gate) for name, gate in averages.items()
                   if gate.average_minutes > config.threshold_minutes),
                  key=lambda pair: pair[1].average_minutes, reverse=True)
    if not over:
        return None
    lines = [
        MARKER,
        f"Over the last {config.window} successful pushes to `main`, these `{WORKFLOW}` gates "
        f"average more than the {config.threshold_minutes:g}-minute threshold "
        f"({requirement_link(repo)}).",
        "",
        "| gate | average | runs |",
        "|---|---|---|",
    ]
    lines += [f"| `{name}` | {gate.average_minutes:.1f} min | {gate.samples} |" for name, gate in over]
    for name, gate in over:
        lines += ["", f"Where the time went in `{name}`, slowest steps first:", ""]
        lines += [f"- `{step}`: {length:.1f} min" for step, length in gate.slowest_steps[:STEPS_SHOWN]]
    lines += ["", "Threshold and window are read from `.github/ci-duration.json`; "
              "this issue is edited in place by `scripts/ci_duration.py`."]
    return "\n".join(lines) + "\n"


def requirement_link(repo: str | None) -> str:
    # An issue body resolves a relative link against /issues/, so only an absolute one works.
    if not repo:
        return "AR-ci.requirements.1"
    return f"[AR-ci.requirements.1](https://github.com/{repo}/blob/main/docs/architecture/AR-ci.md)"


def issue_action(open_issues: list, marker: str = MARKER):
    """Edit the open issue whose body begins with the marker line, or open one when none does.

    An issue that only quotes the marker further down is not the drift issue
    (§AR-ci.requirements.3). The macOS failure issue is found by the same rule under its
    own marker (§AR-ci.14.2)."""
    for issue in open_issues:
        if (issue.get("body") or "").lstrip().startswith(marker):
            return ("update", issue["number"])
    return ("open", None)


def gh(*arguments: str, stdin: str | None = None) -> str:
    binary = shutil.which("gh")
    if binary is None:
        sys.exit("error: gh is not on PATH")
    result = subprocess.run([binary, *arguments], input=stdin, capture_output=True,
                            encoding="utf-8", errors="replace")
    if result.returncode != 0:
        sys.exit(f"error: `gh {' '.join(arguments)}` exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout


def main(argv: list) -> int:
    try:
        config = load_config(argv[1] if len(argv) > 1 else DEFAULT_CONFIG)
    except ConfigError as error:
        sys.exit(f"error: {error}")
    listed = json.loads(gh("api", f"repos/{{owner}}/{{repo}}/actions/workflows/{WORKFLOW}/runs"
                                  f"?branch=main&event=push&status=success&per_page={config.window}"))
    runs = [json.loads(gh("api", f"repos/{{owner}}/{{repo}}/actions/runs/{run['id']}/jobs?per_page=100"))["jobs"]
            for run in listed["workflow_runs"][:config.window]]
    averages = gate_averages(runs, config.window)
    if not averages:
        # §AR-ci.requirements.3: the average is measured, so no run is no pass.
        print(f"error: no successful push to main of {WORKFLOW} was found; nothing was measured",
              file=sys.stderr)
        return 1
    for name, gate in sorted(averages.items()):
        print(f"{gate.average_minutes:6.1f} min over {gate.samples} runs  {name}")
    body = drift_report(averages, config, os.environ.get("GITHUB_REPOSITORY"))
    if body is None:
        print(f"every gate averages at most {config.threshold_minutes:g} minutes; nothing written")
        return 0
    issues = json.loads(gh("issue", "list", "--state", "open", "--limit", "1000", "--json", "number,body"))
    action, number = issue_action(issues)
    title = f"CI gates average over {config.threshold_minutes:g} minutes on main"
    if action == "update":
        gh("issue", "edit", str(number), "--title", title, "--body-file", "-", stdin=body)
        print(f"edited drift issue #{number}")
    else:
        print(gh("issue", "create", "--title", title, "--body-file", "-", stdin=body).strip())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
