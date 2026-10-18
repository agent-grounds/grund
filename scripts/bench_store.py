#!/usr/bin/env python3
"""Keep a push to `main`'s run record for good on the `benchmark-data` branch.

§AR-ci.5.5.3: add the record as `<tier>/<commit>.json` on top of the branch's tip, byte for
byte as the record job merged it (§AR-benchmarks.7.5), and change nothing else.

    bench_store.py --record <first.json> --commit <sha> [--bench-record-conclusion <c>]
        stores the record of <sha> on `benchmark-data` of the `origin` of the checkout it
        runs in; `bench-store.yml` runs it, and so can a person (§AR-ci.5.5.5)

It never checks the branch out, never creates it, and pushes without force. A commit's first
stored record stands. The record is checked by the `bench_record.py` beside this script: the
workflow runs it from a checkout of the record's commit (§AR-ci.5.5.1), and a person from any
checkout whose identity fields are the ones the record was written under.

Exits 0 when the record is stored, was already stored, or does not exist because the record
job failed, was cancelled or was skipped (§AR-ci.5.5.2); 1 naming what refused it, with
nothing stored."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench_record  # noqa: E402

REMOTE, BRANCH = "origin", "benchmark-data"
# §AR-ci.5.5.2: the conclusions of a record job that wrote no record. After any other,
# `success`, `timed_out` and a conclusion the forge did not report among them, a missing
# record is lost.
NO_RECORD = ("failure", "cancelled", "skipped")
# §AR-benchmarks.7.5: the only record the store keeps, a push's, as CI's record job wrote it.
ENVIRONMENT, TIER = "ci", "pr"
# §AR-ci.5.5.3: the pushes a store makes before a race it keeps losing fails it, and the
# pause before each push after the first.
PUSHES, PAUSE_SECONDS = 5, 1.0


class Refused(Exception):
    """The store stored nothing, for the reason the message gives."""


def git(*arguments: str, stdin: bytes | None = None, env: dict | None = None,
        check: bool = True) -> subprocess.CompletedProcess:
    """`git` as `PATH` finds it, in the checkout the store runs in."""
    result = subprocess.run(["git", *arguments], input=stdin, capture_output=True, env=env)
    result.stdout = result.stdout.decode("utf-8", "replace").strip()
    result.stderr = result.stderr.decode("utf-8", "replace").strip()
    if check and result.returncode:
        raise Refused(f"`git {' '.join(arguments)}` failed: {result.stderr}")
    return result


def stored_path(record: object, commit: str) -> str:
    """§AR-benchmarks.7.5: where the record of `commit` is stored, once §AR-ci.5.5.3's
    refusals have passed it: its `commit` is the one given, its `series` is the key of its
    own identity fields (§AR-benchmarks.7.2), and it is a `ci` record of the `pr` tier, the
    one kind CI's record job writes for a push."""
    if not isinstance(record, dict):
        raise Refused("the record is not a JSON object")
    if record.get("commit") != commit:
        raise Refused(f"the record measured {record.get('commit')!r}, not {commit}")
    if record.get("series") != bench_record.series_key(record):
        raise Refused(f"the record's series {record.get('series')!r} is not the key of its "
                      "identity fields")
    if record.get("environment") != ENVIRONMENT:
        raise Refused(f"the record's environment is {record.get('environment')!r}, not "
                      f"`{ENVIRONMENT}`: CI's record job wrote no such record")
    tier = bench_record.tier_of(record)
    if tier != TIER:
        raise Refused(f"the record's tier is {tier!r}, not `{TIER}`: CI's record job of a push "
                      "wrote no such record")
    return f"{tier}/{commit}.json"


def fetch_tip() -> str:
    """The branch's tip on `origin`. Without the branch the store fails: a person creates it
    (§AR-ci.5.5.4), and the store never does."""
    fetched = git("fetch", "--quiet", "--no-tags", REMOTE, f"refs/heads/{BRANCH}", check=False)
    if fetched.returncode:
        raise Refused(f"cannot fetch `{BRANCH}` from `{REMOTE}`, and the store never creates "
                      f"it: {fetched.stderr}")
    return git("rev-parse", "--verify", "FETCH_HEAD^{commit}").stdout


def holds(tip: str, path: str) -> bool:
    return git("cat-file", "-e", f"{tip}:{path}", check=False).returncode == 0


def add(tip: str, path: str, data: bytes) -> str:
    """§AR-ci.5.5.3: one commit whose parent is `tip` and whose tree is the tip's plus `path`,
    built without checking the branch out, and checked to add exactly that file."""
    blob = git("hash-object", "-w", "--stdin", stdin=data).stdout
    with tempfile.TemporaryDirectory(prefix="bench-store-") as scratch:
        index = dict(os.environ, GIT_INDEX_FILE=os.path.join(scratch, "index"))
        git("read-tree", tip, env=index)
        git("update-index", "--add", "--cacheinfo", f"100644,{blob},{path}", env=index)
        tree = git("write-tree", env=index).stdout
    commit = git("commit-tree", tree, "-p", tip, "-m", f"Store {path}").stdout
    added = git("diff-tree", "-r", "--no-renames", "--name-status", tip, commit).stdout
    if added != f"A\t{path}":
        raise Refused(f"the commit for {path} would change more than that one file: {added}")
    return commit


def store(path: str, data: bytes) -> str:
    """§AR-ci.5.5.3: add `path` on the tip and push it without force. A push refused because
    the tip moved is built again on the new tip, up to `PUSHES` pushes."""
    tip, pushes = fetch_tip(), 0
    while not holds(tip, path):
        if pushes == PUSHES:
            raise Refused(f"`{BRANCH}` moved before each of {PUSHES} pushes")
        if pushes:
            time.sleep(PAUSE_SECONDS)
        pushes += 1
        commit = add(tip, path, data)
        pushed = git("push", "--quiet", REMOTE, f"{commit}:refs/heads/{BRANCH}", check=False)
        if pushed.returncode == 0:
            return f"stored {path} on `{BRANCH}` as {commit}"
        before, tip = tip, fetch_tip()
        if tip == before:
            raise Refused(f"`{BRANCH}` refused the push and did not move: {pushed.stderr}")
    return f"{path} is already on `{BRANCH}`: its first record stands, and nothing was pushed"


def parse_args(argv: list) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--record", type=Path, required=True,
                        help="the record CI's record job merged: `first.json` of its artifact")
    parser.add_argument("--commit", required=True, help="the full sha the record measured")
    parser.add_argument("--bench-record-conclusion", default="success",
                        help="how CI's record job ended, as the forge reports it (default: "
                             "success, so a missing record fails)")
    return parser.parse_args(argv)


def main(argv: list) -> int:
    args = parse_args(argv)
    try:
        if not re.fullmatch(r"[0-9a-f]{40}", args.commit):
            raise Refused(f"`--commit {args.commit}` is not a full commit sha")
        if not args.record.is_file():
            conclusion = args.bench_record_conclusion
            if conclusion in NO_RECORD:
                print(f"the record job ended `{conclusion}`, so there is no record to store")
                return 0
            raise Refused(f"no record at {args.record}, though the record job ended "
                          f"`{conclusion or 'unreported'}`: that record is lost")
        try:
            record = bench_record.load(args.record)
        except ValueError as error:
            raise Refused(f"{args.record} is not a run record: {error}") from None
        print(store(stored_path(record, args.commit), args.record.read_bytes()))
        return 0
    except Refused as refused:
        print(f"bench_store: {refused}; nothing was stored", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
