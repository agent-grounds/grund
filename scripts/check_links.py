#!/usr/bin/env python3
"""Check links while resolving this repository's main-branch URLs locally (§AR-ci.3.1), and
settle a github.com 5xx through GitHub's API (§AR-ci.3.1.2)."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path, PurePosixPath
from typing import Callable, Sequence
from urllib.parse import unquote, urlsplit


SELF_PREFIX = "https://github.com/agent-grounds/grund/blob/main/"
SELF_PATH_PREFIX = "/agent-grounds/grund/blob/main/"
SELF_PATH_PREFIXES = {"blob": SELF_PATH_PREFIX, "tree": "/agent-grounds/grund/tree/main/"}


class SelfLinkError(Exception):
    pass


def self_links(dumped: str) -> list[str]:
    return sorted({line for line in dumped.splitlines() if line.startswith(SELF_PREFIX)})


def self_link_target(url: str, repo_root: Path) -> Path:
    """The checkout path a `blob/main` (file) or `tree/main` (directory) self-link names.

    Shared with the printed-link test, so the CLI's guide links and this gate
    agree on what a self-link names (§FS-cli.2.3).
    """
    parsed = urlsplit(url)
    kind = next(
        (k for k, prefix in SELF_PATH_PREFIXES.items() if parsed.path.startswith(prefix)), None
    )
    if (
        parsed.scheme != "https"
        or parsed.netloc != "github.com"
        or kind is None
        or parsed.query
    ):
        raise SelfLinkError(f"malformed canonical self-link: {url}")

    encoded = parsed.path.removeprefix(SELF_PATH_PREFIXES[kind])
    try:
        relative = unquote(encoded, errors="strict")
    except UnicodeError as exc:
        raise SelfLinkError(f"invalid path encoding in canonical self-link: {url}") from exc
    parts = PurePosixPath(relative).parts
    if not relative or any(part in ("", ".", "..") for part in parts):
        raise SelfLinkError(f"unsafe path in canonical self-link: {url}")

    root = repo_root.resolve()
    target = (root / Path(*parts)).resolve()
    try:
        target.relative_to(root)
    except ValueError as exc:
        raise SelfLinkError(f"self-link escapes the repository: {url}") from exc
    exists = target.is_file() if kind == "blob" else target.is_dir()
    if not exists:
        raise SelfLinkError(f"canonical self-link target is missing: {relative} ({url})")
    return target


def _local_url(url: str, repo_root: Path) -> str:
    parsed = urlsplit(url)
    if not parsed.path.startswith(SELF_PATH_PREFIX):
        raise SelfLinkError(f"malformed canonical self-link: {url}")
    target = self_link_target(url, repo_root)
    fragment = f"#{parsed.fragment}" if parsed.fragment else ""
    return f"{target.as_uri()}{fragment}"


def check_links(
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
    runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> int:
    dumped = runner(
        [lychee, "--dump", *inputs],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        text=True,
        check=False,
    )
    if dumped.returncode != 0:
        return dumped.returncode
    urls = self_links(dumped.stdout)
    if urls:
        local_document = "".join(f"[self-link](<{_local_url(url, repo_root)}>)\n" for url in urls)
        local = runner(
            [lychee, "--no-progress", "--include-fragments", "-"],
            cwd=repo_root,
            input=local_document,
            text=True,
            check=False,
        )
        if local.returncode != 0:
            return local.returncode

    command = [lychee, "--no-progress", "--include-fragments"]
    for url in urls:
        command.extend(["--exclude", f"^{re.escape(url)}$"])
    with tempfile.TemporaryDirectory(prefix="grund-links-") as scratch:
        report_path = Path(scratch) / "lychee.json"
        command.extend(["--format", "json", "--output", str(report_path), *inputs])
        returncode = runner(command, cwd=repo_root, check=False).returncode
        if returncode == 0:
            return 0
        failures = _report_failures(report_path)
    if failures is None:
        return returncode
    return _settle(failures, repo_root, runner, returncode)


GITHUB_ENDPOINTS = (
    (re.compile(r"/([^/]+)/([^/]+)/pull/(\d+)"), "repos/{0}/{1}/pulls/{2}"),
    (re.compile(r"/([^/]+)/([^/]+)/issues/(\d+)"), "repos/{0}/{1}/issues/{2}"),
    (re.compile(r"/([^/]+)/([^/]+)/blob/([^/]+)/(.+)"), "repos/{0}/{1}/contents/{3}?ref={2}"),
)


def _report_failures(report_path: Path) -> list[tuple[str, str, dict]] | None:
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
    return str(status.get("code") or status.get("text") or "unknown")


def _settle(
    failures: list[tuple[str, str, dict]],
    repo_root: Path,
    runner: Callable[..., subprocess.CompletedProcess[str]],
    returncode: int,
) -> int:
    """Pass only when every failure is a confirmed github.com 5xx (§AR-ci.3.1.2)."""
    outcomes = {"confirmed": 0, "broken": 0, "unsettled": 0}
    kept = []
    for source, url, status in failures:
        if not _is_candidate(url, status):
            kept.append(f"[{_status_label(status)}] {url} ({source})")
            continue
        outcome = _recheck(url, repo_root, runner)
        outcomes[outcome] += 1
        if outcome != "confirmed":
            kept.append(f"[{_status_label(status)}] {url} ({source}): {outcome} through the API")
    for line in kept:
        print(line)
    checked = sum(outcomes.values())
    if checked:
        print(
            f"{checked} GitHub 5xx re-checked through the API: {outcomes['confirmed']} confirmed, "
            f"{outcomes['broken']} broken, {outcomes['unsettled']} unsettled"
        )
    if kept or not failures:
        return returncode
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check links, resolving canonical same-repository main-branch URLs locally."
    )
    parser.add_argument("--lychee", default="lychee", help="lychee executable (default: lychee)")
    parser.add_argument("inputs", nargs="+", help="files and directories passed to lychee")
    args = parser.parse_args(argv)

    try:
        return check_links(args.inputs, Path.cwd(), args.lychee)
    except SelfLinkError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
