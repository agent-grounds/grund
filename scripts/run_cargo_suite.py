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
for the first `threads`, then at each completion. The estimate is close when the thread
count is the one libtest uses; a test whose completion does not fit the order is charged
the gap since the binary's previous event.

A child process a test runs writes to the same stream as libtest, so its output can run
into libtest's `test <name> ... ok` line on either side, or split it in two on one
thread. The event is looked for anywhere in the line, and a binary's counts are taken
from its own `test result:` line, so an event lost in such a line still counts and a
green suite's last check-in reads `<total>/<total>`.

§AR-ci.13.7: `--shard <k>/<n>`, before cargo's arguments, runs the `k`-th of `n` shards.
The build and the listing are the unsharded ones, so every shard compiles with the same
packages and features; `cargo_shards` deals the listed binaries and is cargo's runner for
the one run, so only the shard's binaries start (see that module).
"""
import collections
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cargo_shards  # noqa: E402
import suite_progress  # noqa: E402

# Every CSI sequence, private ones such as a child's `\x1b[?1049l` included, an OSC, and
# the two-byte escapes.
ANSI = re.compile(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[@-Z\\-_])')
CARGO_STATUS = re.compile(r'^\s*(Compiling|Checking|Finished|Blocking|Downloading|Downloaded|'
                          r'Updating|Locking|Adding|Fresh|Executable)\b')
# Cargo's own line before each binary, indented as its status lines are, so that a test's
# child process printing `Running …` is not taken for one.
BINARY = re.compile(r'^\s+(Running \S.* \(.+\)|Doc-tests \S+)$')
LISTED = re.compile(r'^(.*): (test|benchmark)$')
RUNNING = re.compile(r'^running \d+ tests?$')
STARTED = re.compile(r'test (.+?)(?: - should panic)? \.\.\. ')
VERDICT = re.compile(r'(ok|FAILED|ignored)\b(?:, .*)?')
OPEN_ENDED = re.compile(r'(ok|FAILED)$')
SLOW = re.compile(r'^test .+ has been running for over \d+ seconds$')
RESULT = re.compile(r'^test result: ')
TALLY = re.compile(r'(\d+) (passed|failed|ignored)\b')
CRASHED = re.compile(r'^error: test failed, to rerun pass ')
# §AR-ci.requirements.5.1: what a gate's time went on that no test's own seconds show.
GATE_TIME = re.compile(r'gate time: .*')


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
    """§AR-ci.requirements.5.4: build quietly; a failure prints in full, a warning stays.

    A green build ends with one `gate time: build` line, its compile as a phase of its own.
    """
    started = time.perf_counter()
    result = subprocess.run(['cargo', 'test', '--no-run', '--quiet', *cargo],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    text = result.stdout.decode('utf-8', errors='replace')
    if result.returncode != 0:
        sys.stdout.write(text)
    else:
        for line in text.splitlines():
            if plain(line).strip() and not CARGO_STATUS.match(plain(line)):
                print(line)
        print(f'gate time: build {time.perf_counter() - started:.1f}s')
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
        self.names = set(names)
        self.pending = collections.deque(names)
        self.started = {}
        self.last = now
        self.failed = False
        self.finished = False
        self.open = None
        self.counted = collections.Counter()
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

    def parse(self, text):
        """`(name, verdict, what else the line holds)`; name None when no test ends here.

        A child process's output may run into libtest's line on either side, so the event is
        looked for wherever a `test ` starts, a listed name first. On one thread libtest
        writes `test <name> ... ` before the test runs, and its verdict ends a later line.
        """
        candidates = [STARTED.match(text, at.start()) for at in re.finditer('test ', text)]
        candidates = [match for match in candidates if match]
        known = [match for match in candidates if match.group(1) in self.names]
        match = (known or candidates[-1:] or [None])[0]
        if match is not None:
            rest = text[match.end():]
            verdict = VERDICT.match(rest)
            self.open = None
            if verdict:
                return match.group(1), verdict.group(1), text[:match.start()] + rest[verdict.end():]
            self.open = match.group(1)
            return None, None, text[:match.start()] + rest
        ended = OPEN_ENDED.search(text) if self.open else None
        if ended:
            name, self.open = self.open, None
            return name, ended.group(1), text[:ended.start()]
        return None, None, text

    def unreported(self, result):
        """What libtest's `test result:` line counts that no event of this binary did."""
        counts = dict((outcome, int(number)) for number, outcome in TALLY.findall(result))
        for outcome in ('passed', 'failed', 'ignored'):
            for _ in range(max(0, counts.get(outcome, 0) - self.counted[outcome])):
                yield outcome


