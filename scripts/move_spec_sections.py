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
- a file git does not track. The write set is exactly the tracked set, because
  `git diff` is how a person reviews a pass this size and an untracked file is
  invisible to it. So an untracked, non-ignored file holding a moved coordinate
  is not rewritten silently: the run looks for one *before* it writes anything
  and, finding one, names it, says `git add -N <path>`, writes nothing and exits
  non-zero, because the old coordinate it kept still resolves through the pointer
  the move left behind and no gate below this one will say so. The refusal comes
  first so that each exit code means one thing: non-zero and the tree is
  untouched, so there is nothing for `fmt` to follow; zero and every tracked site
  moved, so the `fmt` after the `&&` below is the other half of the pass.

With `--record`, the table is the `folds` and `moves` of one or more rewrite
records, and the pass is the one a fold needs (§REQ-spec-content.fold, the rule
it serves): a folded section leaves no pointer, so every site must move in the
same change. Each removed coordinate goes to its listed target, and an unlisted
sub-point of one goes to the same target. On top of the two rewrites above, it:

- escapes, rather than retargets, a removed coordinate in a released changelog
  (`docs/changelog/*.md`, and `docs/changelog.md` from its first release on), so a
  released record keeps the coordinate it shipped with;
- reads every coordinate of a manifest line holding several, writes one it retargets
  from the `FS-00N-` spelling in the current one, and keeps a line once where the
  retarget makes it repeat;
- rewrites the `spec.refs` bytes of each `command_agreement_facts/<case>.json`
  whose manifest changed, since `agreement_expectations.rs` compares them;
- retargets the bare coordinates no scanner reads: `citation_ledger.toml`
  points, the cited half of a `POINTERS` row, a moves test's `CHAPTERS`, and the
  schema's `x-grund-spec` id and href, then re-renders the configuration
  reference from the schema;
- deletes the coverage exception rows of removed leaves;
- repoints every link to a heading anchor the rewrite removed or retitled,
  wherever it is written, a commit-pinned GitHub link aside, because `fmt`
  re-derives only a link beside a citation.

It reads the section maps itself, at `--base` with `git show` and in the working
tree, and refuses records it cannot apply safely before it writes anything, with
exit status 2: a line it cannot read, a fold whose target is no ancestor, a target
that is itself removed or absent, a source the base lacks or the tree still has.
A removed section no record lists is printed as not retargeted, and the pass goes
on, since another page's rewrite may still be in progress. What it cannot rewrite
safely it reports, one site per line: a bare mention elsewhere, a removed section
named in a run after a citation, a cited sub-point a record does not list, a moved
section's relative name in a test row, a link to a removed heading that was no
coordinate, an exception row it added, a `POINTERS` row whose pointer is gone, and
every `pins` entry of a record. `--check` writes nothing: it compares each touched
FS declaration's section map at the base with the tree's and exits 1 on a removed
coordinate no record lists or a record problem.

    scripts/move_spec_sections.py --dry-run
    scripts/move_spec_sections.py && grund fmt --write
    scripts/move_spec_sections.py --check --base <commit> --record <record.md>...
    scripts/move_spec_sections.py --base <commit> --record <record.md>... && grund fmt --write
"""

from __future__ import annotations

import argparse
import difflib
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import spec_fold_map as folds  # noqa: E402
from spec_fold_pass import SCHEMA, FoldPass, read_text  # noqa: E402


REPO_ROOT = Path(__file__).resolve().parents[1]

# The move this run performs: the absorbed-scan warning of `FS-check.4.7` became
# the config error of `FS-check.3.30` in grund 0.16.0. Its
# sub-points are renumbered, and two fold into a sibling. `4.7.9` stays where it
# is, as the pointer released changelogs cite, so it maps to itself. Run with the
# changelogs restored afterwards (`git checkout docs/changelog.md docs/changelog/`):
# a released record keeps the coordinate it shipped with.
CHECK_SECTIONS = {
    "4.7": "FS-check.3.30",
    "4.7.1": "FS-check.3.30.1",
    "4.7.2": "FS-check.3.30.2",
    "4.7.3": "FS-check.3.30.3",
    "4.7.4": "FS-check.3.30.2",
    "4.7.5": "FS-check.3.30.4",
    "4.7.6": "FS-check.3.30.5",
    "4.7.7": "FS-check.3.30",
    "4.7.8": "FS-check.3.30",
    "4.7.9": "FS-check.4.7.9",
}

MOVES = {f"FS-check.{section}": target for section, target in CHECK_SECTIONS.items()}

# The `spec.refs` manifests still carry the pre-slug ID this repository declared
# before `[id] format = "{kind}-{slug}"`; both spellings name the same section.
MANIFEST_ALIASES = ("FS-check", "FS-001-check")

# Fixture trees are test inputs, not citations of this repository's spec.
EXCLUDED_DIRECTORIES = ("repo", "expected.repo")

SOURCES = sorted(MOVES, key=lambda source: (-len(source), source))
# A sub-point the table does not name moves with its parent. The lookbehind is
# the escape refusal — `<§>` is not a citation. Sources are tried longest-first
# and the trailing guard refuses a further digit, so `4.7` never matches inside a
# deeper coordinate while a citation that ends a sentence still does.
MARKED = re.compile(
    r"(?<!<)§(" + "|".join(re.escape(source) for source in SOURCES) + r")((?:\.\d+)*)(?!\d)"
)
MANIFEST = re.compile(
    r"^(" + "|".join(re.escape(alias) for alias in MANIFEST_ALIASES) + r")\.("
    + "|".join(re.escape(section) for section in sorted(CHECK_SECTIONS, key=len, reverse=True))
    + r")((?:\.\d+)*)$"
)

# What renders the configuration reference from the schema, run after a fold rewrites it.
RENDER_REFERENCE = "scripts/render_config_reference.py"


def listed_files(*selectors: str, root: Path = REPO_ROOT) -> list[Path]:
    """What `git ls-files` returns under these selectors, fixture trees left out."""
    listing = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", *selectors],
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
        files.append(root / path)
    return files


def tracked_files(root: Path = REPO_ROOT) -> list[Path]:
    """Every tracked file this pass rewrites."""
    return listed_files(root=root)


def untracked_files(root: Path = REPO_ROOT) -> list[Path]:
    """Every file git neither tracks nor ignores — reported, never rewritten."""
    return listed_files("--others", "--exclude-standard", root=root)


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


def repointed(path: Path) -> tuple[str, str] | None:
    """The file's text before and after this pass, or `None` when it is untouched."""
    before = read_text(path)
    if before is None:
        return None
    after = rewrite_manifest(before) if path.name == "spec.refs" else rewrite_marked(before)
    return None if after == before else (before, after)


