"""Settling the failures of a link gate's network pass (§AR-ci.3.1): a github.com 5xx through
GitHub's API (§AR-ci.3.1.2), any other transient failure through one more lychee pass
(§AR-ci.3.1.3), and the sweep's report of what is kept (§AR-ci.3.1.5).
`scripts/check_links.py` runs it."""

from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Callable, Sequence
from urllib.parse import urlsplit

from link_scope import carries


GITHUB_ENDPOINTS = (
    (re.compile(r"/([^/]+)/([^/]+)/pull/(\d+)"), "repos/{0}/{1}/pulls/{2}"),
    (re.compile(r"/([^/]+)/([^/]+)/issues/(\d+)"), "repos/{0}/{1}/issues/{2}"),
    (re.compile(r"/([^/]+)/([^/]+)/blob/([^/]+)/(.+)"), "repos/{0}/{1}/contents/{3}?ref={2}"),
)


def report_failures(report_path: Path) -> list[tuple[str, str, dict]] | None:
    """Every `(source, url, status)` in lychee's JSON report, or None when it is
    missing or unreadable, which keeps lychee's verdict (§AR-ci.3.1.2)."""
    try:
        error_map = json.loads(report_path.read_text(encoding="utf-8"))["error_map"]
        return [
            (source, entry["url"], entry.get("status") or {})
            for source, entries in error_map.items()
            for entry in entries
        ]
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        return None


def _is_candidate(url: str, status: dict) -> bool:
    """A 5xx from exactly `github.com`; nothing else is re-checked (§AR-ci.3.1.2)."""
    code = status.get("code")
    return isinstance(code, int) and 500 <= code <= 599 and urlsplit(url).netloc == "github.com"


def _is_transient(url: str, status: dict) -> bool:
    """A failure lychee may answer differently once more: a 5xx from any host but
    `github.com`, a 403, 408 or 429, or a code-less http(s) failure other than a
    missing fragment (§AR-ci.3.1.3)."""
    parsed = urlsplit(url)
    if parsed.scheme not in ("http", "https"):
        return False
    code = status.get("code")
    if code:
        return code in (403, 408, 429) or (
            isinstance(code, int) and 500 <= code <= 599 and parsed.netloc != "github.com"
        )
    return "Cannot find fragment" not in f"{status.get('text') or ''} {status.get('details') or ''}"


def check_urls(
    urls: Sequence[str],
    repo_root: Path,
    runner: Callable[..., subprocess.CompletedProcess[str]],
    lychee: str,
    label: str,
) -> tuple[int, list[tuple[str, str, dict]] | None]:
    """One lychee pass over just `urls`, from stdin: its exit code and failures, or None for
    a missing or unreadable report (§AR-ci.3.1.3, §AR-ci.3.1.4)."""
    document = "".join(f"[{label}](<{url}>)\n" for url in urls)
    with tempfile.TemporaryDirectory(prefix="grund-recheck-") as scratch:
        report_path = Path(scratch) / "lychee.json"
        command = [lychee, "--no-progress", "--include-fragments", "--format", "json",
                   "--output", str(report_path), "-"]
        try:
            returncode = runner(
                command, cwd=repo_root, input=document, text=True, check=False
            ).returncode
        except OSError:
            return 1, None
        return returncode, report_failures(report_path)


def _recheck_transient(
    urls: Sequence[str],
    repo_root: Path,
    runner: Callable[..., subprocess.CompletedProcess[str]],
    lychee: str,
    pause: float,
) -> dict[str, dict] | None:
    """The `{url: status}` still failing after `pause` seconds and one lychee pass over
    just `urls`, or None when its report is missing or unreadable (§AR-ci.3.1.3)."""
    time.sleep(pause)
    returncode, failures = check_urls(urls, repo_root, runner, lychee, "re-check")
    if failures is None or (returncode != 0 and not failures):
        return None
    return {url: status for _, url, status in failures}


def github_endpoint(url: str) -> str | None:
    """The `gh api` endpoint a github.com pull, issue or blob URL names, its
    fragment dropped, or None for any other shape (§AR-ci.3.1.2)."""
    path = urlsplit(url).path
    for pattern, endpoint in GITHUB_ENDPOINTS:
        match = pattern.fullmatch(path)
        if match:
            return endpoint.format(*match.groups())
    return None


