#!/usr/bin/env python3
"""Prepare and read changelog release sections. §FS-distribution.4"""

from __future__ import annotations

import argparse
import datetime as _datetime
import re
import subprocess
import sys
from pathlib import Path
from typing import Callable, Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))

import changelog_bullets  # noqa: E402  (the shared definitions live beside this script)


VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
RELEASE_RE = re.compile(
    r"^## (?P<number>[0-9]+)\. \[(?P<version>[0-9]+\.[0-9]+\.[0-9]+)\] — (?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2})\s*$"
)
OLDER_RE = re.compile(r"^## (?P<number>[0-9]+)\. Older releases\s*$")
PLACEHOLDER_RE = re.compile(r"(?i)\bPR\s*#\s*TBD\b")
BLAME_HEADER_RE = re.compile(r"^([0-9a-f]{40}) [0-9]+ [0-9]+")


class ChangelogError(Exception):
    pass


def prepare_release(changelog: Path, version: str, release_date: str) -> None:
    _validate_version(version)
    _validate_date(release_date)

    lines = _read_lines(changelog)
    sections = _find_top_level_sections(lines)
    try:
        # Where the section is, is the shared module's answer and not a second one.
        unreleased = changelog_bullets.unreleased_range(lines)[0] - 1
    except changelog_bullets.ChangelogFormatError as exc:
        raise ChangelogError(str(exc)) from exc
    latest = _next_section_after(sections, unreleased, "latest release")
    older = _find_section_after(lines, sections, latest, OLDER_RE, "Older releases")

    latest_match = RELEASE_RE.match(_line_text(lines[latest]))
    if latest_match is None:
        raise ChangelogError(f"expected latest release heading after ## Unreleased, got: {_line_text(lines[latest])}")

    if latest_match.group("version") == version:
        raise ChangelogError(f"docs/changelog.md already has {version} as the inline latest release")

    unreleased_body = _trim_blank_lines(lines[unreleased + 1 : latest])
    if not changelog_bullets.has_bullet(unreleased_body):
        raise ChangelogError("## Unreleased has no bullet entries to promote")

    previous_version = latest_match.group("version")
    previous_date = latest_match.group("date")
    previous_body = lines[latest + 1 : older]
    archived_body = [_rewrite_relative_links_for_archive(line) for line in previous_body]
    summary = _summary_from(previous_body)

    archive_path = changelog.parent / "changelog" / f"{previous_version}.md"
    if archive_path.exists():
        raise ChangelogError(f"archive already exists: {archive_path}")

    archive_lines = [f"# {previous_version} — {previous_date}\n", *archived_body]
    _write_lines(archive_path, archive_lines)

    older_body = lines[older + 1 :]
    older_body = _drop_leading_blank_lines(older_body)
    archive_link = f"- [{previous_version}](changelog/{previous_version}.md) — {previous_date}: {summary}\n"

    new_lines = [
        *lines[: unreleased + 1],
        "\n",
        f"## 2. [{version}] — {release_date}\n",
        "\n",
        *unreleased_body,
        "\n",
        "## 3. Older releases\n",
        "\n",
        archive_link,
        *older_body,
    ]
    _write_lines(changelog, new_lines)


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


def _next_section_after(sections: Sequence[int], after: int, name: str) -> int:
    for section in sections:
        if section > after:
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


def stamp_release_numbers(
    changelog: Path,
    resolve: Callable[[str], set[int]] | None = None,
    blame: Callable[[Path, int, int], list[str]] | None = None,
) -> None:
    """Write each `## Unreleased` bullet's own pull request number into it. §FS-distribution.4.5

    Every line of a bullet is blamed to the commit that wrote it and each commit
    to its pull request; a bullet whose lines all name one and the same pull
    request gains `PR #N`, replacing a `PR #TBD` placeholder in place where the
    author left one. Anything else is warned about once and left as it stands,
    and nothing here ever raises: a release in which nothing resolves is exactly
    today's release.
    """
    resolve = resolve or pull_requests_for_commit
    blame = blame or _blame_commits
    lines = _read_lines(changelog)
    for bullet in reversed(changelog_bullets.bullets(lines)):
        number = _number_for(changelog, bullet, resolve, blame)
        if number is not None:
            lines[bullet.end - 1] = _write_number(lines[bullet.end - 1], bullet, number)
    _write_lines(changelog, lines)


