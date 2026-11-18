"""Scan-scope refusals name the policy remedy (§FS-cochange-recipe.snapshots.scope).

A tracked file a selection classifies but Grund never scans refuses every run; the
refusal names the selection that classified it and the remedy the policy offers,
and the shipped example policy accepts the root commit of `grund init --scaffold`.
"""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import unittest

from cochange_fixture import RECIPE, ROOT, RecipeCase

class ScanScopeTests(RecipeCase):
    def scan_scope(self, report, path):
        rows = [row for row in report["errors"]
                if row["code"] == "scan-scope" and row["path"] == path]
        self.assertEqual(1, len(rows), report)
        self.assertEqual("cover", rows[0]["query"])
        return rows[0]["message"]

    def assert_policy_remedy(self, message, key, other):
        self.assertIn(key, message)
        self.assertNotIn(other, message)
        self.assertRegex(message, re.compile(r"\bnot scan", re.I))
        self.assertRegex(message, re.compile(r"\bnarrow", re.I))
        self.assertRegex(message, re.compile(r"\bremove", re.I))

    def test_hidden_file_under_test_paths_names_test_paths_not_scan_config(self):
        self.fixture.write("tests/integration/.gitkeep", "")
        self.fixture.change(spec=True, test=True)
        report = self.evaluate(2)
        message = self.scan_scope(report, "tests/integration/.gitkeep")
        self.assert_policy_remedy(message, "test_paths", "source_paths")
        self.assertNotRegex(message, re.compile(r"scan configuration", re.I))

    def test_unscanned_source_file_names_source_paths_on_a_test_only_change(self):
        self.fixture.write("src/data.json", '{"a": 1}\n')
        self.fixture.base = self.fixture.commit("Data file")
        self.fixture.edit("tests/test_lib.py")
        report = self.evaluate(2, "Touch only a test")
        self.assertEqual([], report["sources"])
        message = self.scan_scope(report, "src/data.json")
        self.assert_policy_remedy(message, "source_paths", "test_paths")

    def move_destination(self, destination):
        """A source move to an unscanned destination (§FS-cochange-recipe.snapshots.scope.move)."""
        (self.fixture.root / "lib").mkdir()
        self.fixture.git("mv", "src/lib.py", destination)
        message = self.scan_scope(self.evaluate(2), destination)
        self.assertIn("source move", message)
        self.assertRegex(message, re.compile(r"\bnot scan", re.I))
        self.assertRegex(message, re.compile(r"\bmove it to a path", re.I))
        self.assertNotRegex(message, re.compile(r"\bnarrow", re.I))
        return message

    def test_hidden_move_destination_is_not_offered_scan_config(self):
        message = self.move_destination("lib/.lib.py")
        self.assertNotRegex(message, re.compile(r"scan configuration", re.I))

    def test_unscanned_move_destination_is_offered_scan_config(self):
        message = self.move_destination("lib/lib.json")
        self.assertRegex(message, re.compile(r"scan configuration", re.I))

    def test_shipped_policy_accepts_the_scaffold_root_commit(self):
        repo = Path(self.temp.name) / "scaffold"
        repo.mkdir()
        env = {**os.environ, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
        grund = os.environ.get("GRUND_BUILT") or shutil.which("grund")
        subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
        subprocess.run([grund, "init", "--scaffold", "--name", "r"], cwd=repo, check=True,
                       env=env, capture_output=True)
        self.assertTrue((repo / "tests/integration/.gitkeep").exists())
        subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True, env=env)
        message = repo.parent / "scaffold-message"
        message.write_text("root\n", encoding="utf-8")
        result = subprocess.run([sys.executable, str(RECIPE), "--repo", str(repo),
            "--policy", str(ROOT / "examples/cochange/policy.json"), "--grund", grund,
            "commit-msg", "--base", "empty", "--message", str(message)],
            capture_output=True, text=True, cwd=repo, env=env)
        report = json.loads(result.stdout)
        self.assertEqual([], report["errors"], report)
        self.assertEqual(0, result.returncode, report)


if __name__ == "__main__":
    unittest.main()
