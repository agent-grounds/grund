"""The parts of the npm, PyPI, crates.io and Actions OIDC protocols the publisher speaks.

§FS-distribution-candidate.8.2: the only credential is the job's identity token,
exchanged per registry for a short-lived upload token; nothing here reads a stored
token. §FS-distribution-candidate.8.5: `held` answers what a registry already has at
a version and raises when it cannot be asked, so an unanswered question is never read
as an empty registry.
"""

import base64
import email.parser
import hashlib
import io
import json
import os
import tarfile
import uuid
import zipfile
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen

AGENT = "grund-cross-registry-publisher (+https://github.com/agent-grounds/grund)"
NPM_AUDIENCE = "npm:registry.npmjs.org"


class Refused(Exception):
    """A registry or the identity provider said no; the message names to what."""


def call(method, url, body=None, headers=None):
    """(status, decoded JSON or None). A transport failure is a status of 0."""
    request = Request(url, data=body, method=method,
                      headers={"User-Agent": AGENT, "Accept": "application/json", **(headers or {})})
    try:
        with urlopen(request, timeout=120) as response:
            status, data = response.status, response.read()
    except HTTPError as error:
        status, data = error.code, error.read()
    except (URLError, OSError):
        return 0, None
    try:
        return status, json.loads(data) if data else None
    except ValueError:
        return status, None


def identity_token(audience):
    """The workflow's OIDC token for one audience; without `id-token: write` there is none."""
    url, token = (os.environ.get("ACTIONS_ID_TOKEN_REQUEST_URL"),
                  os.environ.get("ACTIONS_ID_TOKEN_REQUEST_TOKEN"))
    if not url or not token:
        raise Refused("no workflow identity: the job needs `permissions: id-token: write`, and "
                      "no stored registry token stands in for it")
    status, body = call("GET", f"{url}&audience={quote(audience, safe='')}",
                        headers={"Authorization": f"bearer {token}"})
    if status != 200 or not (body or {}).get("value"):
        raise Refused(f"the identity provider refused a token for {audience} (HTTP {status})")
    return body["value"]


def npm_escape(name):
    return quote(name, safe="@")


def npm_exchange(registry, name, oidc):
    """§FS-distribution-candidate.8.2: one package's trusted-publisher token, or refusal."""
    status, body = call("POST", f"{registry}/-/npm/v1/oidc/token/exchange/package/{npm_escape(name)}",
                        headers={"Authorization": f"Bearer {oidc}"})
    if status != 200 or not (body or {}).get("token"):
        raise Refused(f"npm refused a publish token for {name} (HTTP {status}): "
                      "is its trusted publisher registered?")
    return body["token"]


def pypi_mint(pypi):
    status, body = call("GET", f"{pypi}/_/oidc/audience")
    if status != 200 or not (body or {}).get("audience"):
        raise Refused(f"pypi gave no trusted-publishing audience (HTTP {status})")
    oidc = identity_token(body["audience"])
    status, body = call("POST", f"{pypi}/_/oidc/mint-token", json.dumps({"token": oidc}).encode(),
                        {"Content-Type": "application/json"})
    if status != 200 or not (body or {}).get("token"):
        raise Refused(f"pypi refused to mint an upload token (HTTP {status}): "
                      "are its trusted publishers registered?")
    return body["token"]


def npm_integrity(data):
    return "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()


def npm_held(registry, name, version):
    """The integrity npm holds for name@version, None when it holds none."""
    status, body = call("GET", f"{registry}/{npm_escape(name)}")
    if status == 404:
        return None
    if status != 200:
        raise Refused(f"npm cannot be asked about {name} (HTTP {status}); "
                      "a registry that does not answer is not read as empty")
    found = (body or {}).get("versions", {}).get(version)
    return found and found.get("dist", {}).get("integrity")


def npm_publish(registry, name, version, data, token):
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
        package = json.loads(tar.extractfile("package/package.json").read())
    file = f"{name.rsplit('/', 1)[-1]}-{version}.tgz"
    package.update({"_id": f"{name}@{version}", "dist": {
        "integrity": npm_integrity(data), "shasum": hashlib.sha1(data).hexdigest(),
        "tarball": f"{registry}/{name}/-/{file}"}})
    document = {"_id": name, "name": name, "description": package.get("description", ""),
                "dist-tags": {"latest": version}, "versions": {version: package}, "access": "public",
                "_attachments": {file: {"content_type": "application/octet-stream",
                                        "data": base64.b64encode(data).decode(), "length": len(data)}}}
    status, body = call("PUT", f"{registry}/{npm_escape(name)}", json.dumps(document).encode(),
                        {"Content-Type": "application/json", "Authorization": f"Bearer {token}"})
    if status not in (200, 201):
        raise Refused(f"npm refused {name}@{version} (HTTP {status}): {(body or {}).get('error', '')}")


def pypi_held(pypi, name, version):
    """{file name: sha256} PyPI holds for name==version."""
    status, body = call("GET", f"{pypi}/pypi/{name}/{version}/json")
    if status == 404:
        return {}
    if status != 200:
        raise Refused(f"pypi cannot be asked about {name} {version} (HTTP {status}); "
                      "a registry that does not answer is not read as empty")
    return {u["filename"]: u["digests"]["sha256"] for u in (body or {}).get("urls", [])}


def _metadata(path, data):
    """The core metadata inside a wheel or an sdist, as the upload form repeats it."""
    if path.name.endswith(".whl"):
        with zipfile.ZipFile(io.BytesIO(data)) as wheel:
            inner = next(n for n in wheel.namelist() if n.endswith(".dist-info/METADATA"))
            text = wheel.read(inner)
    else:
        with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as sdist:
            inner = next(n for n in sdist.getnames() if n.count("/") == 1 and n.endswith("/PKG-INFO"))
            text = sdist.extractfile(inner).read()
    return email.parser.BytesParser().parsebytes(text)


def pypi_upload(url, path, data, token):
    """One file through the legacy upload API, with its metadata, as twine sends it."""
    headers = _metadata(path, data)
    wheel = path.name.endswith(".whl")
    fields = [(":action", "file_upload"), ("protocol_version", "1"),
              ("filetype", "bdist_wheel" if wheel else "sdist"),
              ("pyversion", path.name.split("-")[2] if wheel else "source"),
              ("sha256_digest", hashlib.sha256(data).hexdigest())]
    for key in headers.keys():
        for value in headers.get_all(key):
            fields.append((key.lower().replace("-", "_"), value))
    payload = headers.get_payload()
    if payload:
        fields.append(("description", payload))
    boundary = uuid.uuid4().hex
    parts = [f'--{boundary}\r\nContent-Disposition: form-data; name="{k}"\r\n\r\n{v}\r\n'.encode()
             for k, v in fields]
    parts.append(f'--{boundary}\r\nContent-Disposition: form-data; name="content"; '
                 f'filename="{path.name}"\r\nContent-Type: application/octet-stream\r\n\r\n'.encode()
                 + data + f"\r\n--{boundary}--\r\n".encode())
    auth = base64.b64encode(f"__token__:{token}".encode()).decode()
    status, body = call("POST", url, b"".join(parts), {
        "Content-Type": f"multipart/form-data; boundary={boundary}", "Authorization": f"Basic {auth}"})
    if status != 200:
        raise Refused(f"pypi refused {path.name} (HTTP {status}): {(body or {}).get('message', '')}")


def crate_resolves(crates, name, version):
    status, body = call("GET", f"{crates}/api/v1/crates/{name}/{version}")
    return status == 200 and (body or {}).get("version", {}).get("num") == version
