"""§FS-distribution.4.4.2 and §FS-distribution.4.5: first-stable preparation.

The real selector feeds the existing guard, version writer and forge-backed
changelog preparation in a tagged synthetic history. Workflow assertions pin
the dispatch interface and connect it to the existing candidate safety tests.
"""

import json
import unittest

from release_selection_fixture import GUARD, VERSION_FIXTURE, SelectionRepository
from test_release_candidate_runs import helper_steps
from test_release_workflow import WORKFLOWS, squash, step_index


class ReleaseSelectionAcceptanceTests(SelectionRepository, unittest.TestCase):
    def test_deliberate_first_stable_prepares_complete_release_and_dev_candidates(self):
        # Fail for the reported missing route, before attempting a missing script.
        resolver = helper_steps("release-minor.yml")
        self.assertTrue(any("scripts/release_version.py" in step for step in resolver),
                        "release-minor.yml has no explicit 1.0.0 selection path; "
                        "its resolver only computes the next minor")
        selected = self._outputs(self._select(), "1.0.0")
        release_sha, dev_sha = self._prepare(selected)
        self.assertEqual(release_sha, self._git(self.repo, "rev-parse", dev_sha + "^"))
        for sha, version, python in ((release_sha, "1.0.0", "1.0.0"),
                                     (dev_sha, "1.0.1-dev", "1.0.1.dev0")):
            with self.subTest(version=version):
                self.assertEqual(version, self._candidate("versions", "--ref", sha, "--expect", version))
                self.assertEqual(f"grund {version}", self._git(self.repo, "show", f"{sha}:{VERSION_FIXTURE}"))
                pyproject = self._git(self.repo, "show", f"{sha}:crates/grund-py/pyproject.toml")
                self.assertIn(f'version = "{python}"', pyproject)
                changelog = self._git(self.repo, "show", f"{sha}:docs/changelog.md")
                self.assertIn("## 2. [1.0.0] — 2026-10-21", changelog)
                self.assertIn("Allow the first stable release", changelog)
                self.assertEqual(1, changelog.count("(PR #804)"))
                archive = self._git(self.repo, "show", f"{sha}:docs/changelog/0.21.3.md")
                self.assertIn("Previous release.", archive)
        self.assertEqual(self.tip, self._git(self.repo, "rev-parse", "refs/heads/main"))
        calls = [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]
        self.assertTrue(calls, "the changelog must actually ask the stub forge")
        self.assertTrue(all(call[0] == "api" for call in calls), "preparation dispatched a workflow")

    def test_selected_one_does_not_bypass_a_synthetic_pending_zero_x_promise(self):
        self.tip = self._land("A pending retirement", {
            "crates/grund-core/src/pending.rs":
                'const WARNING: &str = "old syntax becomes an error in grund 0.22.0";\n'})
        self._main_tip(self.tip)
        selected = self._outputs(self._select(), "1.0.0")
        before = self._snapshot()
        refused = self._run(GUARD, selected["next"], "--root", str(self.repo))
        self._refused(before, refused, "this tree cannot be released as 1.0.0", "0.22.0",
                      "crates/grund-core/src/pending.rs")


class ManualSelectionWorkflowTests(unittest.TestCase):
    def test_choices_and_sha_feed_one_selector_before_guard_and_writes(self):
        text = (WORKFLOWS / "release-minor.yml").read_text(encoding="utf-8")
        found = helper_steps("release-minor.yml")
        selector_steps = [step for step in found if "scripts/release_version.py" in step]
        self.assertEqual(1, len(selector_steps), "manual helper has no shared version selector")
        selector = selector_steps[0]
        dispatch = text.split("workflow_dispatch:", 1)[1].split("\npermissions:", 1)[0]
        self.assertRegex(dispatch, r"(?m)^\s+version:\s*$")
        self.assertIn("type: choice", dispatch)
        self.assertIn("default: next-minor", dispatch)
        for option in ("next-minor", "1.0.0"):
            self.assertRegex(dispatch, rf"(?m)^\s+- ['\"]?{option.replace('.', '[.]')}['\"]?\s*$")
        self.assertIn("expected_main_sha:", dispatch)
        for input_name in ("version", "expected_main_sha"):
            self.assertRegex(selector, rf"(?:inputs|github.event.inputs)\.{input_name}\b")
        for flag in ("--version", "--expected-main-sha", "--ref", "--dispatch-sha"):
            self.assertIn(flag, selector)
        self.assertIn("id: versions", selector)
        self.assertIn("GITHUB_OUTPUT", selector)
        selection_index = found.index(selector)
        guard = step_index(found, "Gate - the tree can be released as this version")
        self.assertGreater(guard, selection_index)
        self.assertIn('steps.versions.outputs.next', found[guard])
        self.assertLess(guard, step_index(found, "Build, run and publish the candidates"))
        fetch = squash("\n".join(found[:selection_index + 1]))
        self.assertRegex(fetch, r"git fetch\b.*\bmain\b")
        self.assertIn("github.sha", text)
        self.assertIn("github.ref", text)
        # The remainder is shared for both choices: existing candidate tests pin
        # its three-run gate, tested SHAs and release -> dispatch -> dev sequence.
        rest = "\n".join(found[selection_index + 1:])
        self.assertNotIn("inputs.version", rest)
        self.assertNotIn("inputs.expected_main_sha", rest)


if __name__ == "__main__":
    unittest.main()
