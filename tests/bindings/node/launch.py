"""Test command launchers for §FS-distribution.3.2.3.3 source-package proofs."""

import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
_AUTOMATIC_TIMEOUT = object()


def command(argv, cwd=REPO, env=None, timeout=_AUTOMATIC_TIMEOUT):
    """Apply the build deadline unless the caller supplies one (§AR-ci.3.4.1.1)."""
    args = [str(arg) for arg in argv]
    if timeout is _AUTOMATIC_TIMEOUT:
        timeout = 1800 if build_profile(args) == "release" else 900
    if os.name == "nt" and args[0] == "npm":
        # npm.cmd cannot be passed to CreateProcess; invoke its JS entry directly.
        search_path = (env if env is not None else os.environ).get("PATH")
        npm = shutil.which("npm", path=search_path)
        node = shutil.which("node", path=search_path)
        if not npm or not node:
            raise FileNotFoundError("Node/npm executable missing from command PATH")
        cli = Path(npm).parent / "node_modules/npm/bin/npm-cli.js"
        if not cli.is_file():
            raise FileNotFoundError(f"npm JS entry point missing beside {npm}: {cli}")
        args = [node, str(cli), *args[1:]]
    return subprocess.run(
        args, cwd=cwd, env=env, capture_output=True, text=True,
        encoding="utf-8", timeout=timeout,
    )


def build_profile(args):
    """The profile a build resolves to, or None when the command builds nothing.

    The binding's builder and `build:source` default to release; the CLI's Cargo build
    to dev (§AR-ci.3.4.1.1).
    """
    if any(arg.endswith("build.mjs") for arg in args) or "build:source" in args:
        return args[args.index("--profile") + 1] if "--profile" in args else "release"
    if Path(args[0]).stem == "cargo" and "build" in args:
        return "release" if "--release" in args else "dev"
    return None


def checked(argv, cwd=REPO, env=None):
    args = [str(arg) for arg in argv]
    profile, started = build_profile(args), time.monotonic()
    result = command(argv, cwd, env)
    if profile:
        # §AR-ci.3.4.1.1: one line per build, so CI attributes the acceptance's time.
        target = args[args.index("--target-dir") + 1] if "--target-dir" in args else "-"
        print(f"node build: {profile} {target} {time.monotonic() - started:.1f}s",
              file=sys.stderr, flush=True)
    assert result.returncode == 0, (
        f"command failed ({result.returncode}): {' '.join(map(str, argv))}\n"
        f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    )
    return result


def cargo_probe(directory, source):
    """Put a native Cargo interceptor on PATH, forwarding argv to Python intact."""
    directory.mkdir()
    wrapper = directory / ("cargo.exe" if os.name == "nt" else "cargo")
    wrapper.with_suffix(".py").write_text(source, encoding="utf-8")
    checked(["rustc", "+1.95.0", "--edition=2024",
             Path(__file__).with_name("cargo-probe.rs"), "-o", wrapper])
    return dict(os.environ, PATH=str(directory) + os.pathsep + os.environ["PATH"],
                GRUND_TEST_PROBE_PYTHON=sys.executable)
