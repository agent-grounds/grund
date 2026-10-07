"""§FS-distribution-candidate.8.2, §FS-distribution-candidate.8.3,
§FS-distribution-candidate.8.4, §FS-distribution-candidate.8.5 — the publisher,
against registries that only pretend.

Every registry here is a loopback server speaking the parts of the npm, PyPI,
crates.io and Actions OIDC protocols a publisher uses, so ownership, authority and
query failures, digest drift, partial uploads, readiness timeouts and reruns are
each injected exactly. Readiness waits run with a zero interval; the attempt count
is the real one. Nothing here can reach a real registry: the publisher is only ever
given loopback addresses, and it holds no credential but the fake identity token.

Offline, so it runs in the ordinary gate (§AR-ci.6).
"""

import json
import os
import stat
import sys
import unittest
from pathlib import Path

from distribution_support import (
    PUBLISH, SHA, VERSION, FakeRegistries, file_sha256, npm_integrity,
    run, scratch, synthetic_candidate, tool_missing, write_receipts,
)

BUILD_TOOLS = ("cargo", "rustc", "maturin", "cibuildwheel")


class PublisherFixture(unittest.TestCase):

    def setUp(self):
        self.temp = scratch("grund-publisher-")
        self.addCleanup(self.temp.cleanup)
        self.work = Path(self.temp.name)
        self.root, self.digest = synthetic_candidate(self.work / "candidate")
        self.registries = FakeRegistries()
        self.addCleanup(self.registries.close)
        self.registries.publish_crates(self.root)
        manifest = json.loads((self.root / "manifest.json").read_text(encoding="utf-8"))
        self.uploaded = [a for a in manifest["artifacts"] if a["registry"] in ("npm", "pypi")]
        self.status = self.work / "status.json"
        self.tools = self.work / "tools"
        self.tools.mkdir()
        self.tool_log = self.work / "tools.log"
        for name in BUILD_TOOLS:
            script = self.tools / name
            script.write_text(f'#!/bin/sh\necho "{name} $*" >> "{self.tool_log}"\nexit 1\n')
            script.chmod(script.stat().st_mode | stat.S_IEXEC)
            (self.tools / f"{name}.cmd").write_text(f"@echo {name} %* >> \"{self.tool_log}\"\r\n"
                                                    "@exit /b 1\r\n")

    def env(self, identity=True):
        env = {k: v for k, v in os.environ.items()
               if not k.startswith("ACTIONS_ID_TOKEN_")}
        env["PATH"] = str(self.tools) + os.pathsep + env.get("PATH", "")
        if identity:
            env.update(self.registries.environment())
        return env

    def publish(self, *extra, digest=None, sha=SHA, root=None, identity=True, env=None):
        self.assertTrue(PUBLISH.is_file(), tool_missing(PUBLISH))
        argv = [sys.executable, PUBLISH, "--candidate", root or self.root,
                "--manifest-sha256", digest or self.digest, "--sha", sha,
                *self.registries.args(), "--readiness-interval-seconds", "0",
                "--status-out", self.status, *extra]
        return run(argv, env=env or self.env(identity), timeout=300)

    def state(self):
        status = json.loads(self.status.read_text(encoding="utf-8"))
        self.assertEqual(VERSION, status["version"])
        self.assertEqual(self.digest, status["manifest_sha256"])
        self.assertEqual(sorted(a["path"] for a in self.uploaded),
                         sorted(item["artifact"] for item in status["items"]))
        return status, {item["artifact"]: item["state"] for item in status["items"]}

    def npm_puts(self):
        return [p[len("/npm/"):] for m, p in self.registries.log if m == "PUT"]


