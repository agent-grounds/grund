#!/usr/bin/env python3
"""Require every pull request to add a changelog entry. §FS-distribution.4.6

An entry is one file under `docs/changelog/unreleased/` (§FS-distribution.4.12),
and the branch must add one whose slug the merge base does not hold. One rule in
two halves that take the same input: `--pre-push` resolves the base from the
remote's `main`, pull-request CI is handed the base the event names, and both
then list the directory at the two commits from local git alone, asking the
questions `changelog_bullets` answers (§AR-ci.7). No half calls the GitHub API,
so a fork and a restricted token reach the verdict an owner branch reaches.
"""

from __future__ import annotations

import argparse
import json
import os
import posixpath
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))

import changelog_bullets  # noqa: E402  (the shared definitions live beside this script)
import changelog_gate_git as gate_git  # noqa: E402  (what the gate reads of git, split out for size)


ZERO_SHA_RE = re.compile(r"^0{4,40}$")
SKIP_LINE = "      SKIP=changelog-pr-entry git push"
NO_SLUG = "<slug>"


ChangelogPrError = gate_git.ChangelogPrError
EntryFile = gate_git.EntryFile


def check_changelog_pr_entry(
    changelog: Path,
    base: str | None,
    head: str,
    pr_number: int | None,
    pre_push: bool,
    branch: str | None = None,
) -> None:
    """§FS-distribution.4.6 against one base and one head."""
    repository_path = gate_git.repository_path(changelog)
    directory = posixpath.join(posixpath.dirname(repository_path), changelog_bullets.ENTRY_DIRECTORY)
    head_commit = gate_git.resolve_commit(head)
    head_files = gate_git.entries_at(head_commit, directory)
    merge_base, why_not = (None, gate_git.NO_BASE_REF) if base is None else gate_git.merge_base(base, head_commit)

    if merge_base is None:
        # No merge base can be taken — a shallow clone, or a `main` never
        # fetched. Ask for less rather than for a fetch the contributor did not
        # make (§FS-distribution.4.6).
        print(f"warning: {why_not}; requiring at least one entry in {directory}/", file=sys.stderr)
        if not any(_is_entry(entry) for entry in head_files.values()):
            raise ChangelogPrError(_refusal(directory, "in this clone", branch, pre_push))
        return

    base_files = gate_git.entries_at(merge_base, directory)
    touched = [
        entry
        for name, entry in sorted(head_files.items())
        if name != changelog_bullets.ENTRY_README
        and (name not in base_files or base_files[name].object_id != entry.object_id)
    ]
    _refuse_what_is_not_an_entry(directory, touched, head_files)

    carried = _numbers_by_slug(base_files)
    moved_from = {bullet.key for bullet in _bullets_at(merge_base, repository_path)}
    written = []
    for entry in touched:
        slug = _slug(entry.name)
        if slug not in carried and changelog_bullets.normalise(gate_git.text(entry)) in moved_from:
            continue  # Moved out of the merge base's `## Unreleased`: not written, number unexamined.
        if slug not in carried:
            written.append(entry)
        if pr_number is not None:
            _refuse_a_foreign_number(directory, entry, carried.get(slug, frozenset()), pr_number)
    if not written:
        raise ChangelogPrError(_refusal(directory, f"against {_label(base, merge_base)}", branch, pre_push))


def _refuse_what_is_not_an_entry(
    directory: str, touched: list[EntryFile], head_files: dict[str, EntryFile]
) -> None:
    """Each file the branch adds or changes is an entry with a slug of its own. §FS-distribution.4.6"""
    for entry in touched:
        problem = changelog_bullets.entry_problem(entry.name, gate_git.text(entry))
        if problem is not None:
            raise ChangelogPrError(f"{directory}/{problem}")
    slugs = {name: parsed.slug for name in head_files if (parsed := changelog_bullets.entry_name(name))}
    holders = Counter(slugs.values())
    for entry in touched:
        slug = slugs[entry.name]
        if holders[slug] > 1:
            sharing = ", ".join(name for name, other in sorted(slugs.items()) if other == slug)
            raise ChangelogPrError(
                f"{directory}/: the slug `{slug}` is shared by {sharing}; a slug is unique in the directory, "
                "whatever the category"
            )


