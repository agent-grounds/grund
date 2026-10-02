"""The pending entries as a release reads them. §FS-distribution.4.5

Which commit added each entry, the order the release prints them in, and their
links rebased for where they land (§FS-distribution.4.12). What an entry *is*
is `changelog_bullets`'s answer, which this module asks rather than repeats; the
release helper, `prepare_changelog_release.py`, is the one that writes.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple, Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))

import changelog_bullets  # noqa: E402  (the shared definitions live beside this script)


LINK_RE = re.compile(r"(?P<prefix>\]\()(?P<destination>[^)#][^)#]*)(?P<fragment>#[^)]*)?\)")
# Where entries are written and where an archive is, as directories under `docs/`.
ENTRY_PARTS = tuple(changelog_bullets.ENTRY_DIRECTORY.split("/"))
ARCHIVE_PARTS = ("changelog",)


class ChangelogError(Exception):
    pass


class Entry(NamedTuple):
    """An entry as the release reads it: its file, its section, and its bullet's lines."""

    path: Path
    slug: str
    category: str
    lines: tuple[str, ...]


class Landing(NamedTuple):
    """The commit that added an entry, and how far down the history it sits (0 is the oldest)."""

    position: int
    commit: str


def entry_directory(changelog: Path) -> Path:
    return changelog.parent / changelog_bullets.ENTRY_DIRECTORY


def collect(changelog: Path) -> list[Entry]:
    """The entries, in the order the release prints them. §FS-distribution.4.5

    By category in the order §FS-distribution.4.12 gives, then the oldest-landed
    first, then by file name — for entries that landed in one commit, and after
    every landed one for an entry no commit has added yet.
    """
    directory = entry_directory(changelog)
    found: list[Entry] = []
    for path in sorted(directory.iterdir()) if directory.is_dir() else []:
        if path.name == changelog_bullets.ENTRY_README:
            continue
        text = path.read_bytes().decode("utf-8") if path.is_file() else ""
        problem = changelog_bullets.entry_problem(path.name, text)
        if problem is not None:
            raise ChangelogError(f"{directory.as_posix()}/{problem}")
        parsed = changelog_bullets.entry_name(path.name)
        lines = tuple(changelog_bullets.entry_lines(text))
        found.append(Entry(path, parsed.slug, parsed.category, lines))
    if not found:
        raise ChangelogError(f"{directory.as_posix()}/ holds no entry to release")

    try:
        landed = landings(directory)
    except ChangelogError as exc:
        print(f"warning: ordering the entries by file name alone, {exc}", file=sys.stderr)
        landed = {}

    def order(entry: Entry) -> tuple[int, bool, int, str]:
        landing = landed.get(entry.slug)
        return (
            changelog_bullets.CATEGORIES.index(entry.category),
            landing is None,
            landing.position if landing else 0,
            entry.path.name,
        )

    return sorted(found, key=order)


def release_body(entries: Sequence[Entry]) -> list[str]:
    """One section per category, links rebased from the entry directory to `docs/`. §FS-distribution.4.5"""
    body: list[str] = []
    for category in changelog_bullets.CATEGORIES:
        members = [entry for entry in entries if entry.category == category]
        if not members:
            continue
        if body:
            body.append("\n")
        body += [f"### {category.capitalize()}\n", "\n"]
        body += [rebase_links(line, ENTRY_PARTS, ()) + "\n" for entry in members for line in entry.lines]
    return body


def landings(directory: Path) -> dict[str, Landing]:
    """For each slug the head holds, the commit at which it last appeared. §FS-distribution.4.5

    The directory's first-parent history is walked newest first, undoing each
    commit's additions and deletions, and a slug lands at the newest commit before
    which no file of that slug existed. So an edit leaves an entry where it
    landed, and so does a change of category even when made in the same commit as
    an edit, where git sees no rename; and a slug used again after a release lands
    where it was used again. History rather than blame, because an edit by another
    pull request splits a blame.
    """
    top = _git(directory, "rev-parse", "--show-toplevel").strip()
    prefix = _git(directory, "rev-parse", "--show-prefix").strip()
    held: dict[str, set[str]] = {}
    for path in filter(None, _git(top, "ls-tree", "-z", "--name-only", "HEAD", "--", prefix).split("\0")):
        parsed = changelog_bullets.entry_name(path[len(prefix) :])
        if parsed is not None:
            held.setdefault(parsed.slug, set()).add(path)

    walk = ("--first-parent", "-m", "--no-renames", "--name-status", "--format=@%H", "HEAD", "--", prefix)
    log = _git(top, "log", *walk)
    commits: list[tuple[str, list[tuple[str, str]]]] = []
    for line in log.splitlines():
        if line.startswith("@"):
            commits.append((line[1:], []))
        elif "\t" in line and commits:
            status, _, path = line.partition("\t")
            commits[-1][1].append((status[:1], path))

    landed: dict[str, Landing] = {}
    for newest_first, (commit, changes) in enumerate(commits):
        appeared = set()
        for status, path in changes:
            parsed = changelog_bullets.entry_name(path[len(prefix) :]) if path.startswith(prefix) else None
            if parsed is None or parsed.slug in landed or parsed.slug not in held:
                continue
            if status == "A":
                held[parsed.slug].discard(path)
                appeared.add(parsed.slug)
            elif status == "D":
                held[parsed.slug].add(path)
        for slug in appeared:
            if not held[slug]:
                landed[slug] = Landing(len(commits) - 1 - newest_first, commit)
    return landed


def _git(directory: Path | str, *arguments: str) -> str:
    """git's stdout, run in `directory` and decoded as UTF-8 whatever the locale (§AR-ci.1.1)."""
    result = subprocess.run(
        ["git", "-C", str(directory), *arguments],
        check=False,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        raise ChangelogError(f"git {arguments[0]} failed: {detail[-1] if detail else 'no output'}")
    return result.stdout


def rebase_links(line: str, source: Sequence[str], target: Sequence[str]) -> str:
    """Each relative link written in `source` rewritten to mean the same from `target`. §FS-distribution.4.5

    Both are directories as components under `docs/`: an entry is written in
    `changelog/unreleased`, the inline release is read in `docs/` itself, and an
    archive in `changelog`. An anchor, an absolute path, and a URL mean the same
    thing anywhere; every *relative* destination is resolved against where it was
    written and re-expressed from where it lands, including one that already
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
