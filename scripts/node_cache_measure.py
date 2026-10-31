#!/usr/bin/env python3
"""Dispatch, collect and recover isolated native experiments (§AR-ci.requirements.1.verification)."""

import argparse
import json
from pathlib import Path
import re
import subprocess

from node_cache_experiment_ledger import REPOSITORY, REF, SYSTEMS, api, recover, write

BASE = "d338c4814a095fef54708e83367597c4e7955aad"
CANDIDATE = "6fe17f9d51b666758df0d75f66ea689cffdc434b"


def run(argv, **options):
    return subprocess.run(list(map(str, argv)), check=True, **options)


def dispatch(arguments):
    """The existing registered route; reject scope/ref drift before any repetitions."""
    for sha in (arguments.source, arguments.harness):
        if not re.fullmatch("[0-9a-f]{40}", sha):
            raise ValueError("source and harness must be immutable 40-character SHAs")
    if arguments.source not in (BASE, CANDIDATE):
        raise ValueError("source is outside the fixed base/corrected-candidate pair")
    if not re.fullmatch("[a-zA-Z0-9-]{1,80}", arguments.label):
        raise ValueError("use a unique alphanumeric/hyphen sample label")
    if api("git/ref/heads/" + REF.removeprefix("refs/heads/"))["object"]["sha"] != arguments.harness:
        raise ValueError("permitted fork branch does not point at the pinned harness")
    seed_attempt = None
    if arguments.condition != "seed":
        if not arguments.seed_run:
            raise ValueError("transaction validation and samples require a source-matched seed run")
        seed = api(f"actions/runs/{arguments.seed_run}")
        if (seed["conclusion"] != "success" or seed["head_sha"] != arguments.harness
                or seed["head_branch"] != REF.removeprefix("refs/heads/")
                or seed["event"] != "workflow_dispatch"):
            raise ValueError("seed is not a successful dispatch at this harness/ref")
        seed_attempt = seed["run_attempt"]
    config = {"experiment": "grund-1122", "source": arguments.source, "harness": arguments.harness,
              "condition": arguments.condition, "seed_run": arguments.seed_run, "seed_attempt": seed_attempt,
              "label": arguments.label, "os": arguments.os,
              "node_systems": [SYSTEMS[arguments.os]] if arguments.os != "macOS" else []}
    payload = {"ref": REF.removeprefix("refs/heads/"),
               "inputs": {"node-cache-experiment": json.dumps(config)}}
    run(["gh", "api", f"repos/{REPOSITORY}/actions/workflows/pre-release-checks.yml/dispatches",
         "--method", "POST", "--input", "-"], input=json.dumps(payload), text=True)


def collect(arguments):
    """Retain complete forge times/logs and artifacts before cleanup, even on failure."""
    destination = Path(arguments.out)
    destination.mkdir(parents=True, exist_ok=True)
    prefix = f"actions/runs/{arguments.run}"
    for endpoint, name in ((prefix, "run"), (prefix + "/jobs?per_page=100", "jobs"),
                           (prefix + "/artifacts?per_page=100", "artifacts")):
        write(destination / f"{name}.json", api(endpoint))
    with (destination / "logs.zip").open("wb") as output:
        run(["gh", "api", f"repos/{REPOSITORY}/{prefix}/logs"], stdout=output)
    run(["gh", "run", "download", arguments.run, "--repo", REPOSITORY,
         "--dir", destination / "artifacts"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for operation in ("restore", "save"):
        commands.add_parser(operation).add_argument("cache_id")
    for operation in ("initialize", "validate", "cleanup-local"):
        commands.add_parser(operation)
    send = commands.add_parser("dispatch")
    send.add_argument("--source", required=True)
    send.add_argument("--harness", required=True)
    send.add_argument("--condition", choices=("seed", "validate", "exact", "save"), required=True)
    send.add_argument("--os", choices=tuple(SYSTEMS), required=True)
    send.add_argument("--seed-run", default="")
    send.add_argument("--label", required=True)
    for operation in ("collect", "cleanup"):
        command = commands.add_parser(operation)
        command.add_argument("run")
        command.add_argument("--out", required=True)
    analyze = commands.add_parser("analyze")
    analyze.add_argument("directory")
    analyze.add_argument("--out", required=True)
    analyze.add_argument("--preflight", action="store_true")
    analyze.add_argument("--os", choices=tuple(SYSTEMS))
    args = parser.parse_args()
    if args.command == "dispatch":
        dispatch(args)
    elif args.command == "collect":
        collect(args)
    elif args.command == "cleanup":
        recover(args.run, args.out)
    elif args.command == "analyze":
        from node_cache_measure_analysis import analyze
        if args.preflight and not args.os:
            raise ValueError("preflight analysis requires --os")
        analyze(args.directory, args.out, args.os if args.preflight else None)
    else:
        from node_cache_measure_runner import cleanup_local, initialize, transaction, validate
        if args.command in ("restore", "save"):
            transaction(args.command, args.cache_id)
        else:
            {"initialize": initialize, "validate": validate, "cleanup-local": cleanup_local}[args.command]()


if __name__ == "__main__":
    main()
