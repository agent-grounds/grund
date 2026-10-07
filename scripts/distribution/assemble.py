"""From row candidates to one candidate, and from row rehearsals to its receipts.

§FS-distribution-candidate.5.6: each row is built and rehearsed on its own runner,
yet the publisher trusts one manifest (§FS-distribution-candidate.6.1). Three steps
join them:

- `assemble` merges verified row candidates. Each row's artifacts and payloads come
  from that row's candidate; the row-independent ones (the crates, the npm umbrellas,
  the sdists) from the first row's, so the candidate holds one copy of each. With
  every row it is a `full-release`; with fewer, a `rehearsal`.
- `share` writes one row's share of an assembled candidate — its own artifacts, the
  row-independent ones and its payloads, byte for byte — with a manifest that names
  the candidate's digest. The rehearsal of that row runs against its share.
- `receipts` accepts a share's receipt only when the share's manifest is exactly the
  one `share` derives from this candidate for that row and the receipt names that
  manifest with every check passed. The candidate's receipt then names the candidate's
  digest and the share's (§FS-distribution-candidate.6.5): the bytes the row proved
  are the candidate's own bytes.
"""

import json
import shutil
from pathlib import Path

import matrix
import verify

CHECKS = ("inventory", "install", "cli-parity", "api-parity", "lsp-lifecycle", "provenance",
          "source-install")


def _dump(manifest):
    return json.dumps(manifest, indent=2) + "\n"


def _copy(source, target, path):
    (target / path).parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source / path, target / path)
    if path.startswith("archives/"):
        shutil.copyfile(source / f"{path}.sha256", target / f"{path}.sha256")


def assemble(row_dirs, sha, out):
    """Merge verified row candidates of one commit into one candidate at `out`."""
    found = {}
    for directory in row_dirs:
        problems = verify.verify(directory, sha=sha)
        if problems:
            raise SystemExit("".join(f"error: {directory}: {p}\n" for p in problems).rstrip())
        manifest, _ = verify.read_manifest(directory)
        name, = manifest["rows"]
        if name in found:
            raise SystemExit(f"error: two candidates for row {name}")
        found[name] = (Path(directory), manifest)
    order = [r["row"] for r in matrix.rows() if r["row"] in found]
    first = found[order[0]][1]
    for name in order:
        if (found[name][1]["version"], found[name][1]["source_sha"]) != (first["version"], sha):
            raise SystemExit(f"error: row {name} is not {first['version']} at {sha}")
    out = Path(out)
    if out.exists() and any(out.iterdir()):
        raise SystemExit(f"error: {out} is not empty; a candidate is never rebuilt in place")
    artifacts, payloads, images = [], [], {}
    for name in order:
        directory, manifest = found[name]
        images.update(manifest["toolchain"].get("images", {}))
        payloads += manifest["payloads"]
        for item in manifest["artifacts"]:
            if item["row"] == name or (item["row"] is None and name == order[0]):
                _copy(directory, out, item["path"])
                artifacts.append(item)
    rank = {a["path"]: i for i, a in enumerate(verify.plan.artifacts(first["version"]))}
    every = [r["row"] for r in matrix.rows()]
    manifest = {**first, "scope": "full-release" if order == every else "rehearsal",
                "rows": order, "toolchain": {**first["toolchain"], "images": images},
                "artifacts": sorted(artifacts, key=lambda a: rank[a["path"]]),
                "payloads": payloads}
    (out / "manifest.json").write_text(_dump(manifest), encoding="utf-8")
    print(f"ok: {out} assembles {len(order)} rows, {manifest['scope']}")


def project(manifest, digest, row):
    """§FS-distribution-candidate.5.6: one row's share of a candidate, as a manifest."""
    return {**manifest, "scope": "rehearsal", "rows": [row], "candidate_sha256": digest,
            "artifacts": [a for a in manifest["artifacts"] if a["row"] in (row, None)],
            "payloads": [p for p in manifest["payloads"] if p["row"] == row]}


def share(candidate, row, out):
    candidate, out = Path(candidate), Path(out)
    manifest, digest = verify.read_manifest(candidate)
    if row not in manifest["rows"]:
        raise SystemExit(f"error: {candidate} holds no row {row}")
    projected = project(manifest, digest, row)
    for item in projected["artifacts"]:
        _copy(candidate, out, item["path"])
    (out / "manifest.json").write_text(_dump(projected), encoding="utf-8")
    print(f"ok: {out} is row {row}'s share of {digest}")


def receipts(candidate, shares):
    """Bind each row's rehearsal receipt to the candidate it is a share of."""
    candidate = Path(candidate)
    manifest, digest = verify.read_manifest(candidate)
    problems = []
    for directory in map(Path, shares):
        text = (directory / "manifest.json").read_text(encoding="utf-8")
        row = json.loads(text)["rows"][0]
        if text != _dump(project(manifest, digest, row)):
            problems.append(f"{directory} is not row {row}'s share of {digest}")
            continue
        try:
            receipt = json.loads((directory / "receipts" / f"{row}.json").read_text(encoding="utf-8"))
        except (OSError, ValueError):
            problems.append(f"{directory} holds no receipt for row {row}")
            continue
        share_digest = verify.sha256(text.encode("utf-8"))
        checks = receipt.get("checks") or {}
        if receipt.get("manifest_sha256") != share_digest or receipt.get("row") != row:
            problems.append(f"the receipt in {directory} is not for its share {share_digest}")
        elif any(checks.get(c) != "passed" for c in CHECKS):
            problems.append(f"the receipt for row {row} does not record every check passed")
        else:
            bound = {"schema": 1, "manifest_sha256": digest, "row": row,
                     "runner": receipt.get("runner"), "checks": checks,
                     "share_manifest_sha256": share_digest}
            (candidate / "receipts").mkdir(exist_ok=True)
            (candidate / "receipts" / f"{row}.json").write_text(
                json.dumps(bound, indent=2) + "\n", encoding="utf-8")
            print(f"ok: row {row}'s receipt names {digest}")
    if problems:
        raise SystemExit("".join(f"error: {p}\n" for p in problems).rstrip())
