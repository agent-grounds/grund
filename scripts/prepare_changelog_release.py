#!/usr/bin/env python3
"""Prepare and read changelog release sections. §FS-distribution.4

The pending changes are the entries under `docs/changelog/unreleased/`
(§FS-distribution.4.12): `stamp` numbers them, `preview` shows the section they
make, `prepare` writes it and deletes them (§FS-distribution.4.5), and `notes`
reads a released section back (§FS-distribution.4.7).
"""

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
import changelog_entries  # noqa: E402  (the release's reading of the entries, split out for size)


VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
RELEASE_RE = re.compile(
    r"^## (?P<number>[0-9]+)\. \[(?P<version>[0-9]+\.[0-9]+\.[0-9]+)\] — (?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2})\s*$"
)
OLDER_RE = re.compile(r"^## (?P<number>[0-9]+)\. Older releases\s*$")
# The number an entry ends in, a placeholder included; only the end is ever read or written.
ENDING_NUMBER_RE = re.compile(r"\(\s*PR\s*#\s*(?P<number>[0-9]+|TBD)\s*\)\s*\Z", re.IGNORECASE)
# One error for the helper and the module it reads the entries through.
ChangelogError = changelog_entries.ChangelogError


def prepare_release(changelog: Path, version: str, release_date: str) -> None:
    """The rotation: the entries become the inline release and are deleted. §FS-distribution.4.5

    Everything that can refuse is asked before anything is written, so a refused
    release leaves the tree as it found it.
    """
    _validate_version(version)
    _validate_date(release_date)

    lines = _read_lines(changelog)
    unreleased, latest = _unreleased_and_latest(lines, changelog)
    sections = _find_top_level_sections(lines)
    older = _find_section_after(lines, sections, latest, OLDER_RE, "Older releases")

    latest_match = RELEASE_RE.match(_line_text(lines[latest]))
    if latest_match is None:
        raise ChangelogError(f"expected latest release heading after ## Unreleased, got: {_line_text(lines[latest])}")

    if latest_match.group("version") == version:
        raise ChangelogError(f"docs/changelog.md already has {version} as the inline latest release")

    entries = changelog_entries.collect(changelog)
    pointer = _trim_blank_lines(lines[unreleased + 1 : latest])

    previous_version = latest_match.group("version")
    previous_date = latest_match.group("date")
    previous_body = lines[latest + 1 : older]
    archive_parts = changelog_entries.ARCHIVE_PARTS
    archived_body = [changelog_entries.rebase_links(line, (), archive_parts) for line in previous_body]
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
        *([*pointer, "\n"] if pointer else []),
        f"## 2. [{version}] — {release_date}\n",
        "\n",
        *changelog_entries.release_body(entries),
        "\n",
        "## 3. Older releases\n",
        "\n",
        archive_link,
        *older_body,
    ]
    _write_lines(changelog, new_lines)
    for entry in entries:
        entry.path.unlink()


def preview_release(changelog: Path) -> str:
    """The body `prepare` would write under the new heading; nothing is written. §FS-distribution.4.5"""
    _unreleased_and_latest(_read_lines(changelog), changelog)
    return "".join(changelog_entries.release_body(changelog_entries.collect(changelog)))


def _unreleased_and_latest(lines: Sequence[str], changelog: Path) -> tuple[int, int]:
    """The `## Unreleased` heading and the latest release's, with nothing but the pointer between.

    A bullet under the pointer is refused rather than dropped: the rotation
    writes the entries and keeps the pointer, so a bullet left there out of the
    old habit would be neither released nor kept (§FS-distribution.4.5).
    """
    try:
        # Where the section is, is the shared module's answer and not a second one.
        start, end = changelog_bullets.unreleased_range(lines)
        stray = changelog_bullets.bullets(lines)
    except changelog_bullets.ChangelogFormatError as exc:
        raise ChangelogError(str(exc)) from exc
    if stray:
        directory = changelog_entries.entry_directory(changelog).as_posix()
        raise ChangelogError(
            f"## Unreleased holds a bullet under its pointer, at {changelog.as_posix()}:{stray[0].start}; "
            f"a pending change is an entry under {directory}/, so move it into one rather than have "
            f"the release drop it:\n  {stray[0].lines[0]}"
        )
    if end >= len(lines):
        raise ChangelogError("missing latest release section")
    return start - 1, end


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


