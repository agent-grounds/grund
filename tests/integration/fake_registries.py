"""Fake registries for the publisher tests of §FS-distribution-candidate.8: npm, PyPI,
crates.io and the Actions OIDC endpoint, on loopback.

Not a test module: `unittest discover` collects `test_*.py` only. Each server speaks
only the parts of the real protocols the publisher uses (§FS-distribution-candidate.8.2,
§FS-distribution-candidate.8.5), so every refusal, fault and dropped connection is
injected exactly.
"""

import base64
import email.parser
import email.policy
import hashlib
import json
import re
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, quote, unquote, urlparse

from distribution_support import sha256

DROP = "drop"  # a fault that answers nothing: the client sees a transport failure


class FakeRegistries:
    """One loopback server speaking the parts of the real protocols a publisher uses.

    `faults` maps (registry, method, package) to a list of HTTP statuses served, in order,
    before normal behavior resumes, where `DROP` closes the connection unanswered; `hidden` names packages that accept an upload
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

    def injected(self, status, body):
        if status == DROP:
            self.close_connection = True
            return None
        return self.reply(status, body)

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
                return self.injected(status, {})
            files = [] if name in r.hidden else r.pypi.get((name, version), [])
            if not files:
                return self.reply(404, {"message": "Not Found"})
            return self.reply(200, {"urls": [{"filename": f, "digests": {"sha256": d}}
                                             for f, d in files]})
        match = re.fullmatch(r"/crates/api/v1/crates/([^/]+)/([^/]+)", path)
        if match:
            status = r.fault("crates.io", "GET", match.group(1))
            if status:
                return self.injected(status, {})
            checksum = r.crates.get(match.groups())
            if checksum is None:
                return self.reply(404, {"errors": [{"detail": "Not Found"}]})
            return self.reply(200, {"version": {"num": match.group(2), "checksum": checksum}})
        return self.reply(404, {"message": f"unknown route {path}"})

    def npm(self, method, name):
        r = self.registries
        status = r.fault("npm", method, name)
        if status:
            return self.injected(status, {"error": "injected"})
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
            return self.injected(status, {"message": "injected"})
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
