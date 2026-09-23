"""§FS-distribution.1.1, §FS-distribution.1.1.1, §FS-distribution.1.1.2,
§FS-distribution.1.2, §FS-distribution.1.3 — the package layer of the
distribution spec, read off the manifests and the pre-release name guard that
actually carry it.

None of this reaches a registry. §FS-distribution.1.1's table is asserted against the guard
script's own calls, and the ownership rules are asserted by running the guard
itself against a fake `curl` on `PATH` — the same shell entry point the release
workflows invoke, answering from a fixture instead of the network. The
manifests are parsed rather than grepped, because what §FS-distribution.1.3 forbids is a
dependency edge, not a spelling."""

import json
import os
import re
import subprocess
import tempfile
import tomllib
import unittest
from collections import namedtuple
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CRATES = REPO_ROOT / "crates"
NAME_GUARD = REPO_ROOT / "scripts" / "check-registry-names.sh"

# Any top-level `<helper> "<registry>" "<name>" "<url>"` invocation, matched by
# shape rather than by helper name; the `notice_` prefix is what separates a
# claim from a notice.

# The table keeps pinning all seven claimed names when a registry's ownership
# rule moves to a helper of its own (§FS-distribution.1.1.1).
CALL = re.compile(
    r"""^([a-z_]+)\s+"([^"]+)"\s+"([^"]+)"\s+"([^"]+)"\s*$""",
    re.M,
)

REGISTRY_URL = {
    "crates.io": "https://crates.io/api/v1/crates/{name}",
    "npm": "https://registry.npmjs.org/{name}",
    "pypi": "https://pypi.org/pypi/{name}/json",
}


def guard_calls():
    found = {"claimed": {}, "notice": {}}
    for helper, registry, name, url in CALL.findall(NAME_GUARD.read_text(encoding="utf-8")):
        found["notice" if helper.startswith("notice_") else "claimed"][(registry, name)] = url
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
            set(self.calls["claimed"]),
        )

    def test_the_owner_pattern_accepts_the_repository_it_was_published_from(self):
        """§FS-distribution.1.1: on npm and PyPI ownership is read off metadata the
        last publish wrote, so it names the repository the package was published
        *from*. A repository move reaches those registries only with the next
        release — the one this guard stands in front of — so the former owner
        passes beside the current one, and nobody else does. crates.io no longer
        decides ownership this way (§FS-distribution.1.1.1); this pattern is what
        npm and PyPI keep."""
        pattern = re.search(r"repo_pattern='([^']+)'", NAME_GUARD.read_text()).group(1)
        for owner in ("agent-grounds", "vjovanov"):
            with self.subTest(owner=owner):
                self.assertRegex(f"https://github.com/{owner}/grund", pattern)
        self.assertNotRegex("https://github.com/someone-else/grund", pattern)

    def test_npm_claims_grund_cli_because_the_bare_name_is_occupied(self):
        """The asymmetry §FS-distribution.1.1 records: PyPI takes the bare name, npm takes
        `grund-cli`, and the bare npm name is watched rather than claimed."""
        claimed = set(self.calls["claimed"])
        self.assertIn(("npm", "grund-cli"), claimed)
        self.assertNotIn(("npm", "grund"), claimed)
        self.assertIn(("pypi", "grund"), claimed)
        self.assertEqual({("npm", "grund")}, set(self.calls["notice"]))

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
    the CLI. `grund-lsp` is not asserted here: §FS-distribution.1.2 names siblings "once they
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


CRATES_IO_NAMES = ("grund-core", "grund", "grund-lsp")
CRATE_URL = "https://crates.io/api/v1/crates/{name}"
OWNERS_URL = "https://crates.io/api/v1/crates/{name}/owners"
UNTRUSTED = "error: crates.io/{name} is already taken without trusted owner vjovanov"

OUR_REPOSITORY = "https://github.com/agent-grounds/grund"
MOVED_REPOSITORY = "https://github.com/legacy-owner/grund"

TRUSTED = {
    "id": 4711,
    "login": "vjovanov",
    "name": "Vojin Jovanovic",
    "kind": "user",
    "url": "https://github.com/vjovanov",
}

