"""§FS-distribution-candidate.7.1, §FS-distribution-candidate.7.2,
§FS-distribution-candidate.7.3, §FS-distribution-candidate.7.4, §FS-distribution.4.9 —
each payload of this row is the profile-use build of its own product's training.

The manifest's records are read first; then `scripts/pgo-build.sh` is run per
product with `--evidence`, so the training, the profile key and the digests are
the script's own account rather than the manifest's copy of it. The one allowed
fallback is injected the way it happens: `LLVM_PROFILE_FILE` pointed somewhere the
instrumented binary cannot write, so training produces no profile. Only the Windows
arm64 row may package an LTO build then; on any other host that test is skipped
with the row named, and the refusal on this row is what runs.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6). The `grund` workload is §AR-benchmarks.1.5's list.
"""

import json
import os
import subprocess
import unittest

from candidate_inventory import member
from distribution_support import PGO, candidate
from rehearsal_support import acquire, keep, manifest, row, run_checked

EXCEPTION_ROW = "win32-arm64-msvc"
NO_PROFILE = "training produced no profile"
INSTRUMENTED = (b"__llvm_profile_runtime", b"__llvm_profile_write_file", b"__llvm_prf_cnts", b".lprfc$")
TARGET = {}


def target_dir():
    """One Cargo target for every PGO run of this module, so the runs share a build."""
    return TARGET.setdefault("dir", keep("grund-pgo-target-"))


def pgo(*args, env=None):
    return subprocess.run(["bash", str(PGO), *map(str, args), "--target-dir", str(target_dir())],
                          capture_output=True, text=True, encoding="utf-8", errors="replace",
                          env=env, timeout=7200)


def unwritable_profile_env():
    """A profile path under a regular file, which no process can create."""
    blocker = keep("grund-pgo-fault-") / "not-a-directory"
    blocker.write_text("")
    return dict(os.environ, LLVM_PROFILE_FILE=str(blocker / "%p.profraw"))


def row_payloads():
    return [p for p in manifest()["payloads"] if p["row"] == row()["row"]]


class RecordTests(unittest.TestCase):
    """§FS-distribution-candidate.7.2, §FS-distribution-candidate.7.3: what the manifest says."""

    def setUp(self):
        acquire()

    def test_every_payload_records_its_own_profile(self):
        profiles = []
        for payload in row_payloads():
            with self.subTest(payload=payload["id"]):
                if payload["optimization"] == "lto-exception":
                    self.assertEqual(EXCEPTION_ROW, payload["row"])
                    self.assertEqual(NO_PROFILE, payload["exception"]["failure"])
                    continue
                self.assertEqual("pgo", payload["optimization"])
                self.assertEqual({"generate", "train", "merge", "use"}, set(payload["steps"]))
                self.assertEqual(payload["build_sha256"], payload["steps"]["use"])
                key = payload["profile_key"]
                self.assertEqual((payload["product"], payload["target"], manifest()["source_sha"]),
                                 (key["product"], key["target"], key["source_sha"]))
                self.assertTrue(key["compiler"].startswith("rustc "))
                expected_abi = "abi3-py310" if payload["product"] == "python-extension" else None
                self.assertEqual(expected_abi, key["abi"])
                profiles.append(payload["profile_sha256"])
        self.assertEqual(len(profiles), len(set(profiles)), "two payloads share one profile")

    def test_no_instrumented_binary_is_packaged(self):
        for payload in row_payloads():
            for placement in payload["placements"]:
                with self.subTest(payload=payload["id"], artifact=placement["artifact"]):
                    data = member(placement["artifact"], placement["path"])
                    self.assertEqual([], [m for m in INSTRUMENTED if m in data])


