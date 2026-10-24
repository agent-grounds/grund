"""Real-helper acceptance for §FS-repository-maintenance.2.1,
§FS-repository-maintenance.2.2, §FS-repository-maintenance.2.3 and
§FS-repository-maintenance.2.4; ownership format: §AR-ci.3.4.1.2.

Port the issue #963 two-checkout reproducer: copy the real harness, create both
targets, remove one source checkout, and ask the survivor for its target again.
No Rust build, npm invocation or access to the machine's actual cache is needed.
"""

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "tests" / "bindings" / "node"
CALL = """
import json, os, sys, time
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import support

mode, argument = sys.argv[2:4]
if mode == "denied":
    original_stat = os.stat
    def guarded_stat(path, *args, **kwargs):
        if not isinstance(path, int) and os.path.abspath(os.fsdecode(path)) == argument:
            raise PermissionError(13, "fixture denies checkout lookup", argument)
        return original_stat(path, *args, **kwargs)
    os.stat = guarded_stat
if mode == "race":
    while not os.path.exists(argument):
        time.sleep(0.005)

targets = [str(support.node_target()) for _ in range(20 if mode == "race" else 2)]
print(json.dumps(targets))
"""


def key(checkout):
    return hashlib.sha256(os.fsencode(checkout)).hexdigest()[:16]


