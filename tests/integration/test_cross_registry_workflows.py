"""§FS-distribution-candidate.5.1, §FS-distribution-candidate.8.1,
§FS-distribution-candidate.8.7, §FS-distribution.4.13 — the two cross-registry
workflows, and the wall between them and the Cargo release.

Read as text, like `test_release_workflow.py`, whose job and step readers this
module shares. The isolation half asserts on the existing workflows *and* requires
the new ones to exist, so it cannot pass by there being nothing to isolate.

Offline, so it runs in the ordinary gate (§AR-ci.6).
"""

import re
import unittest

from distribution_support import PUBLISH_WORKFLOW, REHEARSAL_WORKFLOW, WORKFLOWS, spec_matrix
from test_release_workflow import jobs, squash, steps

EXISTING = ("release.yml", "auto-bump.yml", "release-minor.yml", "pre-release-checks.yml", "ci.yml")
PUBLISH_COMMANDS = re.compile(
    r"npm publish|pnpm publish|yarn publish|twine upload|maturin (?:publish|upload)|"
    r"uv publish|pypa/gh-action-pypi-publish|registry\.npmjs\.org|upload\.pypi\.org")
BUILD_COMMANDS = re.compile(r"cargo build|cargo install|maturin build|cibuildwheel|"
                            r"pgo-build\.sh|candidate\.py build|npm pack")


def absent(path):
    return f"workflow absent: {path.relative_to(WORKFLOWS.parents[1]).as_posix()}"


def read(path):
    assert path.is_file(), absent(path)
    return path.read_text(encoding="utf-8")


class WorkflowText(unittest.TestCase):
    """Each test fails on its own while the workflow is absent, rather than the class
    collapsing into one setup error."""

    path = None

    def setUp(self):
        self.assertTrue(self.path.is_file(), absent(self.path))
        self.text = self.path.read_text(encoding="utf-8")
        self.jobs = jobs(self.text)
        self.publishers = {name: body for name, body in self.jobs.items()
                           if "scripts/distribution/publish.py" in body}


def triggers(text):
    """The keys under `on:`, in order."""
    block = re.search(r"(?ms)^on:\n(.*?)^\S", text + "\nend")
    return re.findall(r"(?m)^  ([a-z_]+):", block.group(1)) if block else []


def top_permissions(text):
    block = re.search(r"(?ms)^permissions:\n(.*?)^\S", text + "\nend")
    return dict(re.findall(r"(?m)^  ([a-z-]+): (\w+)", block.group(1))) if block else {}


class RehearsalWorkflowTests(WorkflowText):
    """§FS-distribution-candidate.5.1, §FS-distribution-candidate.5.6."""

    path = REHEARSAL_WORKFLOW

    def test_only_a_person_starts_it(self):
        self.assertEqual(["workflow_dispatch"], triggers(self.text))

    def test_it_holds_no_credential(self):
        self.assertEqual({"contents": "read"}, top_permissions(self.text))
        self.assertNotIn("id-token", self.text)
        self.assertNotIn("secrets.", self.text)
        self.assertNotIn("environment:", self.text)
        self.assertIsNone(PUBLISH_COMMANDS.search(self.text))
        for forbidden in ("git tag", "git push", "gh release", "softprops/action-gh-release"):
            self.assertNotIn(forbidden, self.text)

    def test_every_row_runs_on_its_own_runner(self):
        for row in spec_matrix():
            with self.subTest(row=row["row"]):
                self.assertIn(row["runner"], self.text)
                self.assertIn(row["row"], self.text)

    def test_every_registry_row_proves_every_runtime(self):
        """§FS-distribution-candidate.1.2."""
        flat = squash(self.text)
        for node in ("22", "24"):
            self.assertRegex(flat, rf"node[-_ a-z]*: \[?[^\]]*\b{node}\b")
        for python in ("3.10", "3.11", "3.12", "3.13", "3.14"):
            self.assertIn(python, flat)

    def test_installed_rows_run_the_rehearsal_and_keep_their_receipts(self):
        self.assertIn("tests/integration/rehearsal/run.py", self.text)
        self.assertIn("--receipt", self.text)
        self.assertIn("actions/upload-artifact", self.text)
        self.assertIn("scripts/distribution/candidate.py verify", self.text)


