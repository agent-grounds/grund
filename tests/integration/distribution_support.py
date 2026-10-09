"""Shared fixtures for the candidate tests of §FS-distribution-candidate.

Not a test module: `unittest discover` collects `test_*.py` only. Nothing here is
the candidate tooling; it is what the tests hold that tooling to. The matrix is
read from the specification's own table (§FS-distribution-candidate.1.1), the
inventory is spelled out from §FS-distribution-candidate.2.1 and .3.1, and the
synthetic candidate is the manifest and artifact shape §FS-distribution-candidate.6.1
names, built from bytes rather than compilers so the gate stays offline and cheap.

The candidate tool these fixtures pin is the one §AR-bindings.5 describes.
"""

import base64
import email.parser
import email.policy
import gzip
import hashlib
import io
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, quote, unquote, urlparse

REPO = Path(__file__).resolve().parents[2]
SCRATCH = Path.home() / "ag" / "tmp"
SPEC = REPO / "docs" / "functional-spec" / "FS-distribution-candidate.md"
CANDIDATE = REPO / "scripts" / "distribution" / "candidate.py"
PUBLISH = REPO / "scripts" / "distribution" / "publish.py"
PGO = REPO / "scripts" / "pgo-build.sh"
WORKFLOWS = REPO / ".github" / "workflows"
REHEARSAL_WORKFLOW = WORKFLOWS / "candidate-rehearsal.yml"
PUBLISH_WORKFLOW = WORKFLOWS / "cross-registry-publish.yml"
SHA = "0123456789abcdef0123456789abcdef01234567"
VERSION = "0.17.0"

# What the wheel tag of each registry row is spelled as on disk: auditwheel's
# compressed manylinux2014 tag set, and the plain tag elsewhere.
WHEEL_TAGS = {
    "linux-x64-gnu": "manylinux_2_17_x86_64.manylinux2014_x86_64",
    "linux-arm64-gnu": "manylinux_2_17_aarch64.manylinux2014_aarch64",
    "darwin-x64": "macosx_11_0_x86_64",
    "darwin-arm64": "macosx_11_0_arm64",
    "win32-x64-msvc": "win_amd64",
}
PRODUCTS = ("grund", "grund-lsp", "node-addon", "python-extension")
# What a receipt names, one per rehearsal module (§FS-distribution-candidate.6.5).
RECEIPT_CHECKS = ("inventory", "install", "cli-parity", "api-parity", "lsp-lifecycle",
                  "provenance", "source-install")


def tool_missing(path):
    return f"candidate capability absent: {path.relative_to(REPO).as_posix()} does not exist"


def scratch(prefix):
    SCRATCH.mkdir(parents=True, exist_ok=True)
    return tempfile.TemporaryDirectory(prefix=prefix, dir=SCRATCH)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def file_sha256(path):
    return sha256(Path(path).read_bytes())


def pep440(version):
    """§FS-distribution-candidate.6.2: `X.Y.Z-dev` is `X.Y.Z.dev0` on PyPI."""
    return version[:-4] + ".dev0" if version.endswith("-dev") else version


def run(argv, *, cwd=REPO, env=None, input=None, timeout=600):
    return subprocess.run([str(a) for a in argv], cwd=cwd, env=env, input=input,
                          capture_output=True, text=True, encoding="utf-8",
                          errors="replace", timeout=timeout)


def candidate(*args, env=None, timeout=600):
    """Run the candidate tool; its absence is the failure, never a skip."""
    assert CANDIDATE.is_file(), tool_missing(CANDIDATE)
    return run([sys.executable, CANDIDATE, *args], env=env, timeout=timeout)


def candidate_json(*args):
    result = candidate(*args)
    assert result.returncode == 0, f"candidate.py {' '.join(args)} failed:\n{result.stderr}"
    return json.loads(result.stdout)


# ---------------------------------------------------------------------------
# The matrix, read from the specification table.
# ---------------------------------------------------------------------------

def _cell(text):
    text = text.strip()
    return None if text == "deferred" else text.strip("`")


