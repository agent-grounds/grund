"""§FS-distribution.4.4.2: the manual selector, its default and every refusal.

Temporary specification skips: implement removes the class decorators and os
import. GRUND_804_RED=1 runs the real assertions before the selector exists.
"""

import os
import unittest

from release_selection_fixture import SelectionRepository


@unittest.skipUnless(os.environ.get("GRUND_804_RED") == "1",
                     "red until #804 adds the release selector; implement removes this")
class ReleaseVersionTests(SelectionRepository, unittest.TestCase):
    def test_omitted_inputs_keep_the_next_minor_default(self):
        before = self._snapshot()
        self._outputs(self._select(version=None, expected=None), "0.22.0")
        self.assertEqual(before, self._snapshot())

    def test_explicit_next_minor_accepts_an_optional_matching_sha(self):
        self._outputs(self._select(version="next-minor"), "0.22.0")

    def test_default_next_minor_still_works_after_a_stable_major(self):
        self._candidate("set-version", "1.2.4-dev")
        self._git(self.repo, "tag", "v1.2.3", self.base)
        self._outputs(self._select(version=None, expected=None), "1.3.0", current="1.2.3")

    def test_first_stable_emits_the_existing_four_outputs_without_writes(self):
        before = self._snapshot()
        self._outputs(self._select(), "1.0.0")
        self.assertEqual(before, self._snapshot())

    def test_prerelease_tags_do_not_replace_the_latest_stable_tag(self):
        for tag in ("v1.0.0-rc.1", "v2.0.0-dev"):
            self._git(self.repo, "tag", tag)
        self._outputs(self._select(), "1.0.0")

    def test_a_missing_or_malformed_expected_sha_is_refused(self):
        for sha in (None, "", "abc123", "x" * 40, self.tip[:39], self.tip + "0"):
            with self.subTest(sha=sha):
                before = self._snapshot()
                self._refused(before, self._select(expected=sha), "expected_main_sha")

    def test_an_unsupported_version_is_refused(self):
        for version in ("", "1.1.0", "2.0.0", "v1.0.0", "next-patch"):
            with self.subTest(version=version):
                self._refused(self._snapshot(), self._select(version=version), "version")

    def test_a_dispatch_from_another_branch_or_tag_is_refused(self):
        for ref in ("refs/heads/topic", "refs/tags/v0.21.3", ""):
            with self.subTest(ref=ref):
                self._refused(self._snapshot(), self._select(ref=ref), "main")

    def test_expected_sha_must_match_dispatch_checkout_and_fetched_main(self):
        for mismatch in ("dispatch", "checkout", "main", "expected"):
            with self.subTest(mismatch=mismatch):
                self._git(self.repo, "checkout", "--detach", self.tip)
                self._main_tip(self.tip)
                options = {}
                if mismatch == "dispatch":
                    options["dispatch"] = self.base
                elif mismatch == "checkout":
                    self._git(self.repo, "checkout", "--detach", self.base)
                elif mismatch == "main":
                    self._main_tip(self.base)
                else:
                    options["expected"] = self.base
                self._refused(self._snapshot(), self._select(**options), "SHA")

    def test_a_missing_or_malformed_dispatch_sha_is_refused(self):
        for sha in ("", "1234567", "g" * 40):
            with self.subTest(sha=sha):
                self._refused(self._snapshot(), self._select(dispatch=sha), "SHA")

    def test_a_supplied_next_minor_sha_is_also_validated(self):
        for sha in ("broken", self.base):
            with self.subTest(sha=sha):
                self._refused(self._snapshot(), self._select(version="next-minor", expected=sha),
                              "expected_main_sha")

    def test_a_missing_fetched_main_tip_is_refused(self):
        self._git(self.repo, "update-ref", "-d", "refs/remotes/origin/main")
        self._refused(self._snapshot(), self._select(), "main")

    def test_disagreement_in_any_version_source_is_refused(self):
        for path in ("Cargo.toml", "Cargo.lock", "crates/grund-cli/Cargo.toml",
                     "crates/grund-py/pyproject.toml"):
            with self.subTest(path=path):
                file = self.repo / path
                original = file.read_text(encoding="utf-8")
                file.write_text(original.replace("0.21.4", "0.21.5"), encoding="utf-8")
                self._refused(self._snapshot(), self._select(), path)
                file.write_text(original, encoding="utf-8")

    def test_a_source_already_on_a_stable_major_is_refused(self):
        for version in ("1.0.0", "1.0.1-dev", "2.0.0-dev"):
            with self.subTest(version=version):
                self._candidate("set-version", version)
                self._refused(self._snapshot(), self._select(), "0.x")

    def test_no_stable_tag_history_is_refused(self):
        self._git(self.repo, "tag", "-d", "v0.21.3")
        for tag in (None, "v1.0.0-rc.1"):
            with self.subTest(tag=tag):
                if tag:
                    self._git(self.repo, "tag", tag)
                self._refused(self._snapshot(), self._select(), "tag")

    def test_no_commits_since_the_latest_tag_is_refused(self):
        self._git(self.repo, "tag", "-f", "v0.21.3", self.tip)
        self._refused(self._snapshot(), self._select(), "commits")

    def test_an_unreachable_latest_stable_tag_is_refused(self):
        side = self._land("Not on dispatch main", {"side.txt": "side\n"})
        self._git(self.repo, "tag", "v0.22.0", side)
        self._git(self.repo, "reset", "--hard", self.tip)
        self._refused(self._snapshot(), self._select(), "v0.22.0", "reachable")

    def test_a_stable_tag_at_or_above_one_refuses_even_off_main(self):
        side = self._land("Not on dispatch main", {"side.txt": "side\n"})
        self._git(self.repo, "reset", "--hard", self.tip)
        for tag in ("v1.0.0", "v1.2.3", "v2.0.0"):
            with self.subTest(tag=tag):
                self._git(self.repo, "tag", tag, side)
                self._refused(self._snapshot(), self._select(), tag)
                self._git(self.repo, "tag", "-d", tag)


if __name__ == "__main__":
    unittest.main()
