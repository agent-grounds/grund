"""Test command launchers for §FS-distribution.3.2.3.3 source-package proofs."""

import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]


def command(argv, cwd=REPO, env=None, timeout=900):
    args = [str(arg) for arg in argv]
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


def checked(argv, cwd=REPO, env=None):
    result = command(argv, cwd, env)
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