def spec_matrix():
    """§FS-distribution-candidate.1.1, row by row, as the table spells it."""
    section = SPEC.read_text(encoding="utf-8").split("### 1.1 ", 1)[1].split("\n### ", 1)[0]
    rows = []
    for line in section.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 6 or not cells[0].startswith("`"):
            continue
        row, floor, target, suffix, wheel, runner = cells
        rows.append({
            "row": _cell(row),
            "payload_floor": floor.split("/", 1)[1].strip(),
            "rust_target": _cell(target),
            "npm_suffix": _cell(suffix),
            "wheel_platform": _cell(wheel),
            "runner": re.search(r"`([^`]+)`", runner).group(1),
            "container": "manylinux container" in runner,
            "registry": _cell(suffix) is not None,
        })
    assert len(rows) == 6, f"the spec table must have six rows, read {len(rows)}"
    return rows


def registry_rows():
    return [r for r in spec_matrix() if r["registry"]]


def row_by_id(row_id):
    return next(r for r in spec_matrix() if r["row"] == row_id)


def npm_selectors(suffix):
    """The `os`, `cpu` and `libc` an npm suffix stands for (§FS-distribution-candidate.2.2)."""
    parts = suffix.split("-")
    selectors = {"os": [parts[0]], "cpu": [parts[1]]}
    if parts[0] == "linux":
        selectors["libc"] = ["glibc"]
    return selectors


def host_row():
    """The matrix row this machine is, or None for a host no row describes."""
    machine = platform.machine().lower()
    cpu = {"x86_64": "x64", "amd64": "x64", "aarch64": "arm64", "arm64": "arm64"}.get(machine)
    if sys.platform.startswith("linux"):
        libc, _ = platform.libc_ver()
        return f"linux-{cpu}-gnu" if cpu and libc == "glibc" else None
    if sys.platform == "darwin":
        return f"darwin-{cpu}" if cpu else None
    if sys.platform == "win32":
        return f"win32-{cpu}-msvc" if cpu else None
    return None


def exe(name, row):
    return name + ".exe" if row.startswith("win32") else name


# ---------------------------------------------------------------------------
# The inventory §FS-distribution-candidate.2.1 and .3.1 name.
# ---------------------------------------------------------------------------

def expected_artifacts(version=VERSION):
    pv = pep440(version)
    found = [
        {"path": f"cargo/{name}-{version}.crate", "registry": "crates.io", "package": name,
         "kind": "crate", "row": None, "version": version}
        for name in ("grund-core", "grund", "grund-lsp")
    ]
    for family in ("grund-cli", "grund-lsp"):
        found.append({"path": f"npm/{family}-{version}.tgz", "registry": "npm",
                      "package": family, "kind": "npm", "row": None, "version": version})
        for row in registry_rows():
            found.append({"path": f"npm/{family}-{row['npm_suffix']}-{version}.tgz",
                          "registry": "npm", "package": f"@{family}/{row['npm_suffix']}",
                          "kind": "npm", "row": row["row"], "version": version})
    for dist, tag in (("grund", "cp310-abi3"), ("grund_lsp", "py3-none")):
        package = dist.replace("_", "-")
        for row in registry_rows():
            found.append({"path": f"pypi/{dist}-{pv}-{tag}-{WHEEL_TAGS[row['row']]}.whl",
                          "registry": "pypi", "package": package, "kind": "wheel",
                          "row": row["row"], "version": pv})
        found.append({"path": f"pypi/{dist}-{pv}.tar.gz", "registry": "pypi",
                      "package": package, "kind": "sdist", "row": None, "version": pv})
    for product in ("grund", "grund-lsp"):
        for row in spec_matrix():
            suffix = ".zip" if row["row"].startswith("win32") else ".tar.gz"
            found.append({"path": f"archives/{product}-{version}-{row['rust_target']}{suffix}",
                          "registry": "github", "package": product, "kind": "archive",
                          "row": row["row"], "version": version})
    return found