def run(cargo, libtest, binaries, progress, config=()):
    """Run the suite and print only its failures, its check-ins and what a red binary wrote.

    `config` is cargo's `--config` for a shard's runner; a binary outside `binaries` then
    runs nothing and prints nothing but cargo's `Running` line, which is dropped.
    """
    slots = threads(libtest)
    command = ['cargo', 'test', *config, *cargo] + (['--', *libtest] if libtest else [])
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    binary, failures, held = None, False, []

    def stray(lines):
        # §AR-ci.requirements.5.1: what a red binary's tests or their children wrote outside
        # libtest's capture is part of its failures' output, though not any one test's.
        if lines:
            print(f"output outside libtest's capture from {binary.key}:", flush=True)
            for line in lines:
                print(line, flush=True)

    def close():
        # A binary that never reported a result, `harness = false` or crashed, is judged by
        # cargo: a crash is counted where cargo names it, a green one's lines are dropped.
        if binary is None:
            return
        if binary.failed:
            stray(binary.stray)
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
        if binary is not None and CRASHED.match(text) and not binary.failed:
            # The binary exited non-zero without a failing test: it crashed, or a
            # `harness = false` target failed. An error no test owns.
            progress.error()
            binary.failed = binary.finished = True
            stray(binary.stray)
            binary.stray = []
            print(line, flush=True)
            continue
        if binary is None or binary.finished:
            if text.strip() and not CARGO_STATUS.match(text):
                print(line, flush=True)
            continue
        if RESULT.match(text):
            binary.finished = True
            # A lost event still counts: libtest's own tally is the binary's truth.
            for outcome in binary.unreported(text):
                progress.finished(None, None, outcome)
            binary.failed |= ' FAILED.' in text
            if binary.failed:
                print(line, flush=True)
            continue
        if failures:
            print(line, flush=True)
            continue
        if RUNNING.match(text) or SLOW.match(text):
            continue
        if text == 'failures:':
            # libtest's own report: each failure's `---- <name> stdout ----` block, in full.
            failures = True
            binary.failed = True
            print(f'{binary.key}:', flush=True)
            print(line, flush=True)
            continue
        name, verdict, rest = binary.parse(text)
        if name is not None:
            outcome = {'ok': 'passed', 'FAILED': 'failed', 'ignored': 'ignored'}[verdict]
            binary.failed |= outcome == 'failed'
            binary.counted[outcome] += 1
            progress.finished(name, binary.seconds(name, now), outcome)
        timed = GATE_TIME.search(plain(rest))
        if timed:
            print(timed.group(0), flush=True)
        elif rest.strip():
            binary.stray.append(line if rest == text else rest)
    close()
    status = process.wait()
    if status != 0:
        for line in held:
            print(line, flush=True)
        if progress.failed == 0 and progress.errors == 0:
            progress.error()
    return status


def main(argv):
    started = time.perf_counter()
    shard, argv = cargo_shards.parse(argv)
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(errors='replace')
    cargo, libtest = split(argv)
    status = build(cargo)
    if status != 0:
        return status
    status, binaries = listing(cargo, libtest)
    if status != 0:
        return status
    if shard is None:
        return suite(cargo, libtest, binaries, started)
    index, count = shard
    keys = cargo_shards.deal(binaries, count)[index - 1]
    if not keys:
        print(f'shard {index}/{count} selects no target of {len(binaries)}', file=sys.stderr)
        return cargo_shards.NO_TARGET
    with cargo_shards.runner(keys) as config:
        return suite(cargo, libtest, {key: binaries[key] for key in keys}, started, config)


def suite(cargo, libtest, binaries, started, config=()):
    """Run `binaries` and print one summary counting their tests alone."""
    total = sum(len(names) for names in binaries.values())
    progress = suite_progress.Progress(total, sys.stdout, start=started)
    status = run(cargo, libtest, binaries, progress, config)
    progress.summary()
    return status


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
