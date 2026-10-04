#!/usr/bin/env python3
"""Prepare and read changelog release sections. §FS-distribution.4

Nothing waits in the tree for a release: `prepare` builds the section from the
pull requests merged since the previous tag (§FS-distribution.4.6.1) and the
compatibility notices the decisions gained since then (§FS-distribution.4.6.4),
and rotates it in (§FS-distribution.4.5); `preview` prints the same body and
writes nothing; `notes` reads a released section back (§FS-distribution.4.7).
"""

from __future__ import annotations

import argparse
import datetime as _datetime
import re
import sys
from pathlib import Path
from typing import NamedTuple, Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))

import release_notices  # noqa: E402  (the notices, split out for size)
import release_pull_requests  # noqa: E402  (the forge's list, split out for size)
from release_common import ChangelogError, rebase_links  # noqa: E402  (the shared definitions live beside this script)


VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
RELEASE_RE = re.compile(
    r"^## (?P<number>[0-9]+)\. \[(?P<version>[0-9]+\.[0-9]+\.[0-9]+)\] — (?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2})\s*$"
)
OLDER_RE = re.compile(r"^## (?P<number>[0-9]+)\. Older releases\s*$")
NOTICES_HEADING = "### Compatibility notices"
# Where an archive is, as directories under `docs/`.
ARCHIVE_PARTS = ("changelog",)


class Inline(NamedTuple):
    """The release `docs/changelog.md` keeps inline, and the older-release section after it."""

    lines: list[str]
    latest: int
    older: int
    number: str
    version: str
    date: str


def prepare_release(changelog: Path, version: str, release_date: str) -> None:
    """The rotation: the built section becomes the inline release. §FS-distribution.4.5

    Everything that can refuse is asked before anything is written, so a refused
    release leaves the tree as it found it (§FS-distribution.4.6.3).
    """
    _validate_version(version)
    _validate_date(release_date)

    inline = _inline_release(changelog)
    if inline.version == version:
        raise ChangelogError(f"docs/changelog.md already has {version} as the inline latest release")

    archive_path = changelog.parent / "changelog" / f"{inline.version}.md"
    if archive_path.exists():
        raise ChangelogError(f"archive already exists: {archive_path}")

    body = release_body(changelog, inline)
    lines = inline.lines
    previous_body = lines[inline.latest + 1 : inline.older]
    archived_body = [rebase_links(line, (), ARCHIVE_PARTS) for line in previous_body]
    archive_lines = [f"# {inline.version} — {inline.date}\n", *archived_body]
    archive_link = (
        f"- [{inline.version}](changelog/{inline.version}.md) — {inline.date}: {_summary_from(previous_body)}\n"
    )
    new_lines = [
        *lines[: inline.latest],
        f"## {inline.number}. [{version}] — {release_date}\n",
        "\n",
        *body,
        "\n",
        lines[inline.older],
        "\n",
        archive_link,
        *_drop_leading_blank_lines(lines[inline.older + 1 :]),
    ]
    _write_lines(archive_path, archive_lines)
    _write_lines(changelog, new_lines)


def preview_release(changelog: Path) -> str:
    """The body `prepare` would write under the new heading; nothing is written. §FS-distribution.4.5"""
    return "".join(release_body(changelog, _inline_release(changelog)))


def release_body(changelog: Path, inline: Inline) -> list[str]:
    """The list, newest first, and its compatibility notices after it. §FS-distribution.4.6

    The range is read and every commit in it answered for before the notices are
    read, and both before the caller writes anything (§FS-distribution.4.6.3).
    """
    span = release_pull_requests.release_range(changelog.parent, inline.version)
    body = release_pull_requests.pull_request_lines(span)
    notices = release_notices.published_notices(span.top, changelog.parent, span.head, span.tag)
    if notices:
        body += ["\n", f"{NOTICES_HEADING}\n", "\n", *notices]
    return body


def _inline_release(changelog: Path) -> Inline:
    """The first `## N. [X.Y.Z] — date` heading and the `Older releases` section after it."""
    lines = _read_lines(changelog)
    sections = _find_top_level_sections(lines)
    for latest in sections:
        match = RELEASE_RE.match(_line_text(lines[latest]))
        if match is not None:
            older = _find_section_after(lines, sections, latest, OLDER_RE, "Older releases")
            return Inline(lines, latest, older, match.group("number"), match.group("version"), match.group("date"))
    raise ChangelogError("docs/changelog.md has no inline latest release section `## N. [X.Y.Z] — date`")


