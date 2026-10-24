"""Clean source/sdist/type/docs handoff (§FS-distribution.3.3.7).

The binding is one directory, `crates/grund-py/`: a clean checkout installs from it,
and the sdist built there installs on its own once unpacked.
"""

import os
import json
from pathlib import Path
import re
import shutil
import sys
import tarfile
import time
import unittest
import venv

from support import REPO, checked, python_target, temporary

CRATE = Path("crates/grund-py")
# What a retry can cure: pip never reached the index, or lost it (§AR-bindings.6.2.2).
NETWORK_ERRORS = re.compile(
    r"connection (?:broken|aborted|refused|reset)|NewConnectionError|ConnectTimeoutError"
    r"|ReadTimeoutError|timed out|name resolution|Name or service not known"
    r"|nodename nor servname|getaddrinfo failed", re.IGNORECASE)
INSTALL_ATTEMPTS = 3


def run(arguments, cwd, env=None):
    return checked(arguments, cwd=cwd, env=env)


def install_build(python, cwd, env):
    """Install the sdist's build frontend, retried only on a network failure (§AR-bindings.6.2.2).

    Any other failure, and the last network one, fails the test with pip's own output and
    never skips it (§AR-ci.3.4).
    """
    for attempt in range(1, INSTALL_ATTEMPTS + 1):
        try:
            return run([python, "-m", "pip", "install", "build"], cwd, env)
        except AssertionError as failure:
            if attempt == INSTALL_ATTEMPTS or not NETWORK_ERRORS.search(str(failure)):
                raise
        time.sleep(2 * attempt)


def environment(parent):
    directory = Path(parent) / "venv"
    venv.EnvBuilder(with_pip=True).create(directory)
    python = directory / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    return python


class BuildTests(unittest.TestCase):
    def test_clean_checkout_and_unpacked_sdist_install_independently(self):
        with temporary() as temp:
            source = Path(temp) / "source"
            source.mkdir()
            files = run(["git", "ls-files", "-z"], REPO).stdout.split("\0")
            for name in filter(None, files):
                original = REPO / name
                if not original.exists() and not original.is_symlink():
                    continue
                destination = source / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                if original.is_symlink():
                    destination.symlink_to(os.readlink(original))
                else:
                    shutil.copy2(original, destination)
            python = environment(temp)
            crate = source / CRATE
            # §AR-ci.3.4.5: both installs build into the shared target, from their own sources.
            env = dict(os.environ, CARGO_TARGET_DIR=str(python_target()),
                       GRUND_ACCEPTANCE_SOURCE=str(crate))
            run([python, "-m", "pip", "install", str(crate)], source, env)
            smoke = ('import grund; r=grund.check("tests/e2e/cases/json-report/repo"); '
                     'assert [(f.code,f.line) for f in r.report.errors]==[("dangling",3)]; '
                     'assert "FS-999-missing" in grund.show("FS-001-alpha", '
                     'root="tests/e2e/cases/json-report/repo", mode="brief").body; '
                     'import grund._native; from importlib.metadata import distribution; '
                     'assert not [e for e in distribution("grund").entry_points '
                     'if e.group=="console_scripts"]')
            run([python, "-c", smoke], source, env)
            install_build(python, source, env)
            run([python, "-m", "build", "--sdist", "--outdir", Path(temp) / "dist", crate],
                source, env)
            archive, = (Path(temp) / "dist").glob("*.tar.gz")
            unpacked = Path(temp) / "unpacked"
            unpacked.mkdir()
            with tarfile.open(archive) as package:
                # This locally built artifact is still constrained to its destination.
                for member in package.getmembers():
                    self.assertTrue((unpacked / member.name).resolve().is_relative_to(unpacked))
                    self.assertFalse(member.issym() or member.islnk())
                package.extractall(unpacked)
            sdist, = unpacked.iterdir()
            for name in ("Cargo.toml", "Cargo.lock", "pyproject.toml", "crates/grund-core/src/lib.rs",
                         "crates/grund-py/Cargo.toml", "python/grund/py.typed"):
                self.assertTrue((sdist / name).is_file(), f"sdist omitted {name}")
            # The crate's `license-file` carries the licence, beside the crate manifest.
            self.assertEqual((REPO / "LICENSE").read_bytes(),
                             (sdist / CRATE / "LICENSE").read_bytes())
            separate = Path(temp) / "independent"
            separate.mkdir()
            other = environment(separate)
            # Install while clean checkout is absent: no accidental workspace reach-through.
            shutil.rmtree(source)
            other_env = dict(env, GRUND_ACCEPTANCE_SOURCE=str(sdist))
            run([other, "-m", "pip", "install", str(sdist)], separate, other_env)
            run([other, "-c", 'import grund, grund._native; assert grund.agent_setup_instructions()'],
                separate, other_env)

    def test_native_abi_source_metadata_and_type_marker(self):
        self.assertTrue((REPO / CRATE / "python/grund/py.typed").is_file())
        metadata = json.loads(run(["cargo", "metadata", "--locked", "--format-version", "1"], REPO).stdout)
        identities = {p["id"] for p in metadata["packages"] if p["name"] == "pyo3"}
        features = {f for n in metadata["resolve"]["nodes"] if n["id"] in identities
                    for f in n["features"]}
        self.assertIn("abi3-py310", features, "resolved PyO3 feature policy")
        project = (REPO / CRATE / "pyproject.toml").read_text()
        self.assertIn("maturin", project)
        self.assertIn("grund._native", project)
        self.assertRegex(project, r'requires-python\s*=\s*["\']>=3\.10')
        self.assertTrue((REPO / CRATE / "README.md").is_file())

    def test_user_documentation_and_examples_execute(self):
        document = REPO / "docs/user-facing/python-api.md"
        self.assertTrue(document.is_file())
        content = document.read_text()
        examples = re.findall(r"```python\n(.*?)```", content, re.S)
        runnable = [code for code in examples if "check(" in code and "show(" in code]
        self.assertTrue(runnable, "documentation needs a runnable check/iteration/show block")
        for code in runnable:
            run([sys.executable, "-c", code], REPO)
        example_root = REPO / "examples/python-api"
        scripts = list(example_root.glob("*.py"))
        self.assertTrue(scripts, "publish the tested example workflow")
        for script in scripts:
            run([sys.executable, script], REPO)
        # The README reaches the Python API guide through the installation guide.
        readme = (REPO / "README.md").read_text()
        self.assertIn("(docs/user-facing/installation.md)", readme)
        installation = (REPO / "docs/user-facing/installation.md").read_text()
        self.assertIn("(python-api.md)", installation,
                      "the installation guide must link the Python API guide")
