"""§FS-distribution.1.1, §FS-distribution.1.2, §FS-distribution.1.3 — the
package layer of the distribution spec, read off the manifests and the
pre-release name guard that actually carry it.

None of this reaches a registry: §1.1's table is asserted against the guard
script's own calls rather than against the live registries, so the claim that
the release re-verifies *these* names is testable offline. The manifests are
parsed rather than grepped, because what §1.3 forbids is a dependency edge, not
a spelling."""

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CRATES = REPO_ROOT / "crates"
NAME_GUARD = REPO_ROOT / "scripts" / "check-registry-names.sh"

CALL = re.compile(
    r"^(check_claimed_json_name|notice_external_json_name)"
    r'\s+"([^"]+)"\s+"([^"]+)"\s+"([^"]+)"\s*$',
    re.M,
)

REGISTRY_URL = {
    "crates.io": "https://crates.io/api/v1/crates/{name}",
    "npm": "https://registry.npmjs.org/{name}",
    "pypi": "https://pypi.org/pypi/{name}/json",
}


def guard_calls():
    found = {"check_claimed_json_name": {}, "notice_external_json_name": {}}
    for kind, registry, name, url in CALL.findall(NAME_GUARD.read_text(encoding="utf-8")):
        found[kind][(registry, name)] = url
    return found


def manifest(package):
    return tomllib.loads((CRATES / package / "Cargo.toml").read_text(encoding="utf-8"))


def workspace_manifests():
    root = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    loaded = {}
    for member in root["workspace"]["members"]:
        path = REPO_ROOT / member / "Cargo.toml"
        document = tomllib.loads(path.read_text(encoding="utf-8"))
        loaded[document["package"]["name"]] = document
    return loaded


class PackageNameTests(unittest.TestCase):
    """§FS-distribution.1.1 — the names the release claims, and the one it only
    watches. The guard is where the table lives, so the table is asserted
    there: a rename that forgot a registry would otherwise ship unnoticed."""

    @classmethod
    def setUpClass(cls):
        cls.calls = guard_calls()

    def test_the_claimed_names_are_exactly_the_ones_the_spec_sets(self):
        self.assertEqual(
            {
                ("crates.io", "grund-core"),
                ("crates.io", "grund"),
                ("crates.io", "grund-lsp"),
                ("npm", "grund-cli"),
                ("npm", "grund-lsp"),
                ("pypi", "grund"),
                ("pypi", "grund-lsp"),
            },
            set(self.calls["check_claimed_json_name"]),
        )

    def test_npm_claims_grund_cli_because_the_bare_name_is_occupied(self):
        """The asymmetry §1.1 records: PyPI takes the bare name, npm takes
        `grund-cli`, and the bare npm name is watched rather than claimed."""
        claimed = set(self.calls["check_claimed_json_name"])
        self.assertIn(("npm", "grund-cli"), claimed)
        self.assertNotIn(("npm", "grund"), claimed)
        self.assertIn(("pypi", "grund"), claimed)
        self.assertEqual({("npm", "grund")}, set(self.calls["notice_external_json_name"]))

    def test_every_name_is_queried_at_its_own_registry(self):
        for kind, calls in self.calls.items():
            for (registry, name), url in calls.items():
                with self.subTest(call=kind, registry=registry, name=name):
                    self.assertEqual(REGISTRY_URL[registry].format(name=name), url)

    def test_an_unavailable_claimed_name_fails_the_guard(self):
        """A notice is not a refusal: the two call shapes differ in exactly
        that, which is what makes the npm squat survivable and a lost claim
        not."""
        text = NAME_GUARD.read_text(encoding="utf-8")
        claimed = text.split("check_claimed_json_name() {", 1)[1].split("\n}\n", 1)[0]
        noticed = text.split("notice_external_json_name() {", 1)[1].split("\n}\n", 1)[0]
        self.assertIn("is already taken by another project", claimed)
        self.assertIn("return 1", claimed)
        self.assertNotIn("return 1", noticed)
        self.assertIn("set -euo pipefail", text)


class SupportPackageReadmeTests(unittest.TestCase):
    """§FS-distribution.1.2 — the support package's registry README points at
    the CLI. `grund-lsp` is not asserted here: §1.2 names siblings "once they
    exist" as published packages, and no release has published one yet."""

    @classmethod
    def setUpClass(cls):
        cls.package = manifest("grund-core")

    def test_the_readme_key_resolves_to_a_file_the_package_ships(self):
        readme = self.package["package"]["readme"]
        path = CRATES / "grund-core" / readme
        self.assertTrue(path.is_file(), f"grund-core's readme key names {readme}, which is missing")
        self.assertIn(readme, self.package["package"]["include"], "the README is not packaged")

    def test_the_support_readme_sends_users_to_the_cli_package(self):
        readme = (CRATES / "grund-core" / self.package["package"]["readme"]).read_text(encoding="utf-8")
        cli = manifest("grund-cli")["package"]["name"]
        self.assertIn(f"https://crates.io/crates/{cli}", readme)
        self.assertIn(f"cargo install {cli}", readme)

    def test_the_support_package_is_not_itself_a_cli(self):
        """"Most users should install the CLI crate instead" stays honest only
        while this package ships no command of its own."""
        self.assertNotIn("bin", self.package)
        self.assertFalse((CRATES / "grund-core" / "src" / "main.rs").exists())
        self.assertTrue((CRATES / "grund-cli" / "src" / "main.rs").is_file())


class CliDependencyTests(unittest.TestCase):
    """§FS-distribution.1.3 — installing the CLI installs no language server.
    Asserted over the dependency graph rather than one manifest line, because
    an edge added anywhere on the path from `grund` would break the claim."""

    @classmethod
    def setUpClass(cls):
        cls.workspace = workspace_manifests()

    def path_dependencies(self, package):
        document = self.workspace[package]
        return {
            name
            for name, spec in document.get("dependencies", {}).items()
            if isinstance(spec, dict) and "path" in spec
        }

    def test_the_lsp_is_not_in_the_clis_dependency_closure(self):
        seen, pending = set(), ["grund"]
        while pending:
            package = pending.pop()
            for name in self.path_dependencies(package):
                if name not in seen:
                    seen.add(name)
                    pending.append(name)
        self.assertNotIn("grund-lsp", seen, f"installing the CLI now pulls in: {sorted(seen)}")
        self.assertEqual({"grund-core"}, seen, "the CLI's only in-tree dependency is the engine")

    def test_the_two_installs_are_siblings_over_the_engine(self):
        """Independent packages, not a chain: the LSP depends on the same
        engine the CLI does, and on the CLI not at all."""
        self.assertIn("grund-core", self.path_dependencies("grund-lsp"))
        self.assertNotIn("grund", self.path_dependencies("grund-lsp"))

    def test_the_only_optional_dependency_is_the_benchmark_harness(self):
        """An optional dependency is the one way a dependency edge can appear
        without showing up in the table above, so the set of them is pinned."""
        document = self.workspace["grund"]
        optional = {
            name
            for name, spec in document["dependencies"].items()
            if isinstance(spec, dict) and spec.get("optional")
        }
        self.assertEqual({"iai-callgrind"}, optional)
        self.assertEqual(["dep:iai-callgrind"], document["features"]["bench"])

    def test_the_cli_declares_no_development_dependency_on_the_lsp(self):
        for section in ("dev-dependencies", "build-dependencies"):
            with self.subTest(section=section):
                self.assertNotIn("grund-lsp", self.workspace["grund"].get(section, {}))


if __name__ == "__main__":
    unittest.main()
