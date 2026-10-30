#!/usr/bin/env python3
"""Check that the tree's messages permit a release. §FS-distribution.4.2.5

A ramp is a promise written into a message. A warning names the release it
becomes an error in, and once the ramp lands the error names the release the
change was made in. Both halves are claims about a version, and a release that
disagrees with one publishes a message the binary it ships in is not.

With no positional release, check that the message bounds and shared minimum
publishable release (§FS-distribution.release-floor) leave a nonempty window.
Exit 1 if no release is possible, without choosing a candidate version.

Given the version about to be cut, also check what the core-concerns plan still
owes it: the deprecation ledger's notice statuses and the phase leads
(§FS-distribution.4.2.2.1). Exit 1 when a message, the shared floor, a notice or
a phase disagrees with that release.

Usage: check_release_ramps.py [release] [--root <tree>]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable, NamedTuple, Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))
from deprecation_ledger import notice_problems, phase_lead_problems


REPO_ROOT = Path(__file__).resolve().parents[1]
FLOOR_PATH = "scripts/release-floor.txt"

# The message text: what a user is shown, and the goldens that pin those bytes.
SOURCES = (("crates", "*.rs"), ("tests/e2e/cases", "expected.stdout"), ("tests/e2e/cases", "expected.stderr"))

PENDING = "pending"
LANDED = "landed"

# The clause vocabulary is closed (§FS-distribution.4.2.3): a ramp is written in
# it or this gate does not see it. Each entry is the wording the messages
# already use, and the direction the release named in it constrains.
CLAUSES = (
    ("becomes an error in", PENDING),
    ("became an error in", LANDED),
    # The two tenses of one removal (§FS-distribution.4.2.3): a deprecation names
    # the release its symbol stops working in, and the message that replaces it
    # names the release the removal was made in.
    ("is removed in", PENDING),
    ("was removed in", LANDED),
    ("stopped loading in", LANDED),
    ("unchecked in", LANDED),
    # A wording ramp has only the pending tense (§FS-distribution.4.2.2): the
    # message promising its own wording changes at a named release is rewritten
    # by then, and what replaces it says nothing about when it changed.
    ("wording changes in", PENDING),
)

PATTERNS = tuple(
    (re.compile(re.escape(clause) + r"\s+(?:grund\s+)?\*{0,2}(\d+\.\d+\.\d+)"), clause, direction)
    for clause, direction in CLAUSES
) + (
    # §FS-distribution.4.2.3.1: the landed half of an attribution pair is the bare
    # clause, and it is a substring of both named-error tenses — so it is read only
    # where it opens a clause, never where a word stands in front of it.
    (
        re.compile(r"(?:^|[,;:])\s*an error in\s+(?:grund\s+)?\*{0,2}(\d+\.\d+\.\d+)"),
        "an error in",
        LANDED,
    ),
    # §FS-distribution.4.2.4: scalar migrations say what a command "will exit"
    # before the final `in grund <release>` deadline rather than putting the
    # release directly after the verb phrase.
    (
        re.compile(r"\bwill exit\b[^\n]*?\bin\s+(?:grund\s+)?\*{0,2}(\d+\.\d+\.\d+)"),
        "will exit … in",
        PENDING,
    ),
)

RELEASE_RE = re.compile(r"^\d+\.\d+\.\d+$")


class Claim(NamedTuple):
    path: str
    line: int
    clause: str
    release: str
    direction: str

    def quoted(self) -> str:
        return f"{self.path}:{self.line}: `{self.clause} {self.release}`"


def version(text: str) -> tuple[int, ...]:
    return tuple(int(part) for part in text.split("."))


def scan_text(path: str, text: str) -> Iterable[Claim]:
    for number, line in enumerate(text.splitlines(), start=1):
        for pattern, clause, direction in PATTERNS:
            for match in pattern.finditer(line):
                yield Claim(path, number, clause, match.group(1), direction)


def scan_tree(root: Path) -> list[Claim]:
    claims: list[Claim] = []
    for directory, glob in SOURCES:
        base = root / directory
        if not base.is_dir():
            continue
        for file in sorted(base.rglob(glob)):
            text = file.read_text(encoding="utf-8", errors="replace")
            claims.extend(scan_text(file.relative_to(root).as_posix(), text))
    return claims


def read_release_floor(root: Path) -> str | None:
    """Read the single shared floor from the checked tree. §FS-distribution.release-floor"""
    path = root / FLOOR_PATH
    if not path.exists():
        return None
    lines = path.read_text(encoding="utf-8").splitlines()
    if len(lines) != 1 or not RELEASE_RE.fullmatch(lines[0]):
        raise ValueError(f"{FLOOR_PATH} must contain one release such as 0.19.0 on one line")
    return lines[0]


def release_window(
    claims: Sequence[Claim], shared_floor: str | None = None,
) -> tuple[str | None, str | None]:
    """Combine the inclusive floor and exclusive message ceiling. §FS-distribution.4.2.5"""
    landed = [claim.release for claim in claims if claim.direction == LANDED]
    if shared_floor is not None:
        landed.append(shared_floor)
    pending = [claim.release for claim in claims if claim.direction == PENDING]
    floor = max(landed, key=version) if landed else None
    ceiling = min(pending, key=version) if pending else None
    return floor, ceiling


def violations(claims: Sequence[Claim], release: str) -> tuple[list[Claim], list[Claim]]:
    cut = version(release)
    behind = [c for c in claims if c.direction == LANDED and cut < version(c.release)]
    ahead = [c for c in claims if c.direction == PENDING and cut >= version(c.release)]
    return behind, ahead


def report(claims: Sequence[Claim], release: str | None, shared_floor: str | None = None) -> list[str]:
    """Name every conflicting bound and the window left. §FS-distribution.4.2.5"""
    floor, ceiling = release_window(claims, shared_floor)
    empty = floor is not None and ceiling is not None and version(floor) >= version(ceiling)
    if release is None:
        if not empty:
            return []
        behind = [c for c in claims if c.direction == LANDED and c.release == floor]
        ahead = []
        lines = ["error: this tree cannot be released"]
    else:
        behind, ahead = violations(claims, release)
        below_floor = shared_floor is not None and version(release) < version(shared_floor)
        if not behind and not ahead and not below_floor:
            return []
        lines = [f"error: this tree cannot be released as {release}"]

    if shared_floor is not None:
        lines.append("")
        lines.append(f"  {FLOOR_PATH}:1: minimum publishable release is {shared_floor}.")
    if behind:
        lines.append("")
        if release is None:
            lines.append("  these lines set the earliest release for changes already made:")
        else:
            lines.append("  these lines say a change has already been made in a later release,")
            lines.append("  so cutting this version would ship it under a version that denies it:")
        lines.extend(f"    {claim.quoted()}" for claim in behind)
    if ahead:
        lines.append("")
        lines.append("  these lines promise a change at a release this version has reached,")
        lines.append("  so cutting this version would break the promise rather than keep it:")
        lines.extend(f"    {claim.quoted()}" for claim in ahead)

    lines.append("")
    if empty:
        basis = "landed changes and its shared floor" if shared_floor else "the changes it has already made"
        # §FS-distribution.4.2.5: every pending line at the ceiling, each once, not one example.
        ceiling_lines = [c for c in claims if c.direction == PENDING and c.release == ceiling]
        held = [c for c in ceiling_lines if c not in ahead]
        lines.append(
            f"  this tree can be cut as no release at all: at or above {floor} for {basis}, "
            f"and below {ceiling} for the ones it still only promises"
            + (" —" if held else ", listed above.")
        )
        lines.extend(f"    {claim.quoted()}" for claim in held)
        count = len(ceiling_lines)
        ramps = "that ramp" if count == 1 else f"all {count} ramps naming {ceiling}"
        lines.append(f"  land {ramps} too before cutting anything.")
    else:
        window = f"at or above {floor}" if floor else "any release"
        if ceiling:
            window += f" and below {ceiling}"
        lines.append(f"  this tree can be cut as {window}.")
        if shared_floor:
            lines.append("  respect the shared floor and land or revert the ramps above rather than moving the version.")
        else:
            lines.append("  land or revert the ramps above rather than moving the version.")
    return lines


def main(argv: Sequence[str] | None = None) -> int:
    """Check a candidate, or just that some release exists. §FS-distribution.4.2.5"""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "release", nargs="?",
        help="the version about to be cut, without the leading v; omit to check that some release exists",
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="repository root to scan")
    args = parser.parse_args(argv)

    if args.release is not None and not RELEASE_RE.match(args.release):
        print(f"error: release must look like 0.13.0, got '{args.release}'", file=sys.stderr)
        return 2

    # §FS-distribution.release-floor: manifest rewrites cannot lower the tree's floor.
    try:
        shared_floor = read_release_floor(args.root)
        # §FS-distribution.4.2.2.1: only an explicit candidate has the plan's deadlines.
        ledger_problems = (
            notice_problems(args.root, args.release) + phase_lead_problems(args.root, args.release)
            if args.release is not None else []
        )
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    lines = report(scan_tree(args.root), args.release, shared_floor)
    lines.extend(ledger_problems)
    if lines:
        print("\n".join(lines), file=sys.stderr)
        return 1
    if args.release is None:
        print("the messages and shared floor permit at least one release.")
    else:
        print(f"no message in this tree contradicts a release of {args.release}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
