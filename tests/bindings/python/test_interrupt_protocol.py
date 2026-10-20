"""Positive native interrupt ordering (§AR-bindings.6.2.1, §FS-distribution.3.3.4)."""

import json
import os
from pathlib import Path
import unittest

import test_interrupt_evidence as controls
from support import temporary


@unittest.skipUnless(os.name == "posix", "POSIX signal evidence control")
class InterruptProtocolTests(unittest.TestCase):
    def test_released_helper_proves_pending_signal_before_native_return(self):
        """A held then released helper must establish every stage (§AR-bindings.6.2.1)."""
        with temporary() as temp:
            evidence = Path(temp) / "events.jsonl"
            status, output = controls.InterruptEvidenceTests().probe("scheduled", evidence)
            self.assertEqual(0, status, output)
            self.assertTrue(evidence.is_file(), "scheduling instrumentation never ran")
            events = [json.loads(line) for line in evidence.read_text().splitlines()]
        names = [event["event"] for event in events]
        self.assertEqual(1, names.count("helper-held"), events)
        self.assertEqual(1, names.count("helper-released"), events)
        self.assertLess(names.index("helper-held"), names.index("helper-released"), events)
        self.assertFalse(events[names.index("helper-held")]["gate_released"], events)
        self.assertTrue(events[names.index("helper-released")]["gate_released"], events)
        protocol = [name for name in names if not name.startswith("helper-")]
        self.assertEqual(["ready", "native-reader", "signal-pending", "native-release",
                          "native-return-interrupt", "independent-call"], protocol, events)


if __name__ == "__main__":
    unittest.main(verbosity=2)
