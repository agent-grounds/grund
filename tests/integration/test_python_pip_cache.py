"""§AR-ci-pip-cache — a test's `pip install` leaves no built wheel in the user's pip cache.

Each install that builds grund (§AR-ci-pip-cache.1) is recorded rather than run, the way
`test_python_shared_target.py` records them: a child imports the call site with its runner
replaced and reports the options and environment pip would have got, so no crate is compiled.
A real pip with no index then runs with exactly those on a tiny sdist, and the cache it was
pointed at is read (§AR-ci-pip-cache.3). Nothing here names the setting a site turns the cache
off with: the verdict is what pip did with what the site gave it.
"""

import json
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
import venv
from pathlib import Path
from unittest import mock

REPO = Path(__file__).resolve().parents[2]
BINDINGS = REPO / "tests" / "bindings"
INTEGRATION = REPO / "tests" / "integration"
SCRATCH = Path.home() / "ag/tmp"

# A package named like the binding that builds in milliseconds and needs nothing from an index.
PYPROJECT = """[build-system]
requires = []
build-backend = "backend"
backend-path = ["."]

[project]
name = "grund"
version = "0.0.0"
"""
BACKEND = """import os, zipfile

def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    name, info = "grund-0.0.0-py3-none-any.whl", "grund-0.0.0.dist-info"
    with zipfile.ZipFile(os.path.join(wheel_directory, name), "w") as wheel:
        wheel.writestr(f"{info}/METADATA", "Metadata-Version: 2.1\\nName: grund\\nVersion: 0.0.0\\n")
        wheel.writestr(f"{info}/WHEEL", "Wheel-Version: 1.0\\nGenerator: grund-test\\n"
                                        "Root-Is-Purelib: true\\nTag: py3-none-any\\n")
        wheel.writestr(f"{info}/RECORD", "")
    return name
"""

# What each recorder shares: what an install that builds grund would have been given. The
# frontend's fetch, `pip install build`, builds nothing of ours (§AR-ci-pip-cache.1).
RECORD = """
import json, os, subprocess, sys
sys.dont_write_bytecode = True
sys.path[:0] = sys.argv[1:]
installs = []

def record(argv, *args, env=None, **kwargs):
    argv = [str(arg) for arg in argv]
    if "pip" in argv and "install" in argv and "build" not in argv:
        given = os.environ if env is None else env
        installs.append({"argv": argv, "env": {key: str(value) for key, value in given.items()}})
    return subprocess.CompletedProcess(argv, 0, "", "")
"""

# The module's install (§AR-bindings.6.2).
ACCEPTANCE = RECORD + """
subprocess.run = record
import test_python_binding_acceptance as site
name = "test_python_acceptance_is_in_the_complete_local_gate"
getattr(site.PythonAcceptanceWorkflowTests(name), name)()
print(json.dumps(installs))
"""

# Both of `test_build`'s installs (§FS-distribution.3.3.7). The sdist install comes after the
# frontend's build, so `python -m build` is answered with an archive holding what the test reads.
CLEAN_CHECKOUT_AND_SDIST = RECORD + """
import io, tarfile
from pathlib import Path
import test_build as site

def write_sdist(directory):
    members = {name: b"" for name in (
        "Cargo.toml", "Cargo.lock", "pyproject.toml", "crates/grund-core/src/lib.rs",
        "crates/grund-py/Cargo.toml", "python/grund/py.typed")}
    members["crates/grund-py/LICENSE"] = (site.REPO / "LICENSE").read_bytes()
    directory.mkdir(parents=True)
    with tarfile.open(directory / "grund-0.0.0.tar.gz", "w:gz") as archive:
        for name, data in members.items():
            member = tarfile.TarInfo(f"grund-0.0.0/{name}")
            member.size = len(data)
            archive.addfile(member, io.BytesIO(data))

def run(arguments, cwd, env=None):
    result = record(arguments, env=env)
    if "--sdist" in result.args:
        write_sdist(Path(result.args[result.args.index("--outdir") + 1]))
    return result

site.run = run
site.environment = lambda parent: Path(parent) / "python"
name = "test_clean_checkout_and_unpacked_sdist_install_independently"
getattr(site.BuildTests(name), name)()
print(json.dumps(installs))
"""

# The rehearsal lane's source install (§FS-distribution-candidate.4.2), with no candidate.
SOURCE_INSTALL = RECORD + """
import types
import source_install as site
site.run_checked = record
site.acquire = lambda: None
site.pythons = lambda: [sys.executable]
site.artifact = lambda *args, **kwargs: "grund-0.0.0.tar.gz"
site.registry = lambda: types.SimpleNamespace(url="http://127.0.0.1:9/")
site.PythonSourceTests("test_the_api_sdist_builds_the_extension_and_the_command").install("grund")
print(json.dumps(installs))
"""

