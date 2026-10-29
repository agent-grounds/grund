"""Bounded Node acceptance deadlines (§AR-ci.3.4.1.1, §FS-distribution.3.2.4).

The controlled child ports #913's triage reproducer. It supplements, rather than
replaces, the real release/LTO acceptance. Imports and transport patches live in
an isolated probe process so other binding suites keep their own support modules.
"""

import contextlib
import io
import json
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]
SCALE = 500
# §AR-ci.3.4.1.1: a start-up a loaded runner can spend; the healthy child's own code starts
# no sooner than this after its launch, whatever the real start-up took.
START_UP = 1.5
SLOW_START = f"""
import sys, time
time.sleep(max(0, float(sys.argv[3]) + {START_UP} - time.time()))
"""
# argv: artifact directory, scaled build seconds, launch instant (§AR-ci.3.4.1.1).
CHILD = """
import sys, time
from pathlib import Path
deadline = float(sys.argv[3]) + float(sys.argv[2])
while time.time() < deadline:
    print('controlled build progress', flush=True)
    time.sleep(0.1)
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
(out / 'grund.node').write_bytes(b'controlled stand-in, not a native addon')
print('controlled build completed', flush=True)
"""


def probe(mode, scratch):
    """Substitute process transport, keeping the actual acceptance and launchers."""
    sys.path.insert(0, str(REPO / "tests" / "e2e"))
    import test_node_binding as acceptance
    import launch

    check = unittest.TestCase()
    real_run = subprocess.run

    def run_child(code, out, duration, **options):
        """Run a controlled child charged its scaled time from this launch (§AR-ci.3.4.1.1)."""
        # time.time(): the one clock Python documents with a reference shared across processes.
        launched = time.time()
        return real_run([sys.executable, "-c", code, str(out), str(duration), repr(launched)],
                        **options)

    if mode in ("healthy", "over-budget"):
        selected, outputs = [], []

        def controlled_run(argv, **options):
            args = list(map(str, argv))
            check.assertEqual(str(acceptance.NODE / "build.mjs"), args[1])
            check.assertEqual("release", args[args.index("--profile") + 1])
            budget = options["timeout"]
            check.assertIsNotNone(budget, "automatic build wait must stay bounded")
            selected.append(budget)
            out = Path(args[args.index("--out-dir") + 1])
            outputs.append(out)
            # An outer cap also bounds a broken policy during this regression test.
            options["timeout"] = min(6, budget / SCALE)
            duration = 1100 / SCALE if mode == "healthy" else 4000 / SCALE
            code = SLOW_START + CHILD if mode == "healthy" else CHILD
            return run_child(code, out, duration, **options)

        if mode == "healthy":
            # Same child, ample budget: fixture/tooling failures cannot masquerade as #913.
            control = scratch / "control"
            result = run_child(SLOW_START + CHILD, control, 1100 / SCALE,
                               capture_output=True, text=True, timeout=6)
            check.assertEqual(0, result.returncode, result.stderr)
            check.assertTrue((control / "grund.node").is_file())

        started = time.monotonic()
        with patch.object(acceptance, "SCRATCH", scratch), \
                patch.object(acceptance, "node_target", return_value=scratch / "unused-target"), \
                patch.object(launch.subprocess, "run", controlled_run):
            try:
                # The real test creates scratch, calls checked -> command, and checks the artifact.
                acceptance.NativeSourceBuildTests().test_documented_native_source_builder()
            except subprocess.TimeoutExpired as error:
                if mode == "healthy":
                    # §AR-ci.3.4.1.1: an overrun under the right budget is the harness's timing.
                    if selected == [1800]:
                        check.fail(f"the stand-in child overran the {error.timeout:g}s scaled cap "
                                   "of the correctly selected 1800s release budget: harness "
                                   "timing, not #913's regression")
                    check.fail(f"selected budget {selected[0]}s killed the progressing "
                               "1100-logical-second release build before its artifact")
                progress = error.output or b""
                if isinstance(progress, bytes):
                    progress = progress.decode("utf-8")
                check.assertGreaterEqual(progress.count("controlled build progress"), 5)
                check.assertEqual([1800], selected, "release must get its full finite budget")
                check.assertEqual(1800 / SCALE, error.timeout)
                check.assertLess(time.monotonic() - started, 6)
                check.assertFalse((outputs[0] / "grund.node").exists())
                return
        check.assertEqual("healthy", mode, "over-budget child must be terminated")
        check.assertEqual([1800], selected)
        return

    # No executable prerequisites: even Windows npm's JS entry is a scratch fixture.
    npm_cli = scratch / "node_modules" / "npm" / "bin" / "npm-cli.js"
    npm_cli.parent.mkdir(parents=True)
    npm_cli.write_text("", encoding="utf-8")
    cases = {
        "release": [(["node", acceptance.NODE / "build.mjs"], 1800),
                    (["node", acceptance.NODE / "build.mjs", "--profile", "release"], 1800),
                    (["npm", "run", "build:source"], 1800),
                    (["npm", "run", "build:source", "--", "--profile", "release"], 1800),
                    (["cargo", "+1.95.0", "build", "--release"], 1800)],
        "short": [(["node", acceptance.NODE / "build.mjs", "--profile", "dev"], 900),
                  (["npm", "run", "build:source", "--", "--profile", "dev"], 900),
                  (["cargo", "+1.95.0", "build", "--locked"], 900),
                  (["node", "--version"], 900), (["npm", "pack", "--json"], 900)],
    }
    observed = []
    with patch.object(launch.shutil, "which", side_effect=lambda name, **kw: str(scratch / name)), \
            patch.object(launch.subprocess, "run",
                         return_value=subprocess.CompletedProcess([], 0, "", "")) as transport, \
            contextlib.redirect_stderr(io.StringIO()):
        if mode == "override":
            for args in (["node", acceptance.NODE / "build.mjs"], ["node", "--version"]):
                for budget in (7, 180, 2400):
                    launch.command(args, timeout=budget)
                    observed.append((list(map(str, args)), "command", budget,
                                     transport.call_args.kwargs["timeout"]))
        else:
            for args, budget in cases[mode]:
                for caller in (launch.command, launch.checked):
                    caller(args)
                    observed.append((list(map(str, args)), caller.__name__, budget,
                                     transport.call_args.kwargs["timeout"]))
    print(json.dumps(observed))


class NodeBuildDeadlineTests(unittest.TestCase):
    def run_probe(self, mode):
        scratch = Path.home() / "ag" / "tmp" / "grund-913-deadline-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            result = subprocess.run([sys.executable, __file__, "--probe", mode, directory],
                                    capture_output=True, text=True, timeout=15)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return result

    def test_documented_release_build_survives_the_old_deadline(self):
        self.run_probe("healthy")

    def test_release_deadline_expires_despite_progress(self):
        self.run_probe("over-budget")

    def assert_budgets(self, mode):
        for args, caller, expected, actual in json.loads(self.run_probe(mode).stdout):
            with self.subTest(args=args, caller=caller):
                self.assertEqual(expected, actual)

    def test_release_profiles_share_the_larger_budget_in_both_launchers(self):
        self.assert_budgets("release")

    def test_development_and_non_build_commands_keep_the_short_budget(self):
        self.assert_budgets("short")

    def test_explicit_command_timeouts_take_precedence(self):
        self.assert_budgets("override")


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--probe":
        probe(sys.argv[2], Path(sys.argv[3]))
    else:
        unittest.main()
