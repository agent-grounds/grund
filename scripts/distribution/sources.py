"""The bundled, locked sources a source build reads, for npm and for PyPI alike.

§FS-distribution-candidate.4.1 and §FS-distribution-candidate.4.2: a source build
compiles only the frontends its package installs, from sources the package carries.
The closure is this workspace narrowed to those crates and the engine, with the
workspace `Cargo.lock` pruned to them, so every build from it is `--locked`
(§FS-distribution-candidate.4.3). It is the layout the Node binding's own staging uses
(§FS-distribution.3.2.3.3), widened by the frontend it adds.
"""

import json
import re
import shutil
import subprocess
import tomllib

import matrix


def tracked(prefix):
    """Every file git tracks under `prefix`, so nothing ignored or local is bundled."""
    listed = subprocess.run(["git", "ls-files", "-z", prefix], cwd=matrix.REPO, check=True,
                            capture_output=True).stdout.decode("utf-8")
    return [name for name in listed.split("\0") if name]


def workspace_version():
    text = (matrix.REPO / "Cargo.toml").read_text(encoding="utf-8")
    return tomllib.loads(text)["workspace"]["package"]["version"]


def closure(crates, out, sha):
    """The workspace narrowed to `crates` (the last one its default member) under `out`."""
    out.mkdir(parents=True, exist_ok=True)
    members = ", ".join(f'"crates/{c}"' for c in crates)
    workspace = (matrix.REPO / "Cargo.toml").read_text(encoding="utf-8")
    workspace = re.sub(r"(?m)^members = .*$", f"members = [{members}]", workspace)
    workspace = re.sub(r"(?m)^default-members = .*$",
                       f'default-members = ["crates/{crates[-1]}"]', workspace)
    (out / "Cargo.toml").write_text(workspace, encoding="utf-8")
    for name in ("Cargo.lock", "LICENSE", "README.md"):
        shutil.copyfile(matrix.REPO / name, out / name)
    (out / "rust-toolchain.toml").write_text(
        f'[toolchain]\nchannel = "{matrix.RUST_TOOLCHAIN}"\nprofile = "minimal"\n', encoding="utf-8")
    for crate in crates:
        for name in tracked(f"crates/{crate}"):
            (out / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(matrix.REPO / name, out / name)
    (out / "source-metadata.json").write_text(
        json.dumps({"sourceSha": sha, "engineVersion": workspace_version()}, indent=2) + "\n",
        encoding="utf-8")
    # Lock the narrowed workspace once, here; every build from it is `--locked`.
    subprocess.run(["cargo", f"+{matrix.RUST_TOOLCHAIN}", "metadata", "--format-version", "1",
                    "--manifest-path", str(out / "Cargo.toml")], cwd=out, check=True,
                   stdout=subprocess.DEVNULL)
