#!/usr/bin/env python3
"""Move a spec section and repoint every site that addresses it. §REQ-spec-section-names.permanence

A section that moves takes its citing sites with it. Nothing in `grund` does
that — `fmt` repairs a link once the citation beside it is right, but it will not
change which coordinate a site names — so a migration of the size
[agent-grounds/grund#260](https://github.com/agent-grounds/grund/issues/260) asks
for is this table plus one pass over the tree.

What it rewrites:

- the `§`-marked coordinate token, at any depth, in every tracked text file;
- the unmarked coordinate that an e2e case's `spec.refs` manifest holds, in both
  the current `FS-check.…` and the legacy `FS-001-check.…` spelling. The manifest
  is the coverage gate's evidence (§AR-goal-measurement.1) and is read by no
  scanner, so it is invisible to every citation-aware tool and has to be named
  here or the evidence stays behind while the text moves.

What it refuses to touch, and why each refusal is load-bearing:

- the escape form `<§>FS-check.4.6`. It is how prose writes the *shape* of a
  citation without making one (§FS-check.2.3.1), and `docs/changelog/0.13.0.md`
  uses it to report what a coordinate meant in an earlier release. Rewriting
  those turns a historical record into a false one;
- the e2e fixture trees, `tests/e2e/cases/*/repo/` and `expected.repo/`. A
  coordinate in there is a test input, and rewriting it changes what the case
  proves;
- link targets and `#anchor` fragments. `grund fmt --write` re-derives both from
  the citation beside them under `[fmt.cross_refs]` (§FS-fmt.6), so this pass
  writes the coordinate and `fmt` writes the link. **Run `grund fmt --write`
  after this script, never interleaved with it.**

    scripts/move_spec_sections.py --dry-run
    scripts/move_spec_sections.py && grund fmt --write
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

# The move this run performs. `FS-check`'s ten declaration checks become
# sections of `FS-declarations`, each named by its diagnostic code
# (§REQ-spec-section-names.code).
CHECK_SECTIONS = {
    "3.3": "duplicate",
    "3.4": "broken-stub",
    "3.7": "misplaced-declaration",
    "3.9": "section-heading-level",
    "3.16": "duplicate-section",
    "3.19": "orphan-section",
    "3.23": "section-outside-declaration",
    "4.6": "declaration-near-miss",
    "4.13": "oversized-lead",
    "4.14": "unmarked-heading",
}

MOVES = {
    f"FS-check.{section}": f"FS-declarations.checks.{code}"
    for section, code in CHECK_SECTIONS.items()
}

# The `spec.refs` manifests still carry the pre-slug ID this repository declared
# before `[id] format = "{kind}-{slug}"`; both spellings name the same section.
MANIFEST_ALIASES = ("FS-check", "FS-001-check")

# Fixture trees are test inputs, not citations of this repository's spec.
EXCLUDED_DIRECTORIES = ("repo", "expected.repo")

SOURCES = sorted(MOVES, key=lambda source: (-len(source), source))
# A sub-point moves with its parent: `FS-check.4.14.3` becomes
# `FS-declarations.checks.unmarked-heading.3`. The lookbehind is the escape
# refusal — `<§>` is not a citation. Sources are tried longest-first and the
# trailing guard refuses a further digit, so `3.4` never matches inside a deeper
# coordinate while a citation that ends a sentence still does.
MARKED = re.compile(
    r"(?<!<)§(" + "|".join(re.escape(source) for source in SOURCES) + r")((?:\.\d+)*)(?!\d)"
)
MANIFEST = re.compile(
    r"^(" + "|".join(re.escape(alias) for alias in MANIFEST_ALIASES) + r")\.("
    + "|".join(re.escape(section) for section in sorted(CHECK_SECTIONS, key=len, reverse=True))
    + r")((?:\.\d+)*)$"
)


def tracked_files() -> list[Path]:
    """Every tracked file, fixture trees left out."""
    listing = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files", "-z"],
        capture_output=True,
        check=True,
        text=True,
    ).stdout
    files = []
    for name in listing.split("\0"):
        if not name:
            continue
        path = Path(name)
        if any(part in EXCLUDED_DIRECTORIES for part in path.parts):
            continue
        files.append(REPO_ROOT / path)
    return files


def rewrite_marked(text: str) -> str:
    """Repoint every marked citation of a moved coordinate."""
    return MARKED.sub(lambda match: "§" + MOVES[match.group(1)] + match.group(2), text)


def rewrite_manifest(text: str) -> str:
    """Repoint an e2e case manifest, whose coordinates carry no marker."""
    lines = text.split("\n")
    for index, line in enumerate(lines):
        match = MANIFEST.match(line.strip())
        if match:
            lines[index] = MOVES[f"FS-check.{match.group(2)}"] + match.group(3)
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="report the files that would change")
    arguments = parser.parse_args(argv)

    changed = 0
    sites = 0
    for path in tracked_files():
        try:
            before = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        after = rewrite_manifest(before) if path.name == "spec.refs" else rewrite_marked(before)
        if after == before:
            continue
        moved = sum(
            1
            for line_before, line_after in zip(before.split("\n"), after.split("\n"))
            if line_before != line_after
        )
        changed += 1
        sites += moved
        print(f"{path.relative_to(REPO_ROOT)}: {moved} line(s)")
        if not arguments.dry_run:
            path.write_text(after, encoding="utf-8")
    verb = "would change" if arguments.dry_run else "changed"
    print(f"{verb} {sites} line(s) in {changed} file(s)")
    if not arguments.dry_run:
        print("now run `grund fmt --write` so every link target and anchor follows")
    return 0


if __name__ == "__main__":
    sys.exit(main())
