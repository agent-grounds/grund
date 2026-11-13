"""Gate entry for the two-run suite oracle (§AR-ci.10.2).

The executable end-to-end test is §FS-repository-maintenance.4.1. The skip
lands only for this red specification commit; implement removes the decorator.
Run the executable directly to observe the regression before implementation.
"""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    'fixture_root_lifetime', ROOT / 'tests/e2e/fixture_root_lifetime.py')
ORACLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORACLE)


class FixtureRootLifetime(unittest.TestCase):
    def test_counter_allows_only_an_empty_shared_base(self):
        with tempfile.TemporaryDirectory(prefix='grund-1362-counter-') as scratch:
            root = Path(scratch)
            base = root / 'grund-lib-tests'
            base.mkdir()
            self.assertEqual(ORACLE.entries(root), [])
            leaf = base / 'owned'
            leaf.mkdir()
            (leaf / 'sentinel').write_text('left behind')
            self.assertEqual(set(ORACLE.entries(root)), {base, leaf, leaf / 'sentinel'})

    def test_counter_detects_a_renamed_base(self):
        with tempfile.TemporaryDirectory(prefix='grund-1362-counter-') as scratch:
            root = Path(scratch)
            moved = root / 'another-base'
            moved.mkdir()
            (moved / 'sentinel').write_text('still leaked')
            self.assertEqual(set(ORACLE.entries(root)), {moved, moved / 'sentinel'})

    @unittest.skipUnless(os.name == 'posix', 'requires unprivileged symlinks')
    def test_counter_counts_links_without_following_them(self):
        with tempfile.TemporaryDirectory(prefix='grund-1362-counter-') as scratch:
            root = Path(scratch)
            (root / 'loop').symlink_to(root, target_is_directory=True)
            (root / 'missing').symlink_to(root / 'absent')
            self.assertEqual(set(ORACLE.entries(root)), {root / 'loop', root / 'missing'})

    @unittest.skip('red until #1362 roots own cleanup; implement removes this')
    def test_two_passing_core_suites_leave_no_owned_roots(self):
        with tempfile.TemporaryDirectory(prefix='grund-1362-oracle-logs-') as logs:
            self.assertEqual(ORACLE.run(ROOT, Path(logs)), 0,
                             'passing core suites left owned fixture roots')


if __name__ == '__main__':
    unittest.main()