def _number_for(changelog, bullet, resolve, blame) -> int | None:
    if bullet.numbers:
        return None  # Already stamped, by an earlier run or by its author.
    try:
        commits = blame(changelog, bullet.start, bullet.end)
    except ChangelogError as exc:
        return _unstamped(bullet, str(exc))
    if not commits:
        return _unstamped(bullet, "no committed line to blame")

    numbers: set[int] = set()
    for commit in commits:
        try:
            numbers |= resolve(commit)
        except ChangelogError as exc:
            return _unstamped(bullet, str(exc))
    if len(numbers) != 1:
        return _unstamped(bullet, f"{len(numbers)} pull requests resolve for it")
    return next(iter(numbers))


def _unstamped(bullet: changelog_bullets.Bullet, reason: str) -> None:
    first = bullet.text.splitlines()[0]
    print(f"warning: leaving line {bullet.start} unstamped, {reason}: {first[:100]}", file=sys.stderr)
    return None


def _write_number(line: str, bullet: changelog_bullets.Bullet, number: int) -> str:
    body, ending = _split_ending(line)
    if PLACEHOLDER_RE.search(body):
        return PLACEHOLDER_RE.sub(f"PR #{number}", body, count=1) + ending
    return f"{body.rstrip()} (PR #{number}){ending}"


def _split_ending(line: str) -> tuple[str, str]:
    stripped = line.rstrip("\r\n")
    return stripped, line[len(stripped) :]


def _blame_commits(changelog: Path, start: int, end: int) -> list[str]:
    """The commits that wrote lines `start`..`end`, oldest line first."""
    path = changelog.resolve()
    result = subprocess.run(
        ["git", "-C", str(path.parent), "blame", "--line-porcelain", "-L", f"{start},{end}", "--", str(path)],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise ChangelogError(f"git blame failed: {result.stderr.strip()}")

    commits: list[str] = []
    for line in result.stdout.splitlines():
        match = BLAME_HEADER_RE.match(line)
        if match is None:
            continue
        commit = match.group(1)
        if set(commit) == {"0"}:
            raise ChangelogError("a line of it is not committed yet")
        if commit not in commits:
            commits.append(commit)
    return commits


def pull_requests_for_commit(commit: str) -> set[int]:
    """The pull requests GitHub says a commit landed in. §FS-distribution.4.5"""
    result = subprocess.run(
        ["gh", "api", f"/repos/{{owner}}/{{repo}}/commits/{commit}/pulls", "--jq", ".[].number"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        raise ChangelogError(f"gh could not resolve {commit[:12]}: {detail[-1] if detail else 'gh failed'}")
    return {int(token) for token in result.stdout.split()}


def _summary_from(lines: Sequence[str]) -> str:
    paragraph: list[str] = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            if paragraph:
                break
            continue
        if stripped.startswith("#"):
            continue
        paragraph.append(stripped)

    if not paragraph:
        return "release notes."

    text = re.sub(r"\s+", " ", " ".join(paragraph))
    first_sentence = re.match(r"(.+?[.!?])(?:\s|$)", text)
    if first_sentence is not None:
        return first_sentence.group(1)
    return text


def _rewrite_relative_links_for_archive(line: str) -> str:
    def rewrite(match: re.Match[str]) -> str:
        destination = match.group("destination")
        # An anchor, an absolute path, and a URL all mean the same thing one
        # directory deeper. Every *relative* destination means one directory
        # less, including one that already climbs: `../crates/x.rs` was written
        # against `docs/`, so from `docs/changelog/` it needs a second `../`.
        # Skipping it was how `docs/changelog/0.9.1.md` shipped a link that
        # resolved to `docs/crates/...` and turned the tree's own link check red.
        if destination.startswith(("#", "/", "http://", "https://", "mailto:")):
            return match.group(0)
        fragment = match.group("fragment") or ""
        return f"{match.group('prefix')}../{destination}{fragment})"

    return re.sub(
        r"(?P<prefix>\]\()(?P<destination>[^)#][^)#]*)(?P<fragment>#[^)]*)?\)",
        rewrite,
        line,
    )


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

    prepare = subparsers.add_parser("prepare", help="promote Unreleased into a numbered release")
    prepare.add_argument("version")
    prepare.add_argument("--date", default=_datetime.date.today().isoformat())

    subparsers.add_parser("stamp", help="write each Unreleased bullet's own pull request number into it")

    notes = subparsers.add_parser("notes", help="write release notes for the inline release")
    notes.add_argument("version")
    notes.add_argument("--output", type=Path, required=True)

    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            prepare_release(args.changelog, args.version, args.date)
        elif args.command == "stamp":
            stamp_release_numbers(args.changelog)
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
