#!/usr/bin/env python3
"""What a changelog entry is, where `## Unreleased` is, and what a bullet there is.

The release helper (`prepare_changelog_release.py`) and the module it reads the
entries through (`changelog_entries.py`) both ask what a file under
`docs/changelog/unreleased/` is (§FS-distribution.4.12), and the release asks
`## Unreleased` whether a bullet sits under its pointer (§FS-distribution.4.5).
Two readers answering one question two ways is the defect these answers exist
to close, so they live here once and every reader imports them rather than
carrying a copy (§AR-ci.7).
"""

from __future__ import annotations

import re
from typing import NamedTuple, Sequence


# Where the entries live, beside `docs/changelog.md`, and the one file there that is not one.
ENTRY_DIRECTORY = "changelog/unreleased"
ENTRY_README = "README.md"
# This repository's categories, in the order a release prints them (§FS-distribution.4.12).
CATEGORIES = ("added", "changed", "deprecated", "removed", "fixed", "security")
ENTRY_NAME_RE = re.compile(r"^(?P<slug>[a-z0-9-]+)(?:\.(?P<category>[a-z]+))?\.md$")
ENTRY_BULLET_RE = re.compile(r"^-[ \t]+\S")

UNRELEASED_RE = re.compile(r"^## Unreleased\s*$")
TOP_LEVEL_RE = re.compile(r"^##(?!#)\s+")
HEADING_RE = re.compile(r"^\s{0,3}#{1,6}\s")
BULLET_RE = re.compile(r"^(?P<indent>\s*)-\s+\S")


class ChangelogFormatError(Exception):
    """The changelog does not have the shape §FS-distribution.4 describes."""


class EntryName(NamedTuple):
    """An entry's file name read as the format reads it. §FS-distribution.4.12"""

    slug: str
    category: str | None


class Bullet(NamedTuple):
    """A `- ` line together with its continuation lines, and where they sit.

    `start` and `end` are 1-based inclusive file line numbers, so a refusal can
    say where the bullet it names is.
    """

    lines: tuple[str, ...]
    start: int
    end: int


def entry_name(name: str) -> EntryName | None:
    """`<slug>.<category>.md` or `<slug>.md`, else `None`. §FS-distribution.4.12

    The format takes any lowercase word as a category, and a name without one,
    so that a changelog without sections can use it; which categories this
    repository takes is `entry_problem`'s question, not this one's.
    """
    match = ENTRY_NAME_RE.match(name)
    if match is None:
        return None
    return EntryName(match.group("slug"), match.group("category"))


def entry_problem(name: str, text: str) -> str | None:
    """What keeps `name` holding `text` from being an entry here, or `None`. §FS-distribution.4.12

    The answer opens with the file name, so a caller prefixes the directory and
    has the whole refusal.
    """
    parsed = entry_name(name)
    categories = ", ".join(CATEGORIES)
    if parsed is None:
        return (
            f"{name}: not an entry; an entry is named `<slug>.<category>.md`, "
            "its slug lowercase letters, digits and hyphens"
        )
    if parsed.category is None:
        return f"{name}: no category; name it `{parsed.slug}.<category>.md`, <category> one of: {categories}"
    if parsed.category not in CATEGORIES:
        return f"{name}: the category `{parsed.category}` is not one of: {categories}"
    shape = _bullet_shape_problem(text)
    return None if shape is None else f"{name}: not one bullet; {shape}"


def entry_lines(text: str) -> list[str]:
    """An entry's lines as the release prints them: line endings and trailing blank lines dropped."""
    lines = [line.rstrip("\r") for line in text.split("\n")]
    while lines and not lines[-1].strip():
        lines.pop()
    return lines


def _bullet_shape_problem(text: str) -> str | None:
    """One `- ` line, then only lines indented under it, trailing blank lines aside."""
    lines = entry_lines(text)
    if not lines:
        return "it is empty"
    if ENTRY_BULLET_RE.match(lines[0]) is None:
        return "its first line does not open a bullet with `- `"
    for number, line in enumerate(lines[1:], start=2):
        if not line.strip():
            return f"line {number} is blank, which splits it"
        if not line[0].isspace():
            return f"line {number} is not indented under the bullet"
    return None


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
    """Every bullet under `## Unreleased`, in file order. §FS-distribution.4.5

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
