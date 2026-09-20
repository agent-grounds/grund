#!/usr/bin/env python3
"""Check links while resolving this repository's main-branch URLs locally. §AR-ci.3.1"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Callable, Sequence
from urllib.parse import unquote, urlsplit


SELF_PREFIX = "https://github.com/agent-grounds/grund/blob/main/"
SELF_PATH_PREFIX = "/agent-grounds/grund/blob/main/"


class SelfLinkError(Exception):
    pass


def self_links(dumped: str) -> list[str]:
    return sorted({line for line in dumped.splitlines() if line.startswith(SELF_PREFIX)})


def _local_url(url: str, repo_root: Path) -> str:
    parsed = urlsplit(url)
    if (
        parsed.scheme != "https"
        or parsed.netloc != "github.com"
        or not parsed.path.startswith(SELF_PATH_PREFIX)
        or parsed.query
    ):
        raise SelfLinkError(f"malformed canonical self-link: {url}")

    encoded = parsed.path.removeprefix(SELF_PATH_PREFIX)
    try:
        relative = unquote(encoded, errors="strict")
    except UnicodeError as exc:
        raise SelfLinkError(f"invalid path encoding in canonical self-link: {url}") from exc
    parts = PurePosixPath(relative).parts
    if not relative or any(part in ("", ".", "..") for part in parts):
        raise SelfLinkError(f"unsafe path in canonical self-link: {url}")

    root = repo_root.resolve()
    target = (root / Path(*parts)).resolve()
    try:
        target.relative_to(root)
    except ValueError as exc:
        raise SelfLinkError(f"self-link escapes the repository: {url}") from exc
    if not target.is_file():
        raise SelfLinkError(f"canonical self-link target is missing: {relative} ({url})")

    fragment = f"#{parsed.fragment}" if parsed.fragment else ""
    return f"{target.as_uri()}{fragment}"


def check_links(
    inputs: Sequence[str],
    repo_root: Path,
    lychee: str = "lychee",
    runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> int:
    dumped = runner(
        [lychee, "--dump", *inputs],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        text=True,
        check=False,
    )
    if dumped.returncode != 0:
        return dumped.returncode
    urls = self_links(dumped.stdout)
    if urls:
        local_document = "".join(f"[self-link](<{_local_url(url, repo_root)}>)\n" for url in urls)
        local = runner(
            [lychee, "--no-progress", "--include-fragments", "-"],
            cwd=repo_root,
            input=local_document,
            text=True,
            check=False,
        )
        if local.returncode != 0:
            return local.returncode

    command = [lychee, "--no-progress", "--include-fragments"]
    for url in urls:
        command.extend(["--exclude", f"^{re.escape(url)}$"])
    command.extend(inputs)
    return runner(command, cwd=repo_root, check=False).returncode


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check links, resolving canonical same-repository main-branch URLs locally."
    )
    parser.add_argument("--lychee", default="lychee", help="lychee executable (default: lychee)")
    parser.add_argument("inputs", nargs="+", help="files and directories passed to lychee")
    args = parser.parse_args(argv)

    try:
        return check_links(args.inputs, Path.cwd(), args.lychee)
    except SelfLinkError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
