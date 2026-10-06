"""File-level acceptance for §FS-cochange-recipe.evidence and snapshots."""
from cochange_fixture import RecipeCase, trailer


class EvidenceTests(RecipeCase):
    def test_missing_grounding_cannot_be_waived(self):
        self.fixture.write("src/lib.py", "value = 2\n")
        report = self.evaluate(1, "Refactor" + trailer())
        self.error(report, "missing-grounding", "src/lib.py")
        self.assertIn("grounding", self.source_row(report)["missing"])

    def test_no_evidence_spec_only_and_test_only_fail(self):
        for spec, test, missing in ((False, False, ["spec", "test"]),
                                   (True, False, ["test"]),
                                   (False, True, ["spec"])):
            with self.subTest(spec=spec, test=test):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.change(spec=spec, test=test)
                report = self.evaluate(1)
                self.assertEqual(missing, self.source_row(report)["missing"])
                self.error(report, "missing-evidence", "src/lib.py")

    def test_complete_shared_root_evidence_passes(self):
        self.fixture.change(spec=True, test=True)
        report = self.evaluate(0)
        row = self.source_row(report)
        self.assertEqual([], row["missing"])
        target = next(t for t in row["targets"] if t["id"] == "FS-alpha")
        self.assertEqual(["docs/FS-alpha.md"], target["spec_paths"])
        self.assertEqual(["tests/test_lib.py"], target["test_paths"])
        self.assertEqual("candidate", target["snapshot"])
        self.assertIsNone(target["project"])

    def test_unrelated_and_split_target_evidence_fail(self):
        for targets in (("FS-alpha.1",), ("FS-alpha.1", "FS-beta.1")):
            with self.subTest(targets=targets):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.change(spec=True, targets=targets)
                self.fixture.write("tests/test_lib.py", self.fixture.source(("FS-beta.1",), 2))
                report = self.evaluate(1)
                self.error(report, "missing-evidence", "src/lib.py")

    def test_unchanged_contract_test_and_spec_waiver_pass(self):
        self.fixture.change(test=True)
        reason = "Fix implementation to the existing contract"
        report = self.evaluate(0, "Fix" + trailer(missing=("spec",), reason=reason))
        used = self.source_row(report)["waivers"]
        self.assertTrue(any(w["used"] and w["reason"] == reason
                            and w["missing"] == ["spec"] for w in used))

    def test_refactor_waiver_reason_and_scope_are_visible(self):
        self.fixture.change()
        report = self.evaluate(0, "Refactor" + trailer())
        waiver = self.source_row(report)["waivers"][0]
        self.assertTrue(waiver["used"])
        self.assertEqual("src/lib.py", waiver["path"])
        self.assertEqual("Refactor only", waiver["reason"])
        self.assertEqual(self.fixture.git("rev-parse", "HEAD"), waiver["commit"])

    def test_addition_uses_candidate_and_deletion_uses_base(self):
        for deleted in (False, True):
            with self.subTest(deleted=deleted):
                self.fixture.git("reset", "--hard", self.fixture.base)
                path = "src/lib.py" if deleted else "src/new.py"
                if deleted:
                    self.fixture.git("rm", path)
                else:
                    self.fixture.write(path, self.fixture.source(value=3))
                report = self.evaluate(0, "Move" + trailer(paths=(path,)))
                row = self.source_row(report, path)
                self.assertEqual("deleted" if deleted else "added", row["status"])
                self.assertEqual("base" if deleted else "candidate", row["targets"][0]["snapshot"])
                self.fixture.git("clean", "-fd")

    def test_source_moves_even_pure_moves_keep_obligations(self):
        for edited in (False, True):
            with self.subTest(edited=edited):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.git("mv", "src/lib.py", "src/moved.py")
                if edited:
                    self.fixture.edit("src/moved.py")
                report = self.evaluate(1)
                row = self.source_row(report, "src/moved.py")
                self.assertEqual("renamed", row["status"])
                self.assertEqual(["spec", "test"], row["missing"])

    def test_pure_evidence_moves_and_modes_do_not_supply_edits(self):
        for modes in (False, True):
            with self.subTest(modes=modes):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.change()
                for path in ("docs/FS-alpha.md", "tests/test_lib.py"):
                    if modes:
                        (self.fixture.root / path).chmod(0o755)
                    else:
                        self.fixture.git("mv", path, path.replace("alpha", "renamed").replace("test_lib", "test_moved"))
                if not modes:
                    index = self.fixture.root / "docs/README.md"
                    index.write_text(index.read_text().replace("(FS-alpha.md)", "(FS-renamed.md)"))
                self.assertEqual(["spec", "test"],
                                 self.source_row(self.evaluate(1))["missing"])

    def test_spaces_and_newlines_are_exact_paths(self):
        path = "src/space and\nnewline.py"
        self.fixture.write(path, self.fixture.source())
        report = self.evaluate(0, "Add" + trailer(paths=(path,)))
        self.assertEqual(path, self.source_row(report, path)["waivers"][0]["path"])

    def test_source_mode_only_change_keeps_obligations(self):
        (self.fixture.root / "src/lib.py").chmod(0o755)
        self.assertEqual(["spec", "test"], self.source_row(self.evaluate(1))["missing"])

    def test_deleted_spec_and_test_content_supply_base_evidence(self):
        # Remove the source too, so ordinary check has no dangling candidate citation.
        self.fixture.git("rm", "src/lib.py", "docs/FS-alpha.md", "tests/test_lib.py")
        self.fixture.write("docs/README.md", "- [" + chr(167) + "FS-beta](FS-beta.md)\n")
        report = self.evaluate(0)
        target = self.source_row(report)["targets"][0]
        self.assertEqual("base", target["snapshot"])
        self.assertEqual(["docs/FS-alpha.md"], target["spec_paths"])
        self.assertEqual(["tests/test_lib.py"], target["test_paths"])

    def test_partial_staging_ignores_unstaged_evidence_and_preserves_index(self):
        self.fixture.change()
        self.fixture.git("add", "src/lib.py")
        self.fixture.edit("docs/FS-alpha.md")
        self.fixture.edit("tests/test_lib.py")
        index_path = self.fixture.root / ".git/index"
        index_before = index_path.read_bytes()
        work_before = (self.fixture.root / "tests/test_lib.py").read_bytes()
        report = self.report(self.fixture.invoke("commit-msg"), 1)
        self.assertEqual(["spec", "test"], self.source_row(report)["missing"])
        self.assertEqual(index_before, index_path.read_bytes())
        self.assertEqual(work_before, (self.fixture.root / "tests/test_lib.py").read_bytes())

    def test_valid_empty_diff_still_reports_exact_trees_and_is_deterministic(self):
        first = self.fixture.invoke()
        report = self.report(first, 0)
        self.assertEqual([], report["sources"])
        self.assertEqual(self.fixture.base, report["base"]["commit"])
        self.assertEqual(self.fixture.git("rev-parse", "HEAD^{tree}"), report["candidate"]["tree"])
        self.assertEqual(first.stdout, self.fixture.invoke().stdout)
