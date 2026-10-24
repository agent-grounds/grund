"""The changed-line scope of the link gate (§AR-ci.3.1.4): the diff base, the lines a change
adds or modifies, and the external URLs those lines carry. `scripts/check_links.py` runs it."""

from __future__ import annotations

import html
import os
import re
import subprocess
from pathlib import Path
from typing import Sequence
from urllib.parse import urlsplit


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-c", "core.quotepath=off", *args],
        cwd=repo_root, capture_output=True, text=True, check=False,
    )


def diff_base(repo_root: Path, link_base: str | None) -> str | None:
    """The one diff base every invocation reads: the commit `LINK_BASE` names, else, when it is
    unset or empty, the merge base of `HEAD` with `origin/main`, else None. A set `LINK_BASE`
    that names no commit is None, never the `origin/main` fallback (§AR-ci.3.1.4)."""
    if link_base:
        resolved = _git(repo_root, "rev-parse", "--verify", "--quiet", f"{link_base}^{{commit}}")
        if resolved.returncode == 0 and resolved.stdout.strip():
            return resolved.stdout.strip()
        return None
    merge_base = _git(repo_root, "merge-base", "HEAD", "origin/main")
    if merge_base.returncode == 0 and merge_base.stdout.strip():
        return merge_base.stdout.strip()
    return None


HUNK = re.compile(r"^@@ -\S+ \+(\d+)(?:,(\d+))? @@")


def changed_lines(
    repo_root: Path, base: str, inputs: Sequence[str]
) -> dict[str, dict[int, str]]:
    """`{path: {line: text}}` for every line added or modified between `base` and the working
    tree under `inputs`, every line of an untracked file git does not ignore included; paths
    are relative to `repo_root` (§AR-ci.3.1.4). Every option that shapes the parsed output is
    pinned, so no user diff config (`diff.mnemonicPrefix`, `diff.noprefix`, colour, an external
    diff) changes what the local gate reads from what CI reads (§AR-ci.1.1)."""
    diff = _git(
        repo_root, "diff", "--no-color", "--no-ext-diff", "--no-renames", "--relative",
        "--src-prefix=a/", "--dst-prefix=b/", "--unified=0", base, "--", *inputs,
    )
    if diff.returncode != 0:
        return {}
    numbers: dict[str, set[int]] = {}
    path = None
    for line in diff.stdout.splitlines():
        if line.startswith("+++ "):
            target = line[4:]
            path = None if target == "/dev/null" else os.path.normpath(target.removeprefix("b/"))
        elif path is not None and (hunk := HUNK.match(line)):
            start, count = int(hunk[1]), int(hunk[2] if hunk[2] is not None else 1)
            numbers.setdefault(path, set()).update(range(start, start + count))
    untracked = _git(repo_root, "ls-files", "--others", "--exclude-standard", "--", *inputs)
    whole = {os.path.normpath(name) for name in untracked.stdout.splitlines()
             if untracked.returncode == 0}
    changed: dict[str, dict[int, str]] = {}
    for name in numbers.keys() | whole:
        try:
            text = (repo_root / name).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        wanted = numbers.get(name, set())
        lines = {i: t for i, t in enumerate(text.splitlines(), 1) if name in whole or i in wanted}
        if lines:
            changed[name] = lines
    return changed


DUMPED = re.compile(r"^(\S+) \((.+)\)( \[excluded\])?$")


def changed_urls(
    dumped: str, changed: dict[str, dict[int, str]], excluded: set[str]
) -> dict[str, list[str]]:
    """`{url: ["<file>:<line>", ...]}` for each external URL `lychee --dump --verbose` printed
    that a changed line carries, less the `excluded` self-links (§AR-ci.3.1.4)."""
    locations: dict[str, list[str]] = {}
    for match in map(DUMPED.match, dumped.splitlines()):
        if not match or match[3]:
            continue
        url, source = match[1], os.path.normpath(match[2])
        lines = changed.get(source)
        if not lines or url in excluded or urlsplit(url).scheme not in ("http", "https"):
            continue
        for number, text in lines.items():
            if carries(text, url):
                locations.setdefault(url, []).append(f"{source}:{number}")
    return locations


def carries(line: str, url: str) -> bool:
    """Whether `line` writes `url`: verbatim, entity-decoded, or without the trailing slash
    lychee adds to a bare host. One test, for the gate's scope and the sweep's report."""
    forms = {url, url.rstrip("/")} - {""}
    text = html.unescape(line)
    return any(
        re.search(re.escape(form) + r"(?![\w\-/%~+=&?#]|\.\w)", text) for form in forms
    )
