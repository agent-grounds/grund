"""What every rehearsal module shares: the candidate under test, the row it runs as,
fresh install environments, and a loopback npm registry (§FS-distribution-candidate.5.1,
§FS-distribution-candidate.5.2).

Not a test module. `run.py` sets the candidate and row through the environment, so a
module can also be run on its own with `python -m unittest` once they are set. The
candidate is acquired once per process; a failure to acquire it is remembered, so
every test that needs it fails with the same line rather than rebuilding.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import atexit
import base64
import functools
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import threading
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import quote, unquote

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from distribution_support import (  # noqa: E402
    REPO, candidate, exe, file_sha256, host_row, pep440, row_by_id, scratch,
)

ENV_CANDIDATE = "GRUND_REHEARSAL_CANDIDATE"
ENV_ROW = "GRUND_REHEARSAL_ROW"
ENV_PYTHONS = "GRUND_REHEARSAL_PYTHONS"
PYTHONS = ("3.10", "3.11", "3.12", "3.13", "3.14")
CASES = REPO / "tests" / "e2e" / "cases"
_ACQUIRED = {}


def keep(prefix):
    """A scratch directory that lives until the process ends."""
    temp = scratch(prefix)
    atexit.register(temp.cleanup)
    return Path(temp.name)


def row():
    """The row this process rehearses: the one `run.py` named, or the host's."""
    name = os.environ.get(ENV_ROW) or host_row()
    assert name, "this host is no matrix row; nothing can be rehearsed here"
    return row_by_id(name)


def run_checked(argv, **kwargs):
    result = subprocess.run([str(a) for a in argv], capture_output=True, text=True,
                            encoding="utf-8", errors="replace", **{"cwd": REPO, **kwargs})
    assert result.returncode == 0, (f"{' '.join(map(str, argv))} exited {result.returncode}\n"
                                    f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    return result


def acquire():
    """The candidate directory: the one named, or one `candidate.py build` makes for this row
    (§FS-distribution-candidate.5.6). Raised once, remembered for every later caller."""
    if "error" in _ACQUIRED:
        raise AssertionError(_ACQUIRED["error"])
    if "path" not in _ACQUIRED:
        try:
            _ACQUIRED["path"] = _acquire()
        except AssertionError as error:
            _ACQUIRED["error"] = str(error)
            raise
    return _ACQUIRED["path"]


def _acquire():
    named = os.environ.get(ENV_CANDIDATE)
    if named:
        path = Path(named).resolve()
        assert (path / "manifest.json").is_file(), f"no candidate at {path}: manifest.json absent"
        return path
    out = keep("grund-rehearsal-candidate-") / "candidate"
    sha = run_checked(["git", "rev-parse", "HEAD"]).stdout.strip()
    result = candidate("build", "--row", row()["row"], "--sha", sha, "--out", out,
                       "--target-dir", keep("grund-rehearsal-target-"), timeout=7200)
    assert result.returncode == 0, f"candidate.py build failed:\n{result.stderr}"
    return out


def manifest():
    return json.loads((acquire() / "manifest.json").read_text(encoding="utf-8"))


def manifest_sha256():
    return file_sha256(acquire() / "manifest.json")


def artifact(package, kind, for_row=True):
    """One artifact of the candidate, by package and kind, for this row when it has one."""
    wanted = row()["row"] if for_row else None
    found = [a for a in manifest()["artifacts"]
             if a["package"] == package and a["kind"] == kind and a["row"] == wanted]
    assert len(found) == 1, f"candidate holds {len(found)} {kind} artifacts of {package} for {wanted}"
    return acquire() / found[0]["path"]


def fresh_env(home, **extra):
    """§FS-distribution-candidate.5.2: no Rust on `PATH`, no cache, no index, own home."""
    home = Path(home)
    home.mkdir(parents=True, exist_ok=True)
    rust = ("cargo", "rustc", "rustup")
    path = [d for d in os.environ.get("PATH", "").split(os.pathsep)
            if d and not any(shutil.which(tool, path=d) for tool in rust)]
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(("CARGO", "RUSTUP", "RUST", "PIP_", "PIPX_", "npm_config_",
                                "NPM_CONFIG_", "VIRTUAL_ENV", "PYTHON", "NODE_"))}
    env.update({
        "PATH": os.pathsep.join(path), "HOME": str(home), "USERPROFILE": str(home),
        "XDG_CONFIG_HOME": str(home / ".config"), "XDG_CACHE_HOME": str(home / ".cache"),
        "APPDATA": str(home / "AppData" / "Roaming"), "LOCALAPPDATA": str(home / "AppData" / "Local"),
        "npm_config_cache": str(home / "npm-cache"), "npm_config_update_notifier": "false",
        "npm_config_audit": "false", "npm_config_fund": "false",
        "PIP_NO_INDEX": "1", "PIP_CACHE_DIR": str(home / "pip-cache"),
        "PIP_DISABLE_PIP_VERSION_CHECK": "1",
        "PIPX_HOME": str(home / "pipx"), "PIPX_BIN_DIR": str(home / "pipx-bin"),
    })
    env.update(extra)
    for tool in rust:
        assert not shutil.which(tool, path=env["PATH"]), f"{tool} is still on the fresh PATH"
    return env


