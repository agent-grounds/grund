"""Real npm calls under a throwaway home, and every npm cache they leave, for §AR-ci-npm-cache.4.

Run in a child so the Node harness's `support` never leaks into gate discovery. Each site runs
in a child of its own that exits normally before anything is read, because the temporary
directories go at exit. Only what compiles and the candidate are replaced; npm, `Rehearsal`,
`source_package`, `installed_consumer`, `consumer`, `build_env` and `LoopbackNpm` are real.
Nothing here names the setting a site sends its cache away with: a setting named here is one a
case exports for the site to beat, and the verdict is what npm did with what the site gave it.
"""

import json
import os
from pathlib import Path
import socket
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

# Both spellings with no site in the way: the control for the cases that export them. Its order
# is read where each case's npm would get it, out of this child's own environment.
BOTH_SPELLINGS = """
import json, os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import launch

SPELLINGS = "Object.keys(process.env).filter(k => k.toLowerCase() === 'npm_config_cache').join()"

def asked(*argv, cwd=launch.REPO):
    done = launch.command(argv, cwd=cwd, env=dict(os.environ))
    assert done.returncode == 0, done.stdout + done.stderr
    return done.stdout.strip()

report = {"python": ",".join(key for key in os.environ if key.lower() == "npm_config_cache"),
          "node": asked("node", "-p", SPELLINGS),
          "cache": asked("npm", "config", "get", "cache"),
          "registry": asked("npm", "config", "get", "registry")}
asked("npm", "pack", "--json", cwd=sys.argv[2])
print(json.dumps(report))
"""


def both_spellings(setting, lower, upper):
    """A setting exported twice, lower-case first, as bash, fish and a pyenv shim hand it on."""
    return {f"npm_config_{setting}": str(lower), f"NPM_CONFIG_{setting.upper()}": str(upper)}


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

    def finish(self, script, *args, offline=True, exported=None):
        """Run one process to its normal exit under the throwaway home; what it printed.

        `exported` is what the contributor would have, in its order, after everything else.
        `sys.executable` keeps that order, where a `python3` on `PATH` may be a shim in bash.
        """
        env = {**throwaway(self.root, offline), **(exported or {})}
        result = subprocess.run([sys.executable, "-c", script, *map(str, args)], cwd=REPO,
                                env=env, capture_output=True, text=True,
                                encoding="utf-8", timeout=300)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return result.stdout

    def assert_nothing_left(self, site):
        keys, blobs = npm_caches(self.root)
        if keys or blobs:
            listed = "".join(f"\n  {key}" for key in keys)
            listed += "".join(f"\n  {blob.relative_to(self.root)}" for blob in blobs)
            self.fail(f"{site} left {len(keys)} index entries and {len(blobs)} content blobs "
                      f"in an npm cache after its process exited:{listed}")

    def assert_something_left(self, control):
        keys, blobs = npm_caches(self.root)
        if not (keys and blobs):
            self.fail(f"{control} left {len(keys)} index entries and {len(blobs)} content blobs, "
                      "so this read says nothing about the sites")

    def exported_caches(self):
        """Both spellings of the cache, each under the root, so the verdict's read reaches them.

        On Windows `os.environ` folds names and one value survives; either fails a site that
        obeys it, so nothing here depends on which.
        """
        return both_spellings("cache", self.root / "exported-lower", self.root / "exported-upper")

    def dead_registry(self):
        """A loopback URL that refuses: its port stays bound for the test, and nothing listens."""
        dead = socket.socket()
        self.addCleanup(dead.close)
        dead.bind(("127.0.0.1", 0))
        return f"http://127.0.0.1:{dead.getsockname()[1]}/"

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

    def test_a_test_process_beats_both_exported_spellings_of_the_cache(self):
        """§AR-ci-npm-cache.1.1: `source_package`'s pack and `installed_consumer`'s install."""
        printed = self.finish(TEST_PROCESS, NODE_HARNESS, exported=self.exported_caches())
        report = json.loads(printed.splitlines()[-1])
        for call in (["npm", "pack"], ["npm", "install"]):
            self.assertIn(call, report["ran"], "the site was not reached with real npm")
        self.assert_nothing_left("a test process that inherited both spellings of the cache")

    def test_the_source_install_beats_both_exported_spellings_of_the_cache(self):
        """§AR-ci-npm-cache.1.1: `consumer` under `build_env`, from a port no later run has."""
        self.finish(SOURCE_INSTALL, REHEARSAL, self.source, offline=False,
                    exported=self.exported_caches())
        self.assert_nothing_left("a source install that inherited both spellings of the cache")

    def test_the_source_install_beats_both_exported_spellings_of_the_registry(self):
        """§AR-ci-npm-cache.1.1: `build_env`'s registry, or the install cannot reach `LoopbackNpm`.

        Both spellings refuse, so on Windows it does not matter which one survives, and retries
        are off so that a loser fails at once rather than after npm's backoff.
        """
        dead = self.dead_registry()
        self.finish(SOURCE_INSTALL, REHEARSAL, self.source, offline=False,
                    exported={**both_spellings("registry", dead, dead),
                              "npm_config_fetch_retries": "0"})
        self.assert_nothing_left("a source install that inherited both spellings of the registry")

    @unittest.skipIf(os.name == "nt", "os.environ folds names on Windows, so two spellings of a "
                                      "setting cannot reach this control")
    def test_with_no_site_in_the_way_npm_obeys_the_spelling_exported_last(self):
        """§AR-ci-npm-cache.4: the control for what the cases above export."""
        caches = self.exported_caches()
        lower, upper = map(Path, caches.values())
        dead = self.dead_registry()
        exported = {**caches, **both_spellings("registry", f"{dead}lower/", f"{dead}upper/")}
        report = json.loads(self.finish(BOTH_SPELLINGS, NODE_HARNESS, self.source,
                                        exported=exported).splitlines()[-1])
        order = ",".join(caches)
        self.assertEqual(order, report["python"], "the order did not reach the child's Python")
        self.assertEqual(order, report["node"], "a launcher reordered the environment for node")
        self.assertEqual(str(upper), report["cache"], "npm did not take the last spelling")
        self.assertEqual(f"{dead}upper/", report["registry"], "npm did not take the last spelling")
        self.assertEqual(([], []), npm_caches(lower), "npm filed something under the first one")
        keys, blobs = npm_caches(upper)
        self.assertTrue(keys and blobs, "a plain npm pack filed nothing under the last spelling")


if __name__ == "__main__":
    unittest.main()
