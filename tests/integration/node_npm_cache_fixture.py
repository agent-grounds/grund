"""Real npm calls under a throwaway home, and every npm cache they leave, for §AR-ci-npm-cache.4.

Run in a child so the Node harness's `support` never leaks into gate discovery. Each site runs
in a child of its own that exits normally before anything is read, because the temporary
directories go at exit. Only what compiles and the candidate are replaced; npm, `Rehearsal`,
`source_package`, `installed_consumer`, `consumer`, `build_env` and `LoopbackNpm` are real.
Nothing here names the setting a site sends its cache away with: the verdict is what npm did
with what the site gave it.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from node_consumer_fixture import launch, tarball  # It puts the Node harness on the path.

REPO = Path(__file__).resolve().parents[2]
NODE_HARNESS = REPO / "tests" / "bindings" / "node"
REHEARSAL = REPO / "tests" / "integration" / "rehearsal"

# One test process, as `NodePackageTests.setUp` and `tearDown` have it, with its compiles stubbed.
TEST_PROCESS = """
import json, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
from pathlib import Path
import launch, support

ran = []

def checked(argv, cwd=launch.REPO, env=None):
    args = [str(arg) for arg in argv]
    if "build:source" in args:
        return None
    if args[1].endswith("stage.mjs"):
        stage = Path(args[args.index("--out-dir") + 1])
        stage.mkdir(parents=True)
        (stage / "package.json").write_text(
            '{"name": "grund-cli", "version": "0.0.0", "main": "index.cjs"}', encoding="utf-8")
        (stage / "index.cjs").write_text("module.exports = 1;\\n", encoding="utf-8")
        return None
    ran.append(args[:2])
    return launch.checked(argv, cwd, env)

support.checked = checked
rehearsal = support.Rehearsal()
assert (rehearsal.package / "package.json").is_file(), "the install left no package"
caches = [str(path) for path in rehearsal.consumer.rglob("_cacache")]
installed = sorted(path.name for path in rehearsal.package.iterdir())
rehearsal.close()
print(json.dumps({"ran": ran, "caches_in_the_consumer": caches, "installed": installed}))
"""

# What the rehearsal lane's two npm installs share: a registry serving one tiny tarball.
LOOPBACK = """
import functools, os, shutil, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
from pathlib import Path
import rehearsal_support, source_install as site

site.registry = functools.lru_cache(maxsize=1)(
    lambda: rehearsal_support.LoopbackNpm(Path(sys.argv[2])))
site.manifest = lambda: {"version": "0.0.0"}
"""

# The rehearsal lane's source install (§FS-distribution-candidate.4.1), with no candidate.
SOURCE_INSTALL = LOOPBACK + """
modules = site.consumer("grund-cli", site.build_env())
assert (modules / "grund-cli" / "package.json").is_file(), "the install left no package"
"""

# The same fetch with no site in the way: the control for the read of `SOURCE_INSTALL`.
PLAIN_INSTALL = LOOPBACK + """
consumer = rehearsal_support.keep("grund-1218-plain-")
(consumer / "package.json").write_text('{"private": true}', encoding="utf-8")
env = dict(os.environ, npm_config_registry=site.registry().url)
rehearsal_support.run_checked([shutil.which("npm", path=env["PATH"]), "install",
                               "grund-cli@0.0.0"], cwd=consumer, env=env)
