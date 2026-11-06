"""Development identity reaches every candidate build without compiling payloads.

§FS-distribution-candidate.6.1: only the named, clean checkout supplies the commit.
§FS-distribution-candidate.5.2: gitless installs retain that candidate's identity.
These exercise the builders of §AR-bindings.5 with the compilers stood in for;
the real multi-registry installs run in the rehearsal lane.
"""

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

from distribution_support import REPO, SHA, VERSION

sys.path.insert(0, str(REPO / "scripts" / "distribution"))
import build as builder  # noqa: E402
import matrix  # noqa: E402
import payloads  # noqa: E402
from rehearsal import rehearsal_support  # noqa: E402


class CandidateIdentityTests(unittest.TestCase):
    """§FS-distribution-candidate.6.1: refuse edits before starting any build."""

    def test_staged_unstaged_and_deleted_tracked_files_refuse_the_candidate(self):
        for status in (b" M docs/notes.md\n", b"M  docs/notes.md\n", b" D Cargo.toml\n"):
            with self.subTest(status=status), \
                    mock.patch.object(builder, "_git_head", return_value=SHA), \
                    mock.patch.object(builder.subprocess, "run") as run, \
                    mock.patch.object(builder.versions, "check") as versions:
                run.return_value.stdout = status
                with self.assertRaisesRegex(SystemExit, "uncommitted changes to tracked files"):
                    builder.build({}, SHA, Path("unused-out"), Path("unused-target"))
                versions.assert_not_called()
                self.assertEqual(1, run.call_count)
                self.assertEqual(["git", "--no-optional-locks", "status", "--porcelain",
                                  "--untracked-files=no"], run.call_args.args[0])

    def test_every_pgo_payload_gets_the_candidate_commit(self):
        row = matrix.row("linux-x64-gnu")
        payload = {"id": "grund-linux-x64-gnu", "product": "grund",
                   "target": row["rust_target"], "row": row["row"]}
        # Stop at the compiler failure: no evidence or payload is needed to see
        # the environment the actual subprocess receives.
        with mock.patch.dict(os.environ, GRUND_BUILD_COMMIT="fffffffffe"), \
                mock.patch.object(payloads.subprocess, "run") as run:
            run.return_value.returncode = 1
            with self.assertRaises(SystemExit):
                payloads.build(payload, SHA, VERSION, Path("target"), Path("work"))
            self.assertEqual(SHA[:10], run.call_args.kwargs["env"]["GRUND_BUILD_COMMIT"])

    def test_the_windows_lto_fallback_gets_the_same_commit(self):
        row = matrix.row("win32-arm64-msvc")
        with mock.patch.object(payloads.subprocess, "run") as run:
            run.return_value.returncode = 0
            payloads._exception("grund", row, Path("target"), SHA)
            self.assertEqual(SHA[:10], run.call_args_list[0].kwargs["env"]["GRUND_BUILD_COMMIT"])

    def test_cargo_packaging_gets_the_same_commit(self):
        with mock.patch.object(builder.subprocess, "run") as run, \
                mock.patch.object(builder.shutil, "copyfile"):
            builder._crates(VERSION, Path("target"), Path("out"), SHA)
            self.assertEqual(SHA[:10], run.call_args.kwargs["env"]["GRUND_BUILD_COMMIT"])

    def test_a_crate_install_gets_its_manifests_commit(self):
        rehearsal_support.cargo_install.cache_clear()
        self.addCleanup(rehearsal_support.cargo_install.cache_clear)
        candidate = {"version": VERSION, "source_sha": SHA}
        with mock.patch.object(rehearsal_support, "manifest", return_value=candidate), \
                mock.patch.object(rehearsal_support, "keep", return_value=Path("scratch")), \
                mock.patch.object(rehearsal_support, "artifact", return_value=Path("source.crate")), \
                mock.patch.object(rehearsal_support, "row", return_value={"row": "linux-x64-gnu"}), \
                mock.patch.object(rehearsal_support.tarfile, "open"), \
                mock.patch.object(rehearsal_support, "run_checked") as run:
            rehearsal_support.cargo_install("grund")
            self.assertEqual(SHA[:10], run.call_args.kwargs["env"]["GRUND_BUILD_COMMIT"])