def _refuse_a_foreign_number(directory: str, entry: EntryFile, carried: frozenset[int], pr_number: int) -> None:
    """A number the branch writes into an entry is the pull request's own. §FS-distribution.4.6

    A number the entry's slug already carried at the merge base was not written
    by this branch, so it is not this branch's to answer for.
    """
    foreign = sorted(changelog_bullets.pr_numbers(gate_git.text(entry)) - carried - {pr_number})
    if foreign:
        raise ChangelogPrError(
            f"{directory}/{entry.name} names PR #{foreign[0]}, which is not this pull request "
            f"(PR #{pr_number}); a number is optional, but a number that is written must be the "
            f"pull request's own:\n  {gate_git.text(entry).splitlines()[0]}"
        )


def _numbers_by_slug(files: dict[str, EntryFile]) -> dict[str, frozenset[int]]:
    """Each slug the directory holds, with every number its files carry."""
    numbers: dict[str, frozenset[int]] = {}
    for name, entry in files.items():
        parsed = changelog_bullets.entry_name(name)
        if parsed is not None:
            named = changelog_bullets.pr_numbers(gate_git.text(entry))
            numbers[parsed.slug] = numbers.get(parsed.slug, frozenset()) | named
    return numbers


def _slug(name: str) -> str:
    """The slug of a file `_refuse_what_is_not_an_entry` has already let through."""
    parsed = changelog_bullets.entry_name(name)
    assert parsed is not None, name
    return parsed.slug


def _is_entry(entry: EntryFile) -> bool:
    if entry.name == changelog_bullets.ENTRY_README:
        return False
    return changelog_bullets.entry_problem(entry.name, gate_git.text(entry)) is None


def _refusal(directory: str, against: str, branch: str | None, pre_push: bool) -> str:
    """What a branch that adds no entry reads: the file to add, by name. §FS-distribution.4.6"""
    message = (
        f"{directory}/ gains no entry {against}.\n"
        "  Add one file for this change:\n"
        f"      {directory}/{suggested_slug(branch)}.<category>.md\n"
        f"  <category> is one of: {', '.join(changelog_bullets.CATEGORIES)}.\n"
        "  The file holds one bullet. A number is optional: the release writes it."
    )
    if pre_push:
        message += "\n  If this branch is not becoming a pull request:\n" + SKIP_LINE
    return message


def suggested_slug(branch: str | None) -> str:
    """The branch name made a slug: `fix/issue-379` suggests `fix-issue-379`. §FS-distribution.4.6"""
    name = (branch or "").removeprefix("refs/heads/")
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-") or NO_SLUG


def branch_name(pre_push: bool) -> str | None:
    """The branch each half already has (§AR-ci.7): the local ref `pre-commit`
    hands the hook, and in CI the pull request's head branch, which the runner
    sets in the environment rather than the workflow splicing it into a shell
    line, where a fork's branch name would be code."""
    return os.environ.get("PRE_COMMIT_LOCAL_BRANCH" if pre_push else "GITHUB_HEAD_REF") or None


def _label(base: str, merge_base: str) -> str:
    return base if base != merge_base else f"the merge base {merge_base[:12]}"


def head_contains_nothing_new(base: str | None, head: str) -> bool:
    """Whether the base already holds the head: there is nothing to require."""
    if base is None:
        return False
    return gate_git.git(["merge-base", "--is-ancestor", head, base]) is not None


def base_ref_for_pre_push() -> str | None:
    """The remote's `main`, as `pre-commit` names it, else `origin/main`, else `main`."""
    remote = os.environ.get("PRE_COMMIT_REMOTE_NAME")
    candidates = ([f"{remote}/main"] if remote else []) + ["origin/main", "main"]
    for candidate in candidates:
        if gate_git.git(["rev-parse", "--verify", "--quiet", f"{candidate}^{{commit}}"]) is not None:
            return candidate
    return None