def extract_notes(changelog: Path, version: str, output: Path) -> None:
    _validate_version(version)
    lines = _read_lines(changelog)
    sections = _find_top_level_sections(lines)

    for index, section_start in enumerate(sections):
        match = RELEASE_RE.match(_line_text(lines[section_start]))
        if match is None or match.group("version") != version:
            continue
        section_end = sections[index + 1] if index + 1 < len(sections) else len(lines)
        body = _trim_blank_lines(lines[section_start + 1 : section_end])
        if not body:
            raise ChangelogError(f"release {version} has an empty changelog section")
        _write_lines(output, [*body, "\n"])
        return

    raise ChangelogError(f"release {version} is not the inline changelog release")


def _summary_from(lines: Sequence[str]) -> str:
    """`N pull requests, M compatibility notices.`, counted from the archived section. §FS-distribution.4.5

    M is the bullets under `### Compatibility notices` and N every other bullet,
    so a release archived in the earlier Keep-a-Changelog shape counts each of
    its bullets as a pull request.
    """
    pulls = notices = 0
    in_notices = False
    for line in lines:
        text = _line_text(line)
        if text.startswith("#"):
            in_notices = text.strip() == NOTICES_HEADING
        elif text.startswith("- "):
            if in_notices:
                notices += 1
            else:
                pulls += 1
    return f"{_counted(pulls, 'pull request')}, {_counted(notices, 'compatibility notice')}."


def _counted(count: int, noun: str) -> str:
    return f"{count} {noun}" if count == 1 else f"{count} {noun}s"


def _find_top_level_sections(lines: Sequence[str]) -> list[int]:
    return [index for index, line in enumerate(lines) if line.startswith("## ") and not line.startswith("### ")]


def _find_section_after(
    lines: Sequence[str], sections: Sequence[int], after: int, pattern: re.Pattern[str], name: str
) -> int:
    for section in sections:
        if section <= after:
            continue
        if pattern.match(_line_text(lines[section])):
            return section
    raise ChangelogError(f"missing {name} section")


def _read_lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines(keepends=True)
    except FileNotFoundError as exc:
        raise ChangelogError(f"missing changelog: {path}") from exc


def _write_lines(path: Path, lines: Sequence[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(lines), encoding="utf-8")


def _line_text(line: str) -> str:
    return line.rstrip("\r\n")


def _trim_blank_lines(lines: Sequence[str]) -> list[str]:
    trimmed = list(lines)
    while trimmed and not trimmed[0].strip():
        trimmed.pop(0)
    while trimmed and not trimmed[-1].strip():
        trimmed.pop()
    if trimmed and not trimmed[-1].endswith(("\n", "\r")):
        trimmed[-1] += "\n"
    return trimmed


def _drop_leading_blank_lines(lines: Sequence[str]) -> list[str]:
    trimmed = list(lines)
    while trimmed and not trimmed[0].strip():
        trimmed.pop(0)
    return trimmed


def _validate_version(version: str) -> None:
    if VERSION_RE.match(version) is None:
        raise ChangelogError(f"version must look like 0.1.0, got {version!r}")


def _validate_date(release_date: str) -> None:
    try:
        _datetime.date.fromisoformat(release_date)
    except ValueError as exc:
        raise ChangelogError(f"date must look like YYYY-MM-DD, got {release_date!r}") from exc


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare or read docs/changelog.md release sections.")
    parser.add_argument("--changelog", type=Path, default=Path("docs/changelog.md"))
    subparsers = parser.add_subparsers(dest="command", required=True)

    prepare = subparsers.add_parser("prepare", help="release the pull requests merged since the previous tag")
    prepare.add_argument("version")
    prepare.add_argument("--date", default=_datetime.date.today().isoformat())

    subparsers.add_parser("preview", help="print the release body prepare would write, and write nothing")

    notes = subparsers.add_parser("notes", help="write release notes for the inline release")
    notes.add_argument("version")
    notes.add_argument("--output", type=Path, required=True)

    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            prepare_release(args.changelog, args.version, args.date)
        elif args.command == "preview":
            sys.stdout.write(preview_release(args.changelog))
        elif args.command == "notes":
            extract_notes(args.changelog, args.version, args.output)
        else:
            raise AssertionError(args.command)
    except ChangelogError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