class PublicationTests(PublisherFixture):
    """§FS-distribution-candidate.8.3, §FS-distribution-candidate.8.4."""

    def test_a_verified_candidate_publishes_every_npm_and_pypi_artifact_once(self):
        before = {p: file_sha256(p) for p in self.root.rglob("*") if p.is_file()}
        result = self.publish()
        self.assertEqual(0, result.returncode, result.stderr)
        npm = [a for a in self.uploaded if a["registry"] == "npm"]
        self.assertEqual(sorted(a["package"] for a in npm), sorted(self.npm_puts()))
        for item in npm:
            with self.subTest(package=item["package"]):
                published = self.registries.npm[item["package"]][VERSION]["dist"]["integrity"]
                self.assertEqual(npm_integrity(self.root / item["path"]), published)
        for name in ("grund", "grund-lsp"):
            files = dict(self.registries.pypi[(name, VERSION)])
            expected = {Path(a["path"]).name: a["sha256"] for a in self.uploaded
                        if a["registry"] == "pypi" and a["package"] == name}
            self.assertEqual(expected, files)
        status, states = self.state()
        self.assertTrue(status["complete"])
        self.assertEqual({"published"}, set(states.values()))
        after = {p: file_sha256(p) for p in self.root.rglob("*") if p.is_file()}
        self.assertEqual(before, after, "the candidate was changed while it was published")
        self.assertFalse(self.tool_log.exists(), self.tool_log.read_text()
                         if self.tool_log.exists() else "")

    def test_platform_packages_publish_and_resolve_before_any_umbrella(self):
        result = self.publish()
        self.assertEqual(0, result.returncode, result.stderr)
        log = self.registries.log
        first_umbrella = min(i for i, (m, p) in enumerate(log)
                             if m == "PUT" and p in ("/npm/grund-cli", "/npm/grund-lsp"))
        for i, (method, path) in enumerate(log):
            if method == "PUT" and path.startswith("/npm/@"):
                self.assertLess(i, first_umbrella, path)
                resolved = [j for j, entry in enumerate(log) if j > i and entry == ("GET", path)]
                self.assertTrue(resolved and resolved[0] < first_umbrella,
                                f"{path} was not seen to resolve before the umbrellas")

    def test_the_crates_must_resolve_before_anything_uploads(self):
        """§FS-distribution.4.10: Cargo stays core-first; this lane waits on it."""
        self.registries.crates.clear()
        result = self.publish()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("grund-core", result.stderr)
        self.assertEqual([], self.registries.uploads())
        asked = [p for m, p in self.registries.log if p == f"/crates/api/v1/crates/grund-core/{VERSION}"]
        self.assertEqual(90, len(asked))

    def test_readiness_gives_up_after_ninety_attempts(self):
        hidden = "@grund-cli/linux-x64-gnu"
        self.registries.hidden.add(hidden)
        result = self.publish()
        self.assertNotEqual(0, result.returncode)
        self.assertIn(hidden, result.stderr)
        log = self.registries.log
        put = log.index(("PUT", f"/npm/{hidden}"))
        self.assertEqual(90, log[put + 1:].count(("GET", f"/npm/{hidden}")))
        self.assertNotIn("grund-cli", self.npm_puts())
        status, states = self.state()
        self.assertFalse(status["complete"])

    def test_the_readiness_bound_defaults_to_twenty_seconds(self):
        self.assertTrue(PUBLISH.is_file(), tool_missing(PUBLISH))
        help_text = run([sys.executable, PUBLISH, "--help"]).stdout
        self.assertRegex(help_text, r"--readiness-interval-seconds[\s\S]*20")
        self.assertIn("90", help_text)


class AuthorityTests(PublisherFixture):
    """§FS-distribution-candidate.8.2 — an identity that is refused uploads nothing."""

    def test_a_refused_npm_exchange_uploads_nothing(self):
        refused = "@grund-lsp/darwin-arm64"
        self.registries.refused.add(refused)
        result = self.publish()
        self.assertNotEqual(0, result.returncode)
        self.assertIn(refused, result.stderr)
        self.assertEqual([], self.registries.uploads())

    def test_a_refused_pypi_mint_uploads_nothing(self):
        self.registries.refused.add("pypi")
        result = self.publish()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("pypi", result.stderr.lower())
        self.assertEqual([], self.registries.uploads())

    def test_without_an_identity_token_no_stored_credential_stands_in(self):
        env = self.env(identity=False)
        env.update(NPM_TOKEN="stored", NODE_AUTH_TOKEN="stored", TWINE_PASSWORD="stored",
                   PYPI_API_TOKEN="stored")
        result = self.publish(env=env)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("id-token", result.stderr)
        self.assertEqual([], self.registries.uploads())


class VerificationGateTests(PublisherFixture):
    """§FS-distribution-candidate.8.3 — only the rehearsed bytes, or nothing."""

    def assert_untouched(self, result, naming):
        self.assertNotEqual(0, result.returncode)
        self.assertIn(naming, result.stderr)
        self.assertEqual([], self.registries.log, "a refused candidate asked a registry")

    def test_a_corrupt_artifact_asks_no_registry(self):
        path = self.root / "npm" / f"grund-lsp-{VERSION}.tgz"
        path.write_bytes(path.read_bytes()[:-1])
        self.assert_untouched(self.publish(), f"grund-lsp-{VERSION}.tgz")

    def test_a_missing_artifact_asks_no_registry(self):
        (self.root / "pypi" / f"grund_lsp-{VERSION}.tar.gz").unlink()
        self.assert_untouched(self.publish(), f"grund_lsp-{VERSION}.tar.gz")

    def test_the_manifest_digest_and_the_sha_must_be_the_candidates(self):
        self.assert_untouched(self.publish(digest="0" * 64), "0" * 64)
        self.assert_untouched(self.publish(sha="f" * 40), "f" * 40)

    def test_every_registry_row_needs_a_receipt_for_this_manifest(self):
        (self.root / "receipts" / "darwin-x64.json").unlink()
        self.assert_untouched(self.publish(), "darwin-x64")
        write_receipts(self.root, "1" * 64)
        self.assert_untouched(self.publish(), "receipt")

    def test_only_a_full_release_is_published(self):
        """§FS-distribution-candidate.6.3."""
        for scope in ("binding-only", "rehearsal"):
            with self.subTest(scope=scope):
                root, digest = synthetic_candidate(self.work / scope, scope=scope)
                self.assert_untouched(self.publish(root=root, digest=digest), scope)


