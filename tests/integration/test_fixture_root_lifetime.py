"""Gate entry for the two-run suite oracle (§AR-ci.10.2).

The executable end-to-end test is §FS-repository-maintenance.4.1. When its
build or a suite run fails, the failure carries that child's output and not
only the log it was written to (§AR-ci.requirements.5.1), because the gate
entry removes the log directory with the run. The two stub-cargo cases are
skipped for the red specification commit of #1395; implement removes the
decorator.
"""
import importlib.util
import json
import os
from pathlib import Path
import shlex
import tempfile
import unittest
from unittest import mock

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

    def test_two_passing_core_suites_leave_no_owned_roots(self):
        with tempfile.TemporaryDirectory(prefix='grund-1362-oracle-logs-') as logs:
            self.assertEqual(ORACLE.run(ROOT, Path(logs)), 0,
                             'passing core suites left owned fixture roots')

    # §AR-ci.requirements.5.1: a failure prints its child processes' output.
    @unittest.skip("red until #1395: the oracle's failure carries its child's output; "
                   "implement removes this")
    @unittest.skipUnless(os.name == 'posix', 'stubs cargo with a shell script')
    def test_a_failed_build_carries_the_compiler_error(self):
        with tempfile.TemporaryDirectory(prefix='grund-1395-stub-') as scratch:
            root = Path(scratch)
            failure = raised_by_stub_cargo(self, root, build_error=STUB_BUILD_ERROR)
            self.assertIn(STUB_BUILD_ERROR, failure)

    # §AR-ci.requirements.5.1: a failure prints its child processes' output.
    @unittest.skip("red until #1395: the oracle's failure carries its child's output; "
                   "implement removes this")
    @unittest.skipUnless(os.name == 'posix', 'stubs cargo with a shell script')
    def test_a_failed_suite_run_carries_the_suite_output(self):
        with tempfile.TemporaryDirectory(prefix='grund-1395-stub-') as scratch:
            root = Path(scratch)
            suite = executable(root / 'suite', f"echo {shlex.quote(STUB_SUITE_FAILURE)}\n"
                               "echo 'test result: FAILED. 0 passed; 1 failed'\nexit 101")
            artifact = {'reason': 'compiler-artifact', 'executable': str(suite),
                        'target': {'name': 'grund_core', 'kind': ['lib']},
                        'profile': {'test': True}}
            failure = raised_by_stub_cargo(self, root, artifact=artifact)
            self.assertIn(STUB_SUITE_FAILURE, failure)


STUB_BUILD_ERROR = 'error[E0432]: unresolved import `grund_1395_stub` (the stub compiler error)'
STUB_SUITE_FAILURE = "thread 'grund_1395_stub' panicked: the stub suite failure"


def executable(path, body):
    """A POSIX shell script at `path` running `body`."""
    path.write_text(f'#!/bin/sh\n{body}\n')
    path.chmod(0o755)
    return path


def raised_by_stub_cargo(case, root, build_error=None, artifact=None):
    """The oracle's RuntimeError text with a stub `cargo` first on PATH.

    The stub prints `artifact` as cargo's one JSON line and exits 0, or prints
    `build_error` to stderr and exits 101 as cargo does on a compile error.
    """
    (root / 'bin').mkdir()
    (root / 'crates/grund-core').mkdir(parents=True)
    if build_error is not None:
        body = f'echo {shlex.quote(build_error)} >&2\nexit 101'
    else:
        body = f'echo {shlex.quote(json.dumps(artifact))}'
    executable(root / 'bin/cargo', body)
    path = f"{root / 'bin'}{os.pathsep}{os.environ.get('PATH', '')}"
    with mock.patch.dict(os.environ, {'PATH': path}), case.assertRaises(RuntimeError) as raised:
        ORACLE.run(root, root / 'logs')
    return str(raised.exception)


if __name__ == '__main__':
    unittest.main()
