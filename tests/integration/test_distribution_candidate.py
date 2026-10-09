"""§FS-distribution-candidate.1.1, §FS-distribution-candidate.2.1,
§FS-distribution-candidate.6.1, §FS-distribution-candidate.6.4,
§FS-distribution-candidate.6.5 — the candidate tool's offline half: the matrix it
builds for, the inventory it plans, the npm package trees it writes, the
verification it applies, the receipts it binds and the versions it keeps.

Nothing here compiles or reaches a registry, so it runs in the ordinary gate on
every CI platform; the installed, compiled half is the rehearsal lane under
`tests/integration/rehearsal/` (§FS-distribution-candidate.5.6). The expected
matrix is read from the specification's table, never restated here.

The tool's shape is §AR-bindings.5's and the wheels' §AR-bindings.6's.
"""

import json
import re
import shutil
import subprocess
import tomllib
import unittest
from pathlib import Path

from distribution_support import (
    REPO, SHA, VERSION, WHEEL_TAGS, candidate, candidate_json, expected_artifacts,
    expected_payloads, npm_selectors, pep440, placements, registry_rows,
    repack_npm, rewrite_manifest, row_by_id, scratch, sha256, spec_matrix,
    synthetic_candidate, write_receipts,
)

RELEASE = REPO / ".github" / "workflows" / "release.yml"
FRAGMENT = REPO / "crates" / "grund-node" / "package-api.json"


class MatrixTests(unittest.TestCase):
    """§FS-distribution-candidate.1.1 — the tool prints the specification's table."""

    def test_the_tool_matrix_is_the_spec_table(self):
        printed = candidate_json("matrix")
        keys = ("row", "rust_target", "npm_suffix", "wheel_platform", "runner",
                "payload_floor", "registry")
        self.assertEqual([{k: r[k] for k in keys} for r in spec_matrix()],
                         [{k: r.get(k) for k in keys} for r in printed])

    def test_linux_rows_build_in_the_release_images(self):
        """§FS-distribution.4.8: the candidate uses the digests the release pins."""
        images = re.findall(r'image: "(quay\.io/pypa/manylinux2014_\w+@sha256:[0-9a-f]{64})"',
                            RELEASE.read_text(encoding="utf-8"))
        printed = {r["row"]: r.get("container") for r in candidate_json("matrix")}
        self.assertEqual(sorted(images), sorted(printed[r] for r in printed if r.startswith("linux")))
        for row, container in printed.items():
            if not row.startswith("linux"):
                self.assertIsNone(container, row)

    def test_windows_arm64_is_downloads_only(self):
        """§FS-distribution-candidate.1.4: the deferred row has archives and nothing else."""
        plan = candidate_json("plan", "--version", VERSION, "--sha", SHA)
        kinds = {a["kind"] for a in plan["artifacts"] if a.get("row") == "win32-arm64-msvc"}
        self.assertEqual({"archive"}, kinds)
        products = {p["product"] for p in plan["payloads"] if p["row"] == "win32-arm64-msvc"}
        self.assertEqual({"grund", "grund-lsp"}, products)


class PlanTests(unittest.TestCase):
    """§FS-distribution-candidate.2.1 — thirty-nine artifacts, twenty-two payloads."""

    def test_the_plan_names_exactly_the_inventory(self):
        plan = candidate_json("plan", "--version", VERSION, "--sha", SHA)
        self.assertEqual(VERSION, plan["version"])
        self.assertEqual(SHA, plan["source_sha"])
        keys = ("path", "registry", "package", "kind", "row", "version")
        self.assertEqual(sorted(tuple(a[k] for k in keys) for a in expected_artifacts()),
                         sorted(tuple(a.get(k) for k in keys) for a in plan["artifacts"]))
        self.assertEqual(39, len(plan["artifacts"]))

    def test_the_plan_names_exactly_the_payloads_and_their_placements(self):
        """§FS-distribution-candidate.3.1: the plan says where each payload is placed."""
        plan = candidate_json("plan", "--version", VERSION, "--sha", SHA)
        self.assertEqual(sorted(p["id"] for p in expected_payloads()),
                         sorted(p["id"] for p in plan["payloads"]))
        for payload in plan["payloads"]:
            with self.subTest(payload=payload["id"]):
                self.assertEqual(sorted(placements(payload)),
                                 sorted((p["artifact"], p["path"]) for p in payload["placements"]))

    def test_python_spells_a_development_version_in_pep_440(self):
        """§FS-distribution-candidate.6.2: one version, spelled per registry."""
        plan = candidate_json("plan", "--version", "0.17.1-dev", "--sha", SHA)
        versions = {(a["registry"], a["version"]) for a in plan["artifacts"]}
        self.assertEqual({("crates.io", "0.17.1-dev"), ("npm", "0.17.1-dev"),
                          ("pypi", "0.17.1.dev0"), ("github", "0.17.1-dev")}, versions)

    def test_every_api_wheel_is_abi3_and_every_lsp_wheel_is_py3_none(self):
        """§FS-distribution-candidate.1.2: one stable-ABI wheel per row serves 3.10-3.14."""
        plan = candidate_json("plan", "--version", VERSION, "--sha", SHA)
        wheels = [a["path"] for a in plan["artifacts"] if a["kind"] == "wheel"]
        self.assertEqual(10, len(wheels))
        for wheel in wheels:
            with self.subTest(wheel=wheel):
                tag = "-cp310-abi3-" if wheel.startswith("pypi/grund-") else "-py3-none-"
                self.assertIn(tag, wheel)
                self.assertIn(wheel.rsplit("-", 1)[1].removesuffix(".whl"), WHEEL_TAGS.values())