class RecoveryTests(PublisherFixture):
    """§FS-distribution-candidate.8.5, §FS-distribution.4.11 — reruns resume the same
    candidate, skip what is already there, and never overwrite what differs."""

    def test_a_registry_that_cannot_be_asked_is_never_read_as_empty(self):
        name = "@grund-cli/linux-x64-gnu"
        self.registries.faults[("npm", "GET", name)] = [500] * 200
        result = self.publish()
        self.assertNotEqual(0, result.returncode)
        self.assertIn(name, result.stderr)
        self.assertNotIn(name, self.npm_puts())
        self.assertNotIn("grund-cli", self.npm_puts())

    def test_what_is_already_published_with_the_same_digest_is_skipped(self):
        npm = "@grund-lsp/darwin-x64"
        npm_path = self.root / "npm" / f"grund-lsp-darwin-x64-{VERSION}.tgz"
        self.registries.npm[npm] = {VERSION: {"dist": {"integrity": npm_integrity(npm_path)}}}
        wheel = next(a for a in self.uploaded if a["kind"] == "wheel" and a["package"] == "grund")
        self.registries.pypi[("grund", VERSION)] = [(Path(wheel["path"]).name, wheel["sha256"])]
        result = self.publish()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertNotIn(npm, self.npm_puts())
        self.assertEqual(6, len(self.registries.pypi[("grund", VERSION)]))
        status, states = self.state()
        self.assertTrue(status["complete"])
        self.assertEqual("skipped", states[f"npm/{npm_path.name}"])
        self.assertEqual("skipped", states[wheel["path"]])

    def test_what_is_published_with_another_digest_stops_without_overwrite(self):
        for registry in ("npm", "pypi"):
            with self.subTest(registry=registry):
                self.registries.log.clear()
                if registry == "npm":
                    name, artifact = "@grund-cli/win32-x64-msvc", \
                        f"npm/grund-cli-win32-x64-msvc-{VERSION}.tgz"
                    self.registries.npm[name] = {VERSION: {"dist": {"integrity": "sha512-AAAA"}}}
                    asked = ("GET", f"/npm/{name}")
                else:
                    self.registries.npm.clear()
                    name = "grund-lsp"
                    artifact = next(a["path"] for a in self.uploaded
                                    if a["kind"] == "sdist" and a["package"] == name)
                    self.registries.pypi = {(name, VERSION): [(Path(artifact).name, "0" * 64)]}
                    asked = ("GET", f"/pypi/pypi/{name}/{VERSION}/json")
                result = self.publish()
                self.assertNotEqual(0, result.returncode)
                self.assertIn(Path(artifact).name if registry == "pypi" else name, result.stderr)
                log = self.registries.log
                last = max(i for i, entry in enumerate(log) if entry == asked)
                later = [e for e in log[last:] if e[0] == "PUT" or e[1].endswith("/legacy/")]
                self.assertEqual([], later, "the run uploaded after it found drift")
                status, states = self.state()
                self.assertFalse(status["complete"])
                self.assertEqual("failed", states[artifact])

    def test_a_failed_upload_is_reported_partial_and_a_rerun_completes_it(self):
        name = "@grund-cli/darwin-x64"
        artifact = f"npm/grund-cli-darwin-x64-{VERSION}.tgz"
        self.registries.faults[("npm", "PUT", name)] = [500]
        first = self.publish()
        self.assertNotEqual(0, first.returncode)
        self.assertIn(name, first.stderr)
        status, states = self.state()
        self.assertFalse(status["complete"])
        self.assertEqual("failed", states[artifact])
        self.assertIn("pending", states.values(), "a stopped run names what it did not reach")
        published = {a for a, s in states.items() if s == "published"}
        second = self.publish()
        self.assertEqual(0, second.returncode, second.stderr)
        status, states = self.state()
        self.assertTrue(status["complete"])
        self.assertEqual({"skipped"}, {states[a] for a in published} or {"skipped"})
        self.assertEqual("published", states[artifact])
        puts = self.npm_puts()
        for item in (a for a in self.uploaded if a["registry"] == "npm"):
            expected = 2 if item["package"] == name else 1
            self.assertEqual(expected, puts.count(item["package"]), item["package"])


if __name__ == "__main__":
    unittest.main()
