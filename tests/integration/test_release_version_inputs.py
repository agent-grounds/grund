"""§FS-distribution.4.5: set-version owns fixture/fact bytes in each candidate.

These tests run the writer and the workflows' actual offline write/staging
commands. Input equality is S4's construction predicate, not a substitute for
executing Rust S4; the #888 contract also requires that acceptance on bumped
release and dev trees. §REQ-one-answer.proof.evidence keeps truth independent.
"""

import copy
import json
import os
import shutil
import sys
import tomllib
import unittest

from release_version_fixture import (
    CASE, FACT, REPO, copy_tree, execute_phase, git, inventory, read_bytes, run,
    set_version, workflow_phases,
)


class SetVersionInputTests(unittest.TestCase):
    def check_version(self, version):
        root = copy_tree(self)
        original = json.loads((root / FACT).read_bytes())
        original_inputs = inventory(root)
        expected_bytes = f"grund {version}\n".encode()
        expected_fact = copy.deepcopy(original)
        expected_fact["inputFiles"]["expected.stdout"] = {"bytes": list(expected_bytes)}
        for invocation in range(2):
            result = set_version(root, version)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            with self.subTest(invocation=invocation, representation="fixture"):
                self.assertEqual(expected_bytes, (root / CASE / "expected.stdout").read_bytes())
            with self.subTest(invocation=invocation, representation="authored fact"):
                self.assertEqual(expected_fact, json.loads((root / FACT).read_bytes()),
                                 "only the version-dependent input may change")
            with self.subTest(invocation=invocation, representation="unrelated fixtures"):
                actual = inventory(root)
                actual.pop("expected.stdout")
                self.assertEqual({k: v for k, v in original_inputs.items()
                                  if k != "expected.stdout"}, actual)

    def test_set_version_synchronizes_release_inputs_and_is_idempotent(self):
        self.check_version("0.19.0")

    def test_set_version_synchronizes_dev_inputs_and_is_idempotent(self):
        self.check_version("0.19.1-dev")

    def test_set_version_preserves_unrelated_corruption_and_independent_evidence(self):
        """§REQ-one-answer.proof.evidence: a bump must not launder bad evidence."""
        root = copy_tree(self)
        fact = json.loads((root / FACT).read_bytes())
        fact["inputFiles"]["command.args"]["bytes"] = list(b"--incorrect\n")
        fact["variants"]["whole-tree"]["authoredFrom"] = "independent evidence sentinel"
        (root / FACT).write_text(json.dumps(fact), encoding="utf-8")
        result = set_version(root, "0.19.0")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        expected = copy.deepcopy(fact)
        expected["inputFiles"]["expected.stdout"] = {"bytes": list(b"grund 0.19.0\n")}
        with self.subTest(check="targeted write"):
            self.assertEqual(expected, json.loads((root / FACT).read_bytes()))
        with self.subTest(check="corruption remains detectable by S4 input equality"):
            actual = inventory(root)
            authored = json.loads((root / FACT).read_bytes())["inputFiles"]
            self.assertEqual(["command.args"], sorted(
                k for k in set(actual) | set(authored) if actual.get(k) != authored.get(k)))


class WorkflowVersionInputTests(unittest.TestCase):
    """§FS-distribution.4.5: inspect each index before committing its dev advance."""

    def check_phase(self, helper, release, dev, phase):
        root = copy_tree(self)
        phases = workflow_phases(helper)
        for index, version in enumerate((release, dev)):
            execute_phase(self, root, phases[index], version)
            if index == phase:
                for staged in (False, True):
                    with self.subTest(helper=helper, version=version, staged=staged):
                        expected = f"grund {version}\n".encode()
                        self.assertEqual(expected, read_bytes(root, CASE + "/expected.stdout", staged))
                        fact = json.loads(read_bytes(root, FACT, staged))
                        self.assertEqual(inventory(root, staged), fact["inputFiles"],
                                         "candidate fixture and authored input bytes disagree")
                with self.subTest(check="fact is included in this candidate index"):
                    self.assertIn(FACT, git(root, "diff", "--cached", "--name-only").splitlines())
                return
            # Dev is built over the actual staged release, without sweeping
            # omitted release files into that commit with a test-owned git add.
            git(root, "commit", "-qm", "Disposable release candidate")

    def test_minor_release_index_contains_synchronized_inputs(self):
        self.check_phase("release-minor", "0.19.0", "0.19.1-dev", 0)

    def test_minor_dev_index_contains_synchronized_inputs(self):
        self.check_phase("release-minor", "0.19.0", "0.19.1-dev", 1)

    def test_patch_release_index_contains_synchronized_inputs(self):
        self.check_phase("auto-bump", "0.19.1", "0.19.2-dev", 0)

    def test_patch_dev_index_contains_synchronized_inputs(self):
        self.check_phase("auto-bump", "0.19.1", "0.19.2-dev", 1)


@unittest.skipUnless(os.environ.get("GRUND_895_RED"),
                     "red until #895 derives the bump versions from the tree under test; "
                     "implement removes this")
class CandidateTreeTests(unittest.TestCase):
    """§FS-distribution.4.5: the workflow tests hold on the candidates they gate.

    Candidate CI runs this module on a tree already bumped to the release and to
    its dev successor (#895), so run it there, not only on main's version.
    """

    def check_tree(self, version):
        root = copy_tree(self)
        for name in (".github/workflows/release-minor.yml", ".github/workflows/auto-bump.yml",
                     "tests/integration/release_version_fixture.py",
                     "tests/integration/test_release_workflow.py",
                     "tests/integration/test_release_version_inputs.py"):
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / name, root / name)
        result = set_version(root, version)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        result = run(root / "tests/integration", sys.executable, "-m", "unittest",
                     "test_release_version_inputs.WorkflowVersionInputTests")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def candidates(self):
        manifest = tomllib.loads((REPO / "Cargo.toml").read_text(encoding="utf-8"))
        major, minor, _ = manifest["workspace"]["package"]["version"].split(".", 2)
        return f"{major}.{int(minor) + 1}.0", f"{major}.{int(minor) + 1}.1-dev"

    def test_release_candidate_tree(self):
        self.check_tree(self.candidates()[0])

    def test_dev_candidate_tree(self):
        self.check_tree(self.candidates()[1])


if __name__ == "__main__":
    unittest.main()