def expected_payloads():
    found = []
    for product in PRODUCTS:
        rows = spec_matrix() if product in ("grund", "grund-lsp") else registry_rows()
        for row in rows:
            found.append({"id": f"{product}-{row['row']}", "product": product,
                          "target": row["rust_target"], "row": row["row"]})
    return found


def placements(payload, version=VERSION):
    """Where §FS-distribution-candidate.3.1 puts one payload: (artifact, path) pairs."""
    pv, row = pep440(version), row_by_id(payload["row"])
    suffix, target, product = row["npm_suffix"], row["rust_target"], payload["product"]
    archive = ".zip" if row["row"].startswith("win32") else ".tar.gz"
    if product in ("grund", "grund-lsp"):
        family = "grund-cli" if product == "grund" else "grund-lsp"
        dist = "grund" if product == "grund" else "grund_lsp"
        tag = "cp310-abi3" if product == "grund" else "py3-none"
        binary = exe(product, row["row"])
        found = [(f"archives/{product}-{version}-{target}{archive}",
                  f"{product}-{version}-{target}/{binary}")]
        if suffix:
            found.append((f"npm/{family}-{suffix}-{version}.tgz", f"package/bin/{binary}"))
            found.append((f"pypi/{dist}-{pv}-{tag}-{WHEEL_TAGS[row['row']]}.whl",
                          f"{dist}-{pv}.data/scripts/{binary}"))
        return found
    if product == "node-addon":
        return [(f"npm/grund-cli-{suffix}-{version}.tgz", "package/grund.node")]
    extension = "_native.pyd" if row["row"].startswith("win32") else "_native.abi3.so"
    return [(f"pypi/grund-{pv}-cp310-abi3-{WHEEL_TAGS[row['row']]}.whl", f"grund/{extension}")]


# ---------------------------------------------------------------------------
# A synthetic candidate: real container formats, payloads made of bytes.
# ---------------------------------------------------------------------------

def _tar_gz(members):
    buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=buffer, mode="wb", mtime=0) as gz:
        with tarfile.open(fileobj=gz, mode="w") as tar:
            for name, data in sorted(members.items()):
                info = tarfile.TarInfo(name)
                info.size, info.mtime = len(data), 0
                info.mode = 0o755 if "/bin/" in name or "/scripts/" in name else 0o644
                tar.addfile(info, io.BytesIO(data))
    return buffer.getvalue()


def _zip(members):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(members.items()):
            archive.writestr(zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0)), data)
    return buffer.getvalue()


def _record_hash(data):
    return "sha256=" + base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()


def payload_bytes(payload_id, version):
    return f"synthetic payload {payload_id} {version}\n".encode()


def _npm_package(item, version, members):
    package = {"name": item["package"], "version": item["version"],
               "license": "MIT", "repository": {"type": "git",
               "url": "git+https://github.com/agent-grounds/grund.git"}}
    if item["row"] is None:
        family = item["package"]
        package["bin"] = {"grund" if family == "grund-cli" else "grund-lsp": "launcher.cjs"}
        package["optionalDependencies"] = {
            f"@{family}/{r['npm_suffix']}": version for r in registry_rows()}
        package["grund"] = {"platformPackages": {
            r["rust_target"]: f"@{family}/{r['npm_suffix']}" for r in registry_rows()}}
    else:
        package.update(npm_selectors(row_by_id(item["row"])["npm_suffix"]))
    members["package/package.json"] = (json.dumps(package, indent=2) + "\n").encode()
    members["package/README.md"] = f"# {item['package']}\n".encode()
    members["package/LICENSE"] = (REPO / "LICENSE").read_bytes()
    return _tar_gz(members)