def stamp_release_numbers(changelog: Path, resolve: Callable[[str], set[int]] | None = None) -> None:
    """Write each entry's own pull request number at its end. §FS-distribution.4.5

    The pull request is the one whose commit added the entry
    (`changelog_entries.landings`), so
    whoever edited it since, it keeps the number of the pull request that wrote
    it. Where exactly one resolves, a trailing `(PR #TBD)` is replaced and
    otherwise ` (PR #N)` is appended; a `PR #TBD` in the entry's prose is left as
    it stands — writing into it is how the 0.15.0 cut turned a sentence that told
    authors to write `PR #TBD` into one naming `PR #346`. A number that would go
    into more than one entry goes into none of them, whichever commits added
    them: the guard is per number. A write-up (§FS-distribution.4.6) is its usual
    case, since its unnumbered entries all resolve to the write-up's own number;
    a pull request that brings two unnumbered entries of its own loses its number
    from both. Anything else is warned about once and left, and nothing here
    ever raises or touches `docs/changelog.md`: a release in which nothing
    resolves writes nothing.
    """
    resolve = resolve or pull_requests_for_commit
    directory = changelog_entries.entry_directory(changelog)
    names = sorted(path.name for path in directory.iterdir() if path.is_file()) if directory.is_dir() else []
    try:
        landed, unlanded = changelog_entries.landings(directory), "no commit has added it yet"
    except ChangelogError as exc:
        landed, unlanded = {}, str(exc)
    # Each number and the entries it would go into, gathered before any is written.
    targets: dict[int, list[tuple[Path, str]]] = {}
    for name in names:
        parsed = changelog_bullets.entry_name(name)
        if parsed is None:
            continue  # The README, or a file `prepare` refuses by name.
        path = directory / name
        text = path.read_bytes().decode("utf-8")
        ending = ENDING_NUMBER_RE.search(text)
        if ending is not None and ending.group("number").upper() != "TBD":
            continue  # Already numbered, by an earlier run or by its author.
        if changelog_bullets.entry_problem(name, text) is not None:
            _unstamped(name, "it is not a well-formed entry")
        elif parsed.slug not in landed:
            _unstamped(name, unlanded)
        else:
            number = _number_for(name, landed[parsed.slug].commit, resolve)
            if number is not None:
                targets.setdefault(number, []).append((path, text))
    for number, entries in targets.items():
        if len(entries) == 1:
            path, text = entries[0]
            path.write_bytes(_numbered(text, number).encode("utf-8"))
            continue
        # §FS-distribution.4.5: a number several entries would share is withheld from all of them.
        shared = f"PR #{number} would go into {len(entries)} entries; write each its own (PR #N)"
        for path, _ in entries:
            _unstamped(path.name, shared)


def _number_for(name: str, commit: str, resolve: Callable[[str], set[int]]) -> int | None:
    try:
        numbers = resolve(commit)
    except ChangelogError as exc:
        return _unstamped(name, str(exc))
    if len(numbers) != 1:
        return _unstamped(name, f"{len(numbers)} pull requests resolve for {commit[:12]}, which added it")
    return next(iter(numbers))


def _unstamped(name: str, reason: str) -> None:
    print(f"warning: leaving {name} unstamped, {reason}", file=sys.stderr)
    return None


def _numbered(text: str, number: int) -> str:
    """`text` ending in `(PR #number)`, in place of a trailing placeholder if it has one."""
    body = text.rstrip()
    ending = ENDING_NUMBER_RE.search(body)
    kept = body[: ending.start()].rstrip() if ending is not None else body
    return f"{kept} (PR #{number}){text[len(body):]}"


def pull_requests_for_commit(commit: str) -> set[int]:
    """The pull requests GitHub says a commit landed in. §FS-distribution.4.5"""
    result = subprocess.run(
        ["gh", "api", f"/repos/{{owner}}/{{repo}}/commits/{commit}/pulls", "--jq", ".[].number"],
        check=False,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
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

    prepare = subparsers.add_parser("prepare", help="release the entries as a numbered release")
    prepare.add_argument("version")
    prepare.add_argument("--date", default=_datetime.date.today().isoformat())

    subparsers.add_parser("stamp", help="write each entry's own pull request number at its end")
    subparsers.add_parser("preview", help="print the release body prepare would write, and write nothing")

    notes = subparsers.add_parser("notes", help="write release notes for the inline release")
    notes.add_argument("version")
    notes.add_argument("--output", type=Path, required=True)

    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            prepare_release(args.changelog, args.version, args.date)
        elif args.command == "stamp":
            stamp_release_numbers(args.changelog)
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
