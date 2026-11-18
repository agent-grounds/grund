#!/usr/bin/env python3
"""Print the earliest release this tree may be cut as. §FS-distribution.4.4

The inclusive floor of the release window (§FS-distribution.4.2.5): the highest
release a landed message names, or the shared floor. Prints 0.0.0 when nothing
bounds it. The scheduled patch helper skips a patch below it.

Usage: release_earliest.py [--root <tree>]
"""

import argparse
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_release_ramps import REPO_ROOT, read_release_floor, release_window, scan_tree  # noqa: E402


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="repository root to scan")
    args = parser.parse_args(argv)
    try:
        floor, _ceiling = release_window(scan_tree(args.root), read_release_floor(args.root))
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    print(floor or "0.0.0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
