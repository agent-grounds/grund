"""Publication assertions for §AR-ci.3.4.1.5, run by the Node acceptance module."""

import os
from pathlib import Path
import re
import tempfile
import time

from node_cache_publication_fixture import (
    DEPENDENCIES, WORKSPACE, PublicationFixture, digest, files,
)
from test_ci_cache_saves import with_value


def check_publication(self, workflow):
    scratch = Path.home() / "ag/tmp/grund-1122-target"
    scratch.mkdir(parents=True, exist_ok=True)
    for cargo_hit in (False, True):
        with self.subTest(workflow=workflow, cargo_hit=cargo_hit):
            with tempfile.TemporaryDirectory(prefix="publication-", dir=scratch) as temp:
                fixture = PublicationFixture(Path(temp), workflow)
                before = files(fixture.root)
                fixture.prepare(cargo_hit)
                self.assertEqual(before, files(fixture.root),
                                 "cache preparation changed a live target or ownership record")
                payload = fixture.payload()
                hashes = {digest(data) for data in payload.values()}
                dependencies = fixture.artifacts["dependency"]
                self.assertTrue(dependencies, "fixture built no third-party artifacts")
                self.assertFalse(
                    {digest(before[p]) for p in dependencies}
                    & {digest(before[p]) for p in fixture.artifacts["workspace"]},
                    "fixture workspace and dependency artifact identities overlap",
                )
                for path in dependencies:
                    self.assertIn(digest(before[path]), hashes,
                                  f"publication lost a compiled dependency: {path}")

                # Save evidence before removing the live root for a real restore/rebuild.
                leaked = [str(path.relative_to(fixture.root))
                          for path in fixture.artifacts["workspace"]
                          if digest(before[path]) in hashes]
                extra = [str(p) for p in payload if
                         "incremental" in p.parts or "typescript-5.9.3" in p.parts]
                # Fingerprint/build directories are package-named, even for a renamed lib.
                for path in payload:
                    for parent in path.parents:
                        if (parent.parent.name in (".fingerprint", "build")
                                and parent.name.rsplit("-", 1)[0] in WORKSPACE):
                            extra.append(str(path))
                            break

                fixture.restore_payload(payload)
                for path in dependencies | set(fixture.owners):
                    self.assertTrue(path.is_file(), f"restore lost target-relative path: {path}")
                    self.assertEqual(before[path], path.read_bytes())
                for checkout, target in fixture.targets:
                    # Force workspace compilation, while third-party fingerprints stay warm.
                    source = checkout / "grund/src/lib.rs"
                    source.write_text(source.read_text() + "// new workspace input\n")
                    future = time.time() + 2
                    os.utime(source, (future, future))
                    for profile in ("dev", "release"):
                        messages = fixture.cargo(checkout, target, profile)
                        third_party = [m for m in messages
                                       if Path(m["manifest_path"]).parent.name in DEPENDENCIES]
                        self.assertTrue(third_party, "no dependency reuse evidence")
                        self.assertTrue(all(m["fresh"] for m in third_party),
                                        f"restored {profile} dependencies recompiled")
                        self.assertTrue(any(not m["fresh"] for m in messages
                                            if Path(m["manifest_path"]).parent.name == "grund"),
                                        "fixture did not rebuild changed workspace sources")
                self.assertEqual(0, len(leaked),
                                 "Node publication retains compiled workspace artifacts")
                self.assertEqual([], sorted(extra),
                                 "Node publication retains workspace state, incremental or TypeScript")
                restore = fixture.steps[fixture.restore]
                save = fixture.steps[fixture.save]
                self.assertEqual(with_value(restore, "path"), with_value(save, "path"))
                version = re.search(r"-node-target-v(\d+)-", with_value(restore, "key"))
                self.assertIsNotNone(version, "Node cache key has no representation version")
                self.assertGreater(int(version.group(1)), 3,
                                   "immutable whole-root cache keys must be retired")
                namespace = f"-node-target-v{version.group(1)}-"
                for prefix in with_value(restore, "restore-keys").splitlines():
                    self.assertIn(namespace, prefix, "prefix admits an old whole-root entry")
