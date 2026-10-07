"""One checked workflow for §FS-distribution.3.2.4; absent capability stays red."""

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).parent))
sys.path.insert(0, str(ROOT / "tests" / "integration"))
sys.path.insert(0, str(ROOT / "tests" / "e2e"))
from test_frontend_isolation import FrontendIsolationTests  # noqa: E402

suite = unittest.TestSuite([
    unittest.defaultTestLoader.loadTestsFromName("test_node_binding.NativeSourceBuildTests"),
    unittest.defaultTestLoader.loadTestsFromName("test_node_binding.NodeConsumerTests"),
    unittest.defaultTestLoader.loadTestsFromName("test_package.NodePackageTests"),
    unittest.defaultTestLoader.loadTestsFromName("test_parity.NodeParityTests"),
    unittest.defaultTestLoader.loadTestsFromTestCase(FrontendIsolationTests),
])
sys.exit(not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful())
