"""§FS-distribution-candidate.7.1, §FS-distribution-candidate.7.2,
§FS-distribution-candidate.7.3, §FS-distribution-candidate.7.4, §FS-distribution.4.9 —
each payload of this row is the profile-use build of its own product's training.

The manifest's records are read first; then `scripts/pgo-build.sh` is run per
product with `--evidence`, so the training, the profile key and the digests are
the script's own account rather than the manifest's copy of it. The one allowed
fallback is not injected but met: on the Windows arm64 row rustc crashes compiling
the instrumented build, and that row's build is run again to show it, and that it
packages an LTO build then. Off that row the test is skipped with the row named.
Training that writes no profile is injected the way it would happen, with
`LLVM_PROFILE_FILE` pointed somewhere the instrumented binary cannot write; it is
not the exception, so it fails the build on every row that reaches training.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6). The `grund` workload is §AR-benchmarks.1.5's list.
"""

import json
import os
import shutil
import subprocess
import unittest

from candidate_inventory import member
from distribution_support import PGO, candidate
from rehearsal_support import acquire, keep, manifest, row, run_checked

EXCEPTION_ROW = "win32-arm64-msvc"
COMPILER_CRASH = "rustc crashed compiling the instrumented build"
NO_PROFILE = "training produced no profile"
# Resolved, because on Windows a bare name finds System32's WSL `bash.exe` first.
BASH = shutil.which("bash") or "bash"
INSTRUMENTED = (b"__llvm_profile_runtime", b"__llvm_profile_write_file", b"__llvm_prf_cnts", b".lprfc$")
TARGET = {}


def target_dir():
    """One Cargo target for every PGO run of this module, so the runs share a build."""
    return TARGET.setdefault("dir", keep("grund-pgo-target-"))


def pgo(*args, env=None):
    return subprocess.run([BASH, str(PGO), *map(str, args), "--target-dir", str(target_dir())],
                          capture_output=True, text=True, encoding="utf-8", errors="replace",
                          env=env, timeout=7200)


def unwritable_profile_env():
    """A profile path under a regular file, which no process can create."""
    blocker = keep("grund-pgo-fault-") / "not-a-directory"
    blocker.write_text("")
    return dict(os.environ, LLVM_PROFILE_FILE=str(blocker / "%p.profraw"))


def row_payloads():
    return [p for p in manifest()["payloads"] if p["row"] == row()["row"]]


def trained():
    """This row's products that train: every one but the exception row's LTO builds."""
    return [p["product"] for p in row_payloads()
            if not (p["row"] == EXCEPTION_ROW and p["optimization"] == "lto-exception")]


def skip_untrained(case, product=None):
    """Skip `case` when this row trains `product`, or any product, not at all."""
    untrained = product not in trained() if product else not trained()
    if untrained:
        case.skipTest(f"row {row()['row']} does not train {product or 'any product'}: rustc "
                      "crashed compiling the instrumented build (§FS-distribution-candidate.7.4)")


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
                    self.assertEqual(COMPILER_CRASH, payload["exception"]["failure"])
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
        skip_untrained(self)
        workloads = {}
        for product in trained():
            with self.subTest(product=product):
                record = self.evidence(product)
                self.assertEqual(product, record["product"])
                self.assertEqual(product, record["key"]["product"])
                self.assertEqual(manifest()["source_sha"], record["key"]["source_sha"])
                self.assertTrue(record["training"], "an empty training workload")
                workloads[product] = tuple(record["training"])
        for word in ("check", "show FS-check --full", "fmt --check"):
            if "grund" in workloads:
                self.assertTrue(any(word in step for step in workloads["grund"]), word)
        for method in ("initialize", "textDocument/didOpen", "textDocument/hover", "shutdown"):
            if "grund-lsp" in workloads:
                self.assertTrue(any(method in step for step in workloads["grund-lsp"]), method)
        self.assertEqual(len(workloads), len(set(workloads.values())), "two products share a workload")
        digests = [self.evidence(p)["profile_sha256"] for p in trained()]
        self.assertEqual(len(digests), len(set(digests)))

    def test_a_foreign_profile_is_refused(self):
        skip_untrained(self)
        product = trained()[0]
        foreign = self.evidence(product)["profile"]
        other = "grund-lsp" if product == "grund" else "grund"
        result = pgo("--product", other, "--profile", foreign)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("profile", result.stderr)

    def test_a_missing_profile_is_refused(self):
        missing = pgo("--product", "grund", "--profile", keep("grund-pgo-missing-") / "none.profdata")
        self.assertNotEqual(0, missing.returncode)
        self.assertIn("none.profdata", missing.stderr)


class FailureTests(unittest.TestCase):
    """§FS-distribution-candidate.7.4: one identified failure, on one row, falls back.
    The row is decided before the candidate is read, so a row this host is not skips."""

    def test_a_training_run_that_writes_no_profile_is_not_that_failure(self):
        acquire()
        skip_untrained(self, "grund")
        result = pgo("--product", "grund", env=unwritable_profile_env())
        self.assertNotIn(result.returncode, (0, 3), result.stderr)
        self.assertIn(NO_PROFILE, result.stderr)
        self.assertNotIn(COMPILER_CRASH, result.stderr)

    def test_an_ordinary_build_failure_is_not_that_failure(self):
        acquire()
        env = dict(os.environ, RUSTC=str(keep("grund-pgo-no-rustc-") / "no-such-rustc"))
        result = pgo("--product", "grund", env=env)
        self.assertNotIn(result.returncode, (0, 3))
        self.assertNotIn(NO_PROFILE, result.stderr)
        self.assertNotIn(COMPILER_CRASH, result.stderr)
        # An error rustc reports, as a compile error is, rather than a crash.
        env = dict(os.environ, RUSTFLAGS="-Cno-such-codegen-option")
        result = pgo("--product", "grund", env=env)
        self.assertNotIn(result.returncode, (0, 3), result.stderr)
        self.assertNotIn(COMPILER_CRASH, result.stderr)

    def build_grund(self, env=None):
        sha = run_checked(["git", "rev-parse", "HEAD"]).stdout.strip()
        out = keep("grund-pgo-fallback-") / "candidate"
        result = candidate("build", "--row", row()["row"], "--product", "grund", "--sha", sha,
                           "--out", out, "--target-dir", target_dir(), env=env, timeout=7200)
        return result, out

    def test_the_windows_arm64_row_takes_the_exception(self):
        if row()["row"] != EXCEPTION_ROW:
            self.skipTest(f"row {EXCEPTION_ROW} takes the exception on windows-11-arm; "
                          f"this host is {row()['row']}")
        acquire()
        result, out = self.build_grund()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("STATUS_ACCESS_VIOLATION", result.stderr)
        self.assertIn(COMPILER_CRASH, result.stderr)
        payload, = [p for p in json.loads((out / "manifest.json").read_text())["payloads"]
                    if p["product"] == "grund"]
        self.assertEqual("lto-exception", payload["optimization"])
        self.assertEqual(COMPILER_CRASH, payload["exception"]["failure"])

    def test_every_other_row_fails_the_candidate(self):
        if row()["row"] == EXCEPTION_ROW:
            self.skipTest(f"row {EXCEPTION_ROW} crashes compiling the instrumented build "
                          "before it trains")
        acquire()
        result, _ = self.build_grund(env=unwritable_profile_env())
        self.assertNotEqual(0, result.returncode)
        self.assertIn(row()["row"], result.stderr)
        self.assertIn(NO_PROFILE, result.stderr)


if __name__ == "__main__":
    unittest.main()
