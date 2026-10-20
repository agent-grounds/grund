"""Private POSIX rendezvous for interrupt acceptance (§AR-bindings.6.2.1)."""

import errno
import os
from pathlib import Path
import select
import signal
import sys
import time

BOUND = 5


def event(name):
    """Publish protocol progress without buffering (§AR-bindings.6.2.1)."""
    print(name, flush=True)


def send(fifo, receiver, wakeup):
    """Hold a real Rust read until CPython acknowledges SIGINT (§AR-bindings.6.2.1).

    A nonblocking FIFO writer can open only when a reader exists. Only the
    engine reads this fixture, so this rendezvous establishes native execution.
    Keeping the writer open prevents EOF until the signal's wakeup byte proves
    that CPython's C handler has set the pending-interrupt flag. No Python signal
    callback, workload duration or helper-startup timing supplies that evidence.
    """
    writer = None
    stage = "native-reader"
    try:
        event("ready")
        deadline = time.monotonic() + BOUND
        while writer is None:
            try:
                writer = os.open(fifo, os.O_WRONLY | os.O_NONBLOCK)
            except OSError as error:
                if error.errno != errno.ENXIO:
                    raise
                if time.monotonic() >= deadline:
                    raise TimeoutError("engine never opened the FIFO")
                # Poll readiness, not elapsed time as a reason to send SIGINT.
                time.sleep(.005)
        event("native-reader")
        # Further engine reads see an ordinary empty lock; the open FIFO remains
        # the barrier for this read until its writer closes below.
        replacement = Path(fifo + ".empty")
        replacement.write_bytes(b"")
        os.replace(replacement, fifo)
        stage = "signal-acknowledgment"
        os.kill(receiver, signal.SIGINT)
        if not select.select([wakeup], [], [], BOUND)[0]:
            raise TimeoutError("CPython's native signal handler never acknowledged SIGINT")
        if os.read(wakeup, 1) != bytes([signal.SIGINT]):
            raise RuntimeError("unexpected signal acknowledgment")
        event("signal-pending")
        # This event is sent before EOF, while the engine still cannot return.
        event("native-release")
    except Exception as error:
        event(f"error {stage}: {error}")
        return 1
    finally:
        if writer is not None:
            os.close(writer)
        os.close(wakeup)
    return 0


if __name__ == "__main__":
    sys.exit(send(sys.argv[1], int(sys.argv[2]), int(sys.argv[3])))