def _wheel(item, members):
    dist = "grund" if item["package"] == "grund" else "grund_lsp"
    info = f"{dist}-{item['version']}.dist-info"
    tag = item["path"].rsplit("-", 3)[1:]
    members[f"{info}/METADATA"] = (f"Metadata-Version: 2.1\nName: {item['package']}\n"
                                   f"Version: {item['version']}\n").encode()
    platforms = tag[2].removesuffix(".whl").split(".")
    members[f"{info}/WHEEL"] = ("Wheel-Version: 1.0\nGenerator: synthetic\n"
                                "Root-Is-Purelib: false\n"
                                + "".join(f"Tag: {tag[0]}-{tag[1]}-{p}\n" for p in platforms)).encode()
    record = "".join(f"{name},{_record_hash(data)},{len(data)}\n" for name, data in members.items())
    members[f"{info}/RECORD"] = (record + f"{info}/RECORD,,\n").encode()
    return _zip(members)


def synthetic_candidate(root, version=VERSION, sha=SHA, scope="full-release",
                        exception_row=None, engine_overrides=None, receipts=True):
    """A complete candidate the verifier must accept, in the §FS-distribution-candidate.6.1
    shape. Returns (directory, manifest digest). Tests break it one way at a time."""
    root = Path(root)
    artifacts = expected_artifacts(version)
    payloads = expected_payloads()
    members = {a["path"]: {} for a in artifacts}
    for payload in payloads:
        data = payload_bytes(payload["id"], version)
        payload["placements"] = []
        for artifact, inner in placements(payload, version):
            members[artifact][inner] = data
            payload["placements"].append({"artifact": artifact, "path": inner,
                                          "sha256": sha256(data), "transformations": []})
        lto = payload["row"] == exception_row and payload["product"] in ("grund", "grund-lsp")
        payload.update({
            "build_sha256": sha256(data),
            "engine_version": (engine_overrides or {}).get(payload["id"], version),
            "optimization": "lto-exception" if lto else "pgo",
            "profile_sha256": None if lto else sha256(b"profile " + payload["id"].encode()),
            "training_sha256": None if lto else sha256(b"training " + payload["product"].encode()),
            "exception": {"row": payload["row"],
                          "failure": "rustc crashed compiling the instrumented build"}
            if lto else None,
            "profile_key": None if lto else {
                "compiler": "rustc 1.95.0", "target": payload["target"], "source_sha": sha,
                "product": payload["product"], "features": [],
                "abi": "abi3-py310" if payload["product"] == "python-extension" else None},
        })
        payload["steps"] = None if lto else {
            "generate": sha256(b"generate " + payload["id"].encode()),
            "train": payload["training_sha256"], "merge": payload["profile_sha256"],
            "use": payload["build_sha256"]}
    for item in artifacts:
        path = root / item["path"]
        path.parent.mkdir(parents=True, exist_ok=True)
        inner = members[item["path"]]
        if item["kind"] == "crate":
            base = f"{item['package']}-{version}"
            data = _tar_gz({f"{base}/Cargo.toml": (f'[package]\nname = "{item["package"]}"\n'
                                                   f'version = "{version}"\n').encode()})
        elif item["kind"] == "npm":
            data = _npm_package(item, version, inner)
        elif item["kind"] == "wheel":
            data = _wheel(item, inner)
        elif item["kind"] == "sdist":
            base = f"{item['path'][5:-7]}"
            data = _tar_gz({f"{base}/PKG-INFO": (f"Metadata-Version: 2.1\nName: {item['package']}\n"
                                                 f"Version: {item['version']}\n").encode()})
        else:
            for name in ("LICENSE", "README.md"):
                inner[f"{next(iter(inner)).split('/')[0]}/{name}"] = (REPO / name).read_bytes()
            data = _zip(inner) if item["path"].endswith(".zip") else _tar_gz(inner)
        path.write_bytes(data)
        item["sha256"] = sha256(data)
        if item["kind"] == "archive":
            Path(str(path) + ".sha256").write_text(f"{item['sha256']}  {path.name}\n")
    manifest = {
        "schema": 1, "scope": scope, "version": version, "engine_version": version,
        "source_sha": sha, "rows": [r["row"] for r in spec_matrix()],
        "toolchain": {"rust": "1.95.0"},
        "artifacts": artifacts, "payloads": payloads,
    }
    # §FS-distribution-candidate.6.1: the digest is of the bytes on disk, LF on Windows too.
    data = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    (root / "manifest.json").write_bytes(data)
    digest = sha256(data)
    if receipts:
        write_receipts(root, digest)
    return root, digest


