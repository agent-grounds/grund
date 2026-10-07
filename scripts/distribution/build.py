"""One row's candidate: its payloads, every artifact of its share, and the manifest.

§FS-distribution-candidate.5.6: a row is built on its own runner, and its share is the
plan narrowed to that row plus every row-independent artifact
(§FS-distribution-candidate.2.1). §FS-distribution-candidate.6.1: the manifest records
the commit, the toolchain and the image, every artifact with its digest, and every
payload with each placement's digest; the placed bytes are the payload's own, so no
placement records a transformation. The candidate is scoped `rehearsal`: only
`assemble` makes a full release of the rows' candidates.
"""

import json
import shutil
import subprocess
from pathlib import Path

import matrix
import npm_trees
import packing
import payloads
import plan
import pypi
import versions
from verify import sha256

CRATES = ("grund-core", "grund", "grund-lsp")


def _git_head():
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=matrix.REPO, check=True,
                          capture_output=True, text=True).stdout.strip()


def _host():
    shown = subprocess.run(["rustc", "-vV"], capture_output=True, text=True,
                           env=payloads.environment())
    return next((line.split()[1] for line in shown.stdout.splitlines()
                 if line.startswith("host:")), None)


def _crates(version, target_dir, out):
    """§FS-distribution-candidate.2.1: the three crates, as `cargo publish` would upload them."""
    argv = ["cargo", "package", "--locked", "--no-verify", "--target-dir", str(target_dir)]
    for name in CRATES:
        argv += ["-p", name]
    subprocess.run(argv, cwd=matrix.REPO, check=True, env=payloads.environment())
    for name in CRATES:
        shutil.copyfile(Path(target_dir) / "package" / f"{name}-{version}.crate",
                        out / "cargo" / f"{name}-{version}.crate")


def _pack(item, placed, version, sha, trees, work, out):
    """Write one planned artifact with the payload entries placed in it."""
    path, kind, row = out / item["path"], item["kind"], item["row"] and matrix.row(item["row"])
    path.parent.mkdir(parents=True, exist_ok=True)
    if kind == "npm":
        tree = trees / item["package"]
        executables = {"launcher.cjs"} if not item["row"] else set()
        packing.tgz(path, packing.tree(tree, "package/", executables) + placed)
    elif kind == "wheel":
        pypi.wheel(path, item["package"], version, row, placed)
    elif kind == "sdist":
        pypi.sdist(path, item["package"], version, sha, work)
    elif kind == "archive":
        base = f"{item['package']}-{version}-{row['rust_target']}/"
        extras = [(base + name, (matrix.REPO / name).read_bytes(), packing.FILE)
                  for name in ("LICENSE", "README.md")]
        writer = packing.zip_file if path.name.endswith(".zip") else packing.tgz
        writer(path, extras + placed)
        digest = sha256(path.read_bytes())
        Path(f"{path}.sha256").write_text(f"{digest}  {path.name}\n", encoding="utf-8")


def build(row, sha, out, target_dir, products=None):
    """§FS-distribution-candidate.5.6: build `row`'s candidate into `out`. With
    `products`, only those payloads and the artifacts they alone fill are built."""
    if _git_head() != sha:
        raise SystemExit(f"error: --sha {sha} is not this checkout's HEAD {_git_head()}")
    version = versions.check(matrix.REPO)
    host = _host()
    if host != row["rust_target"]:
        raise SystemExit(f"error: row {row['row']} builds on {row['runner']}; this host is {host}")
    out, target_dir = Path(out).resolve(), Path(target_dir).resolve()
    if out.exists() and any(out.iterdir()):
        raise SystemExit(f"error: {out} is not empty; a candidate is never rebuilt in place")
    work = target_dir / "candidate-work" / row["row"]
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir(parents=True)

    planned = [p for p in plan.payloads(version, [row]) if not products or p["product"] in products]
    built, files = [], {}
    for payload in planned:
        record, kept = payloads.build(payload, sha, version, target_dir, work)
        record["placements"] = [{**p, "sha256": record["build_sha256"], "transformations": []}
                                for p in payload["placements"]]
        built.append(record)
        files[payload["id"]] = kept

    placed = {}
    for payload in planned:
        mode = packing.EXECUTABLE if payload["product"] in ("grund", "grund-lsp") else packing.FILE
        for placement in payload["placements"]:
            placed.setdefault(placement["artifact"], []).append(
                (placement["path"], files[payload["id"]].read_bytes(), mode))
    needs = {}
    for payload in plan.payloads(version, [row]):
        for placement in payload["placements"]:
            needs.setdefault(placement["artifact"], set()).add(payload["id"])
    ids = {p["id"] for p in planned}

    trees = work / "npm"
    (out / "cargo").mkdir(parents=True, exist_ok=True)
    if not products:
        npm_trees.write(trees, version, sha=sha, bundle=True)
        _crates(version, target_dir, out)
    else:
        npm_trees.write(trees, version)
    artifacts = []
    for item in plan.artifacts(version, [row]):
        if (item["row"] is None and products) or not needs.get(item["path"], set()) <= ids:
            continue
        if item["kind"] != "crate":
            _pack(item, placed.get(item["path"], []), version, sha, trees, work, out)
        artifacts.append({**item, "sha256": sha256((out / item["path"]).read_bytes())})
    if not any((out / "cargo").iterdir()):
        (out / "cargo").rmdir()

    manifest = {"schema": 1, "scope": "rehearsal", "version": version, "engine_version": version,
                "source_sha": sha, "rows": [row["row"]],
                "toolchain": {"rust": matrix.RUST_TOOLCHAIN,
                              "images": {row["row"]: row["container"]} if row["container"] else {}},
                "artifacts": artifacts, "payloads": built}
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"ok: {out} holds {len(artifacts)} artifacts and {len(built)} payloads of row {row['row']}")