def pythons():
    """The promised CPython interpreters, every one required (§FS-distribution-candidate.1.2)."""
    named = [p for p in os.environ.get(ENV_PYTHONS, "").split(os.pathsep) if p]
    found = {}
    for candidate_path in named or [shutil.which(f"python{v}") or f"python{v}" for v in PYTHONS]:
        probe = subprocess.run([candidate_path, "-c", "import sys, platform; print("
                                "f'{sys.version_info[0]}.{sys.version_info[1]}', "
                                "platform.python_implementation(), sys._is_gil_enabled() "
                                "if hasattr(sys, '_is_gil_enabled') else True)"],
                               capture_output=True, text=True) if shutil.which(candidate_path) else None
        if probe and probe.returncode == 0:
            version, implementation, gil = probe.stdout.split()
            if implementation == "CPython" and gil == "True":
                found[version] = candidate_path
    missing = [v for v in PYTHONS if v not in found]
    assert not missing, (f"environment prerequisite missing: CPython {', '.join(missing)} "
                         f"(name them in {ENV_PYTHONS})")
    return [found[v] for v in PYTHONS]


def bin_dir(venv):
    return Path(venv) / ("Scripts" if os.name == "nt" else "bin")


def interpreter(environment):
    return bin_dir(environment) / ("python.exe" if os.name == "nt" else "python")


def command(directory, name):
    """An installed command: npm writes a `.cmd` shim on Windows, pip an `.exe`."""
    for suffix in ((".cmd", ".exe", "") if os.name == "nt" else ("",)):
        path = Path(directory) / (name + suffix)
        if path.exists():
            return path
    raise AssertionError(f"{name} is not installed in {directory}")


class LoopbackNpm:
    """An npm registry on loopback serving exactly the candidate's npm tarballs
    (§FS-distribution-candidate.5.1). Anything else is `404`, so an install that
    needs a package the candidate lacks fails here rather than reaching npmjs.org."""

    def __init__(self, directory):
        self.packages = {}
        for tarball in sorted(Path(directory).glob("*.tgz")):
            with tarfile.open(tarball, "r:gz") as tar:
                package = json.loads(tar.extractfile("package/package.json").read())
            data = tarball.read_bytes()
            self.packages.setdefault(package["name"], {})[package["version"]] = (package, tarball, data)
        handler = type("Handler", (_Handler,), {"registry": self})
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.url = f"http://127.0.0.1:{self.server.server_address[1]}/"
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        atexit.register(self.server.shutdown)

    def packument(self, name):
        versions = {}
        for version, (package, tarball, data) in self.packages[name].items():
            integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()
            versions[version] = {**package, "dist": {
                "tarball": f"{self.url}-/{quote(tarball.name)}", "integrity": integrity,
                "shasum": hashlib.sha1(data).hexdigest()}}
        return {"name": name, "dist-tags": {"latest": max(versions)}, "versions": versions}


