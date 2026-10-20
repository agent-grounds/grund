"""Isolated, bounded pending-interrupt evidence (§AR-bindings.6.2.1).

The FIFO is a private section-lock fixture read by the real scanner, rather than
a mock of the operation or an extension API. CPython's set_wakeup_fd supplies a
portable native-handler acknowledgment while that engine read is still held.
"""

import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import time

from interrupt_sender import BOUND


def unavailable(stage, detail):
    """Separate missing evidence from binding delivery (§AR-bindings.6.2.1)."""
    return AssertionError(f"interrupt evidence unavailable ({stage}): {detail}")


def receive(stream, stage):
    """Read one small protocol line within a deadline (§AR-bindings.6.2.1)."""
    deadline = time.monotonic() + BOUND
    data = bytearray()
    while len(data) < 1024:
        remaining = deadline - time.monotonic()
        if remaining <= 0 or not select.select([stream], [], [], remaining)[0]:
            raise unavailable(stage, "helper deadline expired")
        byte = os.read(stream.fileno(), 1)
        if not byte:
            raise unavailable(stage, f"helper closed its event stream: {data!r}")
        if byte == b"\n":
            return data.decode()
        data.extend(byte)
    raise unavailable(stage, "helper protocol line too long")


def record(events):
    """Expose validated ordering to private regression controls (§AR-bindings.6.2.1)."""
    evidence = os.environ.get("GRUND_INTERRUPT_EVIDENCE")
    if evidence:
        with open(evidence, "a") as stream:
            for event in events:
                stream.write(json.dumps({"event": event}) + "\n")


def acceptance(root, independent):
    """Prove pending delivery at this call's return (§FS-distribution.3.3.4)."""
    import grund

    read_ack = write_ack = previous_wakeup = None
    sender = None
    events = []
    interrupted = False
    try:
        stage = "signal-setup"
        try:
            # This is an expendable isolated child; no runner state changes.
            signal.signal(signal.SIGINT, signal.default_int_handler)
            signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGINT})
            stage = "FIFO-fixture"
            fifo = Path(root) / ".grund/sections/FS-001-alpha.lock"
            fifo.parent.mkdir(parents=True, exist_ok=True)
            os.mkfifo(fifo)
            stage = "signal-acknowledgment-pipe"
            read_ack, write_ack = os.pipe()
            os.set_blocking(write_ack, False)
            previous_wakeup = signal.set_wakeup_fd(write_ack)
            stage = "helper-launch"
            sender = subprocess.Popen(
                [sys.executable, str(Path(__file__).with_name("interrupt_sender.py")),
                 str(fifo), str(os.getpid()), str(read_ack)],
                pass_fds=(read_ack,), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        except (OSError, ValueError) as error:
            raise unavailable(stage, str(error)) from error
        os.close(read_ack)
        read_ack = None
        ready = receive(sender.stdout, "helper-ready")
        if ready != "ready":
            raise unavailable("helper-ready", ready)
        events.append(ready)
        # The helper sends only after the engine opens the FIFO, and retains
        # its writer until CPython's C handler acknowledges the pending flag.
        # Only this call, with no setup or cleanup in its try, can satisfy it.
        try:
            grund.check(root)
        except KeyboardInterrupt:
            interrupted = True
        for expected in ("native-reader", "signal-pending", "native-release"):
            observed = receive(sender.stdout, expected)
            if observed != expected:
                raise unavailable(expected, observed)
            events.append(observed)
        if sender.wait(timeout=BOUND) != 0:
            raise unavailable("helper-exit", sender.stderr.read().decode())
        if not interrupted:
            raise AssertionError("pending interrupt was swallowed")
        events.append("native-return-interrupt")
        # §FS-distribution.3.3.3: successful independent work after the interrupt.
        assert grund.check(independent).report is not None
        events.append("independent-call")
        record(events)
    except subprocess.TimeoutExpired as error:
        raise unavailable("helper-exit", str(error)) from error
    finally:
        if sender is not None:
            if sender.poll() is None:
                sender.kill()
            sender.wait(timeout=BOUND)
            sender.stdout.close()
            sender.stderr.close()
        if previous_wakeup is not None:
            signal.set_wakeup_fd(previous_wakeup)
        if read_ack is not None:
            os.close(read_ack)
        if write_ack is not None:
            os.close(write_ack)


def run_child(root, independent):
    """Bound and reap the isolated child and its helpers (§AR-bindings.6.2.1)."""
    code = (f"import sys; sys.path.insert(0, {str(Path(__file__).parent)!r}); "
            "from interrupt_acceptance import acceptance; acceptance(*sys.argv[1:])")
    child = subprocess.Popen([sys.executable, "-c", code, str(root), str(independent)],
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             text=True, start_new_session=True)
    try:
        try:
            out, err = child.communicate(timeout=4 * BOUND)
        except subprocess.TimeoutExpired as error:
            raise unavailable("acceptance-child", "native call or helper deadline expired") from error
        return subprocess.CompletedProcess(child.args, child.returncode, out, err)
    finally:
        # Kill the whole group even after a fast failure orphaned a helper.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.communicate(timeout=BOUND)
