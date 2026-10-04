"""§AR-ci.10.4 — real Git descendants cannot cross strict fixture cleanup.

§FS-distribution.4.5.1 — passing assertions also require strict fixture removal.

Ports the #420 pack-write reproducer to unittest and exercises the three release
suites and commands that bypass GitFixture._git. Linux provides the
LD_PRELOAD seam and /proc process evidence; no timing sleeps hide the race.
"""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import test_prepare_changelog_release_list as listing
import test_prepare_changelog_release_notices as notices
import test_prepare_changelog_release as release
from git_fixture_lifetime_probe import exercise, seed
from release_forge_fixture import ForgeRepository


@unittest.skipUnless(sys.platform.startswith("linux"), "pack-write seam needs Linux")
class GitFixtureLifetimeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not shutil.which("cc"):
            raise unittest.SkipTest("pack-write seam needs a C compiler")
        base = Path.home() / "ag/tmp"
        base.mkdir(parents=True, exist_ok=True)
        cls.scratch = tempfile.TemporaryDirectory(prefix="g420-build-", dir=base)
        cls.addClassCleanup(cls.scratch.cleanup)
        root = Path(cls.scratch.name)
        cls.library = root / "pack.so"
        subprocess.run(["cc", "-shared", "-fPIC", "-Wall", "-Wextra", "-Werror",
                        str(Path(__file__).with_name("git_pack_write_seam.c")),
                        "-ldl", "-o", str(cls.library)], check=True, capture_output=True)
        cls.template = root / "template"
        cls.template.mkdir()
        # Init templates are lower precedence than the fixture's local policy.
        # Force the old GC task on newer Git too, covering both entry points.
        (cls.template / "config").write_text(
            "[gc]\n\tauto = 6700\n\tautoDetach = true\n"
            "[maintenance]\n\tauto = true\n\tautoDetach = true\n\tstrategy = gc\n"
            "[maintenance \"gc\"]\n\tenabled = true\n", encoding="utf-8")

    def _exercise(self, case, construct, operation=None, seed_initial=True):
        # Command-scope overrides could otherwise defeat even correct local
        # policy; those belong to the probe, not the fixture contract.
        with patch.dict(os.environ):
            for key in list(os.environ):
                if key.startswith("GIT_CONFIG_") or key in ("GIT_DIR", "GIT_WORK_TREE"):
                    del os.environ[key]
            exercise(self, case, construct, operation, self.library, self.template, seed_initial)

    def test_list_initialization_owns_git_until_cleanup(self):
        name = "test_prepare_writes_the_release_the_forge_and_the_tree_describe"
        case = listing.EndToEndTests(name)
        self._exercise(case, getattr(case, name))

    def test_notice_initialization_owns_git_until_cleanup(self):
        name = "test_a_notice_added_since_the_tag_is_published"
        case = notices.NoticeTests(name)
        self._exercise(case, getattr(case, name))

    def _release_case(self):
        class ReleaseCase(ForgeRepository, unittest.TestCase):
            pass
        return ReleaseCase()

    def test_release_initialization_owns_git_until_cleanup(self):
        name = "test_prepare_archives_the_previous_latest_and_links_it"
        case = release.ArchiveTests(name)
        self._exercise(case, getattr(case, name))

    def test_direct_dated_commit_owns_git_until_cleanup(self):
        case = self._release_case()

        def land():
            seed(case.repo)
            case._land("A dated commit", {"dated.txt": "A dated change.\n"})

        self._exercise(case, lambda: case._repository({"base.txt": "Base.\n"}),
                       land, seed_initial=False)

    def test_direct_rebase_owns_git_until_cleanup(self):
        case = self._release_case()

        def rebase():
            case._git(case.repo, "checkout", "-q", "-b", "topic")
            case._land("Topic", {"topic.txt": "Topic.\n"})
            case._git(case.repo, "checkout", "-q", "main")
            case._land("Main", {"main.txt": "Main.\n"})
            case._git(case.repo, "checkout", "-q", "topic")
            seed(case.repo)
            # The apply backend also runs automatic maintenance when it
            # finishes on older Git; merge-backend versions can omit it.
            result = subprocess.run(
                ["git", "-C", str(case.repo), "-c", "user.name=Fixture",
                 "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false",
                 "rebase", "--apply", "--quiet", "main"],
                capture_output=True, text=True)
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertEqual("Topic.\n", (case.repo / "topic.txt").read_text())
            self.assertEqual("Main.\n", (case.repo / "main.txt").read_text())

        self._exercise(case, lambda: case._repository({"base.txt": "Base.\n"}),
                       rebase, seed_initial=False)


if __name__ == "__main__":
    unittest.main()