def changed_lines(before: str, after: str) -> int:
    diff = difflib.unified_diff(before.split("\n"), after.split("\n"), n=0, lineterm="")
    return sum(1 for line in diff if line.startswith("-") and not line.startswith("---"))


def stranded_refusal(stranded: list[Path], root: Path) -> int:
    # An untracked file is invisible to `git diff`, so a coordinate it keeps survives the
    # migration through the pointer left behind (§REQ-spec-section-names.permanence); the
    # refusal precedes the write loop to keep `this && grund fmt --write` whole.
    print(f"{len(stranded)} untracked file(s) hold a moved coordinate; nothing was rewritten:")
    for path in stranded:
        relative = path.relative_to(root)
        print(f"  {relative} — `git add -N {relative}`, then run this pass again")
    return 1


def write_all(rewrites: dict[Path, tuple[str, str]], root: Path, dry_run: bool) -> None:
    sites_moved = 0
    for path, (before, after) in sorted(rewrites.items()):
        moved = changed_lines(before, after)
        sites_moved += moved
        print(f"{path.relative_to(root)}: {moved} line(s)")
        if not dry_run:
            path.write_text(after, encoding="utf-8")
    verb = "would change" if dry_run else "changed"
    print(f"{verb} {sites_moved} line(s) in {len(rewrites)} file(s)")


def move_table(root: Path, dry_run: bool) -> int:
    """The fixed move of `CHECK_SECTIONS`."""
    stranded = [path for path in untracked_files(root) if repointed(path) is not None]
    if stranded:
        return stranded_refusal(stranded, root)
    rewrites = {path: pair for path in tracked_files(root) if (pair := repointed(path))}
    write_all(rewrites, root, dry_run)
    # The hint is an instruction to a person, so a pass that wrote nothing does not give it.
    if rewrites and not dry_run:
        print("now run `grund fmt --write` so every link target and anchor follows")
    return 0


def read_records(paths: list[Path]) -> list[folds.Record] | None:
    try:
        return [folds.read_record(path) for path in paths]
    except (folds.RecordError, OSError) as error:
        print(error)
        return None


def fold_records(root: Path, paths: list[Path], base: str, dry_run: bool, check: bool) -> int:
    records = read_records(paths)
    if records is None:
        return 2
    before, after = folds.spec_at(root, base), folds.spec_in(root)
    refused = folds.problems(records, before, after)
    fold, _ = folds.fold_map(records)
    missing, notes = folds.compare(before, after, fold)
    for line in refused:
        print(line)
    for line in missing:
        print(line if check else f"not retargeted: {line}")
    if check:
        for line in notes:
            print(f"note: {line}")
        verdict = "fail" if refused or missing else "ok"
        print(f"check {verdict}: {len(fold.targets)} removed coordinate(s) recorded, "
              f"{len(missing)} in no record, {len(refused)} record problem(s)")
        return 1 if refused or missing else 0
    if refused:
        print(f"{len(refused)} record problem(s); nothing was rewritten")
        return 2
    fold_pass = FoldPass(root, records, before, after)
    stranded, rewrites = fold_pass.plan(tracked_files(root), untracked_files(root))
    if stranded:
        return stranded_refusal(stranded, root)
    write_all(rewrites, root, dry_run)
    if root / SCHEMA in rewrites and not dry_run:
        subprocess.run([sys.executable, str(root / RENDER_REFERENCE)], check=True)
        print(f"rendered the configuration reference from {SCHEMA}")
    fold_pass.report(rewrites and not dry_run)
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="report the files that would change")
    parser.add_argument("--record", action="extend", nargs="+", type=Path, default=[], metavar="FILE",
                        help="a rewrite record whose folds and moves replace the fixed table")
    parser.add_argument("--base", metavar="COMMIT", help="the commit the rewrite started from")
    parser.add_argument("--check", action="store_true",
                        help="fail when a section removed since --base is in no record; write nothing")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="the checkout to work on")
    arguments = parser.parse_args(argv)
    root = arguments.root.resolve()
    if (arguments.record or arguments.check) and not arguments.base:
        parser.error("--record and --check need --base, the commit the rewrite started from")
    if arguments.record or arguments.check:
        return fold_records(root, arguments.record, arguments.base, arguments.dry_run, arguments.check)
    return move_table(root, arguments.dry_run)


if __name__ == "__main__":
    sys.exit(main())
