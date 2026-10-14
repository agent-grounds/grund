"""§FS-distribution.3.3.7, §FS-distribution-candidate.4.2 — PyPI staging strips the
binding's crate-local prefix: the `grund` wheel carries the API package at `grund/`
and the sdist at `python/grund/`, so where the checkout keeps the binding never
reaches what is published.

The archives are staged by `scripts/distribution/pypi.py` itself from a fixture tree
that holds the binding where the checkout does, `crates/grund-py/python/grund/`.
Nothing compiles: the closure's Cargo half is stood in for by the fixture's files, so
what is asserted is only where staging puts the Python package.
"""

import io
import shutil
import subprocess
import sys
import tarfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

from distribution_support import REPO, SHA, VERSION, scratch

sys.path.insert(0, str(REPO / "scripts" / "distribution"))
import matrix  # noqa: E402
import pypi  # noqa: E402
import sources  # noqa: E402

PACKAGE = ("__init__.py", "py.typed", "types.py")
# A tree is laid as the checkout lays it; these are the files the archives read.
FIXTURE = {
    "LICENSE": "MIT\n",
    "crates/grund-py/Cargo.toml": '[package]\nname = "grund-py"\nversion = "0.0.0"\n',
    "crates/grund-py/pyproject.toml": '[project]\nname = "grund"\nversion = "0.0.0"\n',
    **{f"crates/grund-py/python/grund/{name}": f"# {name}\n" for name in PACKAGE},
}


def closure(crates, out, sha):
    """The bundled closure's file copy, without the Cargo lock it also writes."""
    for crate in crates:
        for name in sources.tracked(f"crates/{crate}"):
            (out / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(matrix.REPO / name, out / name)


class StagingTests(unittest.TestCase):
    """The archive paths of the `grund` wheel and sdist, staged from a fixture tree."""

    def setUp(self):
        temp = scratch("grund-pypi-staging-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.tree = self.root / "tree"
        for name, text in FIXTURE.items():
            (self.tree / name).parent.mkdir(parents=True, exist_ok=True)
            (self.tree / name).write_text(text)
        git = ["git", "-C", str(self.tree), "-c", "user.name=t", "-c", "user.email=t@t",
               "-c", "maintenance.auto=false", "-c", "gc.auto=0", "-c", "core.fsmonitor=false"]
        subprocess.run([*git, "init", "-q"], check=True)
        subprocess.run([*git, "add", "-A"], check=True)
        for patch in (mock.patch.object(matrix, "REPO", self.tree),
                      mock.patch.object(sources, "closure", closure)):
            patch.start()
            self.addCleanup(patch.stop)

    def test_the_wheel_carries_the_package_at_grund(self):
        path = self.root / "grund.whl"
        pypi.wheel(path, "grund", VERSION, {"wheel_platform": "win_amd64"}, [])
        names = zipfile.ZipFile(io.BytesIO(path.read_bytes())).namelist()
        self.assertEqual(sorted(f"grund/{name}" for name in PACKAGE),
                         sorted(n for n in names if not n.split("/")[0].endswith(".dist-info")))
        for name in names:
            with self.subTest(member=name):
                self.assertFalse(name.startswith(("python/", "crates/")), name)

    def test_the_sdist_carries_the_package_at_python_grund(self):
        path = self.root / "grund.tar.gz"
        pypi.sdist(path, "grund", VERSION, SHA, self.root / "work")
        top = f"grund-{VERSION}/"
        with tarfile.open(path) as archive:
            names = archive.getnames()
        for name in PACKAGE:
            with self.subTest(member=name):
                self.assertIn(f"{top}python/grund/{name}", names)
        for name in names:
            with self.subTest(member=name):
                self.assertTrue(name.startswith(top), name)
                self.assertFalse(name.startswith((f"{top}grund/", f"{top}crates/grund-py/python/")),
                                 name)
        # The generated pyproject is the build's; the checkout's stays out of it.
        self.assertIn(f"{top}pyproject.toml", names)
        self.assertNotIn(f"{top}crates/grund-py/pyproject.toml", names)


if __name__ == "__main__":
    unittest.main()
