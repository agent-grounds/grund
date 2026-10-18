"""Real preparation receipts pin §AR-ci.3.4.4 and §AR-ci.10.5 for #809.

Cargo builds a tiny offline workspace, not Grund's dependency tree, and Git still
supplies real commits. No elapsed-time threshold stands in for avoided work.
"""

import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from unittest import mock

import distribution_support
import test_distribution_candidate as version_tests

REPO = Path(__file__).resolve().parents[2]
PREPARE = REPO / "scripts/prepare_binding_oracle.py"


class AvoidableWorkTests(unittest.TestCase):
    def setUp(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="grund-809-ci-inputs-", dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def command(self, *argv, **kwargs):
        result = subprocess.run(list(map(str, argv)), cwd=self.root,
                                capture_output=True, text=True, **kwargs)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return result.stdout.strip()

    def git(self, *argv):
        return self.command("git", "-c", "user.name=t", "-c", "user.email=t@t",
                            "-c", "maintenance.auto=false", "-c", "gc.auto=0", *argv)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def workspace(self):
        self.write("Cargo.toml", '[workspace]\nmembers=["core"]\nexclude=["dep"]\n'
                   'resolver="3"\n[profile.dev.package."*"]\nopt-level=1\n')
        self.write("core/Cargo.toml", '[package]\nname="grund-core"\nversion="0.1.0"\n'
                   'edition="2024"\n[dependencies]\ndep-fixture={path="../dep"}\n')
        self.write("core/src/lib.rs", "pub fn unused() {}\n")
        self.write("dep/Cargo.toml", '[package]\nname="dep-fixture"\nversion="0.1.0"\n'
                   'edition="2024"\n')
        self.write("dep/src/lib.rs", "pub fn value() -> u8 { 7 }\n")
        # The non-workspace dependency's build script runs once per compatible
        # target/profile. A second receipt proves real dependency work repeated.
        self.write("dep/build.rs", '''use std::io::Write;
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut log = std::fs::OpenOptions::new().create(true).append(true)
        .open(std::env::var("GRUND_809_DEP_RECEIPT").unwrap()).unwrap();
    writeln!(log, "compiled dependency").unwrap();
}
''')
        self.oracle_source("first")
        self.command("cargo", "+1.95.0", "generate-lockfile", "--offline")
        self.git("init", "-q")
        self.commit()

    def oracle_source(self, behavior):
        self.write("core/examples/grund-binding-oracle.rs", '''fn main() {
    assert_eq!(dep_fixture::value(), 7);
    println!("{{\\\"protocolVersion\\\":1,\\\"sourceSha\\\":\\\"{}\\\",\\\"behavior\\\":\\\"BEHAVIOR\\\"}}",
             env!("GRUND_BINDINGS_SOURCE_SHA"));
}
'''.replace("BEHAVIOR", behavior))

    def commit(self):
        self.git("add", "Cargo.toml", "Cargo.lock", "core", "dep")
        self.git("commit", "-qm", "fixture source")
        return self.git("rev-parse", "HEAD")

    def prepare(self, behavior):
        environment = self.root / "github-env"
        environment.write_text("", encoding="utf-8")
        # Only redirect the old helper's global scratch: the real entrypoint and
        # actual pinned Cargo command run unchanged in the fixture checkout.
        previous = Path.cwd()
        try:
            os.chdir(self.root)
            with mock.patch("pathlib.Path.home", return_value=self.root / "home"), \
                    mock.patch.dict(os.environ, GITHUB_ENV=str(environment)):
                runpy.run_path(str(PREPARE), run_name="__main__")
        finally:
            os.chdir(previous)
        exports = dict(line.split("=", 1) for line in environment.read_text().splitlines())
        oracle = Path(exports["GRUND_BINDINGS_ORACLE"])
        self.assertTrue(oracle.is_absolute())
        metadata = json.loads(self.command(oracle, "--metadata"))
        self.assertEqual(self.git("rev-parse", "HEAD"), metadata["sourceSha"])
        self.assertEqual(1, metadata["protocolVersion"])
        self.assertEqual(behavior, metadata["behavior"], "a stale executable was exported")
        return oracle.parents[2]

    def test_oracle_cold_restored_changed_source_and_incompatible_flags(self):
        """§AR-ci.3.4.4: a cache miss works; a compatible restore avoids dependency work."""
        # Also cover the target override used by local gates, with the same
        # restore/prune/build lifecycle as the default workflow target.
        parent = self.root
        for override in (False, True):
            # TemporaryDirectory owns cleanup, including read-only Git objects
            # on Windows; each variant gets independent history and targets.
            self.root = parent / ("overridden" if override else "default")
            self.root.mkdir()
            with self.subTest(target_override=override), mock.patch.dict(os.environ):
                os.environ.pop("CARGO_ENCODED_RUSTFLAGS", None)
                os.environ.pop("CARGO_TARGET_DIR", None)
                os.environ["CARGO_INCREMENTAL"] = "0"
                os.environ["RUSTFLAGS"] = ""
                receipt = self.root / "dependency-receipt"
                os.environ["GRUND_809_DEP_RECEIPT"] = str(receipt)
                expected = self.root / ("custom-target" if override else "target")
                if override:
                    os.environ["CARGO_TARGET_DIR"] = str(expected)
                self.workspace()
                cold = self.prepare("first")
                counts = [len(receipt.read_text().splitlines())]
                # Model the existing workflow: only its listed target is
                # restored, and workspace artifacts are pruned before saving.
                if cold != expected:
                    shutil.copytree(cold, expected)
                    shutil.rmtree(cold)
                self.command("cargo", "+1.95.0", "clean", "--locked", "-p", "grund-core",
                             "--target-dir", expected)
                self.prepare("first")
                counts.append(len(receipt.read_text().splitlines()))
                self.oracle_source("changed")
                self.commit()
                self.prepare("changed")
                counts.append(len(receipt.read_text().splitlines()))
                os.environ["RUSTFLAGS"] = "-Copt-level=2"
                self.prepare("changed")
                counts.append(len(receipt.read_text().splitlines()))
                self.assertEqual([1, 1, 1, 2], counts,
                                 "compatible restored dependencies were rebuilt; only the "
                                 "cold and incompatible-flag builds may compile them")

    def test_version_fixture_copies_complete_inputs_without_crate_payloads(self):
        """§AR-ci.10.5: byte-complete version inputs, with no unrelated copy operations."""
        helper = version_tests.VersionTests("test_set_version_writes_every_source")
        self.addCleanup(helper.doCleanups)
        manifest = tomllib.loads((REPO / "Cargo.toml").read_text())
        required = {"Cargo.toml", "Cargo.lock", version_tests.PYPROJECT}
        required.update(f"{member}/Cargo.toml" for member in manifest["workspace"]["members"]
                        if (REPO / member / "Cargo.toml").is_file())
        with mock.patch.object(distribution_support, "SCRATCH", self.root), \
                mock.patch.object(shutil, "copyfile", wraps=shutil.copyfile) as copies:
            root = helper.copy()
        present = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
        for name in sorted(required & present):
            self.assertEqual((REPO / name).read_bytes(), (root / name).read_bytes(), name)
        result = version_tests.candidate("versions", "--root", root,
                                         "--expect", helper.workspace_version(root))
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        unrelated = sorted(present - required)
        copied = {Path(call.args[0]).relative_to(REPO).as_posix() for call in copies.call_args_list}
        self.assertEqual(([], [], []),
                         (sorted(required - present), unrelated[:8], sorted(copied - required)[:8]),
                         f"copied {len(unrelated)} files outside the version-input closure; "
                         "all member manifests are required, and copying then deleting "
                         "payloads still pays the cost")


if __name__ == "__main__":
    unittest.main()