class _Handler(BaseHTTPRequestHandler):
    registry = None

    def log_message(self, *args):
        pass

    def do_GET(self):
        path = unquote(self.path.split("?", 1)[0]).lstrip("/")
        body = None
        if path.startswith("-/"):
            for versions in self.registry.packages.values():
                for _, tarball, data in versions.values():
                    if tarball.name == path[2:]:
                        body, kind = data, "application/octet-stream"
        elif path in self.registry.packages:
            body, kind = json.dumps(self.registry.packument(path)).encode(), "application/json"
        if body is None:
            self.send_response(404)
            self.end_headers()
            return
        self.send_response(200)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


@functools.lru_cache(maxsize=1)
def registry():
    return LoopbackNpm(acquire() / "npm")


@functools.lru_cache(maxsize=None)
def npm_install(family):
    """A fresh consumer with one npm family installed, and nothing else (§FS-distribution.1.3)."""
    root = keep(f"grund-npm-{family}-")
    consumer = root / "consumer"
    consumer.mkdir()
    (consumer / "package.json").write_text('{"private": true}\n', encoding="utf-8")
    env = fresh_env(root / "home", npm_config_registry=registry().url)
    version = manifest()["version"]
    npm = shutil.which("npm", path=env["PATH"])
    assert npm, "environment prerequisite missing: npm"
    run_checked([npm, "install", "--no-audit", "--no-fund", f"{family}@{version}"],
                cwd=consumer, env=env)
    return consumer, env


@functools.lru_cache(maxsize=None)
def venv(python, dist):
    """A fresh virtual environment with one wheel of this row installed from its file."""
    root = keep(f"grund-venv-{dist}-")
    env = fresh_env(root / "home")
    run_checked([python, "-m", "venv", root / "venv"], env=env)
    wheel = artifact(dist, "wheel")
    run_checked([interpreter(root / "venv"), "-m", "pip", "install", "--no-index", wheel], env=env)
    return root / "venv", env, wheel


@functools.lru_cache(maxsize=None)
def cargo_install(crate):
    """`cargo install` from the candidate's `.crate` files, the engine patched to its own."""
    root = keep(f"grund-cargo-{crate}-")
    unpacked = {}
    for name in ("grund-core", crate):
        with tarfile.open(artifact(name, "crate", for_row=False), "r:gz") as tar:
            tar.extractall(root / "src", filter="data")
        unpacked[name] = root / "src" / f"{name}-{manifest()['version']}"
    patch = f'patch.crates-io.grund-core.path="{unpacked["grund-core"].as_posix()}"'
    run_checked(["cargo", "install", "--locked", "--path", unpacked[crate], "--root", root / "root",
                 "--target-dir", root / "target", "--config", patch], timeout=7200)
    binary = "grund" if crate == "grund" else "grund-lsp"
    return root / "root" / "bin" / exe(binary, row()["row"])


@functools.lru_cache(maxsize=None)
def archive(product):
    """The row's archive, checked against its `.sha256` and unpacked."""
    path = artifact(product, "archive")
    recorded = Path(str(path) + ".sha256").read_text(encoding="utf-8").split()
    assert recorded == [file_sha256(path), path.name], f"{path.name}: .sha256 disagrees"
    root = keep(f"grund-archive-{product}-")
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as package:
            package.extractall(root)
    else:
        with tarfile.open(path, "r:gz") as package:
            package.extractall(root, filter="data")
    base = path.name.removesuffix(".zip").removesuffix(".tar.gz")
    return root / base / exe(product, row()["row"])


def installed(product):
    """Every installed copy of one command on this row: Cargo, npm, wheel and archive."""
    family = "grund-cli" if product == "grund" else "grund-lsp"
    dist = "grund" if product == "grund" else "grund-lsp"
    consumer, _ = npm_install(family)
    environment, _, _ = venv(pythons()[0], dist)
    return {
        "cargo": cargo_install(product),
        "npm": command(consumer / "node_modules" / ".bin", product),
        "wheel": command(bin_dir(environment), product),
        "archive": archive(product),
    }


def wheel_version():
    return pep440(manifest()["version"])