# Owner records that look like ownership and are not it (§FS-distribution.1.1.1).
# Each differs from TRUSTED in exactly one field, so a rule that reads the wrong
# one is named by the subtest that fails.
NOT_THE_IDENTITY = {
    "the display name only": {"id": 1, "login": "someone-else", "name": "vjovanov", "kind": "user"},
    "the profile URL only": {"id": 2, "login": "someone-else", "kind": "user", "url": "https://github.com/vjovanov"},
    "the github flag only": {"id": 3, "login": "someone-else", "kind": "user", "github_username_matches": True},
    "the organization, bare": {"id": 4, "login": "agent-grounds", "kind": "user", "name": "agent-grounds"},
    "a team": {"id": 5, "login": "github:agent-grounds:publishers", "kind": "team", "name": "publishers"},
    "a team named for the user": {"id": 6, "login": "vjovanov", "kind": "team"},
    "a login differing in case": {"id": 7, "login": "Vjovanov", "kind": "user"},
    "a login with a suffix": {"id": 8, "login": "vjovanov-2", "kind": "user"},
}

# Owner responses that are not owner evidence at all (§FS-distribution.1.1.2).
UNREADABLE_OWNERS = {
    "no users key": '{"teams": []}',
    "users is an object": '{"users": {"login": "vjovanov", "kind": "user"}}',
    "users holds bare logins": '{"users": ["vjovanov"]}',
    "a record with no kind": '{"users": [{"id": 9, "login": "vjovanov"}]}',
    "an HTML error page": "<html><body>502 Bad Gateway</body></html>",
    "an empty body": "",
}

FAKE_CURL = '''#!/usr/bin/env python3
"""A `curl` that answers from GUARD_FAKE_PLAN and logs every URL it is asked for."""

import json
import os
import sys

argv = sys.argv[1:]
url = argv[-1]
destination = argv[argv.index("-o") + 1]
with open(os.environ["GUARD_FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(url + "\\n")

answer = json.loads(os.environ["GUARD_FAKE_PLAN"]).get(url, {"status": 404, "body": "{}"})
if answer.get("transport"):
    sys.stderr.write("curl: (6) Could not resolve host\\n")
    raise SystemExit(6)
with open(destination, "w", encoding="utf-8") as body:
    body.write(answer.get("body", ""))
sys.stdout.write(str(answer["status"]))
'''

GuardRun = namedtuple("GuardRun", "returncode stdout stderr urls")


def run_guard(plan):
    """Run the guard the release workflows run, with `curl` faked on `PATH`.

    Anything the plan does not name answers `404`, so a case declares only the
    packages it is about. Returns the exit status, both streams, and every URL
    the run asked for — the last of these is how "no owner request was made"
    becomes an assertion rather than an inference."""
    with tempfile.TemporaryDirectory() as tmp:
        binaries = Path(tmp) / "bin"
        binaries.mkdir()
        curl = binaries / "curl"
        curl.write_text(FAKE_CURL, encoding="utf-8")
        curl.chmod(0o755)
        log = Path(tmp) / "requests"
        log.write_text("", encoding="utf-8")

        environment = dict(os.environ)
        environment["PATH"] = os.pathsep.join([str(binaries), environment["PATH"]])
        environment["GUARD_FAKE_PLAN"] = json.dumps(plan)
        environment["GUARD_FAKE_LOG"] = str(log)
        done = subprocess.run(
            ["bash", str(NAME_GUARD)],
            cwd=REPO_ROOT,
            env=environment,
            capture_output=True,
            text=True,
        )
        return GuardRun(
            done.returncode, done.stdout, done.stderr, log.read_text(encoding="utf-8").split()
        )


def served(body, status=200):
    return {"status": status, "body": body}


TRANSPORT_FAILURE = {"transport": True}


def crate_metadata(name, repository):
    """A crates.io package response, carrying the repository of the last publish."""
    return json.dumps({"crate": {"name": name, "repository": repository, "homepage": repository}})


def owner_records(*records):
    return json.dumps({"users": list(records)})


def existing_crate(name, repository, owners):
    """A crates.io name that exists, with whatever its owner endpoint answers."""
    return {
        CRATE_URL.format(name=name): served(crate_metadata(name, repository)),
        OWNERS_URL.format(name=name): owners,
    }


def package_metadata(repository, **extra):
    return json.dumps({"repository": {"type": "git", "url": repository}, **extra})


