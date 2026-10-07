"""Hold a candidate to its plan and to its own manifest.

§FS-distribution-candidate.6.4: every refusal names the artifact or payload it is
about. The versions are read from inside each artifact, never from the manifest's
claim about it, and every placement's bytes are read and hashed. Without `release`
a candidate may be one row's share of the plan, the shape a rehearsal row builds
(§FS-distribution-candidate.5.6); with it, the whole plan and nothing less.
"""

import email.parser
import hashlib
import io
import json
import tarfile
import tomllib
import zipfile
from pathlib import Path

import matrix
import plan

EXCEPTION_ROW = "win32-arm64-msvc"
NO_PROFILE = "training produced no profile"
FIELDS = ("registry", "package", "kind", "row", "version")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def member(path, inner):
    """The bytes at one path inside one artifact, or None when it is not there."""
    try:
        if path.name.endswith((".whl", ".zip")):
            with zipfile.ZipFile(path) as package:
                return package.read(inner)
        with tarfile.open(path, "r:gz") as package:
            found = package.extractfile(inner)
            return found.read() if found else None
    except (KeyError, OSError, zipfile.BadZipFile, tarfile.TarError):
        return None


def inner_version(root, item):
    """(file read, version) from inside one artifact (§FS-distribution-candidate.6.4)."""
    path = root / item["path"]
    if item["kind"] == "npm":
        data = member(path, "package/package.json")
        return "package/package.json", data and json.loads(data).get("version")
    if item["kind"] == "crate":
        inner = f"{item['package']}-{item['version']}/Cargo.toml"
        data = member(path, inner)
        return inner, data and tomllib.loads(data.decode("utf-8"))["package"].get("version")
    if item["kind"] in ("wheel", "sdist"):
        dist = plan.DIST[item["package"]]
        base = f"{dist}-{item['version']}"
        inner = f"{base}.dist-info/METADATA" if item["kind"] == "wheel" else f"{base}/PKG-INFO"
        data = member(path, inner)
        headers = data and email.parser.BytesParser().parsebytes(data)
        return inner, headers and headers.get("Version")
    return None, None


def _rows(manifest, release):
    names = manifest.get("rows") or []
    every = [r["row"] for r in matrix.rows()]
    if release and names != every:
        return None, [f"manifest rows {names} are not the full matrix {every}"]
    unknown = [n for n in names if n not in every]
    if unknown or not names:
        return None, [f"manifest rows {names} name no matrix row or an unknown one"]
    return [matrix.row(n) for n in names], []


def _identity(manifest, release, sha, tag):
    problems, version = [], manifest.get("version")
    if release and manifest.get("scope") != "full-release":
        problems.append(f"scope {manifest.get('scope')} is not a release; "
                        "only full-release is (§FS-distribution-candidate.6.3)")
    if release and plan.is_development(version or ""):
        problems.append(f"version {version} is a development version, not a release")
    if manifest.get("scope") == "full-release" and manifest.get("engine_version") != version:
        problems.append(f"engine version {manifest.get('engine_version')} is not version {version}")
    if sha and sha != manifest.get("source_sha"):
        problems.append(f"--sha {sha} is not the manifest's commit {manifest.get('source_sha')}")
    if tag and tag != f"v{version}":
        problems.append(f"--tag {tag} is not the manifest's version v{version}")
    return problems


def _artifacts(root, manifest, planned):
    problems = []
    recorded = {a["path"]: a for a in manifest.get("artifacts", [])}
    for path, item in planned.items():
        if path not in recorded:
            problems.append(f"{path} is planned but not in the manifest")
        elif any(recorded[path].get(k) != item[k] for k in FIELDS):
            problems.append(f"{path}: manifest says {[recorded[path].get(k) for k in FIELDS]}, "
                            f"the plan {[item[k] for k in FIELDS]}")
    problems += [f"{path} is in the manifest but not in the plan"
                 for path in recorded if path not in planned]
    on_disk = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
    for name in sorted(on_disk):
        if name == "manifest.json" or name.startswith("receipts/"):
            continue
        if name.endswith(".sha256") and name[:-7] in planned and name.startswith("archives/"):
            continue
        if name not in planned:
            problems.append(f"{name} is not in the plan")
    for path, item in planned.items():
        file = root / path
        if not file.is_file():
            problems.append(f"{path} is missing")
            continue
        digest = sha256(file.read_bytes())
        if path in recorded and recorded[path].get("sha256") != digest:
            problems.append(f"{path}: its bytes are {digest[:16]}…, not the manifest's")
            continue
        if item["kind"] == "archive":
            problems += _checksum_file(file, digest)
        inner, found = inner_version(root, item)
        if inner and found != item["version"]:
            problems.append(f"{path}: {inner} says version {found}, not {item['version']}")
    return problems


