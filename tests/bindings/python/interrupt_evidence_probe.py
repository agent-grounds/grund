"""Run the live interrupt acceptance under controls (§AR-bindings.6.2.1).

This is regression instrumentation, not the acceptance signal protocol. The held
control prevents child helpers from running at all, modelling descheduling before
delivery; it must never turn missing delivery into evidence of a library failure.
"""

import os
from pathlib import Path
import signal
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import support
import test_boundary


# §AR-bindings.6.2.1: hold every child helper, without matching the old sender source.
# The positive control releases the acknowledged gate and runs the actual helper;
# the negative control never releases it. Inject only into the acceptance child.
HOLD_HELPERS = r'''
import json, os, select, subprocess, sys
_real_popen = subprocess.Popen
def _record(event, **fields):
    with open(_evidence_path, 'a') as stream:
        stream.write(json.dumps(dict(event=event, **fields)) + '\n')
def _held_popen(args, *a, **kw):
    read_gate, write_gate = os.pipe()
    ready_read, ready_write = os.pipe()
    held = [sys.executable, '-c',
            'import os; os.write(' + str(ready_write) + ', b"r"); '
            'os.close(' + str(ready_write) + '); os.read(' + str(read_gate) + ', 1)']
    if _release_helper:
        held[2] += ('; os.close(' + str(read_gate) + '); import runpy, sys; '
                    'sys.argv = ' + repr(list(args[1:])) + '; '
                    'runpy.run_path(sys.argv[0], run_name="__main__")')
    kw['pass_fds'] = (*kw.get('pass_fds', ()), read_gate, ready_write)
    try:
        sender = _real_popen(held, *a, **kw)
    except BaseException:
        os.close(write_gate)
        os.close(ready_read)
        raise
    finally:
        os.close(read_gate)
        os.close(ready_write)
    try:
        ready = select.select([ready_read], [], [], 5)[0]
        if not ready or os.read(ready_read, 1) != b'r':
            raise RuntimeError('held-helper instrumentation never became ready')
    except BaseException:
        sender.kill()
        sender.wait(timeout=5)
        os.close(write_gate)
        raise
    finally:
        os.close(ready_read)
    _record('helper-held', pid=sender.pid, gate_released=False)
    if _release_helper:
        os.write(write_gate, b'r')
        os.close(write_gate)
        write_gate = None
        _record('helper-released', pid=sender.pid, gate_released=True)
    original_wait = sender.wait
    def wait(timeout=None):
        nonlocal write_gate
        error = sys.exc_info()[1]
        _record('helper-wait', exception_before_cleanup=repr(error),
                helper_alive=sender.poll() is None, gate_released=_release_helper)
        try:
            if error is not None and sender.poll() is None:
                sender.kill()
            return original_wait(timeout=min(timeout, 5) if timeout is not None else 5)
        finally:
            if sender.poll() is None:
                sender.kill()
                original_wait(timeout=5)
            if write_gate is not None:
                os.close(write_gate)
                write_gate = None
    sender.wait = wait
    return sender
subprocess.Popen = _held_popen
'''


def run(mode, evidence):
    # The outer test owns this directory, so it can remove live-test fixtures
    # even if its deadline has to kill this probe before finally blocks run.
    support.SCRATCH = evidence.parent
    previous_handler = signal.getsignal(signal.SIGINT)
    previous_mask = None
    original_popen = subprocess.Popen

    def instrument(args, *a, **kw):
        # The live acceptance still uses a -c child, now launched via Popen so
        # its process group can be killed on timeout. Keep the readiness/held
        # acknowledgment mandatory when adapting that transport.
        if len(args) > 2 and args[1] == "-c":
            args = list(args)
            args[2] = (f"_evidence_path = {str(evidence)!r}\n"
                       f"_release_helper = {mode == 'scheduled'!r}\n"
                       + HOLD_HELPERS + args[2])
            if mode == "scheduled":
                kw["env"] = dict(os.environ, GRUND_INTERRUPT_EVIDENCE=str(evidence))
        return original_popen(args, *a, **kw)

    try:
        if mode == "ignored":
            signal.signal(signal.SIGINT, signal.SIG_IGN)
        elif mode == "blocked":
            signal.signal(signal.SIGINT, signal.default_int_handler)
            previous_mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})
        elif mode not in ("held", "scheduled", "ordinary"):
            raise ValueError(mode)
        suite = unittest.TestSuite([test_boundary.BoundaryTests(
            "test_pending_interrupt_is_delivered_when_operation_returns")])
        with patch.object(subprocess, "Popen",
                          instrument if mode in ("held", "scheduled") else original_popen):
            result = unittest.TextTestRunner(verbosity=2).run(suite)
        return int(not result.wasSuccessful())
    finally:
        if previous_mask is not None:
            signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)
        signal.signal(signal.SIGINT, previous_handler)


if __name__ == "__main__":
    sys.exit(run(sys.argv[1], Path(sys.argv[2])))
