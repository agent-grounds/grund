"""Full Python acceptance entrypoint (§FS-distribution.3.3.7, §AR-bindings.6)."""

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
# The shared oracle and corpus sit one level up, beside Node's directory.
sys.path.insert(0, str(HERE.parent))
from adapter import binding  # noqa: E402

# Fail before collection if the capability is missing. No unconditional skips,
# installed-package fallback, or setup failure masquerading as a test failure.
binding()
suite = unittest.defaultTestLoader.discover(str(HERE), "test_*.py")
result = unittest.TextTestRunner(verbosity=2).run(suite)
sys.exit(not result.wasSuccessful())
