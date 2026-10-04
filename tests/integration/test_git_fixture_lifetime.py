"""§AR-ci.10.4 — real Git descendants cannot cross strict fixture cleanup.

Ports the #420 pack-write reproducer to unittest and exercises the three actual
constructors and the commands that bypass GitFixture._git. Linux provides the
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

import test_changelog_entries_merge as merge
import test_prepare_changelog_release as release
import test_prepare_changelog_release_stamp as stamp
from git_fixture_lifetime_probe import exercise, seed


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

    def test_stamp_initialization_owns_git_until_cleanup(self):
        case = stamp.StampTests("test_a_number_that_would_go_into_several_entries_goes_into_none")
        self._exercise(case, case.setUp,
                       case.test_a_number_that_would_go_into_several_entries_goes_into_none)

    def test_merge_initialization_owns_git_until_cleanup(self):
        case = merge.TwoPullRequestsTests()
        self._exercise(case, case.setUp)

    def _release_case(self):
        class ReleaseCase(release.ReleaseRepository, unittest.TestCase):
            pass
        return ReleaseCase()

    def test_release_initialization_owns_git_until_cleanup(self):
        case = self._release_case()
        self._exercise(case, case._repository)

    def test_direct_dated_commit_owns_git_until_cleanup(self):
        case = self._release_case()

        def land():
            seed(case.repo)
            case._land({"dated.added.md": "- A dated entry.\n"}, entries=True)

        self._exercise(case, case._repository, land, seed_initial=False)

    def test_direct_rebase_owns_git_until_cleanup(self):
        case = merge.TwoPullRequestsTests()

        def rebase():
            _, a, b = case._scenario("empty")
            case._git(case.repo, "checkout", "-q", "-B", "trial", b)
            seed(case.repo)
            # The apply backend also runs automatic maintenance when it
            # finishes on older Git; merge-backend versions can omit it.
            result = case._try("rebase", "--apply", "--quiet", a)
            self.assertEqual(0, result.returncode, result.stderr)

        self._exercise(case, case.setUp, rebase, seed_initial=False)


if __name__ == "__main__":
    unittest.main()