def write_receipts(root, digest, rows=None):
    (Path(root) / "receipts").mkdir(exist_ok=True)
    for row in rows or registry_rows():
        receipt = {"schema": 1, "manifest_sha256": digest, "row": row["row"],
                   "runner": row["runner"], "checks": {name: "passed" for name in RECEIPT_CHECKS}}
        (Path(root) / "receipts" / f"{row['row']}.json").write_text(json.dumps(receipt) + "\n")


def rewrite_manifest(root, change):
    """Edit the manifest and return its new digest; receipts follow the new digest."""
    path = Path(root) / "manifest.json"
    manifest = json.loads(path.read_text(encoding="utf-8"))
    change(manifest)
    data = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    path.write_bytes(data)
    digest = sha256(data)
    write_receipts(root, digest)
    return digest


def repack_npm(path, change):
    """Rewrite one npm tarball's members through `change(dict)`; returns its new digest."""
    with tarfile.open(path, "r:gz") as tar:
        members = {m.name: tar.extractfile(m).read() for m in tar.getmembers() if m.isfile()}
    change(members)
    Path(path).write_bytes(_tar_gz(members))
    return file_sha256(path)


# ---------------------------------------------------------------------------
# Fake registries: npm, PyPI, crates.io and the Actions OIDC endpoint, on loopback.
# ---------------------------------------------------------------------------

class FakeRegistries:
    """One loopback server speaking the parts of the real protocols a publisher uses.

    `faults` maps (registry, method, package) to a list of HTTP statuses served, in order,
    before normal behavior resumes; `hidden` names packages that accept an upload
    and never show it. `refused` names npm packages, or `pypi`, whose token exchange is
    refused; `grants` maps one to how many exchanges succeed before every later one is.
    Each exchange issues a new token and only the newest is accepted, so an accepted
    upload used the last exchange before it. Every request is logged as (method, decoded
    path)."""

    def __init__(self):
        self.npm, self.pypi, self.crates = {}, {}, {}
        self.faults, self.hidden, self.refused, self.log = {}, set(), set(), []
        self.grants, self.tokens = {}, {}
        self.lock = threading.Lock()
        handler = type("Handler", (_Handler,), {"registries": self})
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base = f"http://127.0.0.1:{self.server.server_address[1]}"

    def close(self):
        self.server.shutdown()
        self.server.server_close()

    def publish_crates(self, root):
        manifest = json.loads((Path(root) / "manifest.json").read_text())
        for item in manifest["artifacts"]:
            if item["kind"] == "crate":
                self.crates[(item["package"], item["version"])] = item["sha256"]

    def uploads(self):
        return [(m, p) for m, p in self.log if m == "PUT" or p.endswith("/legacy/")]

    def args(self):
        return ["--npm-registry", f"{self.base}/npm", "--pypi-url", f"{self.base}/pypi",
                "--pypi-upload-url", f"{self.base}/pypi/legacy/",
                "--crates-url", f"{self.base}/crates"]

    def environment(self):
        return {"ACTIONS_ID_TOKEN_REQUEST_URL": f"{self.base}/oidc/token?api-version=2.0",
                "ACTIONS_ID_TOKEN_REQUEST_TOKEN": "request-token"}

    def issue(self, key, prefix):
        """A new token for an npm package or `pypi`, or None when the exchange is refused."""
        with self.lock:
            if key in self.refused or self.grants.get(key, 1) < 1:
                return None
            if key in self.grants:
                self.grants[key] -= 1
            self.tokens[key] = f"{prefix}:{len(self.log)}"
            return self.tokens[key]

    def fault(self, registry, method, package):
        with self.lock:
            queue = self.faults.get((registry, method, package))
            return queue.pop(0) if queue else None


