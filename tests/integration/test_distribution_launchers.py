"""§FS-distribution-candidate.3.2, §FS-distribution-candidate.3.3,
§FS-distribution-candidate.3.4, §FS-distribution.2 — the npm launchers, run for real.

The launcher trees come from `candidate.py npm-trees`, laid out the way npm installs
them, with a stand-in payload compiled here in place of the real executable: what
is under test is the launcher between the user and whatever it launches, so the
payload only reports what it was given. The launcher's output is compared with the
payload's own output under the same arguments, directory and environment.

Signals are POSIX: on Windows those tests skip with the row named, and the status,
argument and stream tests run there as everywhere.

The launcher is the plain Node script of §AR-bindings.5.
"""

import json
import os
import shutil
import signal
import subprocess
import sys
import time
import unittest
from pathlib import Path

from distribution_support import (
    VERSION, candidate, compile_payload, host_row, registry_rows, row_by_id, scratch,
)

POSIX = os.name == "posix"
NODE = shutil.which("node")
ARGS = ["check", "a b", "répo-雪", "--x=ü", ""]


PAYLOADS = {}


def setUpModule():
    """One compile of each stand-in for the whole module."""
    PAYLOADS["temp"] = scratch("grund-launcher-payloads-")
    PAYLOADS["dir"] = Path(PAYLOADS["temp"].name)
    for name in ("grund", "grund-lsp", "native"):
        compile_payload(PAYLOADS["dir"], name)


def tearDownModule():
    PAYLOADS["temp"].cleanup()


def signals_skip():
    return f"POSIX signals: row {host_row()} forwards exit status and streams only"