def _recheck(
    url: str, repo_root: Path, runner: Callable[..., subprocess.CompletedProcess[str]]
) -> str:
    """confirmed, broken or unsettled: one `gh api` call, no retry (§AR-ci.3.1.2)."""
    endpoint = github_endpoint(url)
    if endpoint is None:
        return "unsettled"
    try:
        answer = runner(
            ["gh", "api", endpoint],
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return "unsettled"
    if answer.returncode == 0:
        return "confirmed"
    if "(HTTP 404)" in (answer.stderr or ""):
        return "broken"
    return "unsettled"


def _status_label(status: dict) -> str:
    """The label lychee's console prints, which `--output` silences: the code, else
    TIMEOUT or ERROR."""
    if status.get("code"):
        return str(status["code"])
    return "TIMEOUT" if status.get("text") == "Timeout" else "ERROR"


def _status_reason(status: dict) -> str:
    """Lychee's reason as its console printed it: `text`, then `details` when `text`
    does not already carry it (§AR-ci.3.1.2)."""
    text, details = status.get("text") or "", status.get("details") or ""
    if details and details not in text:
        return f"{text}: {details}" if text else details
    return text


def _failure_detail(status: dict, outcome: str | None = None) -> str:
    """What follows a kept failure's label: the API outcome if re-checked, and lychee's
    reason (§AR-ci.3.1.2); the console line and the sweep's report both write it."""
    detail = f": {outcome} through the API" if outcome is not None else ""
    reason = _status_reason(status)
    return f"{detail} | {reason}" if reason else detail


def _failure_line(source: str, url: str, status: dict, outcome: str | None = None) -> str:
    """One kept failure: label, URL, source, the API outcome if re-checked, and
    lychee's reason (§AR-ci.3.1.2)."""
    return f"[{_status_label(status)}] {url} ({source}){_failure_detail(status, outcome)}"


def settle(
    failures: list[tuple[str, str, dict]],
    repo_root: Path,
    runner: Callable[..., subprocess.CompletedProcess[str]],
    returncode: int,
    lychee: str = "lychee",
    report: list[tuple[str, str, dict, str | None]] | None = None,
    pause: float = 5,
) -> int:
    """Pass only when every failure is a confirmed github.com 5xx (§AR-ci.3.1.2) or
    a transient failure the lychee re-check answered (§AR-ci.3.1.3). Each failure kept
    is printed, and added to `report` when given."""
    outcomes = {"confirmed": 0, "broken": 0, "unsettled": 0}
    kept: list[tuple[str, str, dict, str | None]] = []
    transient = []
    for source, url, status in failures:
        if _is_candidate(url, status):
            outcome = _recheck(url, repo_root, runner)
            outcomes[outcome] += 1
            if outcome != "confirmed":
                kept.append((source, url, status, outcome))
        elif _is_transient(url, status):
            transient.append((source, url, status))
        else:
            kept.append((source, url, status, None))
    urls = list(dict.fromkeys(url for _, url, _ in transient))
    still = _recheck_transient(urls, repo_root, runner, lychee, pause) if urls else {}
    for source, url, status in transient:
        # A missing report fails every URL with its first status (§AR-ci.3.1.3).
        if still is None or url in still:
            kept.append((source, url, (still or {}).get(url, status), None))
    for failure in kept:
        print(_failure_line(*failure))
    if report is not None:
        report.extend(kept)
    checked = sum(outcomes.values())
    if checked:
        print(
            f"{checked} GitHub 5xx re-checked through the API: {outcomes['confirmed']} confirmed, "
            f"{outcomes['broken']} broken, {outcomes['unsettled']} unsettled"
        )
    if urls:
        failed = len(urls) if still is None else sum(url in still for url in urls)
        print(
            f"{len(urls)} transient failures re-checked with lychee: "
            f"{len(urls) - failed} passed, {failed} failed"
        )
    if kept or not failures:
        return returncode
    return 0


def write_report(
    path: Path,
    kept: Sequence[tuple[str, str, dict, str | None]],
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
) -> None:
    """The sweep's report: `- <url> - <file>:<line> - <status>` for every line of every input
    file that carries a kept failure's URL (§AR-ci.3.1.5)."""
    listed = subprocess.run(
        [lychee, "--dump-inputs", *inputs],
        cwd=repo_root, capture_output=True, text=True, check=False,
    )
    files = sorted({os.path.normpath(name) for name in listed.stdout.splitlines() if name})
    texts = {}
    for name in files:
        try:
            texts[name] = (repo_root / name).read_text(encoding="utf-8", errors="replace")
        except OSError:
            pass
    lines = []
    for source, url, status, outcome in {failure[1]: failure for failure in kept}.values():
        found = [
            f"{name}:{number}"
            for name, text in texts.items()
            for number, line in enumerate(text.splitlines(), 1)
            if carries(line, url)
        ]
        status_text = f"{_status_label(status)}{_failure_detail(status, outcome)}"
        lines.extend(f"- {url} - {where} - {status_text}" for where in found or [source])
    path.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")
