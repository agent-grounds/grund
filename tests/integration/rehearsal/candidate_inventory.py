"""§FS-distribution-candidate.2.1, §FS-distribution-candidate.1.3,
§FS-distribution-candidate.6.4, §FS-distribution-candidate.7.3,
§FS-distribution.4.8 — the candidate of one row, read off the bytes it holds.

The ordinary gate holds `candidate.py` to these rules on a synthetic candidate; this
module holds a real one to them: the payloads are executables that run, their
linkage stays at or under the row's floor, and every byte the manifest records is
the byte on disk. The floor is read from the payload itself — glibc symbol versions,
the macOS deployment target, the Windows runtime imports — never from a build flag.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import os
import re
import shutil
import subprocess
import tarfile
import unittest
import zipfile

from distribution_support import (
    REPO, candidate, candidate_json, expected_artifacts, expected_payloads, sha256,
)
from rehearsal_support import acquire, keep, manifest, row, run_checked

FLOORS = {"glibc": (2, 17), "macos": (11, 0)}
# Debug or non-14 MSVC runtimes are not the Windows floor (§FS-distribution-candidate.1.3).
FOREIGN_RUNTIME = re.compile(r"(?i)^(msvcr\d+|msvcp(?!140)\d+|vcruntime(?!140)\d+|\w+d\.dll$|ucrtbased)")


def member(path, inner):
    """The bytes at one placement inside one artifact."""
    path = acquire() / path
    if path.name.endswith((".whl", ".zip")):
        with zipfile.ZipFile(path) as package:
            return package.read(inner)
    with tarfile.open(path, "r:gz") as package:
        return package.extractfile(inner).read()


def row_payloads():
    return [p for p in manifest()["payloads"] if p["row"] == row()["row"]]


def deployment_targets(load):
    """The macOS versions `otool -l` records as the deployment target: LC_BUILD_VERSION's
    `minos`, or LC_VERSION_MIN_MACOSX's `version`. A dylib's LC_ID_DYLIB `current version`
    and LC_BUILD_VERSION's tool `version`s are not one."""
    field = {"LC_BUILD_VERSION": "minos", "LC_VERSION_MIN_MACOSX": "version"}
    targets = []
    for command in load.split("Load command")[1:]:
        name = re.search(r"(?m)^\s*cmd (\S+)", command)
        if name and name.group(1) in field:
            found = re.findall(rf"(?m)^\s*{field[name.group(1)]} (\d+)\.(\d+)", command)
            targets += [tuple(map(int, version)) for version in found]
    return targets


class InventoryTests(unittest.TestCase):
    """§FS-distribution-candidate.2.1: nothing unplanned, nothing missing."""

    def setUp(self):
        acquire()

    def test_the_candidate_holds_exactly_this_rows_share_of_the_plan(self):
        version = manifest()["version"]
        mine = {row()["row"], None}
        planned = {a["path"] for a in expected_artifacts(version) if a["row"] in mine}
        self.assertEqual(planned, {a["path"] for a in manifest()["artifacts"]})
        on_disk = {p.relative_to(acquire()).as_posix() for p in acquire().rglob("*") if p.is_file()}
        on_disk = {p for p in on_disk - {"manifest.json"}
                   if not p.startswith("receipts/") and not p.endswith(".sha256")}
        self.assertEqual(planned, on_disk)

    def test_the_payloads_are_the_plans(self):
        planned = {p["id"] for p in expected_payloads() if p["row"] == row()["row"]}
        self.assertEqual(planned, {p["id"] for p in manifest()["payloads"]})

    def test_the_tool_verifies_the_candidate(self):
        result = candidate("verify", acquire(), "--sha", manifest()["source_sha"])
        self.assertEqual(0, result.returncode, result.stderr)

    def test_the_candidate_is_this_checkouts(self):
        """No drift: the commit, the version and every recorded digest are this tree's."""
        head = run_checked(["git", "rev-parse", "HEAD"]).stdout.strip()
        self.assertEqual(head, manifest()["source_sha"])
        versions = candidate("versions", "--root", REPO, "--expect", manifest()["version"])
        self.assertEqual(0, versions.returncode, versions.stderr)
        for item in manifest()["artifacts"]:
            with self.subTest(artifact=item["path"]):
                self.assertEqual(item["sha256"], sha256((acquire() / item["path"]).read_bytes()))

    def test_every_placement_holds_the_recorded_payload(self):
        """§FS-distribution-candidate.7.3: the placed bytes are the build's, or a recorded
        transformation of them."""
        for payload in row_payloads():
            for placement in payload["placements"]:
                with self.subTest(payload=payload["id"], artifact=placement["artifact"]):
                    data = member(placement["artifact"], placement["path"])
                    self.assertEqual(placement["sha256"], sha256(data))
                    if not placement["transformations"]:
                        self.assertEqual(payload["build_sha256"], placement["sha256"])

    def test_the_payloads_are_real_executables_of_this_version(self):
        version = manifest()["version"]
        for payload in row_payloads():
            if payload["product"] not in ("grund", "grund-lsp"):
                continue
            with self.subTest(payload=payload["id"]):
                placement = payload["placements"][0]
                binary = keep("grund-payload-") / f"run-{payload['id']}"
                binary.write_bytes(member(placement["artifact"], placement["path"]))
                binary.chmod(0o755)
                answer = run_checked([binary, "--version"]).stdout.strip()
                self.assertEqual(f"{payload['product']} {version}", answer)


