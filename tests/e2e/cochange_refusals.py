"""Refusal and workspace acceptance for §FS-cochange-recipe.snapshots."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest
from cochange_fixture import CONFIG, ROOT, RecipeCase, trailer


class RefusalTests(RecipeCase):
    def test_missing_base_and_shallow_history_refuse(self):
        self.error(self.report(self.fixture.invoke(base="f" * 40), 2), "git-input")
        self.fixture.change(spec=True, test=True)
        self.fixture.commit("Change")
        clone = Path(self.temp.name) / "shallow"
        subprocess.run(["git", "clone", "-q", "--depth=1",
            self.fixture.root.as_uri(), str(clone)], check=True)
        self.fixture.root = clone
        self.error(self.report(self.fixture.invoke(), 2), "git-input")

    def test_empty_diff_query_failure_and_partial_records_refuse(self):
        for query in ("check", "cover", "list"):
            with self.subTest(query=query):
                result = self.corrupt(query, "failure")
                self.error(self.report(result, 2), "check" if query == "check" else "query")

    def corrupt(self, query, corruption):
        proxy = Path(self.temp.name) / "grund-proxy"
        shutil.copyfile(ROOT / "tests/e2e/cochange_query_proxy.py", proxy)
        proxy.chmod(0o755)
        environment = {**os.environ, "COCHANGE_REAL_GRUND":
            os.environ.get("GRUND_BUILT", shutil.which("grund")),
            "COCHANGE_CORRUPT_QUERY": query, "COCHANGE_CORRUPTION": corruption}
        return self.fixture.invoke(grund=str(proxy), env=environment)

    def test_query_envelopes_missing_duplicate_malformed_mismatched_or_failed_refuse(self):
        self.fixture.change(spec=True, test=True)
        self.fixture.commit("Change")
        for query, corruptions in (
            ("cover", ("duplicate", "malformed")),
            ("list", ("malformed", "failure")),
            ("show", ("missing", "duplicate", "malformed", "mismatched-query",
                      "failed-envelope", "missing-result", "failure"))):
            for corruption in corruptions:
                with self.subTest(query=query, corruption=corruption):
                    report = self.report(self.corrupt(query, corruption), 2)
                    self.assertTrue(any(e["query"] == query for e in report["errors"]), report)
                    self.assertTrue(any(e["code"] in ("query", "resolution")
                                        for e in report["errors"]), report)

    def test_missing_scan_row_differs_from_scanned_zero_citations(self):
        self.fixture.change()
        self.fixture.commit("Change" + trailer())
        self.error(self.report(self.corrupt("cover", "scope"), 2), "scan-scope", "src/lib.py")

    def test_unresolved_coordinate_is_never_evidence(self):
        for target in ("FS-absent", "FS-alpha.99"):
            with self.subTest(target=target):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.change(spec=True, test=True, targets=(target,))
                report = self.evaluate(2, "Change" + trailer())
                self.assertTrue(any(e["code"] in ("resolution", "check")
                                    for e in report["errors"]), report)

    def test_check_errors_cannot_be_waived(self):
        self.fixture.change()
        self.fixture.write("tests/test_lib.py", self.fixture.source(("FS-absent",)))
        self.error(self.evaluate(2, "Refactor" + trailer()), "check")

    def test_invalid_policy_and_config_refuse_even_empty_diff(self):
        policy = json.loads(self.fixture.policy.read_text())
        policy["test_paths"] = ["src/"]
        self.fixture.policy.write_text(json.dumps(policy))
        self.error(self.report(self.fixture.invoke(), 2), "config")
        policy["test_paths"] = ["tests/"]
        self.fixture.policy.write_text(json.dumps(policy))
        self.fixture.write("grund.toml", "not valid TOML = [")
        self.fixture.commit("Bad config")
        self.error(self.report(self.fixture.invoke(base="HEAD"), 2), "config")

    def test_ordinary_config_edits_use_own_snapshots(self):
        self.fixture.change(spec=True, test=True)
        self.fixture.edit("grund.toml")
        self.evaluate(0)

    def test_git_relative_config_root_keeps_git_relative_evidence_paths(self):
        (self.fixture.root / "app").mkdir()
        for path in ("grund.toml", "docs", "src", "tests"):
            self.fixture.git("mv", path, "app/" + path)
        self.fixture.base = self.fixture.commit("Nested baseline")
        self.fixture.policy.write_text(json.dumps({"config_root": "app",
            "source_paths": ["app/src/"], "test_paths": ["app/tests/"],
            "eligible_kinds": ["FS"]}))
        self.fixture.edit("app/src/lib.py")
        self.fixture.edit("app/docs/FS-alpha.md")
        self.fixture.edit("app/tests/test_lib.py")
        report = self.evaluate(0)
        target = self.source_row(report, "app/src/lib.py")["targets"][0]
        self.assertEqual(["app/docs/FS-alpha.md"], target["spec_paths"])
        self.assertEqual(["app/tests/test_lib.py"], target["test_paths"])

    def test_changed_project_identity_and_external_roots_refuse(self):
        for config in (CONFIG.replace('"fixture"', '"renamed"'),
                       CONFIG + '\n[workspace]\nmembers = ["../outside"]\n',
                       CONFIG.replace('folder = "docs"', 'folder = "../facts"'),
                       CONFIG.replace('folder = "docs"',
                           'folder = "../facts"\nresolve = "should"\nfetch = "false"')):
            with self.subTest(config=config):
                self.fixture.git("reset", "--hard", self.fixture.base)
                self.fixture.write("grund.toml", config)
                self.error(self.evaluate(2), "unsupported-input")

    def test_intent_to_add_split_sparse_and_unmerged_index_refuse(self):
        self.fixture.write("src/new.py", self.fixture.source())
        self.fixture.git("add", "-N", "src/new.py")
        self.error(self.report(self.fixture.invoke("commit-msg"), 2), "unsupported-input")
        self.fixture.git("reset", "--hard", self.fixture.base)
        self.fixture.git("update-index", "--split-index")
        self.error(self.report(self.fixture.invoke("commit-msg"), 2), "unsupported-input")
        self.fixture.git("update-index", "--no-split-index")
        # Create a valid conflicted index without changing working-tree bytes.
        blob = self.fixture.git("rev-parse", "HEAD:src/lib.py")
        self.fixture.git("update-index", "--force-remove", "src/lib.py")
        subprocess.run(["git", "-C", str(self.fixture.root), "update-index", "--index-info"],
            input=f"100644 {blob} 1\tsrc/lib.py\n100644 {blob} 2\tsrc/lib.py\n".encode(), check=True)
        self.error(self.report(self.fixture.invoke("commit-msg"), 2), "unsupported-input")
        self.fixture.git("reset", "--hard", self.fixture.base)
        self.fixture.git("sparse-checkout", "init", "--cone", "--sparse-index")
        self.fixture.git("sparse-checkout", "set", "src")
        self.error(self.report(self.fixture.invoke("commit-msg"), 2), "unsupported-input")

    @unittest.skipUnless(os.name == "posix", "Git byte paths require POSIX")
    def test_non_utf8_paths_and_submodules_refuse(self):
        path = os.fsencode(self.fixture.root) + b"/src/non-utf8-\xff.py"
        with open(path, "wb") as stream:
            stream.write(b"value = 1\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-q", "-m", "Byte path")
        self.error(self.report(self.fixture.invoke(), 2), "unsupported-input")
        self.fixture.git("reset", "--hard", self.fixture.base)
        self.fixture.git("update-index", "--add", "--cacheinfo",
            f"160000,{self.fixture.base},vendor")
        self.fixture.commit("Gitlink", stage=False)
        self.error(self.report(self.fixture.invoke(), 2), "unsupported-input")


class WorkspaceTests(RecipeCase):
    def workspace(self):
        self.fixture.write("grund.toml", CONFIG +
            '\n[workspace]\nmembers = ["packages/one", "packages/two"]\n')
        for member in ("one", "two"):
            prefix = f"packages/{member}/"
            self.fixture.write(prefix + "grund.toml", CONFIG.replace('"fixture"', f'"{member}"'))
            self.fixture.write(prefix + "docs/FS-alpha.md", "# FS-alpha: Shared ID\n\n## 1. Behavior\n\nBefore.\n")
            self.fixture.write(prefix + "docs/README.md", "- [" + chr(167) + "FS-alpha](FS-alpha.md)\n")
            self.fixture.write(prefix + "tests/test_lib.py", self.fixture.source())
        self.fixture.base = self.fixture.commit("Workspace baseline")
        policy = json.loads(self.fixture.policy.read_text())
        policy["test_paths"] += ["packages/one/tests/", "packages/two/tests/"]
        self.fixture.policy.write_text(json.dumps(policy))

    def test_qualified_targets_normalize_alias_and_member_local_citations(self):
        self.workspace()
        self.fixture.change(targets=("one/FS-alpha.1",))
        self.fixture.edit("packages/one/docs/FS-alpha.md")
        self.fixture.edit("packages/one/tests/test_lib.py")
        report = self.evaluate(0)
        target = self.source_row(report)["targets"][0]
        self.assertEqual("one", target["project"])
        self.assertEqual("FS-alpha", target["id"])
        self.assertEqual(["packages/one/tests/test_lib.py"], target["test_paths"])

    def test_duplicate_bare_ids_in_other_member_are_unrelated(self):
        self.workspace()
        self.fixture.change(targets=("one/FS-alpha.1",))
        self.fixture.edit("packages/two/docs/FS-alpha.md")
        self.fixture.edit("packages/two/tests/test_lib.py")
        self.error(self.evaluate(1), "missing-evidence", "src/lib.py")

    def test_changed_member_wiring_refuses(self):
        self.workspace()
        self.fixture.write("grund.toml", CONFIG + '\n[workspace]\nmembers = ["packages/one"]\n')
        self.error(self.evaluate(2), "unsupported-input")
