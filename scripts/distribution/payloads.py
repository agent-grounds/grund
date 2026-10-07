"""Each payload of one row: built by `scripts/pgo-build.sh`, recorded from its evidence.

§FS-distribution-candidate.7.1: every product trains on its own workload, one
`pgo-build.sh --product` run each. §FS-distribution-candidate.7.3: the record is the
script's own account of the generate, train, merge and use steps, and the payload
kept is the profile-use build it names. §FS-distribution-candidate.7.4: the script
exits 3 when training wrote no profile; only the Windows arm64 row turns that into an
LTO build, self-checked, and every other failure fails the candidate naming its row.
Linux payloads are built in the row's pinned manylinux image (§FS-distribution.4.8),
macOS payloads for 11.0 (§FS-distribution-candidate.1.3).
"""

import json
import os
import shutil
import subprocess
from pathlib import Path

import matrix
from verify import EXCEPTION_ROW, NO_PROFILE, sha256

PGO = matrix.REPO / "scripts" / "pgo-build.sh"
CRATE = {"grund": "grund", "grund-lsp": "grund-lsp"}


def environment(row=None):
    """Rust 1.95.0 for every build (§FS-distribution-candidate.1.1), macOS 11.0 on its rows."""
    env = dict(os.environ, RUSTUP_TOOLCHAIN=matrix.RUST_TOOLCHAIN)
    if row and row["row"].startswith("darwin"):
        env["MACOSX_DEPLOYMENT_TARGET"] = "11.0"
    return env


def bash():
    """Git's bash on Windows, where `bash` on PATH may be WSL's."""
    if os.name == "nt":
        git = Path(os.environ.get("ProgramFiles", r"C:\Program Files")) / "Git" / "bin" / "bash.exe"
        if git.is_file():
            return str(git)
    return "bash"


def _exception(product, row, target_dir):
    """§FS-distribution-candidate.7.4: the self-checked LTO build the exception packages."""
    if product not in CRATE:
        raise SystemExit(f"error: row {row['row']}: {product} has no LTO exception")
    subprocess.run(["cargo", "build", "--release", "--locked", "-p", CRATE[product],
                    "--target-dir", str(target_dir)], env=environment(row), check=True)
    path = Path(target_dir) / "release" / matrix.exe(product, row["row"])
    check = subprocess.run([str(path), "--version"], capture_output=True, text=True)
    if check.returncode != 0:
        raise SystemExit(f"error: row {row['row']}: the LTO {product} does not self-check:\n"
                         f"{check.stderr}")
    return path


def build(payload, sha, version, target_dir, work):
    """(record, file) for one planned payload. The file is a copy outside Cargo's
    target, so a later product's build cannot change it."""
    product, row = payload["product"], matrix.row(payload["row"])
    evidence = work / "evidence" / f"{payload['id']}.json"
    argv = [bash(), str(PGO), "--product", product, "--target-dir", str(target_dir),
            "--evidence", str(evidence), "--sha", sha]
    if row["container"]:
        argv += ["--container", row["container"]]
    status = subprocess.run(argv, env=environment(row)).returncode
    record = {key: payload[key] for key in ("id", "product", "target", "row")}
    record["engine_version"] = version
    if status == 3 and row["row"] == EXCEPTION_ROW:
        built = _exception(product, row, target_dir)
        record.update({"optimization": "lto-exception", "profile_sha256": None,
                       "training_sha256": None, "profile_key": None, "steps": None,
                       "exception": {"row": row["row"], "failure": NO_PROFILE}})
    elif status != 0:
        why = (f": {NO_PROFILE}, and only {EXCEPTION_ROW} may fall back "
               "(§FS-distribution-candidate.7.4)" if status == 3 else "")
        raise SystemExit(f"error: row {row['row']}: payload {payload['id']} failed "
                         f"(pgo-build.sh exited {status}){why}")
    else:
        found = json.loads(evidence.read_text(encoding="utf-8"))
        built = Path(found["payload"])
        record.update({"optimization": "pgo", "profile_sha256": found["profile_sha256"],
                       "training_sha256": found["training_sha256"], "exception": None,
                       "profile_key": found["key"], "training": found["training"],
                       "steps": {"generate": found["generate_sha256"],
                                 "train": found["training_sha256"],
                                 "merge": found["profile_sha256"], "use": found["build_sha256"]}})
    kept = work / "payloads" / payload["id"] / built.name
    kept.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(built, kept)
    record["build_sha256"] = sha256(kept.read_bytes())
    if record["steps"] and record["steps"]["use"] != record["build_sha256"]:
        raise SystemExit(f"error: payload {payload['id']}: the kept bytes are not the "
                         "profile-use build pgo-build.sh recorded")
    return record, kept