# Each site, the installs that build grund it makes, its recorder and what that recorder imports.
SITES = {"the clean checkout and the unpacked sdist":
         (2, CLEAN_CHECKOUT_AND_SDIST, BINDINGS, BINDINGS / "python"),
         "the rehearsal's source install": (1, SOURCE_INSTALL, INTEGRATION / "rehearsal"),
         "the acceptance module": (1, ACCEPTANCE, INTEGRATION)}


def start_environment(home):
    """What a site starts from: no `PIP_*` of the contributor's decides (§AR-ci-pip-cache.3)."""
    inherited = {key: value for key, value in os.environ.items()
                 if not key.startswith("PIP_") and key not in (
                     "GRUND_PYTHON_TARGET", "CARGO_TARGET_DIR", "PYTHONPATH", "PYTHONHOME")}
    return dict(inherited, HOME=str(home), USERPROFILE=str(home), PIP_CONFIG_FILE=os.devnull,
                PIP_NO_INDEX="1", PIP_DISABLE_PIP_VERSION_CHECK="1", PIP_NO_INPUT="1")


class PipCacheTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        cls.temp = tempfile.TemporaryDirectory(prefix="grund-1209-pip-", dir=SCRATCH)
        cls.root = Path(cls.temp.name)
        venv.EnvBuilder(with_pip=True).create(cls.root / "venv")
        cls.python = cls.root / "venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        # An sdist on every call: on a directory, a pip from 26.2 on passes (§AR-ci-pip-cache.3).
        source = cls.root / "grund-0.0.0"
        source.mkdir()
        (source / "pyproject.toml").write_text(PYPROJECT, encoding="utf-8")
        (source / "backend.py").write_text(BACKEND, encoding="utf-8")
        cls.sdist = cls.root / "grund-0.0.0.tar.gz"
        with tarfile.open(cls.sdist, "w:gz") as archive:
            archive.add(source, arcname=source.name)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def scratch(self, prefix):
        return Path(tempfile.mkdtemp(prefix=prefix, dir=self.root))

    def recorded(self, site):
        """The argv and environment of each install `site` makes, none of them run."""
        count, script, *paths = SITES[site]
        result = subprocess.run([sys.executable, "-c", script, *map(str, paths)], cwd=REPO,
                                env=start_environment(self.scratch("home-")),
                                capture_output=True, text=True, timeout=120)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        found = json.loads(result.stdout.strip().splitlines()[-1])
        self.assertEqual(count, len(found), f"{site} builds grund {len(found)} times: {found}")
        return found

    def cached(self, options, env):
        """The wheels a real pip leaves in an empty cache, given `options` and `env`."""
        cache = self.scratch("cache-")
        result = subprocess.run([str(self.python), "-m", "pip", "install", *options,
                                 str(self.sdist)], cwd=self.root,
                                env=dict(env, PIP_CACHE_DIR=str(cache)),
                                capture_output=True, text=True, timeout=120)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return sorted(wheel.name for wheel in cache.rglob("*.whl"))

    def test_the_recorders_reach_every_install_that_builds_grund(self):
        for site in SITES:
            with self.subTest(site=site):
                self.recorded(site)

    def test_with_the_cache_left_on_pip_caches_the_tiny_sdist(self):
        # The control, and nothing the contributor exported moves it (§AR-ci-pip-cache.3).
        elsewhere = self.scratch("exported-")
        exported = {"PIP_NO_CACHE_DIR": "1", "PIP_CACHE_DIR": str(elsewhere)}
        for given in ({}, exported):
            with self.subTest(exported=sorted(given)), mock.patch.dict(os.environ, given):
                wheels = self.cached([], start_environment(self.scratch("home-")))
                self.assertEqual(["grund-0.0.0-py3-none-any.whl"], wheels,
                                 "this pip did not cache a build from a local sdist, so an "
                                 "empty cache below says nothing about the installs")
        self.assertEqual([], list(elsewhere.iterdir()))

    def test_every_install_that_builds_grund_leaves_no_wheel_in_the_pip_cache(self):
        for site in SITES:
            for install in self.recorded(site):
                argv = install["argv"][install["argv"].index("install") + 1:]
                options = [arg for arg in argv if arg.startswith("-")]
                source = next(Path(arg).name for arg in argv if not arg.startswith("-"))
                with self.subTest(site=site, source=source):
                    self.assertEqual([], self.cached(options, install["env"]),
                                     f"{site} left a wheel in pip's cache, installing {source}")


if __name__ == "__main__":
    unittest.main()