class NpmTreeTests(unittest.TestCase):
    """§FS-distribution-candidate.2.2, §FS-distribution-candidate.2.3,
    §FS-distribution-candidate.2.4 — every npm package as written before payloads."""

    @classmethod
    def setUpClass(cls):
        cls.temp = scratch("grund-npm-trees-")
        cls.root = Path(cls.temp.name)
        result = candidate("npm-trees", "--version", VERSION, "--out", cls.root)
        cls.failure = None if result.returncode == 0 else result.stderr

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def package(self, name):
        self.assertIsNone(self.failure, f"candidate.py npm-trees failed:\n{self.failure}")
        path = self.root / name / "package.json"
        self.assertTrue(path.is_file(), f"npm-trees wrote no {name}/package.json")
        return json.loads(path.read_text(encoding="utf-8"))

    def names(self):
        return ["grund-cli", "grund-lsp"] + [f"@{f}/{r['npm_suffix']}" for f in (
            "grund-cli", "grund-lsp") for r in registry_rows()]

    def test_umbrellas_depend_on_every_platform_package_at_exactly_their_version(self):
        for family in ("grund-cli", "grund-lsp"):
            with self.subTest(family=family):
                package = self.package(family)
                self.assertEqual(VERSION, package["version"])
                expected = {f"@{family}/{r['npm_suffix']}": VERSION for r in registry_rows()}
                self.assertEqual(expected, package["optionalDependencies"])
                self.assertEqual({r["rust_target"]: f"@{family}/{r['npm_suffix']}"
                                  for r in registry_rows()}, package["grund"]["platformPackages"])
                self.assertFalse(package.get("dependencies"), "umbrellas pull in nothing else")

    def test_each_platform_package_selects_its_row_by_os_cpu_and_libc(self):
        for family in ("grund-cli", "grund-lsp"):
            for row in registry_rows():
                with self.subTest(package=f"@{family}/{row['npm_suffix']}"):
                    package = self.package(f"@{family}/{row['npm_suffix']}")
                    self.assertEqual(VERSION, package["version"])
                    selectors = npm_selectors(row["npm_suffix"])
                    self.assertEqual(selectors, {k: package.get(k) for k in selectors})
                    if "libc" not in selectors:
                        self.assertNotIn("libc", package)

    def test_the_binding_fragment_reaches_grund_cli_unchanged(self):
        fragment = json.loads(FRAGMENT.read_text(encoding="utf-8"))
        package = self.package("grund-cli")
        for key, value in fragment.items():
            with self.subTest(key=key):
                if key == "files":
                    self.assertTrue(set(value) <= set(package["files"]))
                elif key == "grund":
                    self.assertEqual(value, {k: package["grund"][k] for k in value})
                    self.assertEqual({"platformPackages"}, set(package["grund"]) - set(value))
                else:
                    self.assertEqual(value, package[key])
        tracked = subprocess.run(["git", "ls-files", "*package.json"], cwd=REPO, check=True,
                                 capture_output=True, text=True).stdout.splitlines()
        umbrellas = [p for p in tracked if '"grund-cli"' in (REPO / p).read_text(encoding="utf-8")
                     or '"grund-lsp"' in (REPO / p).read_text(encoding="utf-8")]
        self.assertEqual([], umbrellas, "no second umbrella manifest is committed")

    def test_the_commands_and_launchers_are_where_the_bins_say(self):
        """§FS-distribution-candidate.3.1: `grund` and `grund-lsp` map to launchers."""
        for family, command in (("grund-cli", "grund"), ("grund-lsp", "grund-lsp")):
            with self.subTest(family=family):
                package = self.package(family)
                self.assertEqual([command], list(package["bin"]))
                self.assertTrue((self.root / family / package["bin"][command]).is_file())
        for family, exports in (("grund-cli", ("addonPath", "executablePath", "metadata")),
                                ("grund-lsp", ("executablePath", "metadata"))):
            for row in registry_rows():
                index = self.root / f"@{family}/{row['npm_suffix']}" / "index.cjs"
                with self.subTest(index=str(index.relative_to(self.root))):
                    self.assertTrue(index.is_file())
                    text = index.read_text(encoding="utf-8")
                    for name in exports:
                        self.assertIn(name, text)

    def test_no_package_compiles_on_install_and_both_offer_a_source_build(self):
        """§FS-distribution-candidate.4.1, §FS-distribution-candidate.4.3."""
        for name in self.names():
            with self.subTest(package=name):
                scripts = self.package(name).get("scripts", {})
                self.assertFalse({"preinstall", "install", "postinstall"} & set(scripts))
        for family in ("grund-cli", "grund-lsp"):
            self.assertIn("build:source", self.package(family)["scripts"])

    def test_every_package_says_what_it_holds_and_carries_the_licence(self):
        """§FS-distribution-candidate.2.4, §FS-distribution.1.2."""
        licence = (REPO / "LICENSE").read_bytes()
        for name in self.names():
            with self.subTest(package=name):
                package = self.package(name)
                self.assertEqual("MIT", package["license"])
                self.assertIn("agent-grounds/grund", json.dumps(package["repository"]))
                self.assertEqual(licence, (self.root / name / "LICENSE").read_bytes())
                readme = (self.root / name / "README.md").read_text(encoding="utf-8")
                self.assertIn("github.com/agent-grounds/grund", readme)
                if name.startswith("@"):
                    family, suffix = name[1:].split("/")
                    self.assertIn(suffix, readme)
                    self.assertIn(f"`{family}`", readme)
                if "grund-lsp" in name:
                    self.assertNotIn("grund-cli", json.dumps(package.get("dependencies", {})))
                    self.assertNotIn("grund-cli", json.dumps(package.get("optionalDependencies", {})))


