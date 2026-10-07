#!/usr/bin/env python3
"""The cross-registry candidate tool: plan, assemble, verify and version one candidate.

§FS-distribution-candidate.2.1 and §FS-distribution-candidate.6 are what each command
answers to; the shape of the tool is §AR-bindings.5's. Every command here works on
files only. Nothing in this directory reaches a registry except `publish.py`, and
that one never builds (§FS-distribution-candidate.8.3).

    candidate.py matrix
    candidate.py plan --version V --sha S [--row R]
    candidate.py npm-trees --version V --out DIR [--sources --sha S]
    candidate.py verify DIR [--release] [--sha S] [--tag vV]
    candidate.py versions [--root DIR] [--ref REF] [--expect V]
    candidate.py set-version V [--root DIR]
    candidate.py build --row R --sha S --out DIR --target-dir DIR [--product P]...
    candidate.py assemble --sha S --out DIR ROW_CANDIDATE...
    candidate.py share CANDIDATE --row R --out DIR
    candidate.py receipts CANDIDATE SHARE...
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import matrix  # noqa: E402
import plan  # noqa: E402


def _print(value):
    sys.stdout.write(json.dumps(value, indent=2) + "\n")


def cmd_matrix(_):
    """§FS-distribution-candidate.1.1: the table, as JSON."""
    _print(matrix.rows())


def cmd_plan(args):
    """§FS-distribution-candidate.2.1: the inventory, before anything is built."""
    rows = [matrix.row(args.row)] if args.row else None
    _print(plan.plan(args.version, args.sha, rows))


def cmd_npm_trees(args):
    import npm_trees
    if args.sources and not args.sha:
        raise SystemExit("error: --sources needs --sha, the commit the sources are recorded as")
    npm_trees.write(Path(args.out), args.version, sha=args.sha, bundle=args.sources)


def cmd_verify(args):
    import verify
    problems = verify.verify(Path(args.candidate), release=args.release, sha=args.sha, tag=args.tag)
    for problem in problems:
        sys.stderr.write(f"error: {problem}\n")
    if problems:
        raise SystemExit(1)
    print(f"ok: {args.candidate} verifies")


def cmd_versions(args):
    import versions
    version = versions.check(Path(args.root), ref=args.ref, expect=args.expect)
    print(version)


def cmd_set_version(args):
    import versions
    versions.set_version(Path(args.root), args.version)


def cmd_build(args):
    import build
    build.build(matrix.row(args.row), args.sha, Path(args.out), Path(args.target_dir),
                products=args.product or None)


def cmd_assemble(args):
    import assemble
    assemble.assemble([Path(p) for p in args.rows], args.sha, Path(args.out))


def cmd_share(args):
    import assemble
    assemble.share(Path(args.candidate), args.row, Path(args.out))


def cmd_receipts(args):
    import assemble
    assemble.receipts(Path(args.candidate), [Path(p) for p in args.shares])


def main(argv=None):
    parser = argparse.ArgumentParser(prog="candidate.py", description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("matrix").set_defaults(run=cmd_matrix)
    p = sub.add_parser("plan")
    p.add_argument("--version", required=True)
    p.add_argument("--sha", required=True)
    p.add_argument("--row")
    p.set_defaults(run=cmd_plan)
    p = sub.add_parser("npm-trees")
    p.add_argument("--version", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--sources", action="store_true",
                   help="also bundle the locked source closure `build:source` builds from")
    p.add_argument("--sha")
    p.set_defaults(run=cmd_npm_trees)
    p = sub.add_parser("verify")
    p.add_argument("candidate")
    p.add_argument("--release", action="store_true")
    p.add_argument("--sha")
    p.add_argument("--tag")
    p.set_defaults(run=cmd_verify)
    p = sub.add_parser("versions")
    p.add_argument("--root", default=str(matrix.REPO))
    p.add_argument("--ref")
    p.add_argument("--expect")
    p.set_defaults(run=cmd_versions)
    p = sub.add_parser("set-version")
    p.add_argument("version")
    p.add_argument("--root", default=str(matrix.REPO))
    p.set_defaults(run=cmd_set_version)
    p = sub.add_parser("build")
    p.add_argument("--row", required=True)
    p.add_argument("--sha", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--target-dir", required=True)
    p.add_argument("--product", action="append", choices=plan.PRODUCTS)
    p.set_defaults(run=cmd_build)
    p = sub.add_parser("assemble")
    p.add_argument("--sha", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("rows", nargs="+", metavar="ROW_CANDIDATE")
    p.set_defaults(run=cmd_assemble)
    p = sub.add_parser("share")
    p.add_argument("candidate")
    p.add_argument("--row", required=True)
    p.add_argument("--out", required=True)
    p.set_defaults(run=cmd_share)
    p = sub.add_parser("receipts")
    p.add_argument("candidate")
    p.add_argument("shares", nargs="+", metavar="SHARE")
    p.set_defaults(run=cmd_receipts)
    args = parser.parse_args(argv)
    args.run(args)


if __name__ == "__main__":
    main()
