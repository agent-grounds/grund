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


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[bytes]:
    """git's output as bytes: no newline translation or locale changes a name (§AR-ci.3.1.4.1)."""
    return subprocess.run(
        ["git", "-c", "core.quotepath=off", *args],
        cwd=repo_root, capture_output=True, check=False,
    )


def _commit(result: subprocess.CompletedProcess[bytes]) -> str | None:
    """The commit a `rev-parse` or `merge-base` printed, or None (§AR-ci.3.1.4)."""
    sha = result.stdout.decode("utf-8").strip()
    return sha if result.returncode == 0 and sha else None


def diff_base(repo_root: Path, link_base: str | None) -> str | None:
    """The one diff base every invocation reads: the commit `LINK_BASE` names, else, when it is
    unset or empty, the merge base of `HEAD` with `origin/main`, else None. A set `LINK_BASE`
    that names no commit is None, never the `origin/main` fallback (§AR-ci.3.1.4)."""
    if link_base:
        return _commit(
            _git(repo_root, "rev-parse", "--verify", "--quiet", f"{link_base}^{{commit}}")
        )
    return _commit(_git(repo_root, "merge-base", "HEAD", "origin/main"))


HUNK = re.compile(rb"^@@ -\S+ \+(\d+)(?:,(\d+))? @@")
C_ESCAPES = {
    b'"': b'"', b"\\": b"\\", b"a": b"\a", b"b": b"\b", b"f": b"\f", b"n": b"\n", b"r": b"\r",
    b"t": b"\t", b"v": b"\v",
}
C_ESCAPE = re.compile(rb"\\([0-7]{3}|.)", re.DOTALL)


def patch_name(header: bytes) -> str | None:
    """The path a `+++ ` line of `git diff` names, as the file system holds it, or None for a
    deleted file (§AR-ci.3.1.4.1). git ends the name with a tab when it holds a space, after
    any closing quote, and C-quotes a name holding a double quote, a backslash or a control
    character, writing a byte it does not name as a `\\NNN` octal escape. So one trailing tab is
    dropped first, the quoting undone second, and the bytes read as UTF-8 last."""
    name = header.removesuffix(b"\t")
    if len(name) > 1 and name.startswith(b'"') and name.endswith(b'"'):
        name = C_ESCAPE.sub(
            lambda escape: bytes([int(escape[1], 8)]) if len(escape[1]) == 3
            else C_ESCAPES.get(escape[1], escape[1]),
            name[1:-1],
        )
    if name == b"/dev/null":
        return None
    return os.path.normpath(name.decode("utf-8").removeprefix("b/"))


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
    for line in diff.stdout.split(b"\n"):  # git ends a line at \n alone (§AR-ci.3.1.4.1)
        if line.startswith(b"+++ "):
            path = patch_name(line[4:])
        elif path is not None and (hunk := HUNK.match(line)):
            start, count = int(hunk[1]), int(hunk[2] if hunk[2] is not None else 1)
            numbers.setdefault(path, set()).update(range(start, start + count))
    untracked = _git(  # -z: each name raw, ended by NUL (§AR-ci.3.1.4.1)
        repo_root, "ls-files", "-z", "--others", "--exclude-standard", "--", *inputs
    )
    whole = {os.path.normpath(name.decode("utf-8")) for name in untracked.stdout.split(b"\0")
             if name and untracked.returncode == 0}
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


def dump_lines(dumped: bytes) -> list[str]:
    """The lines lychee printed, read as UTF-8 and split at `\\n` alone, so a carriage return or
    a line separator in a file name stays in the name (§AR-ci.3.1.4.1). Every reading of a
    lychee dump goes through here."""
    return dumped.decode("utf-8").split("\n")


def changed_urls(
    dumped: bytes, changed: dict[str, dict[int, str]], excluded: set[str]
) -> dict[str, list[str]]:
    """`{url: ["<file>:<line>", ...]}` for each external URL `lychee --dump --verbose` printed
    that a changed line carries, less the `excluded` self-links (§AR-ci.3.1.4)."""
    locations: dict[str, list[str]] = {}
    for match in map(DUMPED.match, dump_lines(dumped)):
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
