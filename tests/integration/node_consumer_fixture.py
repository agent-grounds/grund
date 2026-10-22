"""Real npm preparation and private consumer proof for §AR-ci.3.4.1.4.

Run in a child so the Node harness's `support` never leaks into gate discovery.
Only the expensive native-build input is replaced, by local tarballs. npm,
Rehearsal, the command transport, filesystem copies and Node imports are real.
The existing acceptance owns release/native/fallback and oracle proof.
"""

import io
import json
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tests/bindings/node"))
import launch  # noqa: E402
import support  # noqa: E402


def tarball(root, receipt):
    """Same package name/version, different actual installed source and artifact bytes."""
    stage = root / receipt
    stage.mkdir()
    archive = stage / "grund-cli-0.0.0.tgz"
    files = {
        "package.json": json.dumps({"name": "grund-cli", "version": "0.0.0",
                                    "main": "index.cjs"}),
        "index.cjs": "module.exports=()=>require('node:fs').readFileSync("
                     "require('node:path').join(__dirname,'receipt.txt'),'utf8');\n",
        "receipt.txt": receipt,
        "native/grund.node": "native-" + receipt,
        "build/sources/source.rs": "source-" + receipt,
    }
    with tarfile.open(archive, "w:gz") as output:
        for name, text in files.items():
            data = text.encode()
            staged = stage / name
            staged.parent.mkdir(parents=True, exist_ok=True)
            staged.write_bytes(data)
            info = tarfile.TarInfo("package/" + name)
            info.size = len(data)
            output.addfile(info, io.BytesIO(data))
    return stage, root / "cargo-target", archive


class ConsumerPreparationContract(unittest.TestCase):
    def check_consumer(self, rehearsal, receipt):
        self.assertEqual(receipt, (rehearsal.package / "receipt.txt").read_text())
        self.assertEqual("native-" + receipt,
                         (rehearsal.package / "native/grund.node").read_text())
        self.assertEqual("source-" + receipt,
                         (rehearsal.package / "build/sources/source.rs").read_text())
        self.assertFalse(rehearsal.package.is_symlink())
        self.assertTrue(rehearsal.package.resolve().is_relative_to(rehearsal.consumer.resolve()))
        result = launch.command(["node", "-e", "process.stdout.write(require('grund-cli')())"],
                                cwd=rehearsal.consumer)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(receipt, result.stdout)

    def test_preparation_and_isolation(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="grund-809-consumers-", dir=scratch) as temp:
            root = Path(temp)
            first_source = tarball(root, "source-A")
            second_source = tarball(root, "source-B")
            selected = [first_source]
            installs = []
            real_command = launch.command

            def observed(argv, *args, **kwargs):
                if list(map(str, argv[:2])) == ["npm", "install"]:
                    installs.append(list(map(str, argv)))
                return real_command(argv, *args, **kwargs)

            # Local-only packages; a regression must not silently fetch from a registry.
            env = {"npm_config_offline": "true", "npm_config_cache": str(root / "npm-cache"),
                   "npm_config_update_notifier": "false"}
            with patch.object(support, "SCRATCH", root), \
                    patch.object(support, "source_package", side_effect=lambda: selected[0]), \
                    patch.object(launch, "command", side_effect=observed), \
                    patch.dict(os.environ, env):
                paths = []
                for source in (first_source, second_source):
                    selected[0] = source
                    receipt = source[0].name
                    first = support.Rehearsal()
                    try:
                        self.check_consumer(first, receipt)
                        (first.package / "receipt.txt").write_text("mutated")
                        (first.package / "native/grund.node").unlink()
                        (first.package / "build/sources/source.rs").write_text("mutated")
                        (first.consumer / "private-file").write_text("private")
                        # The template must outlive the preceding test's private consumer.
                        first.close()
                        second = support.Rehearsal()
                        try:
                            self.check_consumer(second, receipt)
                            self.assertFalse((second.consumer / "private-file").exists())
                            paths.extend([first.consumer.resolve(), second.consumer.resolve()])
                        finally:
                            second.close()
                    finally:
                        first.close()
                self.assertEqual(4, len(set(paths)), "every consumer owns a distinct tree")
                self.assertEqual(2, len(installs),
                                 "four private consumers of two staged sources must prepare "
                                 f"two npm installs; observed {len(installs)}")
                for argv in installs:
                    self.assertTrue({"--ignore-scripts", "--no-audit", "--no-fund"} <= set(argv))


if __name__ == "__main__":
    unittest.main()
