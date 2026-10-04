#!/usr/bin/env python3
"""What the release helper's modules share: its one error, git, and link rebasing.

`prepare_changelog_release.py` writes the release (§FS-distribution.4.5) from the
pull requests `release_pull_requests.py` lists (§FS-distribution.4.6.1) and the
notices `release_notices.py` reads (§FS-distribution.4.6.4); each of them asks
git and refuses through the one error here, so a refusal reads the same
whichever module met it (§FS-distribution.4.6.3).
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path
from typing import Sequence


LINK_RE = re.compile(r"(?P<prefix>\]\()(?P<destination>[^)#][^)#]*)(?P<fragment>#[^)]*)?\)")


class ChangelogError(Exception):
    """A refusal: the release names its case and writes nothing (§FS-distribution.4.6.3)."""


def git(directory: Path | str, *arguments: str, nothing_found: int | None = None) -> str:
    """git's stdout, run in `directory` and decoded as UTF-8 whatever the locale (§AR-ci.1.1).

    `nothing_found` is an exit code that means an empty answer rather than a failure.
    """
    result = subprocess.run(
        ["git", "-C", str(directory), *arguments],
        check=False,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode == nothing_found:
        return ""
    if result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        raise ChangelogError(f"git {arguments[0]} failed: {detail[-1] if detail else 'no output'}")
    return result.stdout


def rebase_links(line: str, source: Sequence[str], target: Sequence[str]) -> str:
    """Each relative link written in `source` rewritten to mean the same from `target`. §FS-distribution.4.5

    Both are directories as components under `docs/`: a decision record is
    written in `decisions/<kind>`, the inline release is read in `docs/` itself,
    and an archive in `changelog`. An anchor, an absolute path, and a URL mean the
    same thing anywhere; every *relative* destination is resolved against where it
    was written and re-expressed from where it lands, including one that already
    climbs. `../crates/x.rs` written in `docs/` needs a second `../` from
    `docs/changelog/`; skipping it was how `docs/changelog/0.9.1.md` shipped a link
    that resolved to `docs/crates/...` and turned the tree's own link check red.
    """

    def rewrite(match: re.Match[str]) -> str:
        destination = match.group("destination")
        if destination.startswith(("#", "/", "http://", "https://", "mailto:")):
            return match.group(0)
        fragment = match.group("fragment") or ""
        return f"{match.group('prefix')}{_rebased(destination, source, target)}{fragment})"

    return LINK_RE.sub(rewrite, line)


def _rebased(destination: str, source: Sequence[str], target: Sequence[str]) -> str:
    parts = list(source)
    for part in destination.split("/"):
        if part in ("", "."):
            continue
        if part == ".." and parts and parts[-1] != "..":
            parts.pop()
        else:
            parts.append(part)
    shared = 0
    while shared < min(len(parts), len(target)) and parts[shared] == target[shared]:
        shared += 1
    rebased = "/".join([".."] * (len(target) - shared) + parts[shared:]) or "."
    return f"{rebased}/" if destination.endswith("/") else rebased