class LauncherFixture(unittest.TestCase):
    """One installed `node_modules` per test class, built from the candidate's trees."""

    @classmethod
    def setUpClass(cls):
        cls.temp = scratch("grund-launchers-")
        cls.root = Path(cls.temp.name)
        cls.row = row_by_id(host_row()) if host_row() else None
        cls.payloads = PAYLOADS["dir"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def setUp(self):
        self.assertTrue(NODE, "environment prerequisite missing: node")
        self.assertIsNotNone(self.row, "this host is no matrix row; launchers are not installed here")
        self.assertTrue(self.row["registry"], f"row {self.row['row']} has no npm packages")
        trees = self.root / f"trees-{self.id().rsplit('.', 1)[1]}"
        result = candidate("npm-trees", "--version", VERSION, "--out", trees)
        self.assertEqual(0, result.returncode, f"candidate.py npm-trees failed:\n{result.stderr}")
        self.modules = self.root / f"consumer-{self.id().rsplit('.', 1)[1]}" / "node_modules"
        suffix = self.row["npm_suffix"]
        for family, product in (("grund-cli", "grund"), ("grund-lsp", "grund-lsp")):
            shutil.copytree(trees / family, self.modules / family)
            platform = self.modules / f"@{family}" / suffix
            shutil.copytree(trees / f"@{family}" / suffix, platform)
            binary = product + (".exe" if os.name == "nt" else "")
            (platform / "bin").mkdir(exist_ok=True)
            shutil.copy2(self.payloads / binary, platform / "bin" / binary)
        self.work = self.modules.parent / "wörk dir"
        self.work.mkdir()

    def launcher(self, family="grund-cli"):
        package = json.loads((self.modules / family / "package.json").read_text())
        command = "grund" if family == "grund-cli" else "grund-lsp"
        return self.modules / family / package["bin"][command]

    def payload(self, product="grund"):
        return self.payloads / (product + (".exe" if os.name == "nt" else ""))

    def env(self, **extra):
        return dict(os.environ, FAKE_PAYLOAD_PROBE="π value", **extra)

    def launch(self, *args, family="grund-cli", env=None, input=b"", preload=None):
        argv = [NODE, *(["-r", str(preload)] if preload else []), str(self.launcher(family)), *args]
        return subprocess.run(argv, cwd=self.work, env=env or self.env(), input=input,
                              capture_output=True, timeout=60)

    def direct(self, *args, product="grund", env=None, input=b""):
        return subprocess.run([str(self.payload(product)), *args], cwd=self.work,
                              env=env or self.env(), input=input, capture_output=True, timeout=60)

    def assert_refused(self, result, *naming):
        """§FS-distribution-candidate.3.3: exit 2, one `error:` line, the fix named."""
        self.assertEqual(2, result.returncode, result.stderr)
        self.assertEqual(b"", result.stdout, "a refused launch runs no payload")
        lines = result.stderr.decode("utf-8").splitlines()
        self.assertEqual(1, len(lines), lines)
        self.assertTrue(lines[0].startswith("error:"), lines[0])
        self.assertIn("npm run build:source", lines[0])
        for word in naming:
            self.assertIn(word, lines[0])


class TransparencyTests(LauncherFixture):
    """§FS-distribution-candidate.3.2, §FS-distribution.2 — the launcher adds nothing."""

    def test_arguments_directory_environment_and_streams_are_the_payloads(self):
        launched, direct = self.launch(*ARGS), self.direct(*ARGS)
        self.assertEqual(direct.returncode, launched.returncode)
        self.assertEqual(direct.stdout, launched.stdout)
        self.assertEqual(direct.stderr, launched.stderr)
        report = json.loads(launched.stdout)
        self.assertEqual(ARGS, report["args"])
        self.assertEqual(str(self.work), report["cwd"])
        self.assertEqual("π value", report["probe"])
        self.assertEqual("grund", report["name"])

    def test_standard_input_reaches_the_payload_unchanged(self):
        data = "§FS-check\nü\r\n".encode() + bytes(range(1, 32))
        env = self.env(FAKE_PAYLOAD_MODE="echo-stdin")
        launched = self.launch(env=env, input=data)
        self.assertEqual(0, launched.returncode, launched.stderr)
        self.assertEqual(data, launched.stdout)

    def test_every_exit_status_is_the_payloads(self):
        for status in (0, 1, 2, 3, 101):
            with self.subTest(status=status):
                env = self.env(FAKE_PAYLOAD_STATUS=str(status))
                self.assertEqual(status, self.launch("check", env=env).returncode)

    @unittest.skipUnless(POSIX, signals_skip())
    def test_a_payload_killed_by_a_signal_kills_the_launcher_with_it(self):
        env = self.env(FAKE_PAYLOAD_MODE="raise")
        self.assertEqual(-signal.SIGTERM, self.direct(env=env).returncode)
        self.assertEqual(-signal.SIGTERM, self.launch(env=env).returncode)

    @unittest.skipUnless(POSIX, signals_skip())
    def test_int_term_and_hup_reach_the_payload(self):
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            with self.subTest(signal=sig.name):
                process = subprocess.Popen(
                    [NODE, str(self.launcher())], cwd=self.work, stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE, env=self.env(FAKE_PAYLOAD_MODE="wait"))
                self.assertEqual(b"ready\n", process.stdout.readline())
                process.send_signal(sig)
                try:
                    process.wait(timeout=30)
                finally:
                    if process.poll() is None:
                        process.kill()
                    process.stdout.close()
                    process.stderr.close()
                self.assertEqual(-sig, process.returncode)
                time.sleep(0.2)
                survivors = subprocess.run(["pgrep", "-f", str(self.payloads / "grund")],
                                           capture_output=True, text=True).stdout.split()
                self.assertEqual([], survivors, "the payload outlived its launcher")

    @unittest.skipUnless(POSIX, signals_skip())
    def test_a_closed_output_pipe_ends_both_alike(self):
        def first_byte(argv):
            process = subprocess.Popen(argv, cwd=self.work, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, env=self.env(FAKE_PAYLOAD_MODE="flood"))
            self.assertEqual(b"x", process.stdout.read(1))
            process.stdout.close()
            stderr = process.stderr.read()
            process.stderr.close()
            return process.wait(timeout=30), stderr
        self.assertEqual(first_byte([str(self.payload())]),
                         first_byte([NODE, str(self.launcher())]))


class RefusalTests(LauncherFixture):
    """§FS-distribution-candidate.3.3 — no payload, no substitute."""

    def test_a_missing_platform_package_is_refused(self):
        shutil.rmtree(self.modules / "@grund-cli" / self.row["npm_suffix"])
        self.assert_refused(self.launch("--version"), sys.platform if POSIX else "win32",
                            f"@grund-cli/{self.row['npm_suffix']}")

    def test_a_platform_package_at_another_version_is_refused(self):
        path = self.modules / "@grund-cli" / self.row["npm_suffix"] / "package.json"
        package = json.loads(path.read_text())
        package["version"] = "0.0.1"
        path.write_text(json.dumps(package))
        self.assert_refused(self.launch("--version"), "0.0.1", VERSION)

    def test_another_rows_payload_is_never_run(self):
        """A runnable payload in another row's package is still not this row's."""
        other = next(r for r in registry_rows() if r["row"] != self.row["row"])
        mine = self.modules / "@grund-cli" / self.row["npm_suffix"]
        theirs = self.modules / "@grund-cli" / other["npm_suffix"]
        trees = self.root / f"trees-{self.id().rsplit('.', 1)[1]}"
        shutil.copytree(trees / "@grund-cli" / other["npm_suffix"], theirs)
        shutil.copytree(mine / "bin", theirs / "bin", dirs_exist_ok=True)
        shutil.rmtree(mine)
        self.assert_refused(self.launch("--version"), f"@grund-cli/{self.row['npm_suffix']}")

    def test_an_unsupported_platform_is_refused_by_name(self):
        preload = self.work / "aix.cjs"
        preload.write_text("Object.defineProperty(process, 'platform', { value: 'aix' });\n"
                           "Object.defineProperty(process, 'arch', { value: 'ppc64' });\n")
        self.assert_refused(self.launch("--version", preload=preload), "aix", "ppc64")

    def test_a_matching_source_build_is_preferred(self):
        """§FS-distribution-candidate.4.1: `npm run build:source` output wins when it matches."""
        native = self.modules / "grund-cli" / "native"
        native.mkdir(exist_ok=True)
        shutil.copy2(self.payload("native"), native / ("grund" + (".exe" if os.name == "nt" else "")))
        (native / "metadata.json").write_text(json.dumps({
            "apiSchemaVersion": 1, "engineVersion": VERSION, "packageVersion": VERSION,
            "target": self.row["rust_target"], "napiVersion": 8}))
        shutil.rmtree(self.modules / "@grund-cli" / self.row["npm_suffix"])
        launched = self.launch("--version")
        self.assertEqual(0, launched.returncode, launched.stderr)
        self.assertEqual("native", json.loads(launched.stdout)["name"])


class LanguageServerLauncherTests(LauncherFixture):
    """§FS-distribution-candidate.3.4 — `grund-lsp`'s stdout is the protocol's alone."""

    def test_the_protocol_stream_passes_through_untouched(self):
        message = b'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
        frame = b"Content-Length: %d\r\n\r\n%s" % (len(message), message)
        env = self.env(FAKE_PAYLOAD_MODE="echo-stdin")
        launched = self.launch(family="grund-lsp", env=env, input=frame)
        direct = self.direct(product="grund-lsp", env=env, input=frame)
        self.assertEqual(0, launched.returncode, launched.stderr)
        self.assertEqual(frame, launched.stdout)
        self.assertEqual(direct.stderr, launched.stderr)

    def test_a_refused_launch_writes_nothing_to_stdout(self):
        shutil.rmtree(self.modules / "@grund-lsp" / self.row["npm_suffix"])
        result = self.launch(family="grund-lsp", input=b"Content-Length: 2\r\n\r\n{}")
        self.assert_refused(result, f"@grund-lsp/{self.row['npm_suffix']}")


if __name__ == "__main__":
    unittest.main()
