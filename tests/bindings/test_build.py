"""Clean source/sdist/type/docs handoff (§FS-distribution.3.3.7)."""

import os
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import unittest
import venv

from support import REPO, temporary


def run(arguments, cwd, env=None):
    return subprocess.run([str(a) for a in arguments], cwd=cwd, env=env,
                          capture_output=True, text=True, check=True)


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
            env = dict(os.environ, CARGO_TARGET_DIR=str(Path(temp) / "cargo-target"),
                       GRUND_ACCEPTANCE_SOURCE=str(source))
            run([python, "-m", "pip", "install", str(source)], source, env)
            smoke = ('import grund; r=grund.check("tests/e2e/cases/json-report/repo"); '
                     'assert [(f.code,f.line) for f in r.report]==[("dangling",3)]; '
                     'assert "FS-999-missing" in grund.show("FS-001-alpha", '
                     'root="tests/e2e/cases/json-report/repo", mode="brief").body; '
                     'import grund._native; from importlib.metadata import distribution; '
                     'assert not [e for e in distribution("grund").entry_points '
                     'if e.group=="console_scripts"]')
            run([python, "-c", smoke], source, env)
            run([python, "-m", "pip", "install", "build"], source, env)
            run([python, "-m", "build", "--sdist", "--outdir", Path(temp) / "dist"], source, env)
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
                         "crates/grund-py/Cargo.toml", "python/grund/py.typed", "LICENSE"):
                self.assertTrue((sdist / name).is_file(), f"sdist omitted {name}")
            separate = Path(temp) / "independent"
            separate.mkdir()
            other = environment(separate)
            # Install while clean checkout is absent: no accidental workspace reach-through.
            shutil.rmtree(source)
            other_env = dict(env, CARGO_TARGET_DIR=str(separate / "cargo-target"),
                             GRUND_ACCEPTANCE_SOURCE=str(sdist))
            run([other, "-m", "pip", "install", str(sdist)], separate, other_env)
            run([other, "-c", 'import grund, grund._native; assert grund.agent_setup_instructions()'],
                separate, other_env)

    def test_native_abi_source_metadata_and_type_marker(self):
        self.assertTrue((REPO / "python/grund/py.typed").is_file())
        metadata = json.loads(run(["cargo", "metadata", "--locked", "--format-version", "1"], REPO).stdout)
        identities = {p["id"] for p in metadata["packages"] if p["name"] == "pyo3"}
        features = {f for n in metadata["resolve"]["nodes"] if n["id"] in identities
                    for f in n["features"]}
        self.assertIn("abi3-py310", features, "resolved PyO3 feature policy")
        project = (REPO / "pyproject.toml").read_text()
        self.assertIn("maturin", project)
        self.assertIn("grund._native", project)
        self.assertRegex(project, r'requires-python\s*=\s*["\']>=3\.10')
        self.assertTrue((REPO / "crates/grund-py/README.md").is_file())

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
        self.assertIn("python-api", (REPO / "README.md").read_text())