class _Handler(BaseHTTPRequestHandler):
    registries = None

    def log_message(self, *args):
        pass

    def reply(self, status, body=None):
        data = json.dumps(body if body is not None else {}).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def body(self):
        return self.rfile.read(int(self.headers.get("Content-Length") or 0))

    def do_GET(self):
        self.route("GET")

    def do_PUT(self):
        self.route("PUT")

    def do_POST(self):
        self.route("POST")

    def route(self, method):
        r = self.registries
        url = urlparse(self.path)
        path = unquote(url.path)
        with r.lock:
            r.log.append((method, path))
        if path == "/oidc/token":
            audience = parse_qs(url.query).get("audience", [""])[0]
            ok = self.headers.get("Authorization", "").lower() == "bearer request-token"
            return self.reply(200 if ok else 401, {"value": f"oidc:{audience}"})
        if path.startswith("/npm/-/npm/v1/oidc/token/exchange/package/"):
            name = path.rsplit("/package/", 1)[1]
            token = r.issue(name, f"npm-token:{name}")
            if token is None:
                return self.reply(403, {"message": f"no trusted publisher for {name}"})
            ok = self.headers.get("Authorization") == "Bearer oidc:npm:registry.npmjs.org"
            return self.reply(200 if ok else 401, {"token": token})
        if path.startswith("/npm/"):
            return self.npm(method, path[len("/npm/"):])
        if path == "/pypi/_/oidc/audience":
            return self.reply(200, {"audience": "pypi"})
        if path == "/pypi/_/oidc/mint-token":
            token = json.loads(self.body() or b"{}").get("token")
            token = r.issue("pypi", "pypi-token") if token == "oidc:pypi" else None
            if token is None:
                return self.reply(403, {"message": "invalid-publisher"})
            return self.reply(200, {"success": True, "token": token})
        if path == "/pypi/legacy/" and method == "POST":
            return self.pypi_upload()
        match = re.fullmatch(r"/pypi/pypi/([^/]+)/([^/]+)/json", path)
        if match:
            name, version = canonical(match.group(1)), match.group(2)
            status = r.fault("pypi", "GET", name)
            if status:
                return self.reply(status, {})
            files = [] if name in r.hidden else r.pypi.get((name, version), [])
            if not files:
                return self.reply(404, {"message": "Not Found"})
            return self.reply(200, {"urls": [{"filename": f, "digests": {"sha256": d}}
                                             for f, d in files]})
        match = re.fullmatch(r"/crates/api/v1/crates/([^/]+)/([^/]+)", path)
        if match:
            status = r.fault("crates.io", "GET", match.group(1))
            if status:
                return self.reply(status, {})
            checksum = r.crates.get(match.groups())
            if checksum is None:
                return self.reply(404, {"errors": [{"detail": "Not Found"}]})
            return self.reply(200, {"version": {"num": match.group(2), "checksum": checksum}})
        return self.reply(404, {"message": f"unknown route {path}"})

    def npm(self, method, name):
        r = self.registries
        status = r.fault("npm", method, name)
        if status:
            return self.reply(status, {"error": "injected"})
        if method == "GET":
            versions = {} if name in r.hidden else r.npm.get(name, {})
            if not versions:
                return self.reply(404, {"error": "Not found"})
            return self.reply(200, {"name": name, "versions": versions})
        if method != "PUT":
            return self.reply(405, {})
        if self.headers.get("Authorization") != f"Bearer {r.tokens.get(name)}":
            return self.reply(401, {"error": "unauthorized"})
        document = json.loads(self.body())
        version = next(iter(document["versions"]))
        if version in r.npm.get(name, {}):
            return self.reply(403, {"error": "cannot publish over the previously published versions"})
        tarball = base64.b64decode(next(iter(document["_attachments"].values()))["data"])
        integrity = "sha512-" + base64.b64encode(hashlib.sha512(tarball).digest()).decode()
        with r.lock:
            r.npm.setdefault(name, {})[version] = {"dist": {
                "integrity": integrity, "shasum": hashlib.sha1(tarball).hexdigest()}}
        return self.reply(200, {"ok": True})

    def pypi_upload(self):
        r = self.registries
        expected = "Basic " + base64.b64encode(f"__token__:{r.tokens.get('pypi')}".encode()).decode()
        if self.headers.get("Authorization") != expected:
            return self.reply(403, {"message": "Invalid or non-existent authentication"})
        raw = (f"Content-Type: {self.headers['Content-Type']}\r\n\r\n").encode() + self.body()
        message = email.parser.BytesParser(policy=email.policy.HTTP).parsebytes(raw)
        fields, content, filename = {}, None, None
        for part in message.iter_parts():
            name = part.get_param("name", header="content-disposition")
            if part.get_filename():
                content, filename = part.get_payload(decode=True), part.get_filename()
            else:
                fields[name] = part.get_content().strip()
        key = (canonical(fields.get("name", "")), fields.get("version"))
        status = r.fault("pypi", "POST", key[0])
        if status:
            return self.reply(status, {"message": "injected"})
        if any(f == filename for f, _ in r.pypi.get(key, [])):
            return self.reply(400, {"message": "File already exists"})
        if sha256(content) != fields.get("sha256_digest"):
            return self.reply(400, {"message": "digest mismatch"})
        with r.lock:
            r.pypi.setdefault(key, []).append((filename, sha256(content)))
        return self.reply(200, {})


