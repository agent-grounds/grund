"""Gate entry for the two-run suite oracle (§AR-ci.10.2).

The executable end-to-end test is §FS-repository-maintenance.4.1. When its
build or a suite run fails, the failure carries that child's output and not
only the log it was written to (§AR-ci.requirements.5.1), because the gate
entry removes the log directory with the run.
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
    @unittest.skipUnless(os.name == 'posix', 'stubs cargo with a shell script')
    def test_a_failed_build_carries_the_compiler_error(self):
        with tempfile.TemporaryDirectory(prefix='grund-1395-stub-') as scratch:
            root = Path(scratch)
            stdout = ''.join(f'{json.dumps(line)}\n' for line in [
                compiler_message(root, f'{STUB_DIAGNOSTIC}\n\n'),
                {'reason': 'build-finished', 'success': False}])
            failure = raised_by_stub_cargo(self, root, stdout, STUB_CARGO_STDERR, status=101)
            self.assertIn(STUB_DIAGNOSTIC, failure)
            self.assertLess(failure.index(STUB_DIAGNOSTIC), failure.index(STUB_CARGO_ERROR))
            self.assertNotIn('package.homepage', failure)

    # §AR-ci.requirements.5.1: a failure prints its child processes' output.
    @unittest.skipUnless(os.name == 'posix', 'stubs cargo with a shell script')
    def test_a_failed_suite_run_carries_the_suite_output(self):
        with tempfile.TemporaryDirectory(prefix='grund-1395-stub-') as scratch:
            root = Path(scratch)
            output = f'{STUB_SUITE_FAILURE}\ntest result: FAILED. 0 passed; 1 failed\n'
            failure = raised_by_stub_cargo(self, root, stub_suite(root, output))
            self.assertIn(STUB_SUITE_FAILURE, failure)

    # §AR-ci.requirements.5.1: each failure with its panic or assertion message.
    @unittest.skipUnless(os.name == 'posix', 'stubs cargo with a shell script')
    def test_a_long_failure_message_keeps_its_panic_header(self):
        with tempfile.TemporaryDirectory(prefix='grund-1395-stub-') as scratch:
            root = Path(scratch)
            output = '\n'.join([
                '', 'running 2 tests', '. 1/2', 'grund_1395_stub --- FAILED', '', 'failures:',
                '', '---- grund_1395_stub stdout ----', '', STUB_PANIC,
                *(f'  diff line {number}' for number in range(60)), '', '', 'failures:',
                '    grund_1395_stub', '', 'test result: FAILED. 1 passed; 1 failed; 0 ignored; '
                '0 measured; 0 filtered out; finished in 0.00s', ''])
            failure = raised_by_stub_cargo(self, root, stub_suite(root, output))
            self.assertIn(STUB_PANIC, failure)
            self.assertIn('  diff line 59', failure)

    # §AR-ci.requirements.5.1: a bound on the carried output says where the rest is.
    def test_a_runaway_output_is_cut_in_the_middle_naming_its_log(self):
        lines = [f'line {number}' for number in range(3 * ORACLE.BOUND)]
        carried = ORACLE.carried(lines, 'the/run1.log').splitlines()
        self.assertEqual(len(carried), ORACLE.BOUND + 1)
        self.assertEqual((carried[0], carried[-1]), (lines[0], lines[-1]))
        self.assertIn('the/run1.log', carried[ORACLE.BOUND // 2])


STUB_SUITE_FAILURE = "thread 'grund_1395_stub' panicked: the stub suite failure"
STUB_PANIC = "thread 'grund_1395_stub' (4242) panicked at crates/grund-core/src/lib.rs:8:34:"
# What cargo 1.91 wrote for the unused import #1395 reported, under `-D warnings`: the
# diagnostic only as JSON on stdout, then on stderr the manifest warnings the macOS runner
# prints and cargo's own error.
STUB_DIAGNOSTIC = """error: unused import: `PathBuf`
 --> crates/grund-core/src/api/tests_snapshot_path_text.rs:8:23
  |
8 | use std::path::{Path, PathBuf};
  |                       ^^^^^^^
  |
  = note: `-D unused-imports` implied by `-D warnings`
  = help: to override `-D warnings` add `#[allow(unused_imports)]`"""
STUB_CARGO_ERROR = 'error: could not compile `grund-core` (lib test) due to 1 previous error'
STUB_CARGO_STDERR = f"""warning: `package.homepage` is redundant with `package.repository`
  --> crates/grund-core/Cargo.toml:10:1
   |
 9 | repository.workspace = true
   | ----------
10 | homepage.workspace = true
   | ^^^^^^^^
   |
   = note: `cargo::redundant_homepage` is set to `warn` by default
help: consider removing `package.homepage`
{STUB_CARGO_ERROR}
"""


def compiler_message(root, rendered):
    """A `compiler-message` line as cargo writes it, children and spans left out."""
    crate = root / 'crates/grund-core'
    return {'reason': 'compiler-message', 'package_id': f'path+file://{crate}#0.0.0',
            'manifest_path': str(crate / 'Cargo.toml'),
            'target': {'kind': ['lib'], 'crate_types': ['lib'], 'name': 'grund_core',
                       'src_path': str(crate / 'src/lib.rs'), 'edition': '2024', 'doc': True,
                       'doctest': True, 'test': True},
            'message': {'rendered': rendered, '$message_type': 'diagnostic', 'children': [],
                        'code': {'code': 'unused_imports', 'explanation': None},
                        'level': 'error', 'message': 'unused import: `PathBuf`', 'spans': []}}


def executable(path, body):
    """A POSIX shell script at `path` running `body`."""
    path.write_text(f'#!/bin/sh\n{body}\n')
    path.chmod(0o755)
    return path


def stub_suite(root, output):
    """Cargo's JSON line for a stub core lib-test binary that prints `output` and fails."""
    (root / 'suite.out').write_text(output)
    suite = executable(root / 'suite', f"cat {shlex.quote(str(root / 'suite.out'))}\nexit 101")
    return json.dumps({'reason': 'compiler-artifact', 'executable': str(suite),
                       'target': {'name': 'grund_core', 'kind': ['lib']},
                       'profile': {'test': True}}) + '\n'


def raised_by_stub_cargo(case, root, stdout, stderr='', status=0):
    """The oracle's RuntimeError text with a stub `cargo` first on PATH.

    The stub writes `stdout` and `stderr` as cargo would and exits with `status`.
    """
    (root / 'bin').mkdir()
    (root / 'crates/grund-core').mkdir(parents=True)
    for stream, text in [('stdout', stdout), ('stderr', stderr)]:
        (root / f'cargo.{stream}').write_text(text)
    executable(root / 'bin/cargo', f"cat {shlex.quote(str(root / 'cargo.stdout'))}\n"
               f"cat {shlex.quote(str(root / 'cargo.stderr'))} >&2\nexit {status}")
    path = f"{root / 'bin'}{os.pathsep}{os.environ.get('PATH', '')}"
    with mock.patch.dict(os.environ, {'PATH': path}), case.assertRaises(RuntimeError) as raised:
        ORACLE.run(root, root / 'logs')
    return str(raised.exception)


if __name__ == '__main__':
    unittest.main()
