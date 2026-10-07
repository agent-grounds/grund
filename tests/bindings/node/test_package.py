"""§FS-distribution.3.2.3 — package fragment, source closure, loading and TS."""

import json
import shutil
import unittest

from launch import cargo_probe
from support import NODE, REPO, Rehearsal, checked


class NodePackageTests(unittest.TestCase):
    def setUp(self):
        self.r = Rehearsal()
        self.addCleanup(self.r.close)

    def test_package_contents_metadata_and_single_fragment(self):
        fragment = json.loads((NODE / "package-api.json").read_text())
        manifest = json.loads((self.r.package / "package.json").read_text())
        for field in ("exports", "types"):
            self.assertEqual(fragment[field], manifest[field])
        self.assertEqual("grund-cli", manifest["name"])
        self.assertEqual("^22.0.0 || ^24.0.0", manifest["engines"]["node"])
        self.assertNotIn("bin", manifest, "API-only rehearsal makes no CLI claim")
        self.assertNotIn("grund-lsp", json.dumps(manifest))
        for name in ("index.mjs", "index.cjs", "index.d.mts", "index.d.cts",
                     "README.md", "LICENSE", "native/grund.node", "build/source.mjs"):
            self.assertTrue((self.r.package / name).is_file(), name)
        sources = self.r.package / "build" / "sources"
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                     "crates/grund-core/Cargo.toml", "crates/grund-node/Cargo.toml"):
            self.assertTrue((sources / name).is_file(), "bundled " + name)
        self.assertTrue((sources / "crates/grund-core/assets").is_dir())
        graph = json.loads(checked(["cargo", "metadata", "--no-deps", "--format-version", "1",
                                   "--manifest-path", sources / "Cargo.toml"]).stdout)
        for member in graph["packages"]:
            self.assertTrue(str(member["manifest_path"]).startswith(str(sources)),
                            "source closure reaches outside installed package")
        self.assertFalse((NODE / "package.json").exists(), "no competing assembly manifest")
        self.assertEqual("node build/source.mjs", manifest["scripts"]["build:source"])
        for condition in ("import", "require"):
            keys = list(manifest["exports"]["."][condition])
            self.assertLess(keys.index("types"), keys.index("default"))
        result = checked(["node", "-e",
                          "const n=require('./native/grund.node');"
                          "if ('__test' in n) throw Error('production fault seam');"
                          "process.stdout.write(JSON.stringify(n.metadata));"],
                         cwd=self.r.package)
        metadata = json.loads(result.stdout)
        self.assertEqual(1, metadata["apiSchemaVersion"])
        self.assertEqual(8, metadata["napiVersion"])
        self.assertEqual(manifest["version"], metadata["packageVersion"])
        self.assertEqual(manifest["version"], metadata["engineVersion"])

    def test_bundled_source_fallback_works_without_checkout_access(self):
        native = self.r.package / "native"
        shutil.rmtree(native)
        # Intercept Cargo and refuse any manifest outside the installed source closure.
        sources = self.r.package / "build" / "sources"
        probes = self.r.root / "cargo-probe"
        trace = self.r.root / "cargo-invocations.jsonl"
        real_cargo = shutil.which("cargo")
        env = cargo_probe(probes,
            "import json, os, pathlib, subprocess, sys\n"
            f"allowed = pathlib.Path({str(sources)!r}).resolve()\n"
            "args = sys.argv[1:]\n"
            "manifest = pathlib.Path(args[args.index('--manifest-path') + 1]).resolve() "
            "if '--manifest-path' in args else pathlib.Path.cwd().resolve()\n"
            "assert manifest == allowed or allowed in manifest.parents, "
            "'source build tried a manifest outside bundled sources'\n"
            f"with open({str(trace)!r}, 'a') as log: "
            "log.write(json.dumps({'manifest': str(manifest), 'args': args}) + '\\n')\n"
            f"sys.exit(subprocess.call([{real_cargo!r}, *args]))\n"
        )
        checked(["npm", "run", "build:source", "--", "--target-dir",
                 self.r.root / "fallback-target"], cwd=self.r.package, env=env)
        self.assertTrue(trace.is_file(), "source builder must invoke Cargo")
        invocations = [json.loads(line) for line in trace.read_text().splitlines()]
        self.assertTrue(any("--locked" in call["args"] for call in invocations))
        self.assertTrue((native / "grund.node").is_file())
        self.assertEqual("", self.r.run("operations.mjs"))
        # A bundled build recipe may never embed the checkout's absolute location.
        for recipe in (self.r.package / "build").rglob("*.mjs"):
            self.assertNotIn(str(REPO), recipe.read_text())

    def test_missing_and_corrupt_local_addons_reject_load_after_import(self):
        native = self.r.package / "native" / "grund.node"
        native.unlink()
        self.assertEqual("", self.r.run("load.mjs"))
        native.write_bytes(b"this is not a native addon")
        self.assertEqual("", self.r.run("load.mjs"))
        self.assertEqual(b"this is not a native addon", native.read_bytes(),
                         "loading never downloads or automatically compiles")

    def test_incompatible_local_metadata_fails_without_platform_fallback(self):
        metadata_file = self.r.package / "native" / "metadata.json"
        metadata = json.loads(metadata_file.read_text())
        metadata["packageVersion"] = "0.0.0-mismatch"
        metadata_file.write_text(json.dumps(metadata))
        self.assertEqual("", self.r.run("load.mjs"))

    def test_local_precedence_and_exact_platform_payload_metadata(self):
        native = self.r.package / "native"
        metadata = json.loads((native / "metadata.json").read_text())
        addon = self.r.root / "platform" / "grund.node"
        addon.parent.mkdir()
        shutil.copyfile(native / "grund.node", addon)
        descriptor = {"addonPath": str(addon), "metadata": metadata}
        env = {"GRUND_TEST_PLATFORM_PAYLOAD": json.dumps(descriptor),
               "GRUND_TEST_EXPECT_LOCAL": "yes"}
        self.assertEqual("", self.r.run("platform-load.cjs", extra_env=env))
        env["GRUND_TEST_EXPECT_LOAD"] = "yes"
        (native / "grund.node").write_bytes(b"corrupt local artifact")
        self.assertEqual("", self.r.run("platform-load.cjs", extra_env=env))
        shutil.copyfile(addon, native / "grund.node")
        (native / "metadata.json").write_text(json.dumps(dict(metadata, apiSchemaVersion=99)))
        self.assertEqual("", self.r.run("platform-load.cjs", extra_env=env))
        (native / "metadata.json").write_text(json.dumps(metadata))
        shutil.rmtree(native)
        env["GRUND_TEST_EXPECT_LOCAL"] = "no"
        env["GRUND_TEST_EXPECT_LOAD"] = "no"
        self.assertEqual("", self.r.run("platform-load.cjs", extra_env=env))
        for key, value in (("packageVersion", "0.0.0-mismatch"),
                           ("engineVersion", "0.0.0-mismatch"),
                           ("target", "incompatible-target"),
                           ("apiSchemaVersion", 99), ("napiVersion", 9)):
            with self.subTest(metadata=key):
                wrong = dict(metadata, **{key: value})
                env["GRUND_TEST_PLATFORM_PAYLOAD"] = json.dumps(
                    {"addonPath": str(addon), "metadata": wrong})
                env["GRUND_TEST_EXPECT_LOAD"] = "yes"
                self.assertEqual("", self.r.run("platform-load.cjs", extra_env=env))

    def test_strict_typescript_esm_and_cjs_consumers(self):
        checked(["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund",
                 "--save-dev", "--save-exact", "typescript@5.9.3"],
                cwd=self.r.consumer)
        for name in ("consumer.mts", "consumer.cts"):
            shutil.copyfile(REPO / "tests" / "bindings" / "node" / name,
                            self.r.consumer / name)
        checked(["node", "node_modules/typescript/bin/tsc", "--strict",
                 "--noEmit", "--module", "NodeNext", "--moduleResolution", "NodeNext",
                 "--target", "ES2022", "consumer.mts", "consumer.cts"],
                cwd=self.r.consumer)

    def test_usage_documentation_and_captured_example(self):
        guide = REPO / "docs/user-facing/node-api.md"
        self.assertTrue(guide.is_file(), "approved user-facing Node API documentation")
        text = guide.read_text()
        for name in ("check", "scan", "show", "showBatch", "refs", "list", "listSizes",
                     "cover", "fmt", "proposeId", "init", "completeIds", "effectiveConfig",
                     "validateConfig", "fetch", "integrations", "referenceStyle",
                     "agentSetupInstructions"):
            self.assertIn(name, text)
        self.assertIn("npm run build:source", text)
        self.assertTrue((NODE / "README.md").is_file())
        example = REPO / "examples/node-api/example.mjs"
        golden = REPO / "examples/node-api/expected.stdout"
        self.assertTrue(example.is_file())
        self.assertTrue(golden.is_file())
        shutil.copyfile(example, self.r.consumer / "example.mjs")
        result = checked(["node", "example.mjs", self.r.fixture], cwd=self.r.consumer)
        self.assertEqual("", result.stderr)
        self.assertEqual(golden.read_bytes(), result.stdout.encode())

    def test_source_builder_refuses_abort_panic_strategy(self):
        output = self.r.root / "abort-native"
        from support import command
        # §FS-distribution.3.2.3.3: refusal precedes Cargo even with combined inputs.
        probes = self.r.root / "abort-cargo-probe"
        trace = self.r.root / "abort-cargo-invoked"
        probe_env = cargo_probe(probes,
            "from pathlib import Path\n"
            f"Path({str(trace)!r}).touch()\n"
            "raise SystemExit(77)\n"
        )
        inputs = [
            ("release", {"RUSTFLAGS": "-C panic=abort"}),
            ("release", {"CARGO_ENCODED_RUSTFLAGS": "-C\x1fpanic=abort"}),
            ("release", {"CARGO_PROFILE_RELEASE_PANIC": "abort",
                         "RUSTFLAGS": "-C debuginfo=0"}),
            ("dev", {"CARGO_PROFILE_DEV_PANIC": "abort",
                     "RUSTFLAGS": "-C debuginfo=0"}),
        ]
        for profile, settings in inputs:
            with self.subTest(profile=profile, settings=settings):
                env = {key: value for key, value in probe_env.items() if key not in (
                    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_RELEASE_PANIC",
                    "CARGO_PROFILE_DEV_PANIC")}
                env.update(settings)
                result = command(["node", NODE / "build.mjs", "--source-root", REPO,
                                  "--out-dir", output, "--target-dir", self.r.target,
                                  "--profile", profile], env=env)
                self.assertNotEqual(0, result.returncode, "supported builder requires unwinding")
                self.assertIn("grund-node requires panic=unwind", result.stderr)
                self.assertFalse(trace.exists(), "abort refusal must happen before Cargo")
                self.assertFalse((output / "grund.node").exists())
        # Benign flags and an unused profile's setting must reach the Cargo probe.
        env.update(RUSTFLAGS="-C debuginfo=0", CARGO_PROFILE_RELEASE_PANIC="unwind",
                   CARGO_PROFILE_DEV_PANIC="abort")
        result = command(["node", NODE / "build.mjs", "--source-root", REPO,
                          "--out-dir", output, "--target-dir", self.r.target,
                          "--profile", "release"], env=env)
        self.assertTrue(trace.exists(), "benign active build inputs must not be rejected")
        self.assertIn("cargo failed with status 77", result.stderr)
        self.assertFalse((output / "grund.node").exists())


if __name__ == "__main__":
    unittest.main()
