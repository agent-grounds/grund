"""Full Python acceptance entrypoint (§FS-distribution.3.3.7)."""

import sys
import unittest
from pathlib import Path
from support import binding

# Fail before collection if the capability is missing. No unconditional skips,
# installed-package fallback, or setup failure masquerading as a test failure.
binding()
suite = unittest.defaultTestLoader.discover(str(Path(__file__).parent), "test_*.py")
result = unittest.TextTestRunner(verbosity=2).run(suite)
sys.exit(not result.wasSuccessful())
