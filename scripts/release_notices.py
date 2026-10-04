#!/usr/bin/env python3
"""The compatibility notices a release publishes, read from the decisions. §FS-distribution.4.6.4

A decision record may carry `## release-note: Release note`, one bullet saying
what a verdict change breaks and for whom. The release publishes each one the
decisions gained since the previous tag, matched by decision ID, and refuses one
that is not a single well-formed bullet (§FS-distribution.4.6.3). This module
only reads; `prepare_changelog_release.py` writes.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path, PurePosixPath
from typing import NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent))

from release_common import ChangelogError, git, rebase_links  # noqa: E402  (the shared definitions live beside this script)


NOTICE_HEADING_RE = re.compile(r"^## release-note:[^\n]*$", re.MULTILINE)
DECLARATION_RE = re.compile(r"^# (?P<id>(?:DF|DA)-[a-z0-9-]+):", re.MULTILINE)
HEADING_RE = re.compile(r"^#{1,6} ", re.MULTILINE)
ANCHOR_LINK_RE = re.compile(r"\]\((?P<fragment>#[^)]*)\)")


class Notice(NamedTuple):
    """One decision's `release-note` section as the record holds it."""

    decision: str
    path: str
    text: str


def published_notices(top: Path, docs: Path, head: str, tag: str) -> list[str]:
    """The `### Compatibility notices` bullets' lines, ordered by decision ID. §FS-distribution.4.6.4

    A notice is published when its decision has one at `head` and had none at
    `tag`, so a record moved since the tag is the same record, and a notice
    edited after it shipped is not published again. Only what would be published
    is held to the bullet's shape.
    """
    prefix = docs.resolve().relative_to(top.resolve()).as_posix()
    decisions = f"{prefix}/decisions" if prefix != "." else "decisions"
    shipped = {notice.decision for notice in _notices(top, tag, decisions)}
    added = sorted(
        (notice for notice in _notices(top, head, decisions) if notice.decision not in shipped),
        key=lambda notice: notice.decision,
    )
    docs_relative = prefix if prefix != "." else ""
    return [line for notice in added for line in _published(notice, docs_relative)]


def _notices(top: Path, revision: str, directory: str) -> list[Notice]:
    """Every decision under `directory` at `revision` whose record has a `release-note` section."""
    # `git grep` exits 1 when nothing matches, which is a tree with no notice.
    listed = git(top, "grep", "-l", "-E", "^## release-note:", revision, "--", directory, nothing_found=1)
    found = []
    for entry in listed.splitlines():
        path = entry.split(":", 1)[1] if entry.startswith(f"{revision}:") else entry
        if not path.endswith(".md"):
            continue
        text = git(top, "show", f"{revision}:{path}")
        notice = _notice(path, text)
        if notice is not None:
            found.append(notice)
    return found


def _notice(path: str, text: str) -> Notice | None:
    """The section and the decision it belongs to: the declaration above it."""
    heading = NOTICE_HEADING_RE.search(text)
    if heading is None:
        return None
    declarations = [match for match in DECLARATION_RE.finditer(text) if match.start() < heading.start()]
    if not declarations:
        return None
    following = HEADING_RE.search(text, heading.end())
    body = text[heading.end() : following.start() if following else len(text)]
    return Notice(declarations[-1].group("id"), path, body)


def _published(notice: Notice, docs_relative: str) -> list[str]:
    """The notice's bullet, its links rebased from the record's directory to `docs/`. §FS-distribution.4.6.4"""
    lines = _bullet(notice)
    record = PurePosixPath(notice.path)
    within = record.relative_to(docs_relative) if docs_relative else record
    source = within.parent.parts
    anchored = within.as_posix()
    published = []
    for line in lines:
        line = ANCHOR_LINK_RE.sub(lambda match: f"]({anchored}{match.group('fragment')})", line)
        published.append(rebase_links(line, source, ()) + "\n")
    return published


def _bullet(notice: Notice) -> list[str]:
    """The section as one bullet, continuation lines indented two spaces, or a refusal naming the decision."""
    lines = [line.rstrip("\r") for line in notice.text.split("\n")]
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    problem = None
    if not lines:
        problem = "it is empty"
    elif not lines[0].startswith("- ") or not lines[0][2:].strip():
        problem = "its first line does not open a bullet with `- `"
    else:
        for number, line in enumerate(lines[1:], start=2):
            if not line.strip():
                problem = f"its line {number} is blank, which splits it"
            elif not line.startswith("  "):
                problem = f"its line {number} is not a continuation indented two spaces under the bullet"
            if problem:
                break
    if problem is not None:
        raise ChangelogError(
            f"the release-note section of {notice.decision} ({notice.path}) is not one well-formed bullet: {problem}"
        )
    return lines
