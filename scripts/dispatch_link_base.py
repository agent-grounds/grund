#!/usr/bin/env python3
"""The link gate's diff base for a dispatched run (§AR-ci.3.1.4): fetch the upstream
repository's `main` and print the merge base of `HEAD` with it, so a dispatch on a fork
measures what a pull request from its branch would rather than from the fork's stale `main`.

Run in the checkout as `scripts/dispatch_link_base.py <upstream-url>`. It always exits 0 and
prints `none` when the base cannot be resolved, with the reason on stderr; the gate reads a
`LINK_BASE` that names no commit as no diff base and checks no external URL.
"""

from __future__ import annotations

import subprocess
import sys


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=False)


def upstream_merge_base(upstream_url: str) -> tuple[str | None, str]:
    """The merge base of `HEAD` with `main` at `upstream_url`, or None and the reason."""
    fetched = _git("fetch", "--quiet", "--no-tags", upstream_url, "refs/heads/main")
    if fetched.returncode != 0:
        return None, f"cannot fetch main from {upstream_url}: {fetched.stderr.strip()}"
    merge_base = _git("merge-base", "HEAD", "FETCH_HEAD")
    if merge_base.returncode != 0 or not merge_base.stdout.strip():
        return None, f"HEAD has no merge base with main of {upstream_url}"
    return merge_base.stdout.strip(), ""


def main(argv: list[str]) -> int:
    if len(argv) != 1 or not argv[0]:
        print("none")
        print("usage: dispatch_link_base.py <upstream-url>", file=sys.stderr)
        return 0
    base, reason = upstream_merge_base(argv[0])
    print(base or "none")
    if base is None:
        print(f"dispatch_link_base: {reason}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