class VerifyTests(unittest.TestCase):
    """§FS-distribution-candidate.6.1, §FS-distribution-candidate.6.4 — a complete
    candidate verifies, and each way of breaking one is refused by name."""

    def setUp(self):
        self.temp = scratch("grund-candidate-verify-")
        self.addCleanup(self.temp.cleanup)
        self.root, self.digest = synthetic_candidate(Path(self.temp.name) / "candidate")

    def verify(self, *extra, root=None):
        return candidate("verify", root or self.root, "--release", "--sha", SHA,
                         "--tag", f"v{VERSION}", *extra)

    def refused(self, naming, *extra):
        result = self.verify(*extra)
        self.assertNotEqual(0, result.returncode, "verify accepted a broken candidate")
        self.assertIn(naming, result.stderr)
        return result

    def test_a_complete_candidate_verifies(self):
        result = self.verify()
        self.assertEqual(0, result.returncode, result.stderr)

    def test_a_development_version_is_not_a_release(self):
        dev = Path(self.temp.name) / "dev"
        synthetic_candidate(dev, version="0.17.1-dev")
        result = candidate("verify", dev, "--release", "--sha", SHA, "--tag", "v0.17.1-dev")
        self.assertNotEqual(0, result.returncode)
        self.assertIn("0.17.1-dev", result.stderr)

    def test_the_sha_and_the_tag_must_be_the_manifests(self):
        result = candidate("verify", self.root, "--release", "--sha", "f" * 40, "--tag", f"v{VERSION}")
        self.assertNotEqual(0, result.returncode)
        self.assertIn("f" * 40, result.stderr)
        result = candidate("verify", self.root, "--release", "--sha", SHA, "--tag", "v9.9.9")
        self.assertNotEqual(0, result.returncode)
        self.assertIn("v9.9.9", result.stderr)

    def test_a_missing_artifact_is_named(self):
        (self.root / "pypi" / f"grund-{VERSION}.tar.gz").unlink()
        self.refused(f"pypi/grund-{VERSION}.tar.gz")

    def test_an_unexpected_file_is_named(self):
        (self.root / "npm" / "stray-1.0.0.tgz").write_bytes(b"stray")
        self.refused("npm/stray-1.0.0.tgz")

    def test_an_artifact_whose_bytes_changed_is_named(self):
        path = self.root / "cargo" / f"grund-{VERSION}.crate"
        path.write_bytes(path.read_bytes() + b"\0")
        self.refused(f"cargo/grund-{VERSION}.crate")

    def test_an_archive_checksum_file_must_agree(self):
        """§FS-distribution-candidate.8.6: the `.sha256` beside an archive is checked."""
        name = f"grund-{VERSION}-x86_64-unknown-linux-gnu.tar.gz"
        (self.root / "archives" / f"{name}.sha256").write_text(f"{'0' * 64}  {name}\n")
        self.refused(name)

    def test_a_version_inside_an_artifact_must_be_the_candidates(self):
        """Read from the package itself, not from the manifest's claim about it."""
        path = self.root / "npm" / f"grund-cli-linux-x64-gnu-{VERSION}.tgz"

        def bump(members):
            package = json.loads(members["package/package.json"])
            package["version"] = "0.17.9"
            members["package/package.json"] = json.dumps(package).encode()
        digest = repack_npm(path, bump)
        rewrite_manifest(self.root, lambda m: [a.update(sha256=digest) for a in m["artifacts"]
                                               if a["path"] == f"npm/{path.name}"])
        self.refused(f"npm/{path.name}")

    def test_two_engine_versions_are_refused(self):
        self.root, _ = synthetic_candidate(Path(self.temp.name) / "mixed",
                                           engine_overrides={"grund-lsp-darwin-x64": "0.16.9"})
        result = self.verify(root=self.root)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("grund-lsp-darwin-x64", result.stderr)

    def test_a_placed_payload_must_be_the_built_one(self):
        """§FS-distribution-candidate.7.3: the bytes at a placement are the build's."""
        path = self.root / "npm" / f"grund-cli-darwin-arm64-{VERSION}.tgz"
        digest = repack_npm(path, lambda m: m.update({"package/bin/grund": b"rebuilt\n"}))
        rewrite_manifest(self.root, lambda m: [a.update(sha256=digest) for a in m["artifacts"]
                                               if a["path"] == f"npm/{path.name}"])
        self.refused("grund-darwin-arm64")

    def test_an_lto_exception_is_accepted_only_on_windows_arm64(self):
        """§FS-distribution-candidate.7.4."""
        allowed, _ = synthetic_candidate(Path(self.temp.name) / "arm",
                                         exception_row="win32-arm64-msvc")
        result = self.verify(root=allowed)
        self.assertEqual(0, result.returncode, result.stderr)
        refused, _ = synthetic_candidate(Path(self.temp.name) / "x64", exception_row="linux-x64-gnu")
        result = self.verify(root=refused)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("lto-exception", result.stderr)
        self.assertIn("linux-x64-gnu", result.stderr)
        rewrite_manifest(allowed, lambda m: [p["exception"].update(failure="training produced no profile")
                                             for p in m["payloads"] if p["exception"]])
        result = self.verify(root=allowed)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("lto-exception for 'training produced no profile'", result.stderr)

    def test_a_binding_only_candidate_is_not_a_release(self):
        """§FS-distribution-candidate.6.3."""
        binding, _ = synthetic_candidate(Path(self.temp.name) / "binding", scope="binding-only")
        result = self.verify(root=binding)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("binding-only", result.stderr)

    def test_an_artifact_the_plan_names_cannot_be_dropped_from_the_manifest(self):
        gone = f"npm/grund-lsp-win32-x64-msvc-{VERSION}.tgz"
        (self.root / gone).unlink()
        rewrite_manifest(self.root, lambda m: m.update(
            artifacts=[a for a in m["artifacts"] if a["path"] != gone]))
        self.refused(gone)


