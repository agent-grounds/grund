#!/usr/bin/env python3
"""Where `## Unreleased` is, what a bullet is, and when two bullets are the same.

The changelog gate (`check_changelog_pr_entry.py`) and the release stamper
(`prepare_changelog_release.py stamp`) both ask these three questions of
`docs/changelog.md` (§FS-distribution.4.6). The defect they exist to close was
two halves of one gate answering one question two ways, so the answers live here
once and both scripts import them rather than carrying a copy (§AR-ci.7).
"""

from __future__ import annotations

import re
from typing import NamedTuple, Sequence


UNRELEASED_RE = re.compile(r"^## Unreleased\s*$")
TOP_LEVEL_RE = re.compile(r"^##(?!#)\s+")
HEADING_RE = re.compile(r"^\s{0,3}#{1,6}\s")
BULLET_RE = re.compile(r"^(?P<indent>\s*)-\s+\S")
LINK_DESTINATION_RE = re.compile(r"\]\([^)]*\)")
TRAILING_PR_RE = re.compile(r"\s*\(?\bPR\s*#\s*(?:[0-9]+|TBD)\)?\.?$", re.IGNORECASE)
PR_NUMBER_RES = (
    re.compile(r"(?i)\bPR\s*#\s*([0-9]+)\b"),
    re.compile(r"(?i)\bpull request\s*#\s*([0-9]+)\b"),
    re.compile(r"/pull/([0-9]+)(?:\b|[/#?)])"),
)


class ChangelogFormatError(Exception):
    """The changelog does not have the shape §FS-distribution.4 describes."""


class Bullet(NamedTuple):
    """A `- ` line together with its continuation lines, and where they sit.

    `start` and `end` are 1-based inclusive file line numbers, which is what
    `git blame -L` takes: the stamper blames exactly this range to find the pull
    request that wrote the bullet (§FS-distribution.4.5).
    """

    lines: tuple[str, ...]
    start: int
    end: int

    @property
    def text(self) -> str:
        return "\n".join(line.rstrip("\r\n") for line in self.lines)

    @property
    def key(self) -> str:
        """What makes two bullets the same bullet. §FS-distribution.4.6"""
        return normalise(self.text)

    @property
    def numbers(self) -> frozenset[int]:
        """Every pull request number this bullet names. `PR #TBD` is not one."""
        return pr_numbers(self.text)


def normalise(text: str) -> str:
    """Collapse a bullet to what a rewrap and a formatter cannot change.

    Whitespace collapses, a trailing `PR #N` or `PR #TBD` is dropped, and a
    link's destination is disregarded (§FS-distribution.4.6) — so rewrapping a
    bullet is not a change, rewording one is, and the anchor `grund fmt --write`
    rewrote inside somebody else's bullet is not this branch's edit.
    """
    without_destinations = LINK_DESTINATION_RE.sub("]()", text)
    collapsed = re.sub(r"\s+", " ", without_destinations).strip()
    while True:
        # Every trailing number, not only the last one: appending yours to
        # somebody else's bullet leaves two, and is still not your change.
        shorter = TRAILING_PR_RE.sub("", collapsed).strip()
        if shorter == collapsed:
            return collapsed
        collapsed = shorter


def pr_numbers(text: str) -> frozenset[int]:
    """The pull request numbers named in `text`, in any of the three forms."""
    found: set[int] = set()
    for pattern in PR_NUMBER_RES:
        found.update(int(match) for match in pattern.findall(text))
    return frozenset(found)


def unreleased_range(lines: Sequence[str]) -> tuple[int, int]:
    """The half-open 0-based range of the `## Unreleased` body."""
    start = None
    for index, line in enumerate(lines):
        if UNRELEASED_RE.match(_text(line)):
            start = index + 1
            break
    if start is None:
        raise ChangelogFormatError("missing ## Unreleased section in docs/changelog.md")

    for index in range(start, len(lines)):
        if TOP_LEVEL_RE.match(_text(lines[index])):
            return start, index
    return start, len(lines)


def unreleased_body(lines: Sequence[str]) -> list[str]:
    start, end = unreleased_range(lines)
    return [_text(line) for line in lines[start:end]]


def bullets(lines: Sequence[str]) -> list[Bullet]:
    """Every bullet under `## Unreleased`, in file order. §FS-distribution.4.6

    A bullet runs from its `- ` line to the next `- ` at the same indent or
    less, the next heading, or the end of the section, with trailing blank lines
    left out of it.
    """
    start, end = unreleased_range(lines)
    found: list[Bullet] = []
    index = start
    while index < end:
        opening = BULLET_RE.match(_text(lines[index]))
        if opening is None:
            index += 1
            continue
        indent = len(opening.group("indent"))
        stop = index + 1
        while stop < end:
            text = _text(lines[stop])
            if HEADING_RE.match(text):
                break
            following = BULLET_RE.match(text)
            if following is not None and len(following.group("indent")) <= indent:
                break
            stop += 1
        last = stop
        while last - 1 > index and not _text(lines[last - 1]).strip():
            last -= 1
        found.append(
            Bullet(lines=tuple(_text(line) for line in lines[index:last]), start=index + 1, end=last)
        )
        index = stop
    return found


def has_bullet(lines: Sequence[str]) -> bool:
    """Whether the given lines hold any bullet at all."""
    return any(_text(line).lstrip().startswith("- ") for line in lines)


def _text(line: str) -> str:
    return line.rstrip("\r\n")
