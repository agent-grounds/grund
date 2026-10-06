"""Message scopes and comparison parity for §FS-cochange-recipe.waivers."""
from cochange_fixture import RecipeCase, trailer


class HistoryTests(RecipeCase):
    def test_multiple_merge_bases_refuse_instead_of_guessing(self):
        self.fixture.write("left.txt", "Left\n")
        left = self.fixture.commit("Left")
        self.fixture.git("checkout", "-q", "-b", "right", self.fixture.base)
        self.fixture.write("right.txt", "Right\n")
        right = self.fixture.commit("Right")
        self.fixture.git("merge", "--no-ff", "-m", "Right merge", left)
        right_merge = self.fixture.git("rev-parse", "HEAD")
        self.fixture.git("checkout", "-q", "main")
        self.fixture.git("merge", "--no-ff", "-m", "Left merge", right)
        self.assertEqual(2, len(self.fixture.git("merge-base", "--all", "HEAD", right_merge).splitlines()))
        self.error(self.report(self.fixture.invoke(base=right_merge), 2), "git-input")

    def test_malformed_duplicate_conflicting_and_out_of_scope_trailers_refuse(self):
        values = ["not-json", '{}', '{"paths":[],"missing":["spec"],"reason":"x"}',
            '{"paths":["src/lib.py"],"missing":["other"],"reason":"x"}',
            '{"paths":["src/lib.py"],"missing":["spec"],"reason":" "}',
            '{"paths":["src/lib.py","src/lib.py"],"missing":["spec"],"reason":"x"}',
            '{"paths":["../lib.py"],"missing":["spec"],"reason":"x"}',
            '{"paths":["src/lib.py"],"missing":["spec"],"reason":"x","reason":"y"}',
            '{"paths":["src/lib.py"],"missing":["spec"],"reason":"x","extra":1}']
        messages = ["Change\n\nGrund-Cochange: " + v for v in values]
        messages += ["Change" + trailer() + "\n" + trailer(reason="Conflicting").lstrip("\n"),
                     "Change" + trailer(paths=("src/absent.py",)),
                     "Change" + trailer(paths=("tests/test_lib.py",))]
        for message in messages:
            with self.subTest(message=message):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.change(spec=True, test=True)
                self.error(self.evaluate(2, message), "waiver")

    def test_stale_waiver_on_unchanged_path_refuses_even_empty_diff(self):
        self.error(self.evaluate(2, "Empty" + trailer()), "waiver")

    def test_valid_unused_trailer_is_visible(self):
        self.fixture.change(spec=True, test=True)
        report = self.evaluate(0, "Change" + trailer())
        self.assertFalse(self.source_row(report)["waivers"][0]["used"])

    def test_repeated_edits_each_require_commit_local_waivers(self):
        self.fixture.change()
        first = self.fixture.commit("First" + trailer())
        self.fixture.edit("src/lib.py")
        second = self.fixture.commit("Second")
        report = self.report(self.fixture.invoke(), 1)
        self.error(report, "missing-waiver", "src/lib.py")
        self.assertTrue(any(e["commit"] == second for e in report["errors"]))
        self.fixture.git("reset", "--hard", first)
        self.fixture.edit("src/lib.py")
        self.fixture.commit("Second" + trailer())
        self.report(self.fixture.invoke(), 0)

    def test_whole_pr_evidence_can_be_in_later_commit(self):
        self.fixture.change()
        self.fixture.commit("Implementation")
        self.fixture.edit("docs/FS-alpha.md")
        self.fixture.edit("tests/test_lib.py")
        self.evaluate(0, "Evidence")

    def test_root_commit_and_explicit_amend_base(self):
        self.fixture.change()
        self.fixture.git("add", "-A")
        report = self.report(self.fixture.invoke("commit-msg",
            base="empty", message="Root" + trailer()), 0)
        self.assertIsNone(report["base"]["commit"])
        # Baseline's first parent is empty, so amend compares all root content.
        self.assertEqual("added", self.source_row(report)["status"])
        self.fixture.commit("First" + trailer(), stage=False)
        self.fixture.change(spec=True, test=True)
        self.fixture.git("add", "-A")
        parent = self.fixture.git("rev-parse", "HEAD^")
        amended = self.report(self.fixture.invoke("commit-msg", base=parent), 0)
        self.assertEqual(parent, amended["base"]["commit"])

    def test_squash_message_checked_afresh(self):
        self.fixture.change()
        self.fixture.commit("First" + trailer())
        self.fixture.edit("src/lib.py")
        self.fixture.commit("Second" + trailer())
        self.report(self.fixture.invoke(), 0)
        self.fixture.git("reset", "--soft", self.fixture.base)
        self.fixture.commit("Squash without trailer", stage=False)
        self.error(self.report(self.fixture.invoke(), 1), "missing-evidence")

    def test_merge_scope_uses_first_parent_and_needs_own_trailer(self):
        self.fixture.git("checkout", "-q", "-b", "topic")
        self.fixture.change()
        self.fixture.commit("Topic" + trailer())
        self.fixture.git("checkout", "-q", "main")
        self.fixture.write("note.txt", "Main-only change\n")
        self.fixture.commit("Main")
        self.fixture.git("merge", "--no-ff", "-m", "Import", "topic")
        merge = self.fixture.git("rev-parse", "HEAD")
        report = self.report(self.fixture.invoke(), 1)
        self.error(report, "missing-waiver", "src/lib.py")
        self.assertTrue(any(e["commit"] == merge for e in report["errors"]))
        self.fixture.git("commit", "--amend", "-q", "-m", "Import" + trailer())
        self.report(self.fixture.invoke(), 0)

    def test_index_and_ci_agree_on_same_tree_policy_and_message(self):
        self.fixture.change(test=True)
        self.fixture.git("add", "-A")
        message = "Fix" + trailer(missing=("spec",), reason="Existing contract")
        local = self.report(self.fixture.invoke("commit-msg", message=message), 0)
        self.fixture.commit(message, stage=False)
        ci = self.report(self.fixture.invoke(), 0)
        self.assertEqual(local["candidate"]["tree"], ci["candidate"]["tree"])
        self.assertEqual(local["base"], ci["base"])
        for report in (local, ci):
            for row in report["sources"]:
                for waiver in row["waivers"]:
                    waiver["commit"] = None
        self.assertEqual(local["sources"], ci["sources"])
        self.assertEqual(local["errors"], ci["errors"])

    def test_multiple_disjoint_trailers_can_supply_both_missing_classes(self):
        self.fixture.change()
        message = "Refactor" + trailer(missing=("spec",)) + "\n" + trailer(missing=("test",)).lstrip("\n")
        self.evaluate(0, message)