class TrainingTests(unittest.TestCase):
    """§FS-distribution-candidate.7.1, §FS-distribution-candidate.7.2: the script's account."""

    PRODUCTS = ("grund", "grund-lsp", "node-addon", "python-extension")
    EVIDENCE = {}

    def setUp(self):
        acquire()

    def evidence(self, product):
        if product not in self.EVIDENCE:
            path = keep("grund-pgo-evidence-") / f"{product}.json"
            result = pgo("--product", product, "--evidence", path)
            self.assertEqual(0, result.returncode, f"pgo-build.sh --product {product}:\n{result.stderr}")
            self.EVIDENCE[product] = json.loads(path.read_text(encoding="utf-8"))
        return self.EVIDENCE[product]

    def test_each_product_trains_on_its_own_workload(self):
        workloads = {}
        for product in self.PRODUCTS:
            with self.subTest(product=product):
                record = self.evidence(product)
                self.assertEqual(product, record["product"])
                self.assertEqual(product, record["key"]["product"])
                self.assertEqual(manifest()["source_sha"], record["key"]["source_sha"])
                self.assertTrue(record["training"], "an empty training workload")
                workloads[product] = tuple(record["training"])
        for word in ("check", "show FS-check --full", "fmt --check"):
            self.assertTrue(any(word in step for step in workloads["grund"]), word)
        for method in ("initialize", "textDocument/didOpen", "textDocument/hover", "shutdown"):
            self.assertTrue(any(method in step for step in workloads["grund-lsp"]), method)
        self.assertEqual(len(workloads), len(set(workloads.values())), "two products share a workload")
        digests = [self.evidence(p)["profile_sha256"] for p in self.PRODUCTS]
        self.assertEqual(len(digests), len(set(digests)))

    def test_a_foreign_profile_is_refused(self):
        foreign = self.evidence("grund")["profile"]
        result = pgo("--product", "grund-lsp", "--profile", foreign)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("profile", result.stderr)
        missing = pgo("--product", "grund", "--profile", keep("grund-pgo-missing-") / "none.profdata")
        self.assertNotEqual(0, missing.returncode)
        self.assertIn("none.profdata", missing.stderr)


class FailureTests(unittest.TestCase):
    """§FS-distribution-candidate.7.4: one identified failure, on one row, falls back.
    The row is decided before the candidate is read, so a row this host is not skips."""

    def test_a_training_run_that_writes_no_profile_is_identified(self):
        acquire()
        result = pgo("--product", "grund", env=unwritable_profile_env())
        self.assertEqual(3, result.returncode, result.stderr)
        self.assertIn(NO_PROFILE, result.stderr)

    def test_an_ordinary_build_failure_is_not_that_failure(self):
        acquire()
        env = dict(os.environ, RUSTC=str(keep("grund-pgo-no-rustc-") / "no-such-rustc"))
        result = pgo("--product", "grund", env=env)
        self.assertNotIn(result.returncode, (0, 3))
        self.assertNotIn(NO_PROFILE, result.stderr)

    def build_without_profile(self):
        sha = run_checked(["git", "rev-parse", "HEAD"]).stdout.strip()
        out = keep("grund-pgo-fallback-") / "candidate"
        env = unwritable_profile_env()
        result = candidate("build", "--row", row()["row"], "--product", "grund", "--sha", sha,
                           "--out", out, "--target-dir", target_dir(), env=env, timeout=7200)
        return result, out

    def test_the_windows_arm64_row_takes_the_exception(self):
        if row()["row"] != EXCEPTION_ROW:
            self.skipTest(f"row {EXCEPTION_ROW} takes the exception on windows-11-arm; "
                          f"this host is {row()['row']}")
        acquire()
        result, out = self.build_without_profile()
        self.assertEqual(0, result.returncode, result.stderr)
        payload, = [p for p in json.loads((out / "manifest.json").read_text())["payloads"]
                    if p["product"] == "grund"]
        self.assertEqual("lto-exception", payload["optimization"])
        self.assertEqual(NO_PROFILE, payload["exception"]["failure"])

    def test_every_other_row_fails_the_candidate(self):
        if row()["row"] == EXCEPTION_ROW:
            self.skipTest(f"row {EXCEPTION_ROW} is the one row that may fall back")
        acquire()
        result, _ = self.build_without_profile()
        self.assertNotEqual(0, result.returncode)
        self.assertIn(row()["row"], result.stderr)
        self.assertIn(NO_PROFILE, result.stderr)


if __name__ == "__main__":
    unittest.main()
