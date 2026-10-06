"""Run all contract probes, keeping both failures (§FS-cochange-recipe.examples)."""
import argparse
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("--released", required=True, help="released Grund 0.16.1 executable")
parser.add_argument("--target-dir", default=str(Path.home() / "ag/tmp/grund-474-spec-target"))
args = parser.parse_args()
target = Path(args.target_dir).resolve()
environment = {**os.environ, "GRUND_BUILT": str(target / "debug/grund"),
               "GRUND_RELEASED": str(Path(args.released).resolve())}
build = subprocess.run(["cargo", "build", "-p", "grund", "--locked",
                        "--target-dir", str(target)], cwd=root, env=environment)
if build.returncode:
    sys.exit(build.returncode)
commands = [
    [sys.executable, "-m", "unittest", "discover", "-s", "tests/integration",
     "-p", "test_cochange_recipe.py", "-v"],
    ["cargo", "test", "-p", "grund", "--test", "case_external", "--locked",
     "--target-dir", str(target)],
]
statuses = [subprocess.run(command, cwd=root, env=environment).returncode for command in commands]
sys.exit(1 if any(statuses) else 0)
