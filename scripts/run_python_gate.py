"""Portable inputs for the existing Python gate (§AR-ci.3.4).

Set GRUND_BUILT/GRUND_RELEASED to reuse local artifacts for
§FS-cochange-recipe.examples, and GRUND_BINDINGS_ORACLE to reuse a same-source
binding oracle for §FS-distribution.3.0.3; otherwise build this checkout, install
the exact released crate, and build the oracle from HEAD, without silently
skipping tests. GRUND_NODE_TARGET names the root of the Node acceptance's build targets
(§AR-ci.3.4.1). The run prints only its failures, a check-in each 5% and its slowest
tests before one summary (§AR-ci.requirements.5.1, §AR-ci.3.4.2); of what a green test
wrote it keeps only the lines that point lists, the Node acceptance's `node build:` lines
(§AR-ci.3.4.1.1) and the `gate time:` lines. Its own builds are quiet
(§AR-ci.requirements.5.4).
`--without-acceptance` runs all but the two binding acceptances, `--only-node-acceptance`
and `--only-python-acceptance` one of them alone; no argument runs everything (§AR-ci.3.4.3).
"""
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

import suite_progress

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = Path(__file__).resolve().parent
SLOWEST = 10
# The child keeps `python -m unittest`'s sys.path, with this directory appended to reach discover();
# the argument after it is the selection, empty for everything (§AR-ci.3.4.3).
DISCOVER = ('import sys; sys.path.append(sys.argv.pop(1)); selection = sys.argv.pop(1) or None; '
            'import run_python_gate; sys.exit(run_python_gate.discover(sys.argv[1:], selection))')
# §AR-ci.3.4.3: each `only-` selection and the one acceptance module it runs.
ACCEPTANCES = {'only-node-acceptance': 'test_node_binding_acceptance',
               'only-python-acceptance': 'test_python_binding_acceptance'}
SELECTIONS = ('without-acceptance', *ACCEPTANCES)
USAGE = ('usage: run_python_gate.py '
         '[--without-acceptance | --only-node-acceptance | --only-python-acceptance]')
# §AR-ci.requirements.5.1: what each line a green test keeps begins with, a build line of
# §AR-ci.3.4.1.1, whose marker the acceptance wrapper holds too, or a gate-time line.
KEPT_LINES = ('node build: ', 'gate time: ')


def parse_selection(argv):
    """The selection `argv` names, or exit 2 with the usage line (§AR-ci.3.4.3)."""
    if not argv:
        return None
    if len(argv) == 1 and argv[0].startswith('--') and argv[0][2:] in SELECTIONS:
        return argv[0][2:]
    print(USAGE, file=sys.stderr)
    sys.exit(2)


