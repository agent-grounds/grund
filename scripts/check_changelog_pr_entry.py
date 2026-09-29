#!/usr/bin/env python3
"""Require every pull request to add a bullet to the Unreleased changelog. §FS-distribution.4.6

One rule in two halves that take the same input: `--pre-push` resolves the base
from the remote's `main`, pull-request CI is handed the base the event names,
and both then ask local git the one question `changelog_bullets` defines
(§AR-ci.7). No half calls the GitHub API, so a fork and a restricted token reach
the verdict an owner branch reaches.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))

import changelog_bullets  # noqa: E402  (the shared definitions live beside this script)


ZERO_SHA_RE = re.compile(r"^0{4,40}$")
SKIP_LINE = "      SKIP=changelog-pr-entry git push"
NO_BASE_REF = "no base ref resolves"


class ChangelogPrError(Exception):
    pass


def check_changelog_pr_entry(
    changelog: Path, base: str | None, head: str, pr_number: int | None, pre_push: bool
) -> None:
    """R1 and R2 of §FS-distribution.4.6, against one base and one head."""
    repository_path = _repository_path(changelog)
    head_commit = _resolve_commit(head)
    lines = _lines_at(head_commit, repository_path, changelog)
    merge_base, why_not = _merge_base(base, head_commit) if base is not None else (None, NO_BASE_REF)

    if merge_base is None:
        # No merge base can be taken — a shallow clone, or a `main` never
        # fetched. Ask for less rather than for a fetch the contributor did not
        # make (§FS-distribution.4.6).
        print(
            f"warning: {why_not}; requiring at least one bullet under `## Unreleased` of {changelog.as_posix()}",
            file=sys.stderr,
        )
        if not changelog_bullets.has_bullet(changelog_bullets.unreleased_body(lines)):
            raise ChangelogPrError(_refusal(changelog, "in this clone", pre_push))
        return

    base_keys = {bullet.key for bullet in _bullets_at(merge_base, repository_path)}
    added = [bullet for bullet in changelog_bullets.bullets(lines) if bullet.key not in base_keys]
    if not added:
        raise ChangelogPrError(_refusal(changelog, _label(base, merge_base), pre_push))

    if pr_number is None:
        return
    for bullet in added:
        others = sorted(bullet.numbers - {pr_number})
        if others:
            raise ChangelogPrError(
                f"a `## Unreleased` bullet this branch adds names PR #{others[0]}, which is not this "
                f"pull request (PR #{pr_number}); a number is optional, but a number that is written "
                f"must be the pull request's own:\n  {bullet.text.splitlines()[0]}"
            )


def _refusal(changelog: Path, base_label: str, pre_push: bool) -> str:
    message = (
        f"{changelog.as_posix()} `## Unreleased` has no new or changed bullet against {base_label}.\n"
        "  Add one. A number is optional: write `(PR #TBD)` and the release fills it in."
    )
    if pre_push:
        message += "\n  If this branch is not becoming a pull request:\n" + SKIP_LINE
    return message


def _label(base: str, merge_base: str) -> str:
    return base if base != merge_base else f"the merge base {merge_base[:12]}"


def head_contains_nothing_new(base: str | None, head: str) -> bool:
    """Whether the base already holds the head: there is nothing to require."""
    if base is None:
        return False
    return _git(["merge-base", "--is-ancestor", head, base]) is not None


def base_ref_for_pre_push() -> str | None:
    """The remote's `main`, as `pre-commit` names it, else `origin/main`, else `main`."""
    remote = os.environ.get("PRE_COMMIT_REMOTE_NAME")
    candidates = ([f"{remote}/main"] if remote else []) + ["origin/main", "main"]
    for candidate in candidates:
        if _git(["rev-parse", "--verify", "--quiet", f"{candidate}^{{commit}}"]) is not None:
            return candidate
    return None


