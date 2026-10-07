"""The `grund` sdist's build backend: maturin, with the `grund` executable built first.

§FS-distribution-candidate.4.2: `pip install --no-binary=:all: grund` installs both the
extension and the `grund` command, as the wheel does (§FS-distribution-candidate.3.1).
maturin builds one Cargo target per wheel, so this backend builds the CLI from the same
locked sources into the wheel data maturin packs (`[tool.maturin] data`), and leaves
everything else to maturin. §FS-distribution-candidate.4.3: a missing Cargo is one
line naming it. `scripts/distribution/pypi.py` copies this file into the sdist.
"""

import os
import shutil
import subprocess
from pathlib import Path

import maturin
from maturin import *  # noqa: F401,F403 — every other PEP 517 hook is maturin's own

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent


def _build_cli():
    target = Path(os.environ.get("CARGO_TARGET_DIR") or ROOT / "target")
    try:
        subprocess.run(["cargo", "build", "--release", "--locked", "-p", "grund",
                        "--target-dir", str(target)], cwd=ROOT, check=True)
    except FileNotFoundError:
        raise SystemExit("error: cargo was not found: building grund from source needs "
                         "Rust 1.95.0 and Cargo (https://rustup.rs), a linker and the platform SDK")
    name = "grund.exe" if os.name == "nt" else "grund"
    scripts = HERE / "data" / "scripts"
    scripts.mkdir(parents=True, exist_ok=True)
    shutil.copy2(target / "release" / name, scripts / name)


def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    _build_cli()
    return maturin.build_wheel(wheel_directory, config_settings, metadata_directory)
