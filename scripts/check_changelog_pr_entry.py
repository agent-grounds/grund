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


class ChangelogPrError(Exception):
    pass


def check_changelog_pr_entry(
    changelog: Path, base: str | None, head: str, pr_number: int | None, pre_push: bool
) -> None:
    """R1 and R2 of §FS-distribution.4.6, against one base and one head."""
    lines = _read_lines(changelog)
    merge_base = _merge_base(base, head) if base is not None else None

    if merge_base is None:
        # No base ref resolves — a shallow clone, or a `main` never fetched. Ask
        # for less rather than for a fetch the contributor did not make.
        print(
            f"warning: no base ref resolves; requiring at least one bullet under `## Unreleased` of {changelog}",
            file=sys.stderr,
        )
        if not changelog_bullets.has_bullet(changelog_bullets.unreleased_body(lines)):
            raise ChangelogPrError(_refusal(changelog, "in this clone", pre_push))
        return

    base_keys = {bullet.key for bullet in _bullets_at(merge_base, changelog)}
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
        f"{changelog} `## Unreleased` has no new or changed bullet against {base_label}.\n"
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


def head_ref_for_pre_push() -> str:
    """The local ref `pre-commit` hands the `pre-push` stage, else the checkout."""
    to_ref = os.environ.get("PRE_COMMIT_TO_REF", "").strip()
    if not to_ref or ZERO_SHA_RE.match(to_ref):
        return "HEAD"
    return to_ref


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


def _read_lines(changelog: Path) -> list[str]:
    try:
        return changelog.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError as exc:
        raise ChangelogPrError(f"missing changelog: {changelog}") from exc


def _bullets_at(commit: str, changelog: Path) -> list[changelog_bullets.Bullet]:
    """The bullets `## Unreleased` held at `commit`; none if the file was not there."""
    blob = _git(["show", f"{commit}:{_repository_path(changelog)}"])
    if blob is None:
        return []
    try:
        return changelog_bullets.bullets(blob.splitlines())
    except changelog_bullets.ChangelogFormatError:
        return []


def _repository_path(changelog: Path) -> str:
    prefix = _git(["rev-parse", "--show-prefix"]) or ""
    return f"{prefix}{changelog.as_posix()}" if not changelog.is_absolute() else changelog.as_posix()


def _merge_base(base: str, head: str) -> str | None:
    return _git(["merge-base", base, head])


def _git(arguments: Sequence[str]) -> str | None:
    """`git` output, or `None` where git says no — an unresolvable ref included."""
    try:
        result = subprocess.run(["git", *arguments], check=False, capture_output=True, text=True)
    except FileNotFoundError as exc:
        raise ChangelogPrError("git is not on PATH; the changelog gate reads local git alone") from exc
    if result.returncode != 0:
        return None
    return result.stdout.strip()


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