class CratesIoOwnershipTests(unittest.TestCase):
    """§FS-distribution.1.1.1 — ownership of the three crates.io names is the
    registry's own owner record and nothing the package says about itself. The
    guard is run whole, through `bash`, because that is the entry point
    `release.yml` and `pre-release-checks.yml` invoke (§FS-distribution.4.3)."""

    def test_a_repository_move_does_not_lock_the_project_out_of_its_own_names(self):
        """The reported regression: the metadata still names the repository the
        last publish came from, and the registry says the trusted user owns the
        name. The publish that would correct that metadata is the one this guard
        stands in front of, so reading ownership off it deadlocks the release."""
        for name in CRATES_IO_NAMES:
            with self.subTest(crate=name):
                run = run_guard(
                    existing_crate(name, MOVED_REPOSITORY, served(owner_records(TRUSTED)))
                )
                self.assertEqual(0, run.returncode, run.stderr)
                self.assertIn(f"ok: crates.io/{name} is owned by this project", run.stdout)
                self.assertIn(OWNERS_URL.format(name=name), run.urls)

    def test_copied_repository_metadata_does_not_establish_ownership(self):
        """The same rule from the other side: `repository` is publisher-supplied,
        so any crate may put this repository's URL in its own metadata."""
        impostors = {
            "an untrusted user": NOT_THE_IDENTITY["the display name only"],
            "a team": NOT_THE_IDENTITY["a team"],
        }
        for name in CRATES_IO_NAMES:
            for label, record in impostors.items():
                with self.subTest(crate=name, owner=label):
                    run = run_guard(
                        existing_crate(name, OUR_REPOSITORY, served(owner_records(record)))
                    )
                    self.assertEqual(1, run.returncode)
                    self.assertIn(UNTRUSTED.format(name=name), run.stderr)
                    self.assertIn(OWNERS_URL.format(name=name), run.stderr)
                    self.assertNotIn(f"ok: crates.io/{name} is", run.stdout)

    def test_only_an_exact_user_record_is_the_projects_identity(self):
        """`kind` is exactly `user` and `login` is exactly `vjovanov`. Every other
        field in the response is publisher-supplied, opaque, or a different kind
        of subject, and none of them is a credential."""
        name = "grund-core"
        for label, record in NOT_THE_IDENTITY.items():
            with self.subTest(owner=label):
                run = run_guard(
                    existing_crate(name, OUR_REPOSITORY, served(owner_records(record)))
                )
                self.assertEqual(1, run.returncode)
                self.assertIn(UNTRUSTED.format(name=name), run.stderr)

    def test_an_empty_owner_set_is_untrusted_rather_than_unreadable(self):
        """Well-formed and carrying nobody: the response is readable, so it is the
        taken-without-the-trusted-owner case, not the invalid-evidence one."""
        name = "grund-core"
        run = run_guard(existing_crate(name, OUR_REPOSITORY, served(owner_records())))
        self.assertEqual(1, run.returncode)
        self.assertIn(UNTRUSTED.format(name=name), run.stderr)

    def test_the_trusted_record_is_found_beside_other_owners(self):
        """A crate may have several owners; ownership is the presence of the
        trusted one, not the shape of the whole set."""
        name = "grund-lsp"
        owners = owner_records(NOT_THE_IDENTITY["a team"], TRUSTED, NOT_THE_IDENTITY["a login with a suffix"])
        run = run_guard(existing_crate(name, MOVED_REPOSITORY, served(owners)))
        self.assertEqual(0, run.returncode, run.stderr)
        self.assertIn(f"ok: crates.io/{name} is owned by this project", run.stdout)


class CratesIoEvidenceTests(unittest.TestCase):
    """§FS-distribution.1.1.2 — an existing crate whose ownership cannot be
    established stops the release, and says which of the four things went wrong.
    There is deliberately no fallback to declared metadata: that fallback is
    what both the move deadlock and the copied-URL acceptance are made of."""

    def test_unreadable_owner_data_stops_the_release(self):
        name = "grund-core"
        for label, body in UNREADABLE_OWNERS.items():
            with self.subTest(owners=label):
                run = run_guard(existing_crate(name, OUR_REPOSITORY, served(body)))
                self.assertEqual(1, run.returncode)
                self.assertIn(OWNERS_URL.format(name=name), run.stderr)
                self.assertNotIn(UNTRUSTED.format(name=name), run.stderr)
                self.assertNotIn(f"ok: crates.io/{name} is", run.stdout)

    def test_an_owner_endpoint_that_fails_stops_the_release(self):
        """`404` on the *owner* endpoint is not a free name — the package endpoint
        already said the crate exists — and every other status is the same
        inability to determine ownership."""
        name = "grund"
        for status in (404, 429, 500, 503):
            with self.subTest(status=status):
                run = run_guard(
                    existing_crate(name, OUR_REPOSITORY, served("{}", status=status))
                )
                self.assertEqual(1, run.returncode)
                self.assertIn(str(status), run.stderr)
                self.assertIn(OWNERS_URL.format(name=name), run.stderr)
                self.assertNotIn(f"ok: crates.io/{name} is", run.stdout)
                self.assertNotIn(f"crates.io/{name} is available", run.stdout)

    def test_a_transport_failure_stops_the_release(self):
        """A request that never completes is reported, not fallen through: the
        run must not exit on the raw transport status with nothing said."""
        name = "grund"
        run = run_guard(existing_crate(name, OUR_REPOSITORY, TRANSPORT_FAILURE))
        self.assertEqual(1, run.returncode)
        self.assertIn(OWNERS_URL.format(name=name), run.stderr)
        self.assertNotIn(f"ok: crates.io/{name} is", run.stdout)

    def test_the_four_failures_are_told_apart(self):
        """Distinct diagnostics, because the operator's next move differs: take
        the name back, retry, wait on the registry, or fix the network."""
        name = "grund-core"
        answers = {
            "untrusted": served(owner_records(NOT_THE_IDENTITY["a team"])),
            "unreadable": served('{"teams": []}'),
            "http": served("{}", status=500),
            "transport": TRANSPORT_FAILURE,
        }
        said = {}
        for label, owners in answers.items():
            run = run_guard(existing_crate(name, OUR_REPOSITORY, owners))
            self.assertEqual(1, run.returncode, f"{label} did not stop the release")
            self.assertTrue(run.stderr.strip(), f"{label} said nothing")
            said[label] = run.stderr
        self.assertEqual(len(answers), len(set(said.values())), f"indistinguishable: {said}")


