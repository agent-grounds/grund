"""Checkout-owned terminal setup (§AR-ci-terminal-inputs.1, §FS-repository-maintenance.3).

Port the intake reproducer's setup probe: import the actual harness from two
source roots under one home and intercept Cargo, recording its target and the
binary setup selects. No Rust build or PTY is needed to observe shared output.
"""

import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

HARNESS = Path(__file__).with_name("test_watch_terminal.py")
EXE = "grund.exe" if os.name == "nt" else "grund"


@unittest.skip("red until #988 checkout-owned terminal setup; implement removes this")
class WatchTerminalInputsTests(unittest.TestCase):
    def setUp(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="grund-988-terminal-inputs-", dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.temp = Path(temporary.name).resolve()
        self.home = self.temp / "home"
        self.home.mkdir()

    def setup_from(self, name, **given):
        """Load unmodified harness bytes at another root and run its real setup."""
        root = self.temp / name
        source = root / "tests/integration/test_watch_terminal.py"
        source.parent.mkdir(parents=True)
        source.write_bytes(HARNESS.read_bytes())
        environment = {key: value for key, value in os.environ.items()
                       if key not in ("GRUND_BUILT", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")}
        environment.update(HOME=str(self.home), USERPROFILE=str(self.home), **given)
        spec = importlib.util.spec_from_file_location("terminal_setup_probe", source)
        module = importlib.util.module_from_spec(spec)
        calls = []

        def record(args, **kwargs):
            # Retain only the build input; failure diagnostics must not dump the environment.
            inherited = os.environ if kwargs.get("env") is None else kwargs["env"]
            kwargs["env"] = {key: inherited[key] for key in ("CARGO_TARGET_DIR",)
                             if key in inherited}
            calls.append(mock.call(args, **kwargs))
            return subprocess.CompletedProcess(args, 0)

        with mock.patch.dict(os.environ, environment, clear=True), \
                mock.patch.object(Path, "home", return_value=self.home), \
                mock.patch.object(subprocess, "run", side_effect=record):
            spec.loader.exec_module(module)
            module.WatchTerminalTests.setUpClass()
        binary = Path(module.WatchTerminalTests.binary)
        # A relative executable is interpreted from the invoking process, not Cargo's cwd.
        return root, calls, binary.resolve()

    def build_target(self, root, calls):
        """Record the effective Cargo target, as the original probe driver did."""
        self.assertEqual(1, len(calls), "standalone setup builds once, without cleaning")
        args, kwargs = calls[0].args[0], calls[0].kwargs
        command = [str(arg) for arg in args]
        self.assertEqual("cargo", Path(command[0]).stem)
        self.assertIn("build", command)
        self.assertIn("--locked", command)
        self.assertEqual("grund", command[command.index("-p") + 1])
        self.assertTrue(kwargs.get("check"), "Cargo failure must be an error, never a skip")
        cwd = Path(kwargs["cwd"]).resolve()
        self.assertEqual(root, cwd)
        environment = kwargs.get("env", {})
        target = environment.get("CARGO_TARGET_DIR", "target")
        for index, arg in enumerate(command):
            if arg == "--target-dir":
                target = command[index + 1]
                break
            if arg.startswith("--target-dir="):
                target = arg.split("=", 1)[1]
                break
        return (cwd / target).resolve()

    def test_two_checkouts_under_one_home_build_and_run_their_own_cli(self):
        records = [self.setup_from(name) for name in ("checkout-B", "checkout-A")]
        targets = [self.build_target(root, calls) for root, calls, _ in records]
        binaries = [binary for _, _, binary in records]
        self.assertNotEqual(targets[0], targets[1], "two checkouts selected one shared target")
        self.assertNotEqual(binaries[0], binaries[1], "two checkouts selected one shared CLI")
        for (root, _, binary), target in zip(records, targets):
            self.assertEqual(root / "target", target)
            self.assertEqual(target / "debug" / EXE, binary)

    def test_absolute_explicit_target_selects_its_built_cli(self):
        target = self.temp / "caller-target"
        root, calls, binary = self.setup_from("checkout", CARGO_TARGET_DIR=str(target))
        self.assertEqual(target, self.build_target(root, calls))
        self.assertEqual(target / "debug" / EXE, binary)

    def test_relative_explicit_target_resolves_from_checkout_not_invocation_cwd(self):
        # The test process stays outside this simulated checkout throughout setup.
        root, calls, binary = self.setup_from("checkout", CARGO_TARGET_DIR="caller-target")
        target = root / "caller-target"
        self.assertEqual(target, self.build_target(root, calls))
        self.assertEqual(target / "debug" / EXE, binary)

    def test_gate_binary_is_used_without_building_even_with_an_explicit_target(self):
        supplied = self.temp / EXE
        supplied.write_text("caller-owned artifact", encoding="utf-8")
        _, calls, binary = self.setup_from("checkout", GRUND_BUILT=str(supplied),
                                           CARGO_TARGET_DIR=str(self.temp / "unused-target"))
        self.assertEqual(0, len(calls), "GRUND_BUILT must be reused without a Cargo build")
        self.assertEqual(supplied, binary)


if __name__ == "__main__":
    unittest.main()
