#!/usr/bin/env python3
"""§AR-ci.4: run Grund's full self-check with the catastrophic-regression ceiling."""
import subprocess
import sys

try:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--locked", "--", "check", ".", "--format", "json", "--full"],
        stdout=subprocess.PIPE,
        timeout=30,
    )
except subprocess.TimeoutExpired:
    print("grund self-check exceeded 30s", file=sys.stderr)
    sys.exit(124)

if result.returncode != 0:
    sys.exit(result.returncode)
if result.stdout:
    print("grund self-check wrote unexpected stdout:", file=sys.stderr)
    sys.stderr.buffer.write(result.stdout)
    sys.exit(1)
