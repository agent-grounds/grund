#!/usr/bin/env python3
"""Test-only corruption seam for §FS-cochange-recipe.snapshots query completeness.

Delegates to real Grund. It alters transport facts, never co-change decisions.
"""
import json
import os
import subprocess
import sys

args = sys.argv[1:]
result = subprocess.run([os.environ["COCHANGE_REAL_GRUND"], *args],
                        input=sys.stdin.buffer.read() if "--batch" in args else None,
                        capture_output=True)
query = os.environ["COCHANGE_CORRUPT_QUERY"]
corruption = os.environ["COCHANGE_CORRUPTION"]
if query not in args:
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)
    sys.exit(result.returncode)
if corruption == "failure":
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.write("injected query failure\n")
    sys.exit(2)
rows = [json.loads(line) for line in result.stdout.splitlines()]
if corruption == "missing":
    rows = rows[:-1]
elif corruption == "duplicate":
    rows += rows[:1]
elif corruption == "malformed":
    sys.stdout.write('{"truncated":\n')
    sys.exit(0)
elif corruption == "scope":
    rows = [r for r in rows if r.get("path") != "src/lib.py"]
elif corruption == "mismatched-query":
    if rows:
        rows[0]["query"] = {"id": "FS-wrong", "section": None}
elif corruption == "failed-envelope":
    if rows:
        rows[0]["ok"] = False
        rows[0]["result"] = None
        rows[0]["error"] = {"code": "not-found", "message": "Injected refusal"}
elif corruption == "missing-result":
    if rows:
        rows[0].pop("result", None)
for row in rows:
    print(json.dumps(row))
sys.exit(result.returncode)
