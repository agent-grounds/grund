"""Fresh installed consumers pin §FS-distribution.3.2.4 before implementation."""

import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "bindings" / "node"))
from support import Rehearsal  # noqa: E402
from support import CASES, NODE, REPO, SCRATCH, checked  # noqa: E402

# §FS-fetch.2: grund runs a fetcher directly, with no shell of its own, so a case
# marked `requires-unix-shell` is skipped off unix exactly as the CLI's case runner
# skips it, and the skip is counted and named rather than passing silently.
FETCH_CASE = CASES / "fetch-workspace-folder"
REQUIRES_UNIX_SHELL = (FETCH_CASE / "requires-unix-shell").is_file() and os.name != "posix"
REQUIRES_UNIX_SHELL_SKIP = (
    "the fetch integration is a POSIX shell script grund runs directly; "
    "this platform has no shell to interpret it"
)


class NativeSourceBuildTests(unittest.TestCase):
    def test_documented_native_source_builder(self):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="grund-native-build-", dir=SCRATCH) as directory:
            target = Path(directory)
            checked(["node", NODE / "build.mjs", "--source-root", REPO,
                     "--out-dir", target / "native", "--target-dir", target / "cargo",
                     "--profile", "release"])
            self.assertTrue((target / "native" / "grund.node").is_file())


class NodeConsumerTests(unittest.TestCase):
    def setUp(self):
        self.rehearsal = Rehearsal()
        self.addCleanup(self.rehearsal.close)

    def test_node_local_package(self):
        self.assertEqual("", self.rehearsal.run("operations.mjs"))

    def test_input_query_and_config_failures(self):
        self.assertEqual("", self.rehearsal.run("failures.mjs"))

    def test_independent_owned_requests_and_read_only_trees(self):
        self.assertEqual("", self.rehearsal.run("concurrency.mjs"))

    def test_core_owned_mutations_and_refusals(self):
        self.assertEqual("", self.rehearsal.run("mutations.mjs"))

    @unittest.skipIf(REQUIRES_UNIX_SHELL, REQUIRES_UNIX_SHELL_SKIP)
    def test_explicit_fetch_writes_the_core_snapshot(self):
        self.assertEqual("", self.rehearsal.run("fetch.mjs"))

    def test_native_workers_panics_writer_lease_and_teardown(self):
        checked(["node", NODE / "build.mjs", "--source-root", REPO,
                 "--out-dir", self.rehearsal.package / "native", "--target-dir",
                 self.rehearsal.target, "--profile", "release", "--test-seams"])
        self.assertEqual("", self.rehearsal.run("runtime.mjs"))
        self.assertEqual("finished\n", self.rehearsal.run("pending-liveness.cjs"))


if __name__ == "__main__":
    unittest.main()
