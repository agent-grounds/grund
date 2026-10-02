"""What the changelog gate reads, read from local git alone. §AR-ci.7

Both halves of the gate list the entry directory and read the merge base's
changelog out of commits rather than out of the checkout, and neither calls the
GitHub API, so a fork, a restricted token and an owner branch reach one verdict.
`check_changelog_pr_entry.py` holds the rule (§FS-distribution.4.6); this is
everything it asks git, split out of it for size.
"""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import NamedTuple, Sequence


NO_BASE_REF = "no base ref resolves"
_TEXTS: dict[str, str] = {}  # One read per blob, whichever side of the comparison asks first.


class ChangelogPrError(Exception):
    pass


class EntryFile(NamedTuple):
    """One file of the entry directory as a commit holds it; its text is read on demand."""

    name: str
    object_id: str
    kind: str


def entries_at(commit: str, directory: str) -> dict[str, EntryFile]:
    """The entry directory as `commit` holds it, by file name. §FS-distribution.4.6

    Both sides of the comparison are listed here and not from the checkout, so
    the merge `actions/checkout` leaves in the tree on a `pull_request` event —
    the head merged with the *current* base tip — cannot lend this branch an
    entry the base gained after the branch point, and an uncommitted file is not
    one this push carries. `--full-tree` reads `directory` from the repository
    root wherever the script is run from.
    """
    listing = git_output(["ls-tree", "--full-tree", "-z", commit, "--", f"{directory}/"])
    if listing is None:
        raise ChangelogPrError(f"git cannot list {directory}/ at {commit[:12]}")
    files: dict[str, EntryFile] = {}
    for record in filter(None, listing.split("\0")):
        description, _, path = record.partition("\t")
        _mode, kind, object_id = description.split()
        name = path[len(directory) + 1 :]
        files[name] = EntryFile(name, object_id, kind)
    return files


def text(entry: EntryFile) -> str:
    """The file's text; a directory where a file belongs reads as empty."""
    if entry.kind != "blob":
        return ""
    if entry.object_id not in _TEXTS:
        _TEXTS[entry.object_id] = git_output(["cat-file", "blob", entry.object_id]) or ""
    return _TEXTS[entry.object_id]


def blob_at(commit: str, path: str) -> str | None:
    """The file's bytes at `commit`, or `None` where the commit did not hold it.

    `path` has already been resolved against the repository root, and
    `commit` against the object store, so a `git show` that fails here means the
    file was absent at that commit and nothing else.
    """
    return git_output(["show", f"{commit}:{path}"])


def repository_path(changelog: Path) -> str:
    """`changelog` as git names it, from the repository root down.

    Resolved rather than prefixed: an absolute `--changelog` used to reach
    `git show <commit>:/abs/path`, which git refuses, and a refusal read as
    "the file was not there" makes every moved bullet count as written.
    """
    toplevel = git(["rev-parse", "--show-toplevel"])
    if toplevel is None:
        raise ChangelogPrError("not inside a git repository; the changelog gate reads local git alone")
    try:
        return changelog.resolve().relative_to(Path(toplevel).resolve()).as_posix()
    except ValueError as exc:
        raise ChangelogPrError(f"{changelog.as_posix()} is outside the repository at {toplevel}") from exc


def resolve_commit(revision: str) -> str:
    """`revision` as a commit sha, so an absent file cannot read as an absent commit."""
    sha = git(["rev-parse", "--verify", "--quiet", f"{revision}^{{commit}}"])
    if sha is None:
        raise ChangelogPrError(f"no commit resolves for {revision}")
    return sha


def merge_base(base: str, head: str) -> tuple[str | None, str]:
    """The merge base, or `None` and why none can be taken. §FS-distribution.4.6

    `git merge-base` fails alike for a base no ref resolves and for one that
    resolves but shares no history with the head, and both degrade the same way
    — but a contributor whose base is a shallow clone's grafted commit would
    read "no base ref resolves" and go fetch a ref they already have, so the two
    are told apart here rather than at the call site. The base is deliberately
    not put through `resolve_commit`: failing to resolve is a documented skip
    for the base, where for the head it is an error.
    """
    if git(["rev-parse", "--verify", "--quiet", f"{base}^{{commit}}"]) is None:
        return None, NO_BASE_REF
    found = git(["merge-base", base, head])
    if found is None:
        return None, f"the base {base} and this head share no history"
    return found, ""


def git(arguments: Sequence[str]) -> str | None:
    """`git` output stripped, or `None` where git says no — an unresolvable ref included."""
    output = git_output(arguments)
    return None if output is None else output.strip()


def git_output(arguments: Sequence[str]) -> str | None:
    """git's stdout verbatim, decoded as UTF-8 whatever the platform's locale is.

    Both sides of the comparison come through here, so a section marker or an
    em dash cannot decode two ways the way a Windows clone's locale decoding
    made them (§AR-ci.1.1) — and a byte the changelog should not hold replaces
    itself identically on both sides rather than raising.
    """
    try:
        result = subprocess.run(
            ["git", *arguments], check=False, capture_output=True, encoding="utf-8", errors="replace"
        )
    except FileNotFoundError as exc:
        raise ChangelogPrError("git is not on PATH; the changelog gate reads local git alone") from exc
    if result.returncode != 0:
        return None
    return result.stdout