class NodeTargetLifetimeTests(unittest.TestCase):
    def setUp(self):
        # Explicit scratch home; never touch ~/ag/tmp/grund-node-target.
        scratch = Path.home() / "ag" / "tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="grund-963-lifetime-", dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name).resolve()
        self.shared = self.work / "targets"
        self.shared.mkdir()
        self.environment = {**os.environ, "GRUND_NODE_TARGET": str(self.shared)}

    def checkout(self, name):
        root = self.work / name
        harness = root / "tests" / "bindings" / "node"
        harness.mkdir(parents=True)
        for module in ("support.py", "launch.py"):
            shutil.copyfile(HARNESS / module, harness / module)
        # The shared harness derives the target for both bindings (§AR-ci.3.4.1.2).
        shutil.copyfile(HARNESS.parent / "support.py", harness.parent / "support.py")
        return root

    def argv(self, checkout, mode="normal", argument=""):
        return [sys.executable, "-I", "-c", CALL,
                str(checkout / "tests" / "bindings" / "node"), mode, str(argument)]

    def call(self, checkout, mode="normal", argument=""):
        result = subprocess.run(self.argv(checkout, mode, argument), env=self.environment,
                                cwd=self.work, capture_output=True, text=True, timeout=20)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        targets = json.loads(result.stdout)
        self.assertEqual(1, len(set(targets)), "repeat calls changed the target")
        return Path(targets[0])

    def payload(self, directory):
        artifact = directory / "debug" / "build-payload"
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.write_bytes(b"keep build output\x00\xff")
        return artifact

    def assert_payload(self, artifact):
        self.assertEqual(b"keep build output\x00\xff", artifact.read_bytes())

    def assert_owner(self, target, checkout):
        record = target / "checkout.json"
        self.assertTrue(record.is_file(), "target does not record its resolved checkout")
        self.assertEqual(str(checkout.resolve()),
                         json.loads(record.read_text(encoding="utf-8"))["checkout"])

    def dead_target(self):
        checkout = self.checkout("scratch-review")
        target = self.call(checkout)
        self.payload(target)
        shutil.rmtree(checkout)
        return target

    def assert_reclaimed(self, target):
        self.assertFalse(target.exists(), "removed checkout's nonempty target was not reclaimed")

    def seeded(self, name, record):
        target = self.shared / name
        target.mkdir()
        (target / "checkout.json").write_text(record, encoding="utf-8")
        return target, self.payload(target)

    def test_removed_checkout_target_is_reclaimed_and_live_payloads_survive(self):
        scratch = self.checkout("scratch-review")
        live = self.checkout("live-main")
        dead = self.call(scratch)
        self.payload(dead)
        target = self.call(live)
        artifact = self.payload(target)
        tools = self.payload(self.shared / "tools")
        shutil.rmtree(scratch)

        self.assertEqual(target, self.call(live))
        self.assert_payload(artifact)
        self.assert_payload(tools)
        self.assert_reclaimed(dead)

    def test_existing_target_acquires_resolved_ownership_without_losing_output(self):
        live = self.checkout("live-main")
        target = self.shared / key(live)
        artifact = self.payload(target)
        self.assertEqual(target, self.call(live))
        self.assertEqual(target, self.call(live))  # another process
        self.assert_payload(artifact)
        self.assert_owner(target, live)

    @unittest.skipUnless(os.name == "posix", "fixture checkout alias needs POSIX symlinks")
    def test_checkout_alias_records_the_resolved_source_and_reuses_its_target(self):
        live = self.checkout("live-main")
        alias = self.work / "checkout-alias"
        alias.symlink_to(live, target_is_directory=True)
        target = self.call(alias)
        self.assertEqual(self.shared / key(live), target)
        self.assertEqual(target, self.call(live))
        self.assert_owner(target, live)

    def test_unowned_and_untrusted_entries_survive_an_effective_cleanup(self):
        dead = self.dead_target()
        absent = self.work / "absent"
        valid = json.dumps({"checkout": str(absent)})
        preserved = []
        for index, record in enumerate(("{", "", "[]", '{"checkout": 7}',
                                        '{"checkout": "relative"}',
                                        json.dumps({"checkout": str(self.work / "other")}))):
            # All names look like targets; none supplies trustworthy matching ownership.
            _, artifact = self.seeded(key(self.work / f"invalid-{index}"), record)
            preserved.append(artifact)
        _, unrelated = self.seeded("notes", valid)
        preserved.append(unrelated)
        for owner in ("relative-source", str(self.work / "unused" / ".." / "gone")):
            _, artifact = self.seeded(key(owner), json.dumps({"checkout": owner}))
            preserved.append(artifact)
        legacy = self.shared / key(self.work / "legacy-deleted-source")
        preserved.append(self.payload(legacy))
        record_directory = self.shared / key(absent)
        (record_directory / "checkout.json").mkdir(parents=True)
        preserved.append(self.payload(record_directory))
        # Even plausible ownership cannot bring shared tools into the target namespace.
        _, tools = self.seeded("tools", valid)
        preserved.append(tools)

        self.call(self.checkout("live-main"))
        for artifact in preserved:
            self.assert_payload(artifact)
        self.assertTrue((record_directory / "checkout.json").is_dir())
        self.assertFalse((legacy / "checkout.json").exists())
        self.assert_reclaimed(dead)  # a no-op pruner cannot pass the safety test

    @unittest.skipUnless(os.name == "posix", "fixture cache links need POSIX symlinks")
    def test_target_and_record_symlinks_are_preserved_without_following_them(self):
        dead = self.dead_target()
        absent = self.work / "gone-link-owner"
        outside = self.work / "outside-target-root"
        outside.mkdir()
        (outside / "checkout.json").write_text(json.dumps({"checkout": str(absent)}),
                                                encoding="utf-8")
        artifact = self.payload(outside)
        link = self.shared / key(absent)
        link.symlink_to(outside, target_is_directory=True)
        other = self.work / "gone-record-owner"
        target = self.shared / key(other)
        target.mkdir()
        record = self.work / "external-record.json"
        record.write_text(json.dumps({"checkout": str(other)}), encoding="utf-8")
        (target / "checkout.json").symlink_to(record)
        other_artifact = self.payload(target)

        self.call(self.checkout("live-main"))
        self.assertTrue(link.is_symlink())
        self.assertTrue((target / "checkout.json").is_symlink())
        self.assert_payload(artifact)
        self.assert_payload(other_artifact)
        self.assertEqual({"checkout": str(other)}, json.loads(record.read_text()))
        self.assert_reclaimed(dead)

    def test_uncertain_checkout_lookup_preserves_its_target(self):
        inaccessible = self.checkout("inaccessible-source")
        target = self.call(inaccessible)
        artifact = self.payload(target)
        dead = self.dead_target()
        live = self.checkout("live-main")
        shutil.rmtree(inaccessible)
        # Inject EACCES at the filesystem boundary even when tests run as root.
        self.call(live, "denied", inaccessible)
        self.assert_payload(artifact)
        self.assert_reclaimed(dead)

    def test_concurrent_callers_publish_complete_records_and_keep_live_targets(self):
        dead = self.dead_target()
        checkouts = [self.checkout(name) for name in ("live-main", "live-review")]
        targets = [self.shared / key(checkout) for checkout in checkouts]
        artifacts = [self.payload(target) for target in targets]
        tools = self.payload(self.shared / "tools")
        start = self.work / "start"
        processes = []
        try:
            for checkout in checkouts * 4:
                processes.append(subprocess.Popen(self.argv(checkout, "race", start),
                                                 env=self.environment, cwd=self.work,
                                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                                 text=True))
            start.touch()
            deadline = time.monotonic() + 20
            while any(process.poll() is None for process in processes):
                self.assertLess(time.monotonic(), deadline, "concurrent helpers timed out")
                for checkout, target in zip(checkouts, targets):
                    try:
                        data = (target / "checkout.json").read_text(encoding="utf-8")
                    except FileNotFoundError:  # not published yet
                        continue
                    except PermissionError:  # Windows: a replace holds the name
                        if os.name != "nt":
                            raise
                        continue
                    self.assertEqual(str(checkout), json.loads(data)["checkout"],
                                     "concurrent reader saw incomplete or wrong ownership")
                time.sleep(0.001)
            for index, process in enumerate(processes):
                stdout, stderr = process.communicate(timeout=5)
                self.assertEqual(0, process.returncode, stdout + stderr)
                self.assertEqual([str(targets[index % 2])] * 20, json.loads(stdout))
            self.assertNotEqual(*targets)
            for checkout, target, artifact in zip(checkouts, targets, artifacts):
                self.assert_payload(artifact)
                self.assert_owner(target, checkout)
            self.assert_payload(tools)
            self.assert_reclaimed(dead)
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                process.communicate(timeout=5)


if __name__ == "__main__":
    unittest.main()