class PublisherWorkflowTests(WorkflowText):
    """§FS-distribution-candidate.8.1, §FS-distribution-candidate.8.2,
    §FS-distribution-candidate.8.3, §FS-distribution-candidate.8.6."""

    path = PUBLISH_WORKFLOW

    def test_only_a_person_starts_it_and_names_the_candidate(self):
        self.assertEqual(["workflow_dispatch"], triggers(self.text))
        for name in ("candidate_sha", "manifest_sha256", "rehearsal_run_id"):
            with self.subTest(input=name):
                self.assertRegex(self.text, rf"(?ms)^      {name}:\n(?:        .*\n)*?        required: true")

    def test_every_publishing_job_is_literally_disabled_and_gated(self):
        self.assertTrue(self.publishers, "no job runs the publisher")
        for name, body in self.publishers.items():
            with self.subTest(job=name):
                self.assertRegex(body, r"(?m)^    if: \$\{\{ false \}\}\s*$")
                self.assertRegex(body, r"(?m)^    environment: cross-registry-publish\s*$")

    def test_only_publishing_jobs_hold_an_identity(self):
        self.assertEqual({"contents": "read"}, top_permissions(self.text))
        for name, body in self.jobs.items():
            with self.subTest(job=name):
                if name in self.publishers:
                    self.assertIn("id-token: write", body)
                else:
                    self.assertNotIn("id-token", body)
        self.assertNotIn("secrets.", self.text, "trusted publishing needs no stored token")

    def test_it_publishes_the_rehearsed_artifacts_and_builds_nothing(self):
        self.assertIsNone(BUILD_COMMANDS.search(self.text))
        self.assertIsNone(PUBLISH_COMMANDS.search(self.text))
        self.assertIn("actions/download-artifact", self.text)
        self.assertIn("inputs.rehearsal_run_id", self.text)
        for name, body in self.publishers.items():
            with self.subTest(job=name):
                flat = squash(body)
                self.assertIn("--manifest-sha256 \"${{ inputs.manifest_sha256 }}\"", flat)
                self.assertIn("--sha \"${{ inputs.candidate_sha }}\"", flat)
                self.assertNotIn("--readiness-interval-seconds", flat)
                self.assertIn("--status-out", flat)

    def test_the_release_guard_runs_before_anything_is_published(self):
        """§FS-distribution.4.2.6: the new lane is a publication path too."""
        for name, body in self.publishers.items():
            with self.subTest(job=name):
                names = [squash(step) for step in steps(body)]
                guard = next(i for i, s in enumerate(names) if "scripts/check_release_ramps.py" in s)
                publish = next(i for i, s in enumerate(names) if "scripts/distribution/publish.py" in s)
                self.assertLess(guard, publish)

    def test_provenance_is_attested_before_publication(self):
        """§FS-distribution-candidate.8.6: checksums say integrity, attestations identity."""
        self.assertIn("actions/attest-build-provenance", self.text)
        for name, body in self.publishers.items():
            self.assertIn("attestations: write", body, name)


class IsolationTests(unittest.TestCase):
    """§FS-distribution-candidate.8.7, §FS-distribution.4.3, §FS-distribution.4.4 —
    the Cargo release, its helpers and CI cannot reach the new lane."""

    def setUp(self):
        # Isolation from a lane that does not exist would pass vacuously.
        read(REHEARSAL_WORKFLOW)
        read(PUBLISH_WORKFLOW)

    def test_no_existing_workflow_names_the_new_lane_or_publishes_to_npm_or_pypi(self):
        for name in EXISTING:
            with self.subTest(workflow=name):
                text = read(WORKFLOWS / name)
                for word in (REHEARSAL_WORKFLOW.name, PUBLISH_WORKFLOW.name,
                             "scripts/distribution/publish.py", "cross-registry-publish"):
                    self.assertNotIn(word, text)
                self.assertIsNone(PUBLISH_COMMANDS.search(text))

    def test_the_helpers_dispatch_only_the_cargo_release(self):
        for name in ("auto-bump.yml", "release-minor.yml"):
            with self.subTest(workflow=name):
                dispatched = set(re.findall(r"gh workflow run\s+([\w.-]+\.ya?ml)\b", read(WORKFLOWS / name)))
                self.assertEqual({"release.yml"}, dispatched)

    def test_nothing_triggers_the_new_lane_but_a_person(self):
        for path in (REHEARSAL_WORKFLOW, PUBLISH_WORKFLOW):
            with self.subTest(workflow=path.name):
                text = read(path)
                self.assertEqual(["workflow_dispatch"], triggers(text))
                self.assertNotIn("workflow_call", text)
                self.assertNotIn("workflow_run", text)


if __name__ == "__main__":
    unittest.main()
