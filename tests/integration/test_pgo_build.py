"""§FS-distribution-candidate.5.6, §FS-distribution-candidate.7.3,
§FS-distribution-candidate.7.4 — what `scripts/pgo-build.sh` reports, with Cargo and
rustc stood in for.

The stand-ins answer as the real tools do on the host the script is told it runs
on, so every CI host checks, in seconds and without a profile-guided build, that
the evidence names files this host's Python opens, and that the one failure the
Windows arm64 row may fall back on is told apart from every other failure. The
real builds run in the manual rehearsal
(`tests/integration/rehearsal/profile_payload_provenance.py`).
"""

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PGO = REPO / "scripts" / "pgo-build.sh"
# Resolved, because on Windows a bare name finds System32's WSL `bash.exe` first.
BASH = shutil.which("bash") or "bash"
# Git Bash's wrapper puts its own directories first, so the stand-ins go in front
# from inside the shell (see `UNDER_FAKE_CURL` in test_distribution_packages.py).
UNDER_STAND_INS = 'PATH="$(cygpath -u "$1" 2>/dev/null || printf %s "$1"):$PATH"; shift; exec bash "$@"'
CRASH = "rustc crashed compiling the instrumented build"
NO_PROFILE = "training produced no profile"
ARM64 = "aarch64-pc-windows-msvc"
SHA = "0" * 40

RUSTC = """#!/bin/sh
printf 'rustc 1.95.0 (stand-in)\\nhost: %s\\nrelease: 1.95.0\\n' "$FAKE_HOST"
"""

# Replays a build's output and fails as Cargo does, or writes the payload.
CARGO = """#!/bin/sh
if [ -n "$FAKE_CARGO_LOG" ]; then cat "$FAKE_CARGO_LOG" >&2; exit 101; fi
while [ $# -gt 0 ]; do [ "$1" = --target-dir ] && target="$2"; shift; done
mkdir -p "$target/release" && printf 'payload\\n' > "$target/release/$FAKE_ARTIFACT"
"""

ERROR = "\x1b[1m\x1b[91merror\x1b[0m"
RUSTC_EXE = r"C:\Users\runneradmin\.rustup\toolchains\1.95.0-aarch64-pc-windows-msvc\bin\rustc.exe"
GENERATE = "'-Cprofile-generate=C:/a/_temp/target/pgo-data/grund/raw'"
VIOLATION = "(exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)"


def crashed(crate, flags=GENERATE, status=VIOLATION):
    """Cargo's account of one crate whose rustc did not exit successfully, as
    windows-11-arm runners print it."""
    return (f"{ERROR}: could not compile `{crate}` (lib)\n\nCaused by:\n"
            f"  process didn't exit successfully: `{RUSTC_EXE} --crate-name {crate.replace('-', '_')}"
            f" --edition=2024 --crate-type lib -C opt-level=3 {flags}` {status}\n")


def compile_error(crate):
    return (f"{ERROR}[E0425]: cannot find value `x` in this scope\n"
            f"{ERROR}: could not compile `{crate}` (lib) due to 1 previous error\n")


# Two crates crashing, as release run 37355002760 and rehearsal run 37710056388 met it.
CRASH_LOG = ("   Compiling grund-core v0.16.2-dev\n   Compiling grund v0.16.2-dev\n"
             + crashed("grund")
             + "\x1b[1m\x1b[93mwarning\x1b[0m: build failed, waiting for other jobs to finish...\n"
             + crashed("grund-core"))


