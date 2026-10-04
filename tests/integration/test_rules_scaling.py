"""§AR-rules.3.1, §FS-rules.5.3: run the optimized black-box cost regression.

Discovered by the existing python-test hook in pre-commit and CI. No ignored
test, optional feature, new flag, or timing of an unoptimized build is involved.
"""

import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RulesScalingTests(unittest.TestCase):
    def test_independent_records_do_not_trigger_cubic_rule_evaluation(self):
        target = ROOT / "target"
        subprocess.run(
            ["cargo", "build", "--release", "--locked", "-p", "grund",
             "--target-dir", str(target)], cwd=ROOT, check=True, timeout=900
        )
        result = subprocess.run(
            [sys.executable, str(ROOT / "tests/e2e/rules_scaling.py"),
             str(target / "release/grund")], cwd=ROOT, capture_output=True,
            text=True, timeout=900
        )
        print(result.stdout, end="", flush=True)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
