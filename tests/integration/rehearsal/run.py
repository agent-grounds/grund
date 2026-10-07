"""§FS-distribution-candidate.5.6, §FS-distribution-candidate.6.5,
§FS-distribution.4.13 — the rehearsal of one row, and its receipt.

    python tests/integration/rehearsal/run.py [--row ROW] [--candidate DIR] [--receipt PATH]

Runs every rehearsal module against the candidate for one matrix row: the one named,
or this host's. Without `--candidate` it builds one for the row with `candidate.py
build`. Every other row is named as skipped, with the runner it needs; asking for a
row this host is not fails, and so does a row in which no check ran. A receipt is
written only when every check ran and passed. The manual rehearsal workflow runs this
on each row's own runner; push and pull-request CI never do (§AR-ci.6).
"""

import argparse
import json
import os
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent))

from distribution_support import RECEIPT_CHECKS, host_row, spec_matrix  # noqa: E402

MODULES = {
    "inventory": "candidate_inventory",
    "install": "candidate_install",
    "cli-parity": "installed_cli_parity",
    "api-parity": "installed_api_parity",
    "lsp-lifecycle": "installed_lsp_lifecycle",
    "provenance": "profile_payload_provenance",
    "source-install": "source_install",
}
assert tuple(MODULES) == RECEIPT_CHECKS


def main():
    rows = {r["row"]: r for r in spec_matrix()}
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--row", choices=sorted(rows))
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--only", action="append", choices=sorted(MODULES))
    args = parser.parse_args()
    if args.receipt and args.only:
        parser.error("a receipt records the whole rehearsal; --only cannot write one")

    host = host_row()
    wanted = args.row or host
    if wanted is None:
        print("error: this host is no matrix row; name one with --row on its runner", file=sys.stderr)
        return 1
    if wanted != host:
        print(f"error: row {wanted} runs on {rows[wanted]['runner']}; this host is "
              f"{host or 'no matrix row'}, so nothing of {wanted} can run here", file=sys.stderr)
        return 1
    for name, row in rows.items():
        if name != host:
            print(f"skipped: row {name} runs on {row['runner']} in the rehearsal; this host is {host}")

    os.environ["GRUND_REHEARSAL_ROW"] = wanted
    if args.candidate:
        os.environ["GRUND_REHEARSAL_CANDIDATE"] = str(args.candidate.resolve())
    outcome = {}
    for check in args.only or MODULES:
        print(f"==> row {wanted}: {check}", flush=True)
        suite = unittest.defaultTestLoader.loadTestsFromName(MODULES[check])
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        ran = result.testsRun - len(result.skipped)
        outcome[check] = ("passed" if result.wasSuccessful() and ran else
                          "not run" if result.wasSuccessful() else "failed")
    print(json.dumps({"row": wanted, "checks": outcome}, indent=2))
    if all(state == "not run" for state in outcome.values()):
        print(f"error: row {wanted}: no check ran", file=sys.stderr)
        return 1
    if any(state != "passed" for state in outcome.values()):
        failed = ", ".join(c for c, state in outcome.items() if state != "passed")
        print(f"error: row {wanted}: {failed} did not pass; no receipt", file=sys.stderr)
        return 1
    if args.receipt:
        import rehearsal_support
        receipt = {"schema": 1, "manifest_sha256": rehearsal_support.manifest_sha256(),
                   "row": wanted, "runner": rows[wanted]["runner"], "checks": outcome}
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
