#!/usr/bin/env python3
"""Which catalog rows a benchmark run measures.

§AR-benchmarks.1.2.2: the shards of the `pr` tier are derived from the scenario catalog
here and nowhere else. The rows go, in descending `ceiling_ir` order with ties by name, each
to the shard whose `ceiling_ir` sum is smallest so far, ties to the lowest shard number.
The harness asks this script for its rows, and `tests/integration/test_bench_tiers.py`
holds it to the catalog.

  select            the whole `pr` tier, in catalog order
  select <i>/<n>    the `pr` rows of shard `i` of `n`, in catalog order
  select --tier nightly [<i>/<n>]
                    every row of both tiers (§AR-benchmarks.1.2.1), or shard `i` of them

Plain Python 3.6 and the standard library, as `scripts/distribution/train.py` reads the
catalog (§AR-benchmarks.1.5).
"""

import json
import os
import sys

CATALOG = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                       "crates", "grund-cli", "benches", "scenarios.json")
TIERS = ("pr", "nightly")


def load(path=CATALOG):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def rows(catalog, tier="pr"):
    """§AR-benchmarks.1.2.1: the `pr` tier, or every row when the tier is `nightly`."""
    if tier not in TIERS:
        raise ValueError("unknown tier %r, not one of %s" % (tier, ", ".join(TIERS)))
    return [row for row in catalog["scenarios"] if tier == "nightly" or row["tier"] == "pr"]


def assign(catalog, tier="pr"):
    """§AR-benchmarks.1.2.2: the rows of each of the catalog's `shards` shards, by name, each
    shard in catalog order."""
    count = catalog["shards"]
    selected = rows(catalog, tier)
    order = {row["name"]: index for index, row in enumerate(selected)}
    loads, shards = [0] * count, [[] for _ in range(count)]
    for row in sorted(selected, key=lambda row: (-row["ceiling_ir"], row["name"])):
        lightest = min(range(count), key=lambda shard: (loads[shard], shard))
        loads[lightest] += row["ceiling_ir"]
        shards[lightest].append(row["name"])
    return [sorted(names, key=order.__getitem__) for names in shards]


def select(catalog, tier="pr", shard=None):
    """The names one run measures: a shard `(i, n)` of the tier, or the whole tier."""
    if shard is None:
        return [row["name"] for row in rows(catalog, tier)]
    index, count = shard
    if count != catalog["shards"]:
        raise ValueError("the catalog has %d shards, not %d" % (catalog["shards"], count))
    if not 1 <= index <= count:
        raise ValueError("shard %d is not one of 1..%d" % (index, count))
    return assign(catalog, tier)[index - 1]


def parse_shard(text):
    index, _, count = text.partition("/")
    if not (index.isdigit() and count.isdigit()):
        raise ValueError("a shard is <i>/<n>, not %r" % text)
    return int(index), int(count)


def main(argv):
    if argv[:1] != ["select"]:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    args, tier, shard = argv[1:], "pr", None
    try:
        while args:
            if args[0] == "--tier" and len(args) > 1:
                tier, args = args[1], args[2:]
            elif shard is None and not args[0].startswith("-"):
                shard, args = parse_shard(args[0]), args[1:]
            else:
                raise ValueError("unexpected argument %r" % args[0])
        names = select(load(), tier, shard)
    except ValueError as error:
        print("error: %s" % error, file=sys.stderr)
        return 2
    for name in names:
        print(name)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
