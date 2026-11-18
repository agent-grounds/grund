#!/usr/bin/env python3
"""Read-only manual release selection and preflight. §FS-distribution.4.4.2

Run from the repository root after fetching main and tags. Only the four
workflow output assignments reach stdout; all refusals precede those outputs.
The release guard remains a separate step after selection.
"""

import argparse
import re
import sys
from pathlib import Path

# §FS-distribution.4.4.2: importing the shared reader must not write bytecode.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent / "distribution"))

import versions  # noqa: E402
from release_common import ChangelogError, git  # noqa: E402


SHA = re.compile(r"[0-9a-fA-F]{40}")
STABLE_TAG = re.compile(r"v([0-9]+)\.([0-9]+)\.([0-9]+)")


def check_identity(root, ref, dispatch_sha, expected_sha):
    """§FS-distribution.4.4.2: bind selection to the owner's dispatch main tip."""
    if ref != "refs/heads/main":
        raise ChangelogError("release selection requires refs/heads/main")
    if not expected_sha or not SHA.fullmatch(expected_sha):
        raise ChangelogError("expected_main_sha must be a full 40-hex SHA")
    if not SHA.fullmatch(dispatch_sha):
        raise ChangelogError("dispatch SHA must be a full 40-hex SHA")
    head = git(root, "rev-parse", "HEAD").strip()
    main = git(root, "rev-parse", "--verify", "--quiet", "refs/remotes/origin/main",
               nothing_found=1).strip()
    if not main:
        raise ChangelogError("missing fetched origin/main tip")
    if len({expected_sha.lower(), dispatch_sha.lower(), head, main}) != 1:
        raise ChangelogError(
            "expected_main_sha must match the dispatch SHA, checkout HEAD and fetched origin/main SHA")


def descends(root, name, tag):
    """§FS-distribution.4.4.2: whether tag `name` sits on a commit that contains tag `tag`."""
    base = git(root, "rev-parse", f"refs/tags/{tag}^{{commit}}").strip()
    tip = git(root, "rev-parse", f"refs/tags/{name}^{{commit}}").strip()
    return git(root, "merge-base", base, tip, nothing_found=1).strip() == base


def select(root, args):
    """§FS-distribution.4.4.2: one preflight returns either choice's existing outputs."""
    first_stable = args.version == "1.0.0"
    if args.version not in ("next-minor", "1.0.0"):
        raise ChangelogError("version must be next-minor or 1.0.0")
    if first_stable or args.expected_main_sha is not None:
        check_identity(root, args.ref, args.dispatch_sha, args.expected_main_sha)

    if first_stable:
        # §FS-distribution-candidate.6.2: use the candidate's complete source agreement check.
        source = versions.check(root)
        if source.split(".", 1)[0] != "0":
            raise ChangelogError(f"first-stable selection requires a 0.x source version, found {source}")

    tags = []
    for tag in git(root, "tag", "--list", "v*.*.*").splitlines():
        match = STABLE_TAG.fullmatch(tag)
        if match:
            tags.append((tuple(map(int, match.groups())), tag))
    if not tags:
        raise ChangelogError("no stable vX.Y.Z tag exists")
    number, tag = max(tags)
    if first_stable:
        if number >= (1, 0, 0):
            raise ChangelogError(f"stable tag {tag} already exists; cannot select the first stable release")
        # §FS-distribution.4.4.2: start from the previous release (§FS-distribution.4.6.1); a higher
        # tag may only be its patch on a branch off it (§DF-markdown-homes-refused-in-1-0.3).
        reached = set(git(root, "tag", "--merged", "HEAD", "--list", "v*.*.*").split())
        released = [(n, t) for n, t in tags if t in reached]
        if not released:
            raise ChangelogError("no stable vX.Y.Z tag is reachable from dispatch main")
        number, tag = max(released)
        for other, name in sorted(tags, reverse=True):
            if other > number and not (other[:2] == number[:2] and descends(root, name, tag)):
                raise ChangelogError(f"stable tag {name} is not reachable from dispatch main "
                                     f"and is not a patch of {tag} on a branch off it")

    count = int(git(root, "rev-list", "--count", f"refs/tags/{tag}..HEAD").strip())
    if count == 0:
        raise ChangelogError(f"no commits since {tag}; refusing an empty release")
    current = tag[1:]
    next_version = "1.0.0" if first_stable else f"{number[0]}.{number[1] + 1}.0"
    return {"current": current, "next": next_version,
            "current_tag": tag, "next_tag": f"v{next_version}"}


def main():
    """§FS-distribution.4.4.2: refuse without outputs, or emit the four assignments."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--dispatch-sha", required=True)
    parser.add_argument("--version", default="next-minor")
    parser.add_argument("--expected-main-sha")
    args = parser.parse_args()
    try:
        outputs = select(Path.cwd(), args)
    except (ChangelogError, OSError, ValueError) as error:
        raise SystemExit(f"error: {error}") from None
    for name, value in outputs.items():
        print(f"{name}={value}")


if __name__ == "__main__":
    main()
