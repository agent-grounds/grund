#!/usr/bin/env python3
"""Normalize an instruction-benchmark run into one run record, and compare records.

§AR-benchmarks.7: every run of `crates/grund-cli/benches/instructions.rs` becomes one
compact JSON record whose identity fields name what it was measured with, and two
records are compared only within one series.

  record   write a record from iai-callgrind's `summary.json` files
  compare  print base against head per scenario; refuse two series
  jitter   fail when one binary's two runs differ past the same-binary bound
  probe    run one scenario on a base binary outside Callgrind (§AR-benchmarks.5.1)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path


RECORD_VERSION = 1
# §AR-benchmarks.7.1: the fields that say what a count was measured with.
IDENTITY_FIELDS = (
    "record_version",
    "environment",
    "image",
    "rustc",
    "valgrind",
    "runner",
    "build_flags",
    "locale",
    "timezone",
    "threads",
    "fixtures",
)
# §AR-benchmarks.7.3: two runs of one binary differ by at most this much `Ir`.
JITTER_BOUND_PCT = 1.0
NOT_COMPARED = "no base, not compared"
REPO = Path(__file__).resolve().parents[1]


def build_record(*, commit: str, binary_sha256: str, identity: dict, ir: dict,
                 refused: list) -> dict:
    """§AR-benchmarks.7.1: a record missing an identity field is refused here."""
    missing = [name for name in IDENTITY_FIELDS if name not in identity]
    if missing:
        raise ValueError(f"the run record lacks identity field(s): {', '.join(missing)}")
    record = {name: identity[name] for name in IDENTITY_FIELDS}
    record.update(commit=commit, binary_sha256=binary_sha256,
                  ir=dict(sorted(ir.items())), refused=list(refused))
    record["series"] = series_key(record)
    return record


def series_key(record: dict) -> str:
    """§AR-benchmarks.7.2: the SHA-256 of the identity fields and of nothing else."""
    identity = {name: record.get(name) for name in IDENTITY_FIELDS}
    text = json.dumps(identity, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def read_refused(path: Path) -> list:
    """The scenarios `probe` found the base binary refusing, in order, once each."""
    path = Path(path)
    if not path.is_file():
        return []
    names = [line.strip() for line in path.read_text(encoding="utf-8").splitlines()]
    return list(dict.fromkeys(name for name in names if name))


def series_differences(base: dict, head: dict) -> list:
    """The identity fields on which two records differ (§AR-benchmarks.7.2)."""
    return [name for name in IDENTITY_FIELDS if base.get(name) != head.get(name)]


def refuse_two_series(base: dict, head: dict, what: str) -> bool:
    differing = series_differences(base, head)
    for name in differing:
        print(f"different series, not {what}: `{name}` is {json.dumps(base.get(name))} "
              f"in the first record and {json.dumps(head.get(name))} in the second",
              file=sys.stderr)
    return bool(differing)


def load(path: Path) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def compare(base: dict, head: dict) -> tuple:
    """§AR-benchmarks.7.2, §AR-benchmarks.5.1: one Markdown table line per scenario.

    Only a scenario both binaries ran is compared; one the base refused, or never
    measured, reads `no base, not compared`. The limits that fail a change are the
    harness's (§AR-ci.5.2), so a comparison within one series always succeeds."""
    if refuse_two_series(base, head, "compared"):
        return 1, []
    lines = [
        f"Series `{head['series'][:12]}`: base `{base['commit'][:12]}` against "
        f"head `{head['commit'][:12]}`.",
        "",
        "| Scenario | Base `Ir` | Head `Ir` | Change |",
        "|---|---:|---:|---:|",
    ]
    base_ir, head_ir = base["ir"], head["ir"]
    for name in sorted(set(base_ir) | set(head_ir)):
        if name not in head_ir:
            lines.append(f"| {name} | {base_ir[name]:,} | | not measured on head |")
        elif name not in base_ir:
            lines.append(f"| {name} | | {head_ir[name]:,} | {NOT_COMPARED} |")
        else:
            change = (head_ir[name] - base_ir[name]) / base_ir[name] * 100
            lines.append(f"| {name} | {base_ir[name]:,} | {head_ir[name]:,} | {change:+.3f}% |")
    return 0, lines


def jitter(first: dict, second: dict) -> tuple:
    """§AR-benchmarks.7.3: each scenario's two counts lie within the same-binary bound."""
    if refuse_two_series(first, second, "checked for jitter"):
        return 1, []
    lines, failed = [], False
    for name in sorted(set(first["ir"]) | set(second["ir"])):
        a, b = first["ir"].get(name), second["ir"].get(name)
        if a is None or b is None:
            failed = True
            lines.append(f"{name}: measured in one run only")
            continue
        drift = abs(b - a) / a * 100
        over = drift > JITTER_BOUND_PCT
        failed |= over
        verdict = f"past the {JITTER_BOUND_PCT}% bound" if over else "within the bound"
        lines.append(f"{name}: {a:,} then {b:,}, {drift:.4f}% apart, {verdict}")
    return int(failed), lines


def probe(binary: str, scenario: str, refused: Path, args: list) -> int:
    """§AR-benchmarks.5.1: run a scenario's invocation once, outside Callgrind; a
    refusal is recorded so the harness leaves the scenario out of the base run."""
    result = subprocess.run([binary, *args], stdout=subprocess.DEVNULL,
                            stderr=subprocess.PIPE, text=True, check=False)
    if result.returncode == 0:
        return 0
    refused = Path(refused)
    refused.parent.mkdir(parents=True, exist_ok=True)
    with refused.open("a", encoding="utf-8") as out:
        out.write(scenario + "\n")
    reason = (result.stderr.strip().splitlines() or ["no message"])[0]
    print(f"the base binary refuses `{scenario}` (exit {result.returncode}: {reason}); "
          f"it is {NOT_COMPARED}", file=sys.stderr)
    return 1