class ShareTests(unittest.TestCase):
    """§FS-distribution-candidate.6.5 — a row's receipt binds to the candidate it is a
    share of, whichever runner wrote the share: every manifest is LF bytes and hashed
    as written (§FS-distribution-candidate.6.1). Windows is where a text-mode write
    would differ, so the windows-latest gate is the one that holds this."""

    ROW = "win32-x64-msvc"

    def setUp(self):
        temp = scratch("grund-share-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.built, self.digest = synthetic_candidate(self.root / "candidate", receipts=False)
        result = candidate("share", self.built, "--row", self.ROW, "--out", self.root / "share")
        self.assertEqual(0, result.returncode, result.stderr)

    def bind(self, data):
        """Receipt the share as run.py does, naming its manifest's bytes, then bind it."""
        (self.root / "share" / "manifest.json").write_bytes(data)
        write_receipts(self.root / "share", sha256(data), [row_by_id(self.ROW)])
        return candidate("receipts", self.built, self.root / "share")

    def test_a_shares_receipt_binds_to_the_candidate(self):
        data = (self.root / "share" / "manifest.json").read_bytes()
        self.assertNotIn(b"\r", data)
        result = self.bind(data)
        self.assertEqual(0, result.returncode, result.stderr)
        bound = json.loads((self.built / "receipts" / f"{self.ROW}.json").read_text())
        self.assertEqual([self.digest, sha256(data)],
                         [bound["manifest_sha256"], bound["share_manifest_sha256"]])

    def test_a_crlf_share_is_not_the_candidates_share(self):
        lines = (self.root / "share" / "manifest.json").read_bytes().splitlines()
        result = self.bind(b"\r\n".join(lines) + b"\r\n")
        self.assertNotEqual(0, result.returncode, "receipts bound a CRLF share")
        self.assertIn(f"is not row {self.ROW}'s share of {self.digest}", result.stderr)

    def test_an_assembled_manifest_is_lf(self):
        out = self.root / "assembled"
        result = candidate("assemble", "--sha", SHA, "--out", out, self.root / "share")
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertNotIn(b"\r", (out / "manifest.json").read_bytes())


class VersionTests(unittest.TestCase):
    """§FS-distribution-candidate.6.2, §FS-distribution.4.5 — every version source,
    read and written by one tool."""

    def copy(self):
        temp = scratch("grund-versions-")
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        tracked = subprocess.run(["git", "ls-files", "Cargo.toml", "Cargo.lock", "pyproject.toml",
                                  "crates", "python"], cwd=REPO, capture_output=True, text=True,
                                 check=True).stdout.splitlines()
        for name in tracked:
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / name, root / name)
        return root

    def workspace_version(self, root=REPO):
        return tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]

    def test_this_tree_carries_one_version(self):
        version = self.workspace_version()
        result = candidate("versions", "--root", REPO, "--expect", version)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(pep440(version),
                         tomllib.loads((REPO / "pyproject.toml").read_text())["project"]["version"])

    def test_a_disagreeing_source_is_named(self):
        root = self.copy()
        pyproject = root / "pyproject.toml"
        pyproject.write_text(re.sub(r'(?m)^version = ".*"$', 'version = "9.9.9"',
                                    pyproject.read_text(), count=1))
        result = candidate("versions", "--root", root)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("pyproject.toml", result.stderr)

    def test_an_unexpected_version_is_refused(self):
        result = candidate("versions", "--root", REPO, "--expect", "9.9.9")
        self.assertNotEqual(0, result.returncode)
        self.assertIn("9.9.9", result.stderr)

    def test_a_ref_is_read_from_history_not_the_working_tree(self):
        """§FS-distribution.4.3: the verify job reads the release source ref, the tag
        when it recovers one, so the working tree must not stand in for it."""
        root = self.copy()
        # Nothing git starts may write into the repository after the test returns.
        git = ["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t",
               "-c", "maintenance.auto=false", "-c", "gc.auto=0", "-c", "core.fsmonitor=false"]
        subprocess.run([*git, "init", "-q"], check=True)
        subprocess.run([*git, "add", "-A"], check=True)
        subprocess.run([*git, "commit", "-qm", "release"], check=True)
        version = self.workspace_version(root)
        candidate("set-version", "9.9.9", "--root", root)
        result = candidate("versions", "--root", root, "--ref", "HEAD", "--expect", version)
        self.assertEqual(0, result.returncode, result.stderr)

    def test_set_version_writes_every_source(self):
        root = self.copy()
        for version in ("0.17.0", "0.17.1-dev"):
            with self.subTest(version=version):
                result = candidate("set-version", version, "--root", root)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertEqual(version, self.workspace_version(root))
                self.assertEqual(pep440(version), tomllib.loads(
                    (root / "pyproject.toml").read_text())["project"]["version"])
                for crate in ("grund-cli", "grund-lsp"):
                    manifest = tomllib.loads((root / "crates" / crate / "Cargo.toml").read_text())
                    self.assertEqual(version, manifest["dependencies"]["grund-core"]["version"])
                lock = (root / "Cargo.lock").read_text()
                self.assertIn(f'name = "grund-core"\nversion = "{version}"', lock)
                result = candidate("versions", "--root", root, "--expect", version)
                self.assertEqual(0, result.returncode, result.stderr)


if __name__ == "__main__":
    unittest.main()
