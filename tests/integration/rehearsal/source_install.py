"""§FS-distribution-candidate.4.1, §FS-distribution-candidate.4.2,
§FS-distribution-candidate.4.3 — the source fallback of each package, run.

A source build is asked for, never implied: npm through `npm run build:source` in an
installed package whose platform package was left out, Python through `pip install
--no-binary=:all:` of the candidate's sdist. Each builds the commands its package
installs and nothing else. Run without Rust, the npm build names what is missing.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import json
import os
import shutil
import subprocess
import unittest

from rehearsal_support import (
    CASES, acquire, artifact, bin_dir, command, fresh_env, interpreter, keep, manifest, pythons,
    registry, run_checked,
)


def consumer(family, env):
    """An installed package with no platform package: the source build's starting point."""
    root = keep(f"grund-source-{family}-") / "consumer"
    root.mkdir()
    (root / "package.json").write_text('{"private": true}\n', encoding="utf-8")
    run_checked([shutil.which("npm", path=env["PATH"]), "install", "--omit=optional",
                 f"{family}@{manifest()['version']}"], cwd=root, env=env)
    return root / "node_modules"


def build_env(**extra):
    """This host's environment, Rust included, with the npm registry on loopback."""
    return dict(os.environ, npm_config_registry=registry().url,
                CARGO_TARGET_DIR=str(keep("grund-source-target-")), **extra)


class NpmSourceTests(unittest.TestCase):
    """§FS-distribution-candidate.4.1."""

    def setUp(self):
        acquire()
        self.version = manifest()["version"]

    def test_the_cli_package_builds_the_command_and_the_addon_into_native(self):
        env = build_env()
        modules = consumer("grund-cli", env)
        run_checked([shutil.which("npm"), "run", "build:source"], cwd=modules / "grund-cli",
                    env=env, timeout=7200)
        native = modules / "grund-cli" / "native"
        metadata = json.loads((native / "metadata.json").read_text(encoding="utf-8"))
        self.assertEqual(self.version, metadata["packageVersion"])
        self.assertTrue(list(native.glob("*.node")), "no addon was built")
        answer = run_checked([command(modules / ".bin", "grund"), "--version"], env=env)
        self.assertEqual(f"grund {self.version}", answer.stdout.strip())

    def test_the_server_package_builds_only_the_server(self):
        env = build_env()
        modules = consumer("grund-lsp", env)
        run_checked([shutil.which("npm"), "run", "build:source"], cwd=modules / "grund-lsp",
                    env=env, timeout=7200)
        native = modules / "grund-lsp" / "native"
        self.assertEqual([], list(native.glob("*.node")))
        self.assertFalse(any(native.glob("grund")) or any(native.glob("grund.exe")))
        answer = run_checked([command(modules / ".bin", "grund-lsp"), "--version"], env=env)
        self.assertEqual(f"grund-lsp {self.version}", answer.stdout.strip())

    def test_a_missing_rust_toolchain_is_named(self):
        """§FS-distribution-candidate.4.3: a line naming the missing prerequisite."""
        env = fresh_env(keep("grund-source-norust-") / "home", npm_config_registry=registry().url)
        modules = consumer("grund-cli", env)
        result = subprocess.run([shutil.which("npm", path=env["PATH"]), "run", "build:source"],
                                cwd=modules / "grund-cli", env=env, capture_output=True, text=True)
        self.assertNotEqual(0, result.returncode)
        self.assertTrue(any("cargo" in line.lower() for line in result.stderr.splitlines()
                            if line.startswith("error:")), result.stderr)


class PythonSourceTests(unittest.TestCase):
    """§FS-distribution-candidate.4.2: the sdists build what their wheels install."""

    def setUp(self):
        acquire()
        self.version = manifest()["version"]

    def install(self, dist):
        root = keep(f"grund-sdist-{dist}-")
        run_checked([pythons()[0], "-m", "venv", root / "venv"])
        run_checked([interpreter(root / "venv"), "-m", "pip", "install", "--no-binary=:all:",
                     "--only-binary=maturin", artifact(dist, "sdist", for_row=False)],
                    env=build_env(), timeout=7200)
        return root / "venv"

    def test_the_api_sdist_builds_the_extension_and_the_command(self):
        environment = self.install("grund")
        answer = run_checked([command(bin_dir(environment), "grund"), "--version"])
        self.assertEqual(f"grund {self.version}", answer.stdout.strip())
        probe = ("import grund, grund._native, sys; "
                 "assert [(f.code, f.line) for f in grund.check(sys.argv[1]).report] == [('dangling', 3)]")
        run_checked([interpreter(environment), "-c", probe, CASES / "json-report" / "repo"],
                    cwd=keep("grund-sdist-probe-"))

    def test_the_server_sdist_builds_only_the_server(self):
        environment = self.install("grund-lsp")
        answer = run_checked([command(bin_dir(environment), "grund-lsp"), "--version"])
        self.assertEqual(f"grund-lsp {self.version}", answer.stdout.strip())
        self.assertFalse(any(bin_dir(environment).glob("grund")) or
                         any(bin_dir(environment).glob("grund.exe")))


if __name__ == "__main__":
    unittest.main()