def inputs():
    """Fresh-checkout built, released, oracle and Node target inputs (§AR-ci.3.4)."""
    environment = dict(os.environ)
    suffix = '.exe' if os.name == 'nt' else ''
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    # Both builds share this checkout's target, so the oracle reuses grund-core's artifacts.
    target = Path(environment.get('CARGO_TARGET_DIR', str(ROOT / 'target'))).expanduser().resolve()
    if not environment.get('GRUND_BUILT'):
        subprocess.run(['cargo', 'build', '--quiet', '-p', 'grund', '--locked',
                        '--target-dir', str(target)], cwd=ROOT, check=True)
        environment['GRUND_BUILT'] = str(target / ('debug/grund' + suffix))
    if not environment.get('GRUND_RELEASED'):
        release = scratch / 'grund-cochange-release-0.16.1'
        executable = release / ('bin/grund' + suffix)
        if not executable.is_file():
            subprocess.run(['cargo', 'install', '--quiet', 'grund', '--version', '0.16.1', '--locked',
                            '--root', str(release), '--target-dir', str(scratch / 'grund-release-build')],
                           cwd=ROOT, check=True)
        environment['GRUND_RELEASED'] = str(executable)
    if not environment.get('GRUND_BINDINGS_ORACLE'):
        # §AR-ci.3.4: built from HEAD each run, as the workflow builds it; cargo is incremental.
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True,
                              text=True, check=True).stdout.strip()
        subprocess.run(['cargo', 'build', '--quiet', '--locked', '-p', 'grund-core', '--example',
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


class Capture:
    """Holds what the tests write, their child processes included, until it is asked for.

    §AR-ci.requirements.5.1: file descriptors 1 and 2 point at one temporary file for the
    whole run, so a child process that inherits them writes there too; on Windows the
    process's standard handles follow, because a child inherits those instead. What is
    printed of it, a failing test's whole output or the lines a green one keeps, is
    `TimedResult`'s call.
    """

    def __init__(self):
        self.file = tempfile.TemporaryFile()
        self.saved = None

    def start(self):
        self.flush()
        self.saved = (os.dup(1), os.dup(2), sys.stdout, sys.stderr)
        for fd in (1, 2):
            os.dup2(self.file.fileno(), fd)
        sink = io.TextIOWrapper(open(self.file.fileno(), 'wb', buffering=0, closefd=False),
                                encoding='utf-8', errors='replace', write_through=True)
        sys.stdout = sys.stderr = sink
        self.handles()

    def take(self):
        """What was written since the last take, and an empty file for what comes next."""
        self.flush()
        self.file.seek(0)
        data = self.file.read()
        self.file.seek(0)
        self.file.truncate()
        return data.decode('utf-8', errors='replace')

    def stop(self):
        self.flush()
        out, err, sys.stdout, sys.stderr = self.saved
        os.dup2(out, 1)
        os.dup2(err, 2)
        os.close(out)
        os.close(err)
        self.handles()

    @staticmethod
    def flush():
        for stream in (sys.stdout, sys.stderr):
            try:
                stream.flush()
            except (AttributeError, OSError, ValueError):
                pass

    @staticmethod
    def handles():
        if os.name != 'nt':
            return
        import ctypes
        import msvcrt
        for fd, handle in ((1, -11), (2, -12)):  # STD_OUTPUT_HANDLE, STD_ERROR_HANDLE
            ctypes.windll.kernel32.SetStdHandle(handle, msvcrt.get_osfhandle(fd))


class TimedResult(unittest.TestResult):
    """Times each test, fixtures included (§AR-ci.3.4.2), and prints what failed or is kept.

    §AR-ci.requirements.5.1: what a test and its fixtures wrote is printed under its
    name when it fails and dropped when it passes or is skipped, all but the lines that
    point keeps, the build lines of §AR-ci.3.4.1.1 and the `gate time:` lines, which are
    printed once each, as lines of their own. A green run's leftover, what a teardown
    wrote after the last test, keeps them the same way. A check-in each 5% and the
    summary go through `suite_progress`, as the cargo wrapper's do.
    """

    def __init__(self, stream, total):
        super().__init__()
        self.stream = stream
        self.progress = suite_progress.Progress(total, stream)
        self.capture = Capture()
        self.timings = []
        self.mark = time.perf_counter()
        self.counts = (0, 0, 0)
        self.skips = 0
        self.running = False

    def problems(self):
        return (len(self.failures), len(self.errors), len(self.unexpectedSuccesses))

    def startTestRun(self):
        super().startTestRun()
        self.mark = self.progress.start = time.perf_counter()
        self.capture.start()

    def startTest(self, test):
        # A test is charged from the previous test's stop, so setUpModule/setUpClass count,
        # and so is what they wrote.
        self.started = self.mark
        self.counts = self.problems()
        self.skips = len(self.skipped)
        self.running = True
        super().startTest(test)

    def stopTest(self, test):
        super().stopTest(test)
        self.running = False
        self.mark = time.perf_counter()
        seconds = self.mark - self.started
        self.timings.append([seconds, test.id()])
        output = self.capture.take()
        if self.problems() != self.counts:
            self.report(test.id(), output, self.counts)
            outcome = 'failed'
        else:
            # §AR-ci.requirements.5.1: a pass, a skip or an expected failure shows what it keeps.
            self.show_kept(output)
            outcome = 'ignored' if len(self.skipped) != self.skips else 'passed'
        self.progress.finished(test.id(), seconds, outcome)

    def addError(self, test, err):
        super().addError(test, err)
        if not self.running:
            # A class or module fixture failed outside any test (`_ErrorHolder`).
            before = (len(self.failures), len(self.errors) - 1, len(self.unexpectedSuccesses))
            self.report(str(test), self.capture.take(), before)
            self.progress.error()

    def report(self, name, output, before):
        failures, errors, unexpected = before
        self.stream.write(f'FAIL {name}\n')
        if output.strip():
            self.stream.write(output if output.endswith('\n') else output + '\n')
        for _, trace in self.failures[failures:] + self.errors[errors:]:
            self.stream.write(trace if trace.endswith('\n') else trace + '\n')
        if len(self.unexpectedSuccesses) > unexpected:
            self.stream.write('unexpected success\n')
        self.stream.flush()

    def show_kept(self, output):
        """Print the lines of a green test's `output` that §AR-ci.requirements.5.1 keeps:
        each line that begins with one of `KEPT_LINES`, ended with a plain newline, so a
        Windows child's CRLF comes out neither doubled nor with a stray carriage return."""
        kept = ''.join(line + '\n' for line in output.splitlines() if line.startswith(KEPT_LINES))
        if kept:
            self.stream.write(kept)
            self.stream.flush()

    def stopTestRun(self):
        # The last class's and module's teardown is charged to the last test.
        if self.timings:
            self.timings[-1][0] += time.perf_counter() - self.mark
        leftover = self.capture.take()
        self.capture.stop()
        if self.wasSuccessful():
            self.show_kept(leftover)
        elif leftover.strip():
            self.stream.write(leftover)
        super().stopTestRun()


def module_of(test):
    """The module a test comes from; a module that failed to import keeps its name."""
    if isinstance(test, unittest.loader._FailedTest):
        name = test._testMethodName
    else:
        name = type(test).__module__
    return name.rsplit('.', 1)[-1]


def select(suite, selection):
    """The tests of `suite` that `selection` runs, cut by module (§AR-ci.3.4.3)."""
    if selection is None:
        return suite
    if selection not in SELECTIONS:
        raise ValueError(f'unknown selection {selection!r}')
    chosen = unittest.TestSuite()
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            chosen.addTest(select(item, selection))
        elif (ACCEPTANCES.get(selection) == module_of(item) if selection in ACCEPTANCES
              else module_of(item) not in ACCEPTANCES.values()):
            chosen.addTest(item)
    return chosen


class SelectingLoader(unittest.TestLoader):
    """Discovers as `unittest` does, then keeps what the selection runs (§AR-ci.3.4.3)."""

    def __init__(self, selection):
        super().__init__()
        self.selection = selection

    def discover(self, *args, **kwargs):
        return select(super().discover(*args, **kwargs), self.selection)


class QuietRunner:
    """A `unittest` runner whose output is §AR-ci.requirements.5.1's, the lines it keeps
    of a green test included, and nothing else."""

    def __init__(self, stream):
        self.stream = stream

    def run(self, test):
        result = TimedResult(self.stream, test.countTestCases())
        result.startTestRun()
        try:
            test(result)
        finally:
            result.stopTestRun()
        return result


def discover(argv, selection=None):
    """Run `unittest` with `argv` on the selected tests, then print the slowest
    (§AR-ci.3.4.2) and the summary."""
    # The report goes to the descriptor the run started with, which Capture leaves alone.
    stream = open(os.dup(1), 'w', encoding='utf-8', errors='replace')
    runner = QuietRunner(stream)
    result = unittest.main(module=None, argv=['python -m unittest', *argv],
                           testLoader=SelectingLoader(selection),
                           testRunner=runner, exit=False).result
    stream.write('Slowest tests\n')
    for seconds, test in sorted(result.timings, reverse=True)[:SLOWEST]:
        stream.write(f'{seconds:8.2f}s {test}\n')
    result.progress.summary()
    stream.close()
    if result.testsRun == 0 and not result.skipped:
        return 5  # §AR-ci.3.4.2: `python -m unittest` on 3.12+ exits 5 when it finds no tests.
    return 0 if result.wasSuccessful() else 1


def run_tests(start, environment, selection=None):
    """Discover, select and time the tests under `start`; return the exit status (§AR-ci.3.4.2)."""
    return subprocess.run([sys.executable, '-c', DISCOVER, str(SCRIPTS), selection or '',
                           'discover', '-s', str(start), '-p', 'test_*.py'],
                          cwd=ROOT, env=environment).returncode


if __name__ == '__main__':
    # §AR-ci.3.4.3: the argument is judged before inputs() builds anything.
    chosen = parse_selection(sys.argv[1:])
    sys.exit(run_tests('tests/integration', inputs(), chosen))
