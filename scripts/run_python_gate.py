"""Portable inputs for the existing Python gate (§AR-ci.3.4).

Set GRUND_BUILT/GRUND_RELEASED to reuse local artifacts for
§FS-cochange-recipe.examples, and GRUND_BINDINGS_ORACLE to reuse a same-source
binding oracle for §FS-distribution.3.0.3; otherwise build this checkout, install
the exact released crate, and build the oracle from HEAD, without silently
skipping tests. GRUND_NODE_TARGET names the Node acceptance's shared build target
(§AR-ci.3.4.1). The run ends with its slowest tests (§AR-ci.3.4.2).
"""
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = Path(__file__).resolve().parent
SLOWEST = 10
# The child keeps `python -m unittest`'s sys.path, with this directory appended to reach discover().
DISCOVER = ('import sys; sys.path.append(sys.argv.pop(1)); import run_python_gate; '
            'sys.exit(run_python_gate.discover(sys.argv[1:]))')


def inputs():
    """Fresh-checkout built, released, oracle and Node target inputs (§AR-ci.3.4)."""
    environment = dict(os.environ)
    suffix = '.exe' if os.name == 'nt' else ''
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    # Both builds share this checkout's target, so the oracle reuses grund-core's artifacts.
    target = Path(environment.get('CARGO_TARGET_DIR', str(ROOT / 'target'))).expanduser().resolve()
    if not environment.get('GRUND_BUILT'):
        subprocess.run(['cargo', 'build', '-p', 'grund', '--locked', '--target-dir', str(target)],
                       cwd=ROOT, check=True)
        environment['GRUND_BUILT'] = str(target / ('debug/grund' + suffix))
    if not environment.get('GRUND_RELEASED'):
        release = scratch / 'grund-cochange-release-0.16.1'
        executable = release / ('bin/grund' + suffix)
        if not executable.is_file():
            subprocess.run(['cargo', 'install', 'grund', '--version', '0.16.1', '--locked',
                            '--root', str(release), '--target-dir', str(scratch / 'grund-release-build')],
                           cwd=ROOT, check=True)
        environment['GRUND_RELEASED'] = str(executable)
    if not environment.get('GRUND_BINDINGS_ORACLE'):
        # §AR-ci.3.4: built from HEAD each run, as the workflow builds it; cargo is incremental.
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True,
                              text=True, check=True).stdout.strip()
        subprocess.run(['cargo', 'build', '--locked', '-p', 'grund-core', '--example',
                        'grund-binding-oracle', '--target-dir', str(target)], cwd=ROOT,
                       env=dict(environment, GRUND_BINDINGS_SOURCE_SHA=head), check=True)
        oracle = target / ('debug/examples/grund-binding-oracle' + suffix)
        if not oracle.is_file():
            raise FileNotFoundError(oracle)
        environment['GRUND_BINDINGS_ORACLE'] = str(oracle)
    if not environment.get('GRUND_NODE_TARGET'):
        # §AR-ci.3.4.1: outside the checkout and kept between runs; CI's cargo cache restores it.
        environment['GRUND_NODE_TARGET'] = str(scratch / 'grund-node-target')
    Path(environment['GRUND_NODE_TARGET']).expanduser().mkdir(parents=True, exist_ok=True)
    for key in ('GRUND_BUILT', 'GRUND_RELEASED', 'GRUND_BINDINGS_ORACLE', 'GRUND_NODE_TARGET'):
        environment[key] = str(Path(environment[key]).expanduser().resolve())
    version = subprocess.run([environment['GRUND_RELEASED'], '--version'],
                             capture_output=True, text=True, check=True).stdout
    if version != 'grund 0.16.1\n':
        raise ValueError('GRUND_RELEASED must be released Grund 0.16.1')
    return environment


class TimedResult(unittest.TextTestResult):
    """Records each test's wall-clock, fixtures included, for the slowest table (§AR-ci.3.4.2)."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.timings = []
        self.mark = time.perf_counter()

    def startTestRun(self):
        super().startTestRun()
        self.mark = time.perf_counter()

    def startTest(self, test):
        # A test is charged from the previous test's stop, so setUpModule/setUpClass count.
        self.started = self.mark
        super().startTest(test)

    def stopTest(self, test):
        super().stopTest(test)
        self.mark = time.perf_counter()
        self.timings.append([self.mark - self.started, test.id()])

    def stopTestRun(self):
        # The last class's and module's teardown is charged to the last test.
        if self.timings:
            self.timings[-1][0] += time.perf_counter() - self.mark
        super().stopTestRun()


def discover(argv):
    """Run `unittest` with `argv`, then print the slowest tests (§AR-ci.3.4.2)."""
    runner = unittest.TextTestRunner(resultclass=TimedResult)
    result = unittest.main(module=None, argv=['python -m unittest', *argv],
                           testRunner=runner, exit=False).result
    stream = runner.stream
    stream.writeln('Slowest tests')
    for seconds, test in sorted(result.timings, reverse=True)[:SLOWEST]:
        stream.writeln(f'{seconds:8.2f}s {test}')
    stream.flush()
    if result.testsRun == 0 and not result.skipped:
        return 5  # §AR-ci.3.4.2: `python -m unittest` on 3.12+ exits 5 when it finds no tests.
    return 0 if result.wasSuccessful() else 1


def run_tests(start, environment):
    """Discover and time the tests under `start`; return the exit status (§AR-ci.3.4.2)."""
    return subprocess.run([sys.executable, '-c', DISCOVER, str(SCRIPTS), 'discover', '-s',
                           str(start), '-p', 'test_*.py'], cwd=ROOT, env=environment).returncode


if __name__ == '__main__':
    sys.exit(run_tests('tests/integration', inputs()))