class FreeNameTests(unittest.TestCase):
    """§FS-distribution.1.1 — the package endpoint's own `404` is the only thing
    that makes a name free, and nothing later turns an existing crate back into
    one."""

    def test_a_name_no_registry_answers_for_is_free_and_no_owner_is_asked(self):
        run = run_guard({})
        self.assertEqual(0, run.returncode, run.stderr)
        for name in CRATES_IO_NAMES:
            with self.subTest(crate=name):
                self.assertIn(f"ok: crates.io/{name} is available", run.stdout)
                self.assertNotIn(OWNERS_URL.format(name=name), run.urls)

    def test_a_package_endpoint_failure_is_never_read_as_free(self):
        name = "grund-core"
        for label, answer in (("HTTP 500", served("{}", status=500)), ("transport", TRANSPORT_FAILURE)):
            with self.subTest(failure=label):
                run = run_guard({CRATE_URL.format(name=name): answer})
                self.assertNotEqual(0, run.returncode)
                self.assertNotIn(f"crates.io/{name} is available", run.stdout)
                self.assertNotIn(f"ok: crates.io/{name} is", run.stdout)


class NpmAndPyPiTests(unittest.TestCase):
    """§FS-distribution.1.1 — npm and PyPI keep the repository-metadata rule and
    its diagnostics. The crates.io identity decides nothing here, in either
    direction: it neither rescues a name nor refuses one."""

    def claimed(self, registry, name):
        return REGISTRY_URL[registry].format(name=name)

    def test_repository_metadata_still_owns_an_npm_or_pypi_name(self):
        for registry, name in (("pypi", "grund"), ("npm", "grund-cli")):
            with self.subTest(registry=registry, name=name):
                url = self.claimed(registry, name)
                run = run_guard({url: served(package_metadata(OUR_REPOSITORY))})
                self.assertEqual(0, run.returncode, run.stderr)
                self.assertIn(f"ok: {registry}/{name} is owned by this project", run.stdout)
                self.assertNotIn(f"{url}/owners", run.urls)

    def test_unrelated_metadata_still_fails_an_npm_or_pypi_name(self):
        for registry, name in (("pypi", "grund-lsp"), ("npm", "grund-lsp")):
            with self.subTest(registry=registry, name=name):
                url = self.claimed(registry, name)
                run = run_guard({url: served(package_metadata("https://github.com/someone/else"))})
                self.assertEqual(1, run.returncode)
                self.assertIn(f"error: {registry}/{name} is already taken by another project", run.stderr)

    def test_an_owner_record_does_not_reach_npm_or_pypi(self):
        """A `users` array in an npm or PyPI document is just a document: it must
        not rescue a name whose metadata names somebody else's repository."""
        for registry, name in (("pypi", "grund-lsp"), ("npm", "grund-lsp")):
            with self.subTest(registry=registry, name=name):
                url = self.claimed(registry, name)
                body = package_metadata("https://github.com/someone/else", users=[TRUSTED])
                run = run_guard({url: served(body)})
                self.assertEqual(1, run.returncode)
                self.assertIn(f"error: {registry}/{name} is already taken by another project", run.stderr)

    def test_the_bare_npm_name_stays_an_informational_notice(self):
        url = REGISTRY_URL["npm"].format(name="grund")
        run = run_guard({url: served(package_metadata("https://github.com/someone/else"))})
        self.assertEqual(0, run.returncode, run.stderr)
        self.assertIn("notice: npm/grund is occupied by an external package as documented", run.stdout)



if __name__ == "__main__":
    unittest.main()