def head_ref_for_pre_push() -> str | None:
    """The local ref `pre-commit` hands the `pre-push` stage, else the checkout.

    `None` where the push deletes the ref: `pre-commit` hands the all-zero sha,
    there is no head to list entries from, and a deletion adds no entry to
    require (§FS-distribution.4.6).
    """
    to_ref = os.environ.get("PRE_COMMIT_TO_REF", "").strip()
    if ZERO_SHA_RE.match(to_ref):
        return None
    return to_ref or "HEAD"


def pr_number_from_event(event_path: Path) -> int | None:
    pull_request = _pull_request_from_event(event_path)
    if pull_request is None:
        return None
    number = pull_request.get("number")
    if number is None:
        return None
    if not isinstance(number, int) or number <= 0:
        raise ChangelogPrError(f"invalid pull request number in event: {number!r}")
    return number


def base_sha_from_event(event_path: Path) -> str | None:
    """The base commit the `pull_request` event names — the CI half's base."""
    pull_request = _pull_request_from_event(event_path)
    if pull_request is None:
        return None
    base = pull_request.get("base")
    sha = base.get("sha") if isinstance(base, dict) else None
    return sha if isinstance(sha, str) and sha else None


def _pull_request_from_event(event_path: Path) -> dict | None:
    try:
        event = json.loads(event_path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ChangelogPrError(f"missing GitHub event file: {event_path}") from exc
    except json.JSONDecodeError as exc:
        raise ChangelogPrError(f"invalid GitHub event JSON: {event_path}: {exc}") from exc
    pull_request = event.get("pull_request")
    return pull_request if isinstance(pull_request, dict) else None


def _bullets_at(commit: str, repository_path: str) -> list[changelog_bullets.Bullet]:
    """The bullets `## Unreleased` held at `commit`; none if the file was not there."""
    blob = gate_git.blob_at(commit, repository_path)
    if blob is None:
        return []
    try:
        return changelog_bullets.bullets(blob.splitlines())
    except changelog_bullets.ChangelogFormatError:
        return []


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check that the branch adds a changelog entry, and that any number in one is this PR's."
    )
    parser.add_argument("--changelog", type=Path, default=Path("docs/changelog.md"))
    parser.add_argument("--pr-number", type=int)
    parser.add_argument("--event-path", type=Path, default=None)
    parser.add_argument("--base-sha", default=None, help="the base commit to take the merge base against")
    parser.add_argument("--head-sha", default=None, help="the head commit, defaulting to the checkout")
    parser.add_argument(
        "--pre-push",
        action="store_true",
        help="resolve the base from the remote's main and the head from the pushed ref",
    )
    args = parser.parse_args(argv)

    try:
        event_path = args.event_path
        if event_path is None:
            raw_event_path = os.environ.get("GITHUB_EVENT_PATH")
            event_path = Path(raw_event_path) if raw_event_path else None

        pr_number = args.pr_number
        if pr_number is None and event_path is not None:
            pr_number = pr_number_from_event(event_path)
        if pr_number is not None and pr_number <= 0:
            raise ChangelogPrError(f"invalid pull request number: {pr_number}")

        if args.pre_push:
            base = args.base_sha or base_ref_for_pre_push()
            head = args.head_sha or head_ref_for_pre_push()
            if head is None:
                print("the push deletes a ref and adds nothing; skipping the changelog entry check")
                return 0
        else:
            base = args.base_sha
            if base is None and event_path is not None:
                base = base_sha_from_event(event_path)
            head = args.head_sha or "HEAD"

        if head_contains_nothing_new(base, head):
            print("the base already contains this head; skipping the changelog entry check")
            return 0

        branch = branch_name(args.pre_push)
        check_changelog_pr_entry(args.changelog, base, head, pr_number, args.pre_push, branch)
    except (ChangelogPrError, changelog_bullets.ChangelogFormatError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
