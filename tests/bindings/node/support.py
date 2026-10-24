"""Test-only local package rehearsal for §FS-distribution.3.2.3.3."""

import hashlib
import importlib.util
import json
import atexit
import os
import re
import shutil
import tempfile
import time
from pathlib import Path
from functools import lru_cache

from launch import checked, command

REPO = Path(__file__).resolve().parents[3]
SCRATCH = Path.home() / "ag" / "tmp"
NODE = REPO / "crates" / "grund-node"
CASES = REPO / "tests" / "e2e" / "cases"
TYPESCRIPT = "5.9.3"
# What a retry can cure: the registry was not reached (§AR-ci.3.4.1.3).
NETWORK_ERRORS = ("ENOTFOUND", "EAI_AGAIN", "ETIMEDOUT", "ECONNRESET", "ECONNREFUSED")
INSTALL_ATTEMPTS = 3
_INSTALLED_CONSUMERS = {}


def _bindings():
    """`tests/bindings/support.py`, loaded by path: this module is `support` too."""
    spec = importlib.util.spec_from_file_location(
        "grund_bindings_support", Path(__file__).resolve().parents[1] / "support.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# The per-checkout target both bindings share (§AR-ci.3.4.1.2).
bindings = _bindings()


def node_root():
    """The shared root every checkout's Node target lives under (§AR-ci.3.4.1)."""
    given = os.environ.get("GRUND_NODE_TARGET")
    return Path(given).expanduser() if given else SCRATCH / "grund-node-target"


def node_target():
    """Reuse this checkout's persistent build output (§FS-repository-maintenance.2.1).

    One per source checkout under `node_root()`, as `tests/bindings/support.py` derives it
    for both bindings (§AR-ci.3.4.1.2).
    """
    return bindings.checkout_target(node_root())


def typescript_compiler():
    """The pinned TypeScript compiler, prepared once beside the targets (§AR-ci.3.4.1.3).

    A prepared directory is returned with nothing run. A cold one is installed into a
    fresh sibling, retried only on a network-class npm error, and renamed into place
    only once the install succeeded, so no failed install leaves it looking ready.
    """
    tools = node_root() / "tools"
    home = tools / f"typescript-{TYPESCRIPT}"
    tsc = home / "node_modules" / "typescript" / "bin" / "tsc"
    if tsc.is_file():
        return tsc.resolve()
    tools.mkdir(parents=True, exist_ok=True)
    for attempt in range(1, INSTALL_ATTEMPTS + 1):
        sibling = Path(tempfile.mkdtemp(prefix=f".{home.name}-", dir=tools))
        try:
            (sibling / "package.json").write_text('{"private":true}\n', encoding="utf-8")
            argv = ["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund",
                    "--save-exact", "--prefix", sibling, f"typescript@{TYPESCRIPT}"]
            result = command(argv, cwd=sibling)
            if result.returncode == 0:
                try:
                    sibling.rename(home)
                except OSError:
                    # Another run renamed its own copy into place first; theirs is as good.
                    if not tsc.is_file():
                        raise
                return tsc.resolve()
        finally:
            shutil.rmtree(sibling, ignore_errors=True)
        output = f"{result.stdout}\n{result.stderr}"
        network = any(re.search(rf"\b{code}\b", output) for code in NETWORK_ERRORS)
        retry = network and attempt < INSTALL_ATTEMPTS
        if not retry:
            # A failed preparation fails the test, never skips it (§AR-ci.3.4).
            raise AssertionError(
                f"command failed ({result.returncode}) on attempt {attempt}: "
                f"{' '.join(map(str, argv))}\nstdout:\n{result.stdout}\n"
                f"stderr:\n{result.stderr}"
            )
        time.sleep(2 * attempt)


def dev_env(base=None):
    """A dev build's environment: no incremental state for the cache to archive (§AR-ci.3.4.1)."""
    return {**(os.environ if base is None else base), "CARGO_INCREMENTAL": "0"}


@lru_cache(maxsize=1)
def source_package():
    """Build once per suite; each test still gets a fresh installed consumer."""
    for prerequisite in ("node", "npm", "cargo", "rustc"):
        assert shutil.which(prerequisite), f"environment prerequisite missing: {prerequisite}"
    major = int(checked(["node", "--version"]).stdout.lstrip("v").split(".")[0])
    assert major in (22, 24), f"environment prerequisite: Node 22/24, got {major}"
    assert (NODE / "build.mjs").is_file(), (
        "Node binding capability absent: crates/grund-node/build.mjs "
        "(documented native source builder)"
    )
    assert (NODE / "stage.mjs").is_file(), "Node API-only staging capability absent"
    SCRATCH.mkdir(parents=True, exist_ok=True)
    temp = tempfile.TemporaryDirectory(prefix="grund-node-source-", dir=SCRATCH)
    atexit.register(temp.cleanup)
    stage, target = Path(temp.name) / "package", node_target()
    checked(["node", NODE / "stage.mjs", "--out-dir", stage, "--target-dir", target])
    # §AR-ci.3.4.1.1: the source closure is proven by a dev build, without incremental state.
    checked(["npm", "run", "build:source", "--", "--target-dir", target, "--profile", "dev"],
            cwd=stage, env=dev_env())
    packed = checked(["npm", "pack", "--json"], cwd=stage)
    tarball = stage / json.loads(packed.stdout)[0]["filename"]
    return stage, target, tarball


def installed_consumer(tarball):
    """One pristine real npm installation per staged tarball (§AR-ci.3.4.1.4).

    Both the resolved path and bytes identify the run's input, never its version.
    Only successful preparations are retained, outside any rehearsal's lifetime;
    their temporary directories are discarded at process exit.
    """
    tarball = tarball.resolve()
    digest = hashlib.sha256()
    with tarball.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    key = (tarball, digest.digest())
    if key not in _INSTALLED_CONSUMERS:
        temp = tempfile.TemporaryDirectory(prefix="grund-809-node-installed-", dir=SCRATCH)
        consumer = Path(temp.name) / "consumer"
        try:
            consumer.mkdir()
            (consumer / "package.json").write_text(
                '{"private":true,"type":"module"}\n', encoding="utf-8",
            )
            checked(["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund",
                     tarball], cwd=consumer)
        except BaseException:
            temp.cleanup()
            raise
        atexit.register(temp.cleanup)
        _INSTALLED_CONSUMERS[key] = temp
    return Path(_INSTALLED_CONSUMERS[key].name) / "consumer"


@lru_cache(maxsize=1)
def checkout_cli():
    """Independent same-source mutation byte oracle (§FS-distribution.3.2.3.1).

    Built once per run into this checkout's target, and reused by every test (§AR-ci.3.4.1).
    """
    target = node_target()
    # No incremental state: the CI cache could not leave it out, and a fresh checkout
    # cannot reuse it (§AR-ci.3.4.1).
    checked(["cargo", "+1.95.0", "build", "-p", "grund", "--locked", "--target-dir", target],
            env=dev_env())
    return target / "debug" / ("grund.exe" if os.name == "nt" else "grund")


class Rehearsal:
    """No production builder/loader lives here; invoke the declared workflow."""

    def __init__(self):
        """Private writable copies of the run's pristine install (§AR-ci.3.4.1.4)."""
        self.stage, self.target, self.tarball = source_package()
        template = installed_consumer(self.tarball)
        self.temp = tempfile.TemporaryDirectory(prefix="grund-node-contract-", dir=SCRATCH)
        self.root = Path(self.temp.name)
        self.consumer = self.root / "consumer"
        shutil.copytree(template, self.consumer)
        self.package = self.consumer / "node_modules" / "grund-cli"
        self.fixture = self.root / "répo-雪"
        shutil.copytree(CASES / "json-report" / "repo", self.fixture)

    def run(self, script, *args, extra_env=None):
        # Copy the consumer so bare package import resolves against the fresh install.
        source = REPO / "tests" / "bindings" / "node" / script
        destination = self.consumer / script
        shutil.copyfile(source, destination)
        env = dict(os.environ)
        env["GRUND_TEST_REPO"] = str(REPO)
        if script == "mutations.mjs":
            env["GRUND_TEST_CLI"] = str(checkout_cli())
        isolated_home = self.root / "home"
        isolated_home.mkdir(exist_ok=True)
        env.update({
            "HOME": str(isolated_home), "USERPROFILE": str(isolated_home),
            "XDG_CONFIG_HOME": str(isolated_home / ".config"),
            "APPDATA": str(isolated_home / "AppData" / "Roaming"),
            "LOCALAPPDATA": str(isolated_home / "AppData" / "Local"),
        })
        env.update(extra_env or {})
        result = command(["node", destination, self.fixture, *args],
                         cwd=self.consumer, env=env, timeout=180)
        assert result.returncode == 0, (
            f"{script} failed ({result.returncode})\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        assert result.stderr == "", f"embedding emitted stderr: {result.stderr!r}"
        return result.stdout

    def close(self):
        self.temp.cleanup()
