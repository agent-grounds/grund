"""Run `cargo test` and print its failures, a check-in each 5% and one summary.

§AR-ci.requirements.5.2: `python scripts/run_cargo_suite.py <cargo test args>` is how
`ci.yml` and the `cargo-test` hook run a cargo suite. It needs the standard library and
the toolchain, nothing else, and drives `cargo test` itself, so every test still runs
in its own binary with cargo's environment and threads (§AR-ci.3.3).

Three cargo runs, all with the same arguments: `cargo test --no-run --quiet` builds the
tests and prints only what the compiler says; `cargo test -- --list` counts the suite up
front, binary by binary; `cargo test` runs it, and its libtest output is read line by
line and reduced to what §AR-ci.requirements.5.1 asks. The exit status is cargo's.

A test's time is estimated, because stable libtest reports when a test ends and not when
it starts. libtest starts tests in the order `--list` names them, as many at once as it
has threads, and starts the next one when a slot frees, so a test is taken to start when
the test before it in that order was started and a slot freed: at the binary's start
for the first `threads`, then at each completion. The estimate is exact on one thread
and close when the thread count is the one libtest uses; a test whose completion does
not fit the order is charged the gap since the binary's previous event.
"""
import collections
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import suite_progress  # noqa: E402

ANSI = re.compile(r'\x1b\[[0-9;]*[A-Za-z]')
CARGO_STATUS = re.compile(r'^\s*(Compiling|Checking|Finished|Blocking|Downloading|Downloaded|'
                          r'Updating|Locking|Adding|Fresh|Executable)\b')
# Cargo's own line before each binary, indented as its status lines are, so that a test's
# child process printing `Running …` is not taken for one.
BINARY = re.compile(r'^\s+(Running \S.* \(.+\)|Doc-tests \S+)$')
LISTED = re.compile(r'^(.*): (test|benchmark)$')
RUNNING = re.compile(r'^running \d+ tests?$')
EVENT = re.compile(r'^test (.+?)(?: - should panic)? \.\.\. (ok|FAILED|ignored)\b')
SLOW = re.compile(r'^test .+ has been running for over \d+ seconds$')
RESULT = re.compile(r'^test result: ')


def split(argv):
    """`cargo test`'s own arguments and the libtest arguments after `--`."""
    if '--' in argv:
        at = argv.index('--')
        return argv[:at], argv[at + 1:]
    return argv, []


def threads(libtest):
    """How many tests libtest runs at once: `--test-threads`, `RUST_TEST_THREADS`, the CPUs."""
    for index, arg in enumerate(libtest):
        if arg.startswith('--test-threads='):
            return max(1, int(arg.split('=', 1)[1]))
        if arg == '--test-threads' and index + 1 < len(libtest):
            return max(1, int(libtest[index + 1]))
    if os.environ.get('RUST_TEST_THREADS', '').isdigit():
        return max(1, int(os.environ['RUST_TEST_THREADS']))
    if hasattr(os, 'sched_getaffinity'):
        return max(1, len(os.sched_getaffinity(0)))
    return os.cpu_count() or 1


def plain(line):
    return ANSI.sub('', line).rstrip()


def build(cargo):
    """§AR-ci.requirements.5.4: build quietly; a failure prints in full, a warning stays."""
    result = subprocess.run(['cargo', 'test', '--no-run', '--quiet', *cargo],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    text = result.stdout.decode('utf-8', errors='replace')
    if result.returncode != 0:
        sys.stdout.write(text)
    else:
        for line in text.splitlines():
            if plain(line).strip() and not CARGO_STATUS.match(plain(line)):
                print(line)
    sys.stdout.flush()
    return result.returncode


def listing(cargo, libtest):
    """The tests of each binary, in libtest's order, keyed by cargo's `Running` line."""
    result = subprocess.run(['cargo', 'test', *cargo, '--', '--list', *libtest],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    text = result.stdout.decode('utf-8', errors='replace')
    if result.returncode != 0:
        sys.stdout.write(text)
        sys.stdout.flush()
        return result.returncode, None
    binaries, current = {}, None
    for line in map(plain, text.splitlines()):
        if BINARY.match(line):
            current = binaries.setdefault(line.strip(), [])
        elif current is not None and LISTED.match(line):
            current.append(LISTED.match(line).group(1))
    return 0, binaries


class Binary:
    """One test binary's run: its estimated start times, its stray lines, its failures."""

    def __init__(self, key, names, slots, now):
        self.key = key
        self.pending = collections.deque(names)
        self.started = {}
        self.last = now
        self.failed = False
        self.finished = False
        self.stray = []
        for _ in range(slots):
            self.start(now)

    def start(self, now):
        if self.pending:
            self.started[self.pending.popleft()] = now

    def seconds(self, name, now):
        """How long `name` ran, by libtest's order of starts (see the module comment)."""
        begun = self.started.pop(name, None)
        if begun is None:
            if name in self.pending:
                self.pending.remove(name)
            begun = self.last
        else:
            self.start(now)
        self.last = now
        return now - begun


def run(cargo, libtest, binaries, progress):
    """Run the suite and print only its failures, its check-ins and what a red binary wrote."""
    slots = threads(libtest)
    command = ['cargo', 'test', *cargo] + (['--', *libtest] if libtest else [])
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    binary, failures, held = None, False, []

    def close():
        # §AR-ci.requirements.5.1: what a red binary's tests or their children wrote
        # outside libtest's capture is part of its failures' output. A binary that never
        # reported a result, crashed or `harness = false`, is judged by cargo's status.
        if binary is None:
            return
        if binary.failed:
            for line in binary.stray:
                print(line, flush=True)
        elif not binary.finished:
            held.extend(binary.stray)

    for raw in process.stdout:
        line = raw.decode('utf-8', errors='replace').rstrip('\r\n')
        text = plain(line)
        now = time.perf_counter()
        if BINARY.match(text):
            close()
            key = text.strip()
            binary, failures = Binary(key, binaries.get(key, []), slots, now), False
            continue
        if binary is None or binary.finished:
            if text.strip() and not CARGO_STATUS.match(text):
                print(line, flush=True)
            continue
        if RESULT.match(text):
            binary.finished = True
            if binary.failed:
                print(line, flush=True)
            continue
        if failures:
            print(line, flush=True)
            continue
        event = EVENT.match(text)
        if event:
            name, verdict = event.group(1), event.group(2)
            outcome = {'ok': 'passed', 'FAILED': 'failed', 'ignored': 'ignored'}[verdict]
            binary.failed |= outcome == 'failed'
            progress.finished(name, binary.seconds(name, now), outcome)
        elif text == 'failures:':
            # libtest's own report: each failure's `---- <name> stdout ----` block, in full.
            failures = True
            binary.failed = True
            print(f'{binary.key}:', flush=True)
            print(line, flush=True)
        elif text.strip() and not RUNNING.match(text) and not SLOW.match(text):
            binary.stray.append(line)
    close()
    status = process.wait()
    if status != 0:
        for line in held:
            print(line, flush=True)
        if progress.failed == 0:
            progress.error()
    return status


def main(argv):
    started = time.perf_counter()
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(errors='replace')
    cargo, libtest = split(argv)
    status = build(cargo)
    if status != 0:
        return status
    status, binaries = listing(cargo, libtest)
    if status != 0:
        return status
    total = sum(len(names) for names in binaries.values())
    progress = suite_progress.Progress(total, sys.stdout, start=started)
    status = run(cargo, libtest, binaries, progress)
    progress.summary()
    return status


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
