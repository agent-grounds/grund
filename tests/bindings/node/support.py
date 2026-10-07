"""Test-only local package rehearsal for §FS-distribution.3.2.3.3."""

import json
import atexit
import os
import shutil
import tempfile
from pathlib import Path
from functools import lru_cache

from launch import checked, command

REPO = Path(__file__).resolve().parents[3]
SCRATCH = Path.home() / "ag" / "tmp"
NODE = REPO / "crates" / "grund-node"
CASES = REPO / "tests" / "e2e" / "cases"


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
    root = Path(temp.name)
    stage, target = root / "package", root / "cargo-target"
    checked(["node", NODE / "stage.mjs", "--out-dir", stage, "--target-dir", target])
    checked(["npm", "run", "build:source", "--", "--target-dir", target], cwd=stage)
    packed = checked(["npm", "pack", "--json"], cwd=stage)
    tarball = stage / json.loads(packed.stdout)[0]["filename"]
    return stage, target, tarball


@lru_cache(maxsize=1)
def checkout_cli():
    """Independent same-source mutation byte oracle (§FS-distribution.3.2.3.1)."""
    SCRATCH.mkdir(parents=True, exist_ok=True)
    temp = tempfile.TemporaryDirectory(prefix="grund-node-cli-oracle-", dir=SCRATCH)
    atexit.register(temp.cleanup)
    target = Path(temp.name) / "cargo-target"
    checked(["cargo", "+1.95.0", "build", "-p", "grund", "--locked", "--target-dir", target])
    return target / "debug" / ("grund.exe" if os.name == "nt" else "grund")


class Rehearsal:
    """No production builder/loader lives here; invoke the declared workflow."""

    def __init__(self):
        self.stage, self.target, self.tarball = source_package()
        self.temp = tempfile.TemporaryDirectory(prefix="grund-node-contract-", dir=SCRATCH)
        self.root = Path(self.temp.name)
        self.consumer = self.root / "consumer"
        self.consumer.mkdir()
        (self.consumer / "package.json").write_text(
            '{"private":true,"type":"module"}\n', encoding="utf-8",
        )
        checked(["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund",
                 self.tarball], cwd=self.consumer)
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