class FloorTests(unittest.TestCase):
    """§FS-distribution-candidate.1.3: each payload is inspected against its row's floor."""

    def setUp(self):
        acquire()
        self.dump = keep("grund-floors-")

    def payload_files(self):
        for payload in row_payloads():
            placement = payload["placements"][0]
            path = self.dump / f"{payload['id']}{os.path.splitext(placement['path'])[1]}"
            path.write_bytes(member(placement["artifact"], placement["path"]))
            yield payload["id"], path

    def test_no_payload_needs_more_than_its_rows_floor(self):
        system = row()["row"].split("-", 1)[0]
        for payload, path in self.payload_files():
            with self.subTest(payload=payload):
                if system == "linux":
                    symbols = run_checked(["objdump", "-T", path]).stdout
                    needed = {tuple(map(int, v.split("."))) for v in re.findall(r"GLIBC_([\d.]+)", symbols)}
                    self.assertTrue(needed, "a Linux payload with no glibc symbol versions was not read")
                    self.assertLessEqual(max(needed), FLOORS["glibc"], f"{payload} needs glibc {max(needed)}")
                elif system == "darwin":
                    load = run_checked(["otool", "-l", path]).stdout
                    targets = deployment_targets(load)
                    self.assertTrue(targets, "no deployment target recorded")
                    self.assertLessEqual(max(targets), FLOORS["macos"], f"{payload} targets macOS {max(targets)}")
                else:
                    imports = run_checked(["dumpbin", "/dependents", path]).stdout
                    dlls = re.findall(r"(?im)^\s+(\S+\.dll)\s*$", imports)
                    self.assertTrue(dlls, "no imports read")
                    self.assertEqual([], [d for d in dlls if FOREIGN_RUNTIME.match(d)])

    def test_linux_payloads_run_inside_the_old_container(self):
        if not row()["container"]:
            self.skipTest(f"the old-container run is for the Linux rows; this row is {row()['row']}")
        docker = shutil.which("docker")
        self.assertTrue(docker, "environment prerequisite missing: docker")
        image = next(r["container"] for r in candidate_json("matrix") if r["row"] == row()["row"])
        version = manifest()["version"]
        for payload, path in self.payload_files():
            if not payload.startswith(("grund-linux", "grund-lsp-linux")):
                continue
            with self.subTest(payload=payload):
                path.chmod(0o755)
                ran = subprocess.run([docker, "run", "--rm", "-v", f"{self.dump}:/p:ro", image,
                                      f"/p/{path.name}", "--version"], capture_output=True, text=True)
                self.assertEqual(0, ran.returncode, ran.stderr)
                self.assertIn(version, ran.stdout)


if __name__ == "__main__":
    unittest.main()
