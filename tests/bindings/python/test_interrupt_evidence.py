"""Regressions for live POSIX interrupt evidence (§AR-bindings.6.2.1).

The library promise remains §FS-distribution.3.3.4; independent calls remain
§FS-distribution.3.3.3. These tests exercise the existing acceptance harness,
including its native call and assertions, rather than copying its signal sender.
"""

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from support import REPO, temporary

PROBE = Path(__file__).with_name("interrupt_evidence_probe.py")


@unittest.skipUnless(os.name == "posix", "POSIX signal evidence controls")
class InterruptEvidenceTests(unittest.TestCase):
    def probe(self, mode, evidence):
        # A private process group confines signals and makes even a hung live
        # harness and all of its descendants bounded and reapable by this test.
        child = subprocess.Popen([sys.executable, str(PROBE), mode, str(evidence)],
                                 cwd=REPO, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 text=True, start_new_session=True)
        try:
            try:
                stdout, stderr = child.communicate(timeout=30)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                stdout, stderr = child.communicate(timeout=5)
                self.fail(f"interrupt evidence control timed out ({mode}):\n{stdout}{stderr}")
            return child.returncode, stdout + stderr
        finally:
            # Also clean up orphan helpers after a fast failure in the live test.
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait(timeout=5)

    def test_live_acceptance_with_ignoring_parent(self):
        with temporary() as temp:
            status, output = self.probe("ignored", Path(temp) / "events.jsonl")
        self.assertEqual(0, status, output)

    @unittest.skipUnless(hasattr(signal, "pthread_sigmask"), "requires POSIX signal masks")
    def test_live_acceptance_with_blocked_parent(self):
        with temporary() as temp:
            status, output = self.probe("blocked", Path(temp) / "events.jsonl")
        self.assertEqual(0, status, output)

    def test_held_sender_reports_unavailable_evidence(self):
        with temporary() as temp:
            evidence = Path(temp) / "events.jsonl"
            status, output = self.probe("held", evidence)
            self.assertTrue(evidence.is_file(), "control never reached the live helper")
            events = [json.loads(line) for line in evidence.read_text().splitlines()]
        held = [event for event in events if event["event"] == "helper-held"]
        self.assertTrue(held, f"control did not hold a helper: {events}\n{output}")
        self.assertTrue(all(not event["gate_released"] for event in events), events)
        self.assertNotEqual(0, status, "acceptance passed without signal delivery")
        self.assertNotIn("pending interrupt was swallowed", output,
                         f"no helper ran or sent SIGINT; events={events}\n{output}")
        self.assertIn("interrupt evidence unavailable", output,
                      f"missing bounded synchronization diagnostic: {events}\n{output}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
