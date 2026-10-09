"""§FS-distribution-candidate.5.2, §FS-distribution-candidate.5.1,
§FS-distribution-candidate.1.2, §FS-distribution.1.3, §FS-lsp.2.1 — every package of
this row installs fresh, without Rust, and each family installs without the other.

npm installs come from a loopback registry serving only the candidate's tarballs,
Python installs from the candidate's wheel files with no index, and the crates from
the candidate's `.crate` files. Every promised interpreter is required: a missing
CPython fails the check rather than shrinking it.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import shutil
import subprocess
import unittest

from rehearsal_support import (
    CASES, acquire, archive, artifact, bin_dir, cargo_install, command, fresh_env, interpreter,
    keep, manifest, npm_install, pythons, registry, row, run_checked, venv, wheel_version,
)

FAMILIES = {"grund-cli": ("grund", "grund-lsp"), "grund-lsp": ("grund-lsp", "grund")}
DISTS = {"grund": ("grund", "grund-lsp"), "grund-lsp": ("grund-lsp", "grund")}


class Rehearsed(unittest.TestCase):
    def setUp(self):
        acquire()
        self.version = manifest()["version"]

    def assert_answers(self, binary, product, env=None):
        answer = run_checked([binary, "--version"], env=env).stdout.strip()
        self.assertEqual(f"{product} {self.version}", answer)


class NpmInstallTests(Rehearsed):
    """§FS-distribution-candidate.5.2: `npm install` and `npx`, each family alone."""

    def test_each_family_installs_alone_and_answers(self):
        for family, (product, other) in FAMILIES.items():
            with self.subTest(family=family):
                consumer, env = npm_install(family)
                modules = consumer / "node_modules"
                self.assert_answers(command(modules / ".bin", product), product, env)
                self.assertFalse((modules / ".bin" / other).exists(), f"{family} pulled in {other}")
                suffix = row()["npm_suffix"]
                self.assertTrue((modules / f"@{family}" / suffix).is_dir(),
                                f"the {suffix} platform package was not selected")
                installed = {p.name for p in (modules / f"@{family}").iterdir()}
                self.assertEqual({suffix}, installed, "another row's platform package installed")

    def test_npx_runs_each_command(self):
        for family, (product, _) in FAMILIES.items():
            with self.subTest(family=family):
                home = keep("grund-npx-")
                env = fresh_env(home / "home", npm_config_registry=registry().url)
                npx = shutil.which("npx", path=env["PATH"])
                self.assertTrue(npx, "environment prerequisite missing: npx")
                ran = run_checked([npx, "--yes", "--package", f"{family}@{self.version}", "--",
                                   product, "--version"], cwd=home, env=env)
                self.assertEqual(f"{product} {self.version}", ran.stdout.strip())

    def test_an_npm_install_builds_nothing(self):
        """§FS-distribution-candidate.4.1: no Rust on PATH, and no `native/` appears."""
        for family in FAMILIES:
            with self.subTest(family=family):
                consumer, _ = npm_install(family)
                self.assertFalse((consumer / "node_modules" / family / "native").exists())


class PythonInstallTests(Rehearsed):
    """§FS-distribution-candidate.1.2: every CPython 3.10–3.14 installs the row's wheels."""

    def test_each_wheel_installs_alone_into_every_interpreter(self):
        for python in pythons():
            for dist, (product, other) in DISTS.items():
                with self.subTest(python=python, dist=dist):
                    environment, env, _ = venv(python, dist)
                    self.assert_answers(command(bin_dir(environment), product), product, env)
                    self.assertFalse([p for p in bin_dir(environment).glob(f"{other}*")
                                      if p.stem == other], f"the {dist} wheel installed {other}")
                    shown = run_checked([interpreter(environment), "-m", "pip", "show", dist],
                                        env=env).stdout
                    self.assertIn(f"Version: {wheel_version()}", shown)

    def test_the_api_wheel_imports_its_extension_on_every_interpreter(self):
        probe = ("import grund, grund._native, sys; "
                 "assert grund._native.__file__.startswith(grund.__path__[0]); "
                 "report = grund.check(sys.argv[1]).report; "
                 "assert [(f.code, f.line) for f in report] == [('dangling', 3)], report")
        for python in pythons():
            with self.subTest(python=python):
                environment, env, _ = venv(python, "grund")
                run_checked([interpreter(environment), "-c", probe, CASES / "json-report" / "repo"],
                            env=env, cwd=keep("grund-api-probe-"))

    def test_pipx_installs_each_command(self):
        for dist, (product, _) in DISTS.items():
            with self.subTest(dist=dist):
                home = keep("grund-pipx-")
                # pipx's pip backend, as on a host without uv. Making its shared venv
                # reinstalls pip from an index, so that is off; the wheel installs with none.
                env = fresh_env(home / "home", PIPX_DEFAULT_BACKEND="pip",
                                PIPX_DISABLE_SHARED_LIBS_AUTO_UPGRADE="1")
                pipx = shutil.which("pipx")
                self.assertTrue(pipx, "environment prerequisite missing: pipx")
                run_checked([pipx, "install", "--python", pythons()[0],
                             "--pip-args=--no-index", artifact(dist, "wheel")], env=env)
                self.assert_answers(command(env["PIPX_BIN_DIR"], product), product, env)


class CargoAndArchiveTests(Rehearsed):
    """§FS-distribution-candidate.5.2: the crates and the archives stand alone too."""

    def test_cargo_installs_each_crate_from_its_file(self):
        for crate in ("grund", "grund-lsp"):
            with self.subTest(crate=crate):
                self.assert_answers(cargo_install(crate), crate)

    def test_each_archive_runs_alone(self):
        for product in ("grund", "grund-lsp"):
            with self.subTest(product=product):
                binary = archive(product)
                env = fresh_env(keep("grund-archive-run-") / "home")
                self.assert_answers(binary, product, env)
                ran = subprocess.run([binary, "--version"], capture_output=True, env=env)
                self.assertEqual(b"", ran.stderr)


if __name__ == "__main__":
    unittest.main()