class SourceIdentityTests(unittest.TestCase):
    """§FS-distribution-candidate.5.2: a source bundle's metadata reaches Cargo."""

    def setUp(self):
        scratch = Path.home() / "ag" / "tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temp = tempfile.TemporaryDirectory(prefix="grund-1175-source-identity-", dir=scratch)
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)

    @unittest.skipIf(os.name == "nt", "the manylinux container route runs on Linux")
    def test_the_container_receives_only_a_set_commit_variable(self):
        """§FS-distribution-candidate.6.1.1: optional Docker argv survives Bash 3.2.

        §AR-ci.6: the real entrypoint stops at a stand-in, before any PGO build.
        The interpreter is the builder's normal Bash, including macOS's Bash 3.2.
        """
        tools = self.root / "tools"
        tools.mkdir()
        docker = tools / "docker"
        docker.write_text("#!/bin/sh\n"
                          "if [ \"$1\" = info ]; then\n"
                          "  echo \"$GRUND_TEST_DOCKER_MODE\"; exit 0\n"
                          "fi\n"
                          "printf '%s\\0' \"$@\" > \"$GRUND_TEST_DOCKER_LOG\"\n"
                          "exit 43\n")
        docker.chmod(0o755)
        log = self.root / "docker-args"
        target = self.root / "target with spaces"
        home = target / "pgo-container-home"
        rustflags = "-C opt-level=2 -C link-arg=a b\n-C debuginfo=1"
        env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ["PATH"],
                   GRUND_TEST_DOCKER_LOG=str(log), RUSTFLAGS=rustflags,
                   RUSTUP_TOOLCHAIN="1.95.0")
        for name in ("BASH_ENV", "ENV", "SHELLOPTS", "BASHOPTS", "GRUND_BUILD_COMMIT"):
            env.pop(name, None)
        # The first container exits before setup or compilation. Its argv is
        # the one all subsequent container builds receive from in_build.
        for mode in ("rootless", "rootful"):
            for given in (None, SHA[:10], ""):
                with self.subTest(mode=mode, commit=given):
                    log.unlink(missing_ok=True)
                    env["GRUND_TEST_DOCKER_MODE"] = mode
                    env.pop("GRUND_BUILD_COMMIT", None)
                    if given is not None:
                        env["GRUND_BUILD_COMMIT"] = given
                    result = subprocess.run([payloads.bash(), str(REPO / "scripts" / "pgo-build.sh"),
                                             "--sha", SHA, "--target-dir", str(target),
                                             "--container", "stand-in"], env=env,
                                            capture_output=True, text=True)
                    self.assertEqual(43, result.returncode, result.stderr)
                    recorded = log.read_bytes()
                    self.assertTrue(recorded.endswith(b"\0"))
                    args = os.fsdecode(recorded[:-1]).split("\0")
                    self.assertEqual("run", args[0])
                    image = args.index("stand-in")
                    actual = {}
                    index = 1
                    while index < image:
                        option = args[index]
                        if option == "--rm":
                            actual.setdefault(option, []).append(None)
                            index += 1
                        else:
                            self.assertLess(index + 1, image, args)
                            actual.setdefault(option, []).append(args[index + 1])
                            index += 2
                    expected = {
                        "--rm": [None],
                        "-v": [f"{REPO}:{REPO}", f"{target}:{target}"],
                        "-w": [str(REPO)],
                        "-e": [f"HOME={home}", f"CARGO_HOME={home}/cargo",
                               f"RUSTUP_HOME={home}/rustup", "RUSTUP_TOOLCHAIN=1.95.0",
                               f"RUSTFLAGS={rustflags}",
                               "PYO3_PYTHON=/opt/python/cp310-cp310/bin/python3"],
                    }
                    if mode == "rootful":
                        expected["--user"] = [f"{os.getuid()}:{os.getgid()}"]
                    if given is not None:
                        expected["-e"].insert(0, f"GRUND_BUILD_COMMIT={given}")
                    self.assertEqual(expected.keys(), actual.keys())
                    for option, values in expected.items():
                        self.assertCountEqual(values, actual[option], option)
                    self.assertEqual(["bash", "-c"], args[image + 1:image + 3])
                    self.assertEqual(["bash", "bash", "-c"], args[image + 4:image + 7])
                    self.assertEqual(image + 8, len(args))
                    self.assertIn("rustup toolchain install", args[-1])

    def test_the_sdist_cli_build_uses_the_bundled_commit(self):
        script = REPO / "scripts" / "distribution" / "grund_build.py"
        spec = importlib.util.spec_from_file_location("grund_identity_backend", script)
        backend = importlib.util.module_from_spec(spec)
        # The backend delegates the extension to maturin; this test only asks
        # for its CLI build, so no installed backend or native tool is needed.
        with mock.patch.dict(sys.modules, maturin=types.ModuleType("maturin")):
            spec.loader.exec_module(backend)
        (self.root / "source-metadata.json").write_text(json.dumps({"sourceSha": SHA}))
        backend.ROOT = self.root
        backend.HERE = self.root / "build_backend"
        backend.HERE.mkdir()
        with mock.patch.dict(os.environ, GRUND_BUILD_COMMIT="fffffffffe"), \
                mock.patch.object(backend.subprocess, "run") as run, \
                mock.patch.object(backend.shutil, "copy2"):
            backend._build_cli()
            self.assertEqual(SHA[:10], run.call_args.kwargs["env"]["GRUND_BUILD_COMMIT"])

    def test_the_npm_cli_build_uses_the_bundled_commit(self):
        node = shutil.which("node")
        self.assertIsNotNone(node, "environment prerequisite missing: node")
        sources = self.root / "build" / "sources"
        sources.mkdir(parents=True)
        (self.root / "package.json").write_text(json.dumps({
            "name": "grund-cli", "bin": {"grund": "launcher.cjs"}}))
        (sources / "source-metadata.json").write_text(json.dumps({"sourceSha": SHA}))
        script = sources.parent / "source.mjs"
        shutil.copyfile(REPO / "scripts" / "distribution" / "npm" / "source-build.mjs", script)
        target = self.root / "target"
        binary = target / "x86_64-unknown-linux-gnu" / "release" / "grund"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"payload")
        # Run the actual ESM entrypoint; intercept only its subprocess calls.
        # No addon or compiler runs, and the CLI copy still uses a real file.
        intercept = """
import cp from 'node:child_process';
import {syncBuiltinESMExports} from 'node:module';
const builds = [];
cp.spawnSync = (program, args, options) => {
  if (program === 'cargo' && args.includes('build'))
    builds.push(options.env.GRUND_BUILD_COMMIT);
  return {status: 0, stdout: 'host: x86_64-unknown-linux-gnu\\n'};
};
syncBuiltinESMExports();
process.argv = ['node', 'source.mjs', '--target-dir', process.argv[2]];
await import(process.env.GRUND_TEST_SOURCE_SCRIPT);
console.log(JSON.stringify(builds));
"""
        result = subprocess.run([node, "--input-type=module", "-e", intercept, "unused", str(target)],
                                capture_output=True, text=True, check=True,
                                env=dict(os.environ, GRUND_BUILD_COMMIT="fffffffffe",
                                         GRUND_TEST_SOURCE_SCRIPT=script.as_uri()))
        self.assertEqual([SHA[:10]], json.loads(result.stdout))


if __name__ == "__main__":
    unittest.main()