def _checksum_file(file, digest):
    """§FS-distribution-candidate.8.6: the `.sha256` beside an archive must agree."""
    beside = Path(str(file) + ".sha256")
    expected = f"{digest}  {file.name}\n"
    if not beside.is_file():
        return [f"{file.name}.sha256 is missing beside {file.name}"]
    if beside.read_text(encoding="utf-8") != expected:
        return [f"{file.name}.sha256 disagrees with {file.name}"]
    return []


def _payloads(root, manifest, version, rows):
    problems = []
    planned = {p["id"]: p for p in plan.payloads(version, rows)}
    recorded = {p.get("id"): p for p in manifest.get("payloads", [])}
    problems += [f"payload {i} is planned but not in the manifest" for i in planned if i not in recorded]
    problems += [f"payload {i} is not in the plan" for i in recorded if i not in planned]
    engine = manifest.get("engine_version")
    profiles = {}
    for pid, payload in recorded.items():
        if pid not in planned:
            continue
        want = planned[pid]
        where = sorted((p.get("artifact"), p.get("path")) for p in payload.get("placements", []))
        if where != sorted((p["artifact"], p["path"]) for p in want["placements"]):
            problems.append(f"payload {pid}: placements are not the plan's")
        if payload.get("engine_version") != engine:
            problems.append(f"payload {pid}: engine version {payload.get('engine_version')}, "
                            f"not the candidate's {engine}")
        for placement in payload.get("placements", []):
            data = member(root / placement["artifact"], placement["path"])
            if data is None or sha256(data) != placement.get("sha256"):
                problems.append(f"payload {pid}: the bytes at {placement['artifact']}:"
                                f"{placement['path']} are not the recorded ones")
            elif not placement.get("transformations") and placement["sha256"] != payload.get("build_sha256"):
                problems.append(f"payload {pid}: {placement['artifact']} holds bytes that are not "
                                "the build's, and records no transformation")
        problems += _optimization(pid, payload, manifest.get("source_sha"))
        if payload.get("profile_sha256"):
            profiles.setdefault(payload["profile_sha256"], []).append(pid)
    problems += [f"payloads {', '.join(ids)} share one profile"
                 for ids in profiles.values() if len(ids) > 1]
    return problems


def _optimization(pid, payload, sha):
    """§FS-distribution-candidate.7.2, .7.3 and .7.4."""
    kind = payload.get("optimization")
    if kind == "lto-exception":
        if payload.get("row") != EXCEPTION_ROW:
            return [f"payload {pid} is an lto-exception on {payload.get('row')}; only "
                    f"{EXCEPTION_ROW} may fall back (§FS-distribution-candidate.7.4)"]
        failure = (payload.get("exception") or {}).get("failure")
        return [] if failure == NO_PROFILE else [f"payload {pid}: lto-exception for {failure!r}"]
    if kind != "pgo":
        return [f"payload {pid}: optimization {kind!r} is neither pgo nor lto-exception"]
    key, steps = payload.get("profile_key") or {}, payload.get("steps") or {}
    expected = {"product": payload.get("product"), "target": payload.get("target"), "source_sha": sha,
                "abi": "abi3-py310" if payload.get("product") == "python-extension" else None}
    problems = [f"payload {pid}: profile key {k} is {key.get(k)!r}, not {v!r}"
                for k, v in expected.items() if key.get(k) != v]
    if not str(key.get("compiler", "")).startswith("rustc "):
        problems.append(f"payload {pid}: the profile key names no compiler")
    if set(steps) != {"generate", "train", "merge", "use"} or steps.get("use") != payload.get("build_sha256"):
        problems.append(f"payload {pid}: generate, train, merge and use are not recorded, "
                        "or the use step is not the packaged build")
    if not payload.get("profile_sha256") or not payload.get("training_sha256"):
        problems.append(f"payload {pid}: no profile or training digest")
    return problems


def verify(root, release=False, sha=None, tag=None):
    """Every problem with the candidate at `root`, as lines naming what they are about."""
    root = Path(root)
    try:
        manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        return [f"{root / 'manifest.json'}: {error}"]
    rows, problems = _rows(manifest, release)
    problems += _identity(manifest, release, sha, tag)
    if rows is None:
        return problems
    version = manifest.get("version")
    planned = {a["path"]: a for a in plan.artifacts(version, rows)}
    problems += _artifacts(root, manifest, planned)
    problems += _payloads(root, manifest, version, rows)
    return problems


def read_manifest(root):
    text = (Path(root) / "manifest.json").read_bytes()
    return json.load(io.BytesIO(text)), sha256(text)