def canonical(name):
    """PEP 503 normalization, which PyPI applies to every project name it is asked for."""
    return re.sub(r"[-_.]+", "-", name).lower()


def npm_integrity(path):
    return "sha512-" + base64.b64encode(hashlib.sha512(Path(path).read_bytes()).digest()).decode()


def npm_escape(name):
    return quote(name, safe="@")


# ---------------------------------------------------------------------------
# A stand-in payload, compiled at test time so launchers can be run for real.
# ---------------------------------------------------------------------------

PAYLOAD_SOURCE = r'''
use std::io::{Read, Write};
unsafe extern "C" { fn raise(sig: i32) -> i32; fn signal(sig: i32, handler: usize) -> usize; }
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = std::env::var("FAKE_PAYLOAD_MODE").unwrap_or_default();
    let mut out = std::io::stdout();
    match mode.as_str() {
        "echo-stdin" => { let mut b = Vec::new(); std::io::stdin().read_to_end(&mut b).unwrap();
                          out.write_all(&b).unwrap(); }
        "flood" => { #[cfg(unix)] unsafe { signal(13, 0); }
                     let line = [b'x'; 4096];
                     loop { if out.write_all(&line).is_err() { std::process::exit(141); } } }
        "wait" => { println!("ready"); out.flush().unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(120)); }
        "raise" => { unsafe { raise(15); } }
        _ => {}
    }
    let cwd = std::env::current_dir().unwrap();
    let probe = std::env::var("FAKE_PAYLOAD_PROBE").unwrap_or_default();
    let line = format!("{{\"args\":{:?},\"cwd\":{:?},\"probe\":{:?},\"name\":{:?}}}",
                       args, cwd.to_string_lossy(), probe, env!("FAKE_PAYLOAD_NAME"));
    if mode.is_empty() || mode == "status" { println!("{}", line); }
    eprintln!("payload stderr");
    let status = std::env::var("FAKE_PAYLOAD_STATUS").ok().and_then(|s| s.parse().ok());
    std::process::exit(status.unwrap_or(0));
}
'''


def compile_payload(directory, name):
    """Compile the stand-in as `name` (with `.exe` on Windows) into `directory`."""
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    source = directory / f"{name}-payload.rs"
    source.write_text(PAYLOAD_SOURCE, encoding="utf-8")
    output = directory / (name + (".exe" if os.name == "nt" else ""))
    env = dict(os.environ, FAKE_PAYLOAD_NAME=name)
    rustc = shutil.which("rustc")
    assert rustc, "environment prerequisite missing: rustc"
    result = run([rustc, "--edition=2024", "-O", "-o", output, source], env=env)
    assert result.returncode == 0, f"stand-in payload did not compile:\n{result.stderr}"
    return output