class StandIns(unittest.TestCase):
    """A scratch root with the stand-in `rustc` and `cargo`, and the script run under them."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="grund-pgo-stand-in-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        tools = self.root / "tools"
        tools.mkdir()
        for name, text in (("rustc", RUSTC), ("cargo", CARGO)):
            (tools / name).write_text(text, encoding="utf-8", newline="\n")
            (tools / name).chmod(0o755)

    def pgo(self, host, log=None, *args, artifact="grund.exe"):
        env = {k: v for k, v in os.environ.items() if k not in ("RUSTC", "RUSTFLAGS", "FAKE_CARGO_LOG")}
        env.update(FAKE_HOST=host, FAKE_ARTIFACT=artifact)
        if log is not None:
            path = self.root / "cargo.log"
            path.write_text(log, encoding="utf-8", newline="\n")
            env["FAKE_CARGO_LOG"] = path.as_posix()
        argv = [BASH, "-c", UNDER_STAND_INS, "bash", str(self.root / "tools"), str(PGO),
                "--target-dir", str(self.root / "target"), "--sha", SHA, *map(str, args)]
        return subprocess.run(argv, capture_output=True, text=True, encoding="utf-8",
                              errors="replace", env=env, timeout=600)


class EvidenceTests(StandIns):
    """§FS-distribution-candidate.7.3: `candidate.py build` keeps the payload the
    evidence names, so on a Windows row it must be a path native Python opens, not
    Git Bash's `/c/...` (§FS-distribution-candidate.5.6)."""

    def test_the_evidence_names_files_this_host_opens(self):
        host, artifact = {"nt": ("x86_64-pc-windows-msvc", "grund_node.dll")}.get(
            os.name, ("x86_64-unknown-linux-gnu", "libgrund_node.so"))
        key = (f'{{"compiler": "rustc 1.95.0", "target": "{host}", "source_sha": "{SHA}", '
               '"product": "node-addon", "features": [], "abi": null}')
        profile = self.root / "node-addon.profdata"
        profile.write_bytes(b"profile\n")
        Path(f"{profile}.key").write_text(f"key: {key}\ntraining: [\"stand-in\"]\n"
                                          "training_sha256: t\ngenerate_sha256: g\n",
                                          encoding="utf-8", newline="\n")
        evidence = self.root / "evidence.json"
        result = self.pgo(host, None, "--product", "node-addon", "--profile", profile,
                          "--evidence", evidence, artifact=artifact)
        self.assertEqual(0, result.returncode, result.stderr)
        record = json.loads(evidence.read_text(encoding="utf-8"))
        payload = Path(record["payload"])
        self.assertTrue(payload.is_file(), f"evidence names {record['payload']}")
        self.assertEqual(artifact, payload.name)
        self.assertEqual(hashlib.sha256(payload.read_bytes()).hexdigest(), record["build_sha256"])
        self.assertTrue(Path(record["profile"]).is_file(), f"evidence names {record['profile']}")
        self.assertTrue(Path(record["profile"]).samefile(profile))


class FailureTests(StandIns):
    """§FS-distribution-candidate.7.4: only rustc's crash on the Windows arm64 host exits 3."""

    def test_the_crash_on_the_windows_arm64_host_exits_3(self):
        result = self.pgo(ARM64, CRASH_LOG)
        self.assertEqual(3, result.returncode, result.stderr)
        self.assertIn(CRASH, result.stderr)
        self.assertIn("could not compile `grund-core`", result.stderr, "the build's own output")

    def test_the_crash_on_another_host_is_an_ordinary_failure(self):
        result = self.pgo("x86_64-pc-windows-msvc", CRASH_LOG)
        self.assertEqual(101, result.returncode, result.stderr)
        self.assertNotIn(CRASH, result.stderr)

    def test_a_compile_error_is_not_the_crash(self):
        result = self.pgo(ARM64, compile_error("grund"))
        self.assertEqual(101, result.returncode, result.stderr)
        self.assertNotIn(CRASH, result.stderr)

    def test_a_compile_error_beside_a_crash_is_not_the_crash(self):
        result = self.pgo(ARM64, crashed("grund") + compile_error("grund-core"))
        self.assertEqual(101, result.returncode, result.stderr)
        self.assertNotIn(CRASH, result.stderr)

    def test_a_crash_without_the_instrumentation_is_not_the_crash(self):
        result = self.pgo(ARM64, crashed("grund", flags="-C strip=debuginfo"))
        self.assertEqual(101, result.returncode, result.stderr)
        self.assertNotIn(CRASH, result.stderr)

    def test_another_exit_status_is_not_the_crash(self):
        result = self.pgo(ARM64, crashed("grund", status="(exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)"))
        self.assertEqual(101, result.returncode, result.stderr)
        self.assertNotIn(CRASH, result.stderr)

    def test_training_that_writes_no_profile_is_not_the_crash(self):
        """The instrumented build succeeds, but its payload cannot run, so no profile."""
        result = self.pgo(ARM64)
        self.assertEqual(1, result.returncode, result.stderr)
        self.assertIn(NO_PROFILE, result.stderr)
        self.assertNotIn(CRASH, result.stderr)


if __name__ == "__main__":
    unittest.main()
