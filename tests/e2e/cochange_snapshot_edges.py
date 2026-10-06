"""Hook index and symlink refusals for §FS-cochange-recipe.snapshots."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import unittest

from cochange_fixture import RECIPE, RecipeCase, trailer


class SnapshotEdgeTests(RecipeCase):
    @unittest.skipUnless(os.name == "posix", "The real hook fixture uses a POSIX shell")
    def test_real_commit_only_hook_uses_candidate_index_and_matches_committed_tree(self):
        """Git's temporary hook index governs §FS-cochange-recipe.snapshots and exit."""
        fixture = self.fixture
        fixture.change(spec=True, test=True)
        fixture.git("add", "-A")
        report_path = Path(self.temp.name) / "hook-report.json"
        index_path = Path(self.temp.name) / "hook-index.txt"
        binary = os.environ.get("GRUND_BUILT") or shutil.which("grund")
        command = shlex.join([sys.executable, str(RECIPE), "--repo", str(fixture.root),
            "--policy", str(fixture.policy), "--grund", binary,
            "commit-msg", "--base", "HEAD", "--message"])
        hook = fixture.root / ".git/hooks/commit-msg"
        fixture.git("config", "core.hooksPath", str(hook.parent))
        hook.write_text('#!/bin/sh\nprintf "%s\\n" "$GIT_INDEX_FILE" > ' +
            shlex.quote(str(index_path)) + '\n' + command + ' "$1" > ' +
            shlex.quote(str(report_path)) + '\n', encoding="utf-8")
        hook.chmod(0o755)
        working = {path: path.read_bytes() for path in fixture.root.rglob("*")
                   if path.is_file() and ".git" not in path.parts}
        index_bytes = (fixture.root / ".git/index").read_bytes()

        def commit(message):
            return subprocess.run(["git", "-C", str(fixture.root), "commit", "--only",
                "src/lib.py", "-m", message], capture_output=True, text=True)

        rejected = commit("Implementation only")
        self.assertNotEqual(0, rejected.returncode, rejected.stdout + rejected.stderr)
        self.assertTrue(index_path.read_text().strip())
        self.assertNotEqual(str(fixture.root / ".git/index"), index_path.read_text().strip())
        refused = json.loads(report_path.read_text())
        self.assertEqual(["spec", "test"], self.source_row(refused)["missing"])
        self.error(refused, "missing-evidence", "src/lib.py")
        self.assertEqual(fixture.base, fixture.git("rev-parse", "HEAD"))
        self.assertEqual(index_bytes, (fixture.root / ".git/index").read_bytes())

        accepted = commit("Authorized refactor" + trailer())
        self.assertEqual(0, accepted.returncode, accepted.stdout + accepted.stderr)
        approved = json.loads(report_path.read_text())
        self.assertEqual([], approved["errors"])
        self.assertEqual([], self.source_row(approved)["missing"])
        target = self.source_row(approved)["targets"][0]
        self.assertEqual([], target["spec_paths"])
        self.assertEqual([], target["test_paths"])
        self.assertTrue(all(waiver["used"] for waiver in self.source_row(approved)["waivers"]))
        self.assertEqual(fixture.git("rev-parse", "HEAD^{tree}"), approved["candidate"]["tree"])
        self.assertEqual("src/lib.py", fixture.git("diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"))
        ci = self.report(fixture.invoke(), 0)
        self.assertEqual(approved["candidate"]["tree"], ci["candidate"]["tree"])
        self.assertEqual(self.source_row(approved)["targets"], self.source_row(ci)["targets"])
        self.assertEqual(working, {path: path.read_bytes() for path in working})

    def test_relative_and_absolute_active_indexes_preserve_both_indexes_and_worktree(self):
        """Active index paths follow Git -C semantics (§FS-cochange-recipe.snapshots)."""
        fixture = self.fixture
        fixture.change(spec=True, test=True)
        fixture.git("add", "-A")
        active = fixture.root / ".git/alternate-index"
        environment = {**os.environ, "GIT_INDEX_FILE": str(active)}
        for args in (("read-tree", "HEAD"), ("add", "src/lib.py")):
            subprocess.run(["git", "-C", str(fixture.root), *args], env=environment, check=True)
        paths = [fixture.root / ".git/index", active, fixture.root / "src/lib.py",
                 fixture.root / "docs/FS-alpha.md", fixture.root / "tests/test_lib.py"]
        expected = subprocess.run(["git", "-C", str(fixture.root), "write-tree"],
            env=environment, capture_output=True, text=True, check=True).stdout.strip()
        before = {path: path.read_bytes() for path in paths}
        for index in (str(active), active.relative_to(fixture.root).as_posix()):
            with self.subTest(index=index):
                report = self.report(fixture.invoke("commit-msg", env={**environment,
                    "GIT_INDEX_FILE": index}), 1)
                self.assertEqual(expected, report["candidate"]["tree"])
                self.assertEqual(["spec", "test"], self.source_row(report)["missing"])
                self.assertEqual(before, {path: path.read_bytes() for path in paths})

    @unittest.skipUnless(os.name == "posix", "Tracked symlink fixture requires POSIX")
    def test_tracked_symlink_cycle_is_a_deterministic_json_refusal(self):
        """One report, exit 2, empty stderr (§FS-cochange-recipe.output, §FS-cochange-recipe.exit)."""
        for path, target in (("a", "b"), ("b", "a"), ("c", "a")):
            (self.fixture.root / path).symlink_to(target)
        self.fixture.commit("Tracked symlink cycle")
        first = self.fixture.invoke()
        report = self.report(first, 2)
        self.error(report, "unsupported-input", "c")
        self.assertIn("symlink cycle", report["errors"][0]["message"])
        self.assertIn("non-cyclic", report["errors"][0]["message"])
        self.assertEqual(1, len(first.stdout.splitlines()))
        self.assertNotIn(str(Path.home() / "ag/tmp"), first.stdout)
        second = self.fixture.invoke()
        self.report(second, 2)
        self.assertEqual(first.stdout, second.stdout)

    @unittest.skipUnless(os.name == "posix", "Tracked symlink fixture requires POSIX")
    def test_tracked_symlink_escaping_snapshot_still_refuses(self):
        (self.fixture.root / "escape").symlink_to("../outside")
        self.fixture.commit("Escaping link")
        report = self.report(self.fixture.invoke(), 2)
        self.error(report, "unsupported-input", "escape")
        self.assertIn("leaves snapshot", report["errors"][0]["message"])