def head_ref_for_pre_push() -> str | None:
    """The local ref `pre-commit` hands the `pre-push` stage, else the checkout.

    `None` where the push deletes the ref: `pre-commit` hands the all-zero sha,
    there is no head to read a changelog from, and a deletion adds no line to
    require a bullet for (§FS-distribution.4.6).
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


def _lines_at(commit: str, repository_path: str, changelog: Path) -> list[str]:
    """The changelog as `commit` holds it. §FS-distribution.4.6

    The head side of the comparison is read here and not from the checkout, so
    the merge `actions/checkout` leaves in the tree on a `pull_request` event —
    the head merged with the *current* base tip — cannot lend this branch a
    bullet the base gained after the branch point.
    """
    blob = _blob_at(commit, repository_path)
    if blob is None:
        raise ChangelogPrError(f"missing changelog: {changelog.as_posix()} is not in {commit[:12]}")
    return blob.splitlines()


def _bullets_at(commit: str, repository_path: str) -> list[changelog_bullets.Bullet]:
    """The bullets `## Unreleased` held at `commit`; none if the file was not there."""
    blob = _blob_at(commit, repository_path)
    if blob is None:
        return []
    try:
        return changelog_bullets.bullets(blob.splitlines())
    except changelog_bullets.ChangelogFormatError:
        return []


def _blob_at(commit: str, repository_path: str) -> str | None:
    """The file's bytes at `commit`, or `None` where the commit did not hold it.

    `repository_path` has already been resolved against the repository root, and
    `commit` against the object store, so a `git show` that fails here means the
    file was absent at that commit and nothing else.
    """
    return _git_output(["show", f"{commit}:{repository_path}"])


def _repository_path(changelog: Path) -> str:
    """`changelog` as git names it, from the repository root down.

    Resolved rather than prefixed: an absolute `--changelog` used to reach
    `git show <commit>:/abs/path`, which git refuses, and a refusal read as
    "the file was not there" makes every bullet count as added.
    """
    toplevel = _git(["rev-parse", "--show-toplevel"])
    if toplevel is None:
        raise ChangelogPrError("not inside a git repository; the changelog gate reads local git alone")
    try:
        return changelog.resolve().relative_to(Path(toplevel).resolve()).as_posix()
    except ValueError as exc:
        raise ChangelogPrError(f"{changelog.as_posix()} is outside the repository at {toplevel}") from exc


def _resolve_commit(revision: str) -> str:
    """`revision` as a commit sha, so an absent file cannot read as an absent commit."""
    sha = _git(["rev-parse", "--verify", "--quiet", f"{revision}^{{commit}}"])
    if sha is None:
        raise ChangelogPrError(f"no commit resolves for {revision}")
    return sha


def _merge_base(base: str, head: str) -> tuple[str | None, str]:
    """The merge base, or `None` and why none can be taken. §FS-distribution.4.6

    `git merge-base` fails alike for a base no ref resolves and for one that
    resolves but shares no history with the head, and both degrade the same way
    — but a contributor whose base is a shallow clone's grafted commit would
    read "no base ref resolves" and go fetch a ref they already have, so the two
    are told apart here rather than at the call site. The base is deliberately
    not put through `_resolve_commit`: failing to resolve is a documented skip
    for the base, where for the head it is an error.
    """
    if _git(["rev-parse", "--verify", "--quiet", f"{base}^{{commit}}"]) is None:
        return None, NO_BASE_REF
    merge_base = _git(["merge-base", base, head])
    if merge_base is None:
        return None, f"the base {base} and this head share no history"
    return merge_base, ""


def _git(arguments: Sequence[str]) -> str | None:
    """`git` output stripped, or `None` where git says no — an unresolvable ref included."""
    output = _git_output(arguments)
    return None if output is None else output.strip()


def _git_output(arguments: Sequence[str]) -> str | None:
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


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check that `## Unreleased` gains a bullet, and that any number in it is this PR's."
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
                print("the push deletes a ref and adds nothing; skipping the changelog bullet check")
                return 0
        else:
            base = args.base_sha
            if base is None and event_path is not None:
                base = base_sha_from_event(event_path)
            head = args.head_sha or "HEAD"

        if head_contains_nothing_new(base, head):
            print("the base already contains this head; skipping the changelog bullet check")
            return 0

        check_changelog_pr_entry(args.changelog, base, head, pr_number, args.pre_push)
    except (ChangelogPrError, changelog_bullets.ChangelogFormatError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
