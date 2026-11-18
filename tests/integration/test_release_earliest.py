"""§FS-distribution.4.4: the earliest release a tree may be cut as, which the
scheduled patch helper skips below. It is the inclusive floor of the release
window of §FS-distribution.4.2.5, read by the guard's own functions."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EARLIEST = ROOT / "scripts/release_earliest.py"
LANDED = "crates/grund-core/src/lib.rs"
PENDING = "tests/e2e/cases/promise/expected.stderr"
FLOOR = "scripts/release-floor.txt"


class EarliestReleaseTests(unittest.TestCase):
    def earliest(self, landed=None, floor=None):
        with tempfile.TemporaryDirectory(prefix="grund-earliest-") as name:
            root = Path(name)
            files = {PENDING: "warning: wording changes in grund 2.0.0\n"}
            if landed:
                files[LANDED] = f'const MESSAGE: &str = "became an error in {landed}";\n'
            if floor:
                files[FLOOR] = floor + "\n"
            for path, text in files.items():
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_text(text, encoding="utf-8")
            result = subprocess.run([sys.executable, str(EARLIEST), "--root", str(root)],
                                    capture_output=True, text=True, check=False)
        self.assertEqual(("", 0), (result.stderr, result.returncode))
        return result.stdout

    def test_an_unbounded_tree_may_be_cut_as_anything(self):
        self.assertEqual("0.0.0\n", self.earliest())

    def test_the_shared_floor_bounds_a_tree_with_no_landed_change(self):
        self.assertEqual("0.19.0\n", self.earliest(floor="0.19.0"))

    def test_a_landed_change_dated_later_raises_the_floor(self):
        self.assertEqual("1.0.0\n", self.earliest(landed="1.0.0", floor="0.19.0"))

    def test_a_landed_change_below_the_shared_floor_does_not_lower_it(self):
        self.assertEqual("0.19.0\n", self.earliest(landed="0.18.0", floor="0.19.0"))


if __name__ == "__main__":
    unittest.main()
