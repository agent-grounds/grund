"""§AR-ci.10.7 - a fixture shared across a process ends with the process.

`release_queue_fixture.template()` builds the origin every release-helper case
copies, once per process. This runs a process that builds it and exits, with a
private, empty temporary directory, and names whatever that process left there.

The test lands skipped, one commit before the fix, because the pre-commit hook
runs this suite; implement removes the decorator. Run the module with the
decorator lifted to see it fail before the fix.
"""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from changelog_gate_fixture import environment

HERE = Path(__file__).resolve().parent
BUILD_AND_EXIT = "import release_queue_fixture; release_queue_fixture.template()"


class SharedFixtureLifetimeTests(unittest.TestCase):
    @unittest.skip("red until #1392 removes the fixture's root; implement removes this")
    def test_the_queue_template_is_gone_once_its_process_exits(self):
        with tempfile.TemporaryDirectory(prefix="grund-1392-template-tmp-") as private:
            run = subprocess.run(
                [sys.executable, "-c", BUILD_AND_EXIT], cwd=HERE,
                env=environment({"TMPDIR": private, "TMP": private, "TEMP": private}),
                capture_output=True, text=True, timeout=300)
            # Only a process that built the template says anything about its removal.
            self.assertEqual(run.returncode, 0, f"building the template failed:\n{run.stderr}")
            self.assertEqual(
                sorted(os.listdir(private)), [],
                "a process built the queue template, exited, and left these in its "
                f"temporary directory:\n{run.stderr}")


if __name__ == "__main__":
    unittest.main()
