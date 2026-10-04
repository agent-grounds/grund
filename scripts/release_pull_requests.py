#!/usr/bin/env python3
"""The pull requests merged since the previous tag, as the release lists them. §FS-distribution.4.6.1

`Range` is what git says the release covers; `pull_request_lines` asks the forge
about every commit in it and writes one line per merged pull request
(§FS-distribution.4.6.2). Both only read: `prepare_changelog_release.py` is the
one that writes, and only once everything here has answered
(§FS-distribution.4.6.3).
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent))

from release_common import ChangelogError, git  # noqa: E402  (the shared definitions live beside this script)


TAG_RE = re.compile(r"^v(?P<version>[0-9]+\.[0-9]+\.[0-9]+)$")
# The commit opening the next development version has no pull request (§FS-distribution.4.1).
DEV_OPENER_RE = re.compile(r"^Open [0-9]+\.[0-9]+\.[0-9]+-dev for development$")
# Each of these is escaped with a backslash in a title; `§` is written `&sect;` instead.
ESCAPED = frozenset("\\`*_[]<>&")


class Commit(NamedTuple):
    sha: str
    subject: str


class Range(NamedTuple):
    """`HEAD`, read once, the previous release's tag, and the commits between, newest first."""

    top: Path
    head: str
    tag: str
    commits: tuple[Commit, ...]


class PullRequest(NamedTuple):
    number: int
    title: str
    url: str
    merge_commit: str


def release_range(directory: Path, inline_version: str) -> Range:
    """The range a release covers. §FS-distribution.4.6.1

    `HEAD` is frozen first and everything after reads that commit. The previous
    release is the highest `vX.Y.Z` tag reachable from it, and it must name the
    release `docs/changelog.md` keeps inline.
    """
    top = Path(git(directory, "rev-parse", "--show-toplevel").strip())
    if git(top, "rev-parse", "--is-shallow-repository").strip() == "true":
        raise ChangelogError(
            "the checkout is a shallow clone, so the range since the previous tag cannot be read whole; "
            "fetch the full history first (fetch-depth: 0)"
        )
    head = git(top, "rev-parse", "HEAD").strip()
    versions = []
    for tag in git(top, "tag", "--merged", head, "--list", "v*").split():
        match = TAG_RE.match(tag)
        if match is not None:
            versions.append((tuple(int(part) for part in match.group("version").split(".")), tag))
    if not versions:
        raise ChangelogError("no vX.Y.Z tag is reachable from HEAD, so there is no previous release to list from")
    tag = max(versions)[1]
    if tag != f"v{inline_version}":
        raise ChangelogError(
            f"the previous tag {tag} does not name the release docs/changelog.md keeps inline, {inline_version}"
        )
    log = git(top, "log", "--topo-order", "--format=%H%x1f%s", f"{tag}..{head}")
    commits = tuple(Commit(*line.split("\x1f", 1)) for line in log.splitlines() if line)
    return Range(top, head, tag, commits)


def pull_request_lines(span: Range) -> list[str]:
    """One line per pull request merged in `span`, newest first by git. §FS-distribution.4.6.2

    Every commit is asked about, so a list that has answered has seen every pull
    request: merges are rebase-only, and each pull request's commits land on
    `main`. A commit the forge does not answer for is refused rather than
    skipped (§FS-distribution.4.6.3).
    """
    repository = _repository_url(span.top)
    position = {commit.sha: index for index, commit in enumerate(span.commits)}
    listed: dict[int, PullRequest] = {}
    for commit in span.commits:
        kept = [pull for pull in _pulls_for(span.top, commit) if pull.merge_commit in position]
        for pull in kept:
            expected = f"{repository}/pull/{pull.number}"
            if pull.url != expected:
                raise ChangelogError(
                    f"pull request #{pull.number} links {pull.url}, not this repository's {expected}"
                )
            listed.setdefault(pull.number, pull)
        if not kept and DEV_OPENER_RE.match(commit.subject) is None:
            print(
                f"warning: commit {commit.sha[:12]} ({commit.subject}) belongs to no pull request merged "
                "into main in this range, so the release does not list it",
                file=sys.stderr,
            )
    if not listed:
        raise ChangelogError(f"{span.tag}..HEAD holds no pull request merged into main; there is nothing to release")
    ordered = sorted(listed.values(), key=lambda pull: position[pull.merge_commit])
    return [f"- [{escape_title(pull.title)}]({pull.url}) (PR #{pull.number})\n" for pull in ordered]


def escape_title(title: str) -> str:
    """A title collapsed to one line and escaped, so it is text and never a citation. §FS-distribution.4.6.2"""
    collapsed = " ".join(title.split())
    return "".join("&sect;" if char == "§" else f"\\{char}" if char in ESCAPED else char for char in collapsed)


def _pulls_for(top: Path, commit: Commit) -> list[PullRequest]:
    """The merged pull requests into `main` the forge says `commit` belongs to. §FS-distribution.4.6.1"""
    answer = _forge(top, f"repos/{{owner}}/{{repo}}/commits/{commit.sha}/pulls", f"commit {commit.sha[:12]} ({commit.subject})")
    if not isinstance(answer, list):
        raise ChangelogError(f"the forge answered for commit {commit.sha[:12]} with something other than a list")
    pulls = []
    for pull in answer:
        base = (pull.get("base") or {}).get("ref")
        if pull.get("merged_at") and base == "main" and pull.get("merge_commit_sha"):
            pulls.append(PullRequest(int(pull["number"]), str(pull["title"]), str(pull["html_url"]), pull["merge_commit_sha"]))
    return pulls


def _repository_url(top: Path) -> str:
    """This repository's address, as the forge answers for it."""
    answer = _forge(top, "repos/{owner}/{repo}", "this repository")
    url = answer.get("html_url") if isinstance(answer, dict) else None
    if not url:
        raise ChangelogError("the forge did not say which repository this is")
    return str(url).rstrip("/")


def _forge(top: Path, endpoint: str, about: str) -> object:
    """`gh api <endpoint>`'s JSON, read here rather than through `--jq`; any failure refuses."""
    if shutil.which("gh") is None:
        raise ChangelogError("gh is not on PATH; the release reads the pull requests it lists from the forge with it")
    result = subprocess.run(
        ["gh", "api", endpoint],
        cwd=top,
        check=False,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        raise ChangelogError(f"the forge did not answer for {about}: {detail[-1] if detail else 'gh failed'}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise ChangelogError(f"the forge's answer for {about} is not JSON: {exc}") from exc
