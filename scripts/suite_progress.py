"""What a suite step prints while it runs and when it ends (§AR-ci.requirements.5.1).

Both suite wrappers, `scripts/run_cargo_suite.py` and `scripts/run_python_gate.py`,
report through one `Progress`, so the check-in and the summary are spelled once.
"""
import math
import time

CHECK_INS = 20  # one each 5% of the suite
SLOWEST = 3


class Progress:
    """Counts finished tests and prints a check-in each 5% and one summary line."""

    def __init__(self, total, stream, start=None):
        self.total, self.stream = total, stream
        self.start = time.perf_counter() if start is None else start
        self.done = self.passed = self.failed = self.ignored = 0
        self.timings = []
        self.reported = 0
        # §AR-ci.requirements.5.1: the done counts at which a check-in is due, at most twenty.
        self.due = sorted({math.ceil(k * total / CHECK_INS) for k in range(1, CHECK_INS + 1)} - {0})

    def finished(self, name, seconds, outcome):
        """One test ended `passed`, `failed` or `ignored` after `seconds`."""
        self.done += 1
        setattr(self, outcome, getattr(self, outcome) + 1)
        if outcome != 'ignored':
            self.timings.append((seconds, name))
        if self.due and self.done >= self.due[0]:
            while self.due and self.done >= self.due[0]:
                self.due.pop(0)
            self.check_in()

    def error(self):
        """A failure no test owns: a class or module fixture's, or a binary that crashed."""
        self.failed += 1

    def check_in(self):
        """`<done>/<total> tests, <failed> failed, <elapsed>s elapsed, slowest: …`."""
        self.reported = self.done
        slowest = sorted(self.timings, reverse=True)[:SLOWEST]
        names = ', '.join(f'{name} {seconds:.2f}s' for seconds, name in slowest) or '-'
        self.write(f'{self.done}/{self.total} tests, {self.failed} failed, '
                   f'{self.elapsed():.1f}s elapsed, slowest: {names}')

    def summary(self):
        """The last line: `<total> tests: <passed> passed, <failed> failed, <ignored> ignored, <s>s`."""
        if self.done and self.reported != self.done:
            self.check_in()  # a suite that stopped early still reports where it stopped
        self.write(f'{self.total} tests: {self.passed} passed, {self.failed} failed, '
                   f'{self.ignored} ignored, {self.elapsed():.1f}s')

    def elapsed(self):
        return time.perf_counter() - self.start

    def write(self, line):
        self.stream.write(line + '\n')
        self.stream.flush()
