"""Build the same-source binding oracle for a CI job and export it (§FS-distribution.3.0.3).

Built from the actual checkout, a pull request's merge checkout included, with the
pinned binding toolchain, before the Python gate runs; the path is written to
`GITHUB_ENV` as `GRUND_BINDINGS_ORACLE`. A step of each job that runs the gate
(§AR-ci.13.1), rather than a heredoc copied into every one of them.
"""
import os
import subprocess
from pathlib import Path

source_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
# §AR-ci.3.4.4: reuse restored dependencies; Cargo rebuilds the oracle from this HEAD.
target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).expanduser().resolve()
build_env = dict(os.environ, GRUND_BINDINGS_SOURCE_SHA=source_sha)
subprocess.run([
    "cargo", "+1.95.0", "build", "--quiet", "--locked", "-p", "grund-core",
    "--example", "grund-binding-oracle", "--target-dir", str(target),
], env=build_env, check=True)
executable = "grund-binding-oracle.exe" if os.name == "nt" else "grund-binding-oracle"
oracle = target / "debug/examples" / executable
if not oracle.is_file():
    raise FileNotFoundError(oracle)
with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as output:
    output.write(f"GRUND_BINDINGS_ORACLE={oracle}\n")