"""


def throwaway(root, offline=True):
    """An environment with every home and scratch under `root`, and no npm setting inherited."""
    home, scratch = root / "home", root / "tmp"
    roaming, local = home / "AppData" / "Roaming", home / "AppData" / "Local"
    # On Windows npm stats its global prefix, `%APPDATA%\\npm`, even for a local install.
    for directory in (roaming / "npm", local, scratch):
        directory.mkdir(parents=True)
    inherited = {key: value for key, value in os.environ.items()
                 if not key.lower().startswith("npm_config_")}
    settings = {"npm_config_offline": "true"} if offline else {}
    return {**inherited, **settings,
            "HOME": str(home), "USERPROFILE": str(home), "APPDATA": str(roaming),
            "LOCALAPPDATA": str(local), "XDG_CACHE_HOME": str(home / ".cache"),
            "TMPDIR": str(scratch), "TEMP": str(scratch), "TMP": str(scratch),
            "GRUND_NODE_TARGET": str(root / "target"), "npm_config_update_notifier": "false"}


def npm_caches(root):
    """The keys and content blobs of every npm cache under `root`, wherever npm put it."""
    keys, blobs = [], []
    for cache in root.rglob("_cacache"):
        for index in cache.glob("index-*/**/*"):
            if index.is_file():
                keys += [json.loads(line.split("\t", 1)[1])["key"]
                         for line in index.read_text(encoding="utf-8").splitlines() if "\t" in line]
        blobs += [path for path in cache.glob("content-*/**/*") if path.is_file()]
    return sorted(keys), blobs


class NpmCacheContract(unittest.TestCase):
    def setUp(self):
        scratch = Path.home() / "ag" / "tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temp = tempfile.TemporaryDirectory(prefix="grund-1218-npm-cache-", dir=scratch)
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name) / "throwaway"
        self.root.mkdir()
        self.source = tarball(Path(temp.name), "source")[0]

    def finish(self, script, *args, offline=True):
        """Run one process to its normal exit under the throwaway home; what it printed."""
        result = subprocess.run([sys.executable, "-c", script, *map(str, args)], cwd=REPO,
                                env=throwaway(self.root, offline), capture_output=True, text=True,
                                encoding="utf-8", timeout=300)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return result.stdout

    def assert_nothing_left(self, site):
        keys, blobs = npm_caches(self.root)
        if keys or blobs:
            listed = "".join(f"\n  {key}" for key in keys)
            self.fail(f"{site} left {len(keys)} index entries and {len(blobs)} content blobs "
                      f"in an npm cache after its process exited:{listed}")

    def assert_something_left(self, control):
        keys, blobs = npm_caches(self.root)
        if not (keys and blobs):
            self.fail(f"{control} left {len(keys)} index entries and {len(blobs)} content blobs, "
                      "so this read says nothing about the sites")

    def test_a_test_process_leaves_nothing_in_any_npm_cache(self):
        """§AR-ci-npm-cache.1: `source_package`'s pack and `installed_consumer`'s install."""
        report = json.loads(self.finish(TEST_PROCESS, NODE_HARNESS).splitlines()[-1])
        for call in (["npm", "pack"], ["npm", "install"]):
            self.assertIn(call, report["ran"], "the site was not reached with real npm")
        self.assert_nothing_left("a test process's npm pack and npm install")
        self.assertEqual([], report["caches_in_the_consumer"],
                         "a cache inside the consumer is copied by every Rehearsal")
        self.assertEqual(["index.cjs", "package.json"], report["installed"],
                         "a cache inside the stage is packed into the tarball")

    def test_the_source_install_leaves_nothing_in_any_npm_cache(self):
        """§AR-ci-npm-cache.1: `consumer` under `build_env`, from a port no later run has."""
        self.finish(SOURCE_INSTALL, REHEARSAL, self.source, offline=False)
        self.assert_nothing_left("the rehearsal's source install")

    def test_a_plain_npm_pack_shows_up_in_the_read(self):
        """§AR-ci-npm-cache.4: the control for what a test process's pack would leave."""
        packed = launch.command(["npm", "pack", "--json"], cwd=self.source,
                                env=throwaway(self.root))
        self.assertEqual(0, packed.returncode, packed.stdout + packed.stderr)
        self.assert_something_left("a plain npm pack")

    def test_a_plain_install_from_the_loopback_shows_up_in_the_read(self):
        """§AR-ci-npm-cache.4: the control for what the source install would leave."""
        self.finish(PLAIN_INSTALL, REHEARSAL, self.source, offline=False)
        self.assert_something_left("a plain npm install from the loopback registry")


if __name__ == "__main__":
    unittest.main()
