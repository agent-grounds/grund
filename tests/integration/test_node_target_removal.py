"""Real-helper removal acceptance for §FS-repository-maintenance.2.4.

Protected output/ownership follow §FS-repository-maintenance.2.1 and eligibility
follows §FS-repository-maintenance.2.3; helper placement: §AR-ci.3.4.1.2.
"""

import errno
import json
from pathlib import Path
import subprocess
import sys
import unittest

FIXTURE = Path(__file__).with_name("node_target_removal_fixture.py")


class NodeTargetRemovalTests(unittest.TestCase):
    def scenario(self, mode, location):
        # A subprocess makes nontermination fail rather than hang the whole suite.
        result = subprocess.run([sys.executable, "-I", "-B", str(FIXTURE), mode, location],
                                capture_output=True, text=True, timeout=20)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        observed = json.loads(result.stdout)
        self.assertTrue(observed["injected"], "real shutil did not reach the removal boundary")
        self.assertTrue(observed["protected"], "cleanup changed protected output or ownership")
        self.assertTrue(observed["owner"], "caller ownership was incomplete or incorrect")
        return observed

    def assert_completed(self, observed):
        if observed["error"] is not None:
            self.fail(observed["error"]["traceback"])
        self.assertTrue(observed["returned"], "helper did not return the caller's stable target")
        self.assertTrue(observed["absent_at_return"], "helper returned before removal completed")

    def test_peer_removal_after_denial_completes_before_return(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                observed = self.scenario("peer", location)
                self.assertTrue(observed["peer_finished"])
                self.assert_completed(observed)

    def test_successful_removal_after_transient_denial(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                self.assert_completed(self.scenario("retry", location))

    def test_persistent_denial_is_a_bounded_visible_failure(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                observed = self.scenario("persistent", location)
                self.assertEqual("PermissionError", observed["error"]["type"])
                self.assertTrue(observed["boundary_retained"])
                self.assertFalse(observed["returned"])

    def test_already_removed_output_is_success(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                self.assert_completed(self.scenario("missing", location))

    def test_unrelated_removal_error_remains_visible(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                observed = self.scenario("unrelated", location)
                self.assertEqual(errno.EIO, observed["error"]["errno"])
                self.assertTrue(observed["boundary_retained"])
                self.assertFalse(observed["returned"])

    def test_uncertain_absence_does_not_turn_denial_into_success(self):
        for location in ("root", "directory", "file"):
            with self.subTest(location=location):
                observed = self.scenario("uncertain", location)
                self.assertEqual("PermissionError", observed["error"]["type"])
                self.assertTrue(observed["boundary_retained"])
                self.assertFalse(observed["returned"])


if __name__ == "__main__":
    unittest.main()
