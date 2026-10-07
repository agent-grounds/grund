"""§FS-distribution-candidate.5.4, §FS-distribution.2 — the binding acceptance corpus,
run against the installed packages of this row rather than a checkout build.

Python: the acceptance modules of `tests/bindings` run under each promised
interpreter with the row's wheel installed from its file, so `binding()` accepts it
only because `GRUND_ACCEPTANCE_SOURCE` names that wheel. Node: the parity corpus of
`tests/bindings/node` runs against `grund-cli` installed from the loopback registry,
by the same tests with only their consumer replaced. Both compare with the
same-source oracle, so neither can pass by agreeing with itself.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6). The wheel is the one per row of §AR-bindings.6.
"""

import shutil
import subprocess
import sys
import unittest

from distribution_support import REPO
from rehearsal_support import (
    CASES, acquire, fresh_env, interpreter, keep, manifest, pythons, registry, run_checked, venv,
)

sys.path.insert(0, str(REPO / "tests" / "bindings" / "node"))
import test_parity as node_parity  # noqa: E402

BINDINGS = REPO / "tests" / "bindings"
ACCEPTANCE = ("test_api", "test_parity", "test_regressions")


class PythonApiTests(unittest.TestCase):
    """Every interpreter, the installed wheel, the whole acceptance corpus."""

    def setUp(self):
        acquire()

    def test_the_acceptance_corpus_passes_against_every_installed_wheel(self):
        for python in pythons():
            with self.subTest(python=python):
                environment, env, wheel = venv(python, "grund")
                env = dict(env, GRUND_ACCEPTANCE_SOURCE=str(wheel))
                ran = subprocess.run([str(interpreter(environment)), "-m", "unittest", "-v",
                                      *ACCEPTANCE], cwd=BINDINGS, env=env, capture_output=True,
                                     text=True, encoding="utf-8", errors="replace", timeout=3600)
                self.assertEqual(0, ran.returncode, ran.stderr[-4000:])


class InstalledConsumer:
    """The shape of `tests/bindings/node`'s `Rehearsal`, over an installed `grund-cli`."""

    def __init__(self):
        self.root = keep("grund-node-installed-")
        self.consumer = self.root / "consumer"
        self.consumer.mkdir()
        (self.consumer / "package.json").write_text('{"private":true,"type":"module"}\n',
                                                     encoding="utf-8")
        env = fresh_env(self.root / "home", npm_config_registry=registry().url)
        node = shutil.which("node", path=env["PATH"])
        major = int(run_checked([node, "--version"]).stdout.lstrip("v").split(".")[0])
        assert major in (22, 24), f"environment prerequisite: Node 22/24, got {major}"
        run_checked([shutil.which("npm", path=env["PATH"]), "install", "--no-audit", "--no-fund",
                     f"grund-cli@{manifest()['version']}"], cwd=self.consumer, env=env)
        self.package = self.consumer / "node_modules" / "grund-cli"
        self.fixture = self.root / "répo-雪"
        shutil.copytree(CASES / "json-report" / "repo", self.fixture)

    def close(self):
        pass


class InstalledNodeParityTests(node_parity.NodeParityTests):
    """The Node parity corpus, unchanged but for where `grund-cli` comes from."""

    def setUp(self):
        acquire()
        self.r = InstalledConsumer()


if __name__ == "__main__":
    unittest.main()