def summary_ir(iai_dir: Path) -> dict:
    """The `Ir` of this run per scenario, from iai-callgrind's `summary.json` files.

    A run's own count is the first of `Both` (the second is the baseline's) or `Left`."""
    counts = {}
    for path in sorted(Path(iai_dir).rglob("summary.json")):
        summary = load(path)
        # A bench added through `binary_benchmark_attribute!` carries its own name as its id.
        name, bench_id = summary["function_name"], summary.get("id")
        if bench_id and bench_id != name:
            name = f"{name}.{bench_id}"
        for profile in summary["profiles"]:
            if profile["tool"] != "Callgrind":
                continue
            metrics = profile["summaries"]["total"]["summary"]["Callgrind"]["Ir"]["metrics"]
            value = metrics["Both"][0] if "Both" in metrics else metrics["Left"]
            counts[name] = value["Int"]
    return counts


def fixture_identities(fixtures_dir: Path) -> dict:
    """§AR-benchmarks.1.1: each generated fixture's schema version and digest."""
    identities = {}
    for manifest in sorted(Path(fixtures_dir).glob("*/manifest.json")):
        data = load(manifest)
        identities[manifest.parent.name] = {"schema_version": data["schema_version"],
                                            "digest": data["digest"]}
    return identities


def tool_version(*command: str) -> str:
    try:
        result = subprocess.run(command, capture_output=True, text=True, check=False)
    except FileNotFoundError:
        return "absent"
    return (result.stdout.strip() or result.stderr.strip() or "unknown").splitlines()[0]


def runner_version() -> str:
    """`iai-callgrind-runner` has no `--version`; the refusal it prints names its version."""
    try:
        result = subprocess.run(["iai-callgrind-runner", "--version"], capture_output=True,
                                text=True, check=False)
    except FileNotFoundError:
        return "absent"
    found = re.search(r"iai-callgrind-runner \((\d[\w.+-]*)\)", result.stdout + result.stderr)
    return found.group(1) if found else "unknown"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with Path(path).open("rb") as data:
        for chunk in iter(lambda: data.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def environment_identity(environment: str, image: str, fixtures_dir: Path) -> dict:
    """§AR-benchmarks.7.1, §AR-benchmarks.7.4: read from the environment the run took."""
    threads = os.environ.get("RAYON_NUM_THREADS")
    return {
        "record_version": RECORD_VERSION,
        "environment": environment,
        "image": image if environment == "ci" else "local",
        "rustc": tool_version("rustc", "--version"),
        "valgrind": tool_version("valgrind", "--version"),
        "runner": runner_version(),
        "build_flags": {
            "RUSTFLAGS": os.environ.get("RUSTFLAGS", ""),
            "CARGO_INCREMENTAL": os.environ.get("CARGO_INCREMENTAL", ""),
            "profile": "bench",
        },
        "locale": os.environ.get("LC_ALL", ""),
        "timezone": os.environ.get("TZ", ""),
        "threads": int(threads) if threads and threads.isdigit() else None,
        "fixtures": fixture_identities(fixtures_dir),
    }


def git_head() -> str:
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True,
                          text=True, check=True).stdout.strip()


def parse_args(argv: list) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    modes = parser.add_subparsers(dest="mode", required=True)

    record = modes.add_parser("record", help="write the run record of the last harness run")
    record.add_argument("--out", type=Path, required=True)
    record.add_argument("--binary", type=Path, default=REPO / "target/release/grund",
                        help="the binary the run measured")
    record.add_argument("--commit", help="the commit the binary was built from (default: HEAD)")
    record.add_argument("--environment", choices=("ci", "local"), default="local")
    record.add_argument("--image", default=os.environ.get("BENCH_IMAGE", "local"),
                        help="the container image's digest (default: $BENCH_IMAGE)")
    record.add_argument("--iai", type=Path, default=REPO / "target/iai")
    record.add_argument("--fixtures", type=Path, default=REPO / "target/bench-fixtures")
    record.add_argument("--refused", type=Path, help="the file `probe` appended refusals to")

    for name in ("compare", "jitter"):
        mode = modes.add_parser(name)
        mode.add_argument("first", type=Path)
        mode.add_argument("second", type=Path)

    probe_mode = modes.add_parser("probe", help="run one scenario on a base binary")
    probe_mode.add_argument("--bin", required=True)
    probe_mode.add_argument("--scenario", required=True)
    probe_mode.add_argument("--refused", type=Path, required=True)
    probe_mode.add_argument("args", nargs=argparse.REMAINDER)
    return parser.parse_args(argv)


def main(argv: list) -> int:
    args = parse_args(argv)
    if args.mode == "probe":
        rest = args.args[1:] if args.args[:1] == ["--"] else args.args
        return probe(args.bin, args.scenario, args.refused, rest)
    if args.mode == "record":
        ir = summary_ir(args.iai)
        if not ir:
            print(f"no iai-callgrind summary.json under {args.iai}", file=sys.stderr)
            return 1
        record = build_record(
            commit=args.commit or git_head(),
            binary_sha256=sha256_file(args.binary),
            identity=environment_identity(args.environment, args.image, args.fixtures),
            ir=ir,
            refused=read_refused(args.refused) if args.refused else [],
        )
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n",
                            encoding="utf-8")
        print(f"recorded {len(ir)} scenario(s) of series {record['series'][:12]} to {args.out}")
        return 0
    check = compare if args.mode == "compare" else jitter
    status, lines = check(load(args.first), load(args.second))
    for line in lines:
        print(line)
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
