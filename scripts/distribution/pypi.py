"""The two PyPI distributions: a wheel per registry row, and an sdist.

§FS-distribution-candidate.3.1: the `grund` wheel carries the PGO-built executable as
wheel data in `scripts/` and the abi3 extension inside the package, so an installer
puts the payload itself on `PATH`; the `grund-lsp` wheel carries only the server.
Wheels are packed from the profile-use payloads directly — the extension is built by
`scripts/pgo-build.sh` with the binding's own `extension-module` feature and abi3 floor
(§AR-bindings.6), and the executable cannot pass through a second build at all.
§FS-distribution-candidate.4.2: each sdist carries the locked closure of what its
wheel holds and builds it with maturin; `grund`'s backend builds the CLI first and
hands it to maturin as wheel data. §FS-distribution-candidate.2.4: the README, the
licence and this repository's URL travel in the metadata of both.
"""

import base64
import hashlib
import shutil

import matrix
import packing
import plan
import readmes
import sources

HERE = matrix.REPO / "scripts" / "distribution"
SUMMARY = {"grund": "The grund CLI and its Python API: check and read ID-based citations "
                    "across docs and code",
           "grund-lsp": "The grund language server, alone: diagnostics, hover and completion "
                        "for ID-based citations"}
CRATES = {"grund": ("grund-core", "grund-cli", "grund-py"), "grund-lsp": ("grund-core", "grund-lsp")}


def metadata(product, version):
    """Core metadata, shared by the wheel's `METADATA` and the sdist's `PKG-INFO`."""
    lines = ["Metadata-Version: 2.1", f"Name: {product}", f"Version: {plan.pep440(version)}",
             f"Summary: {SUMMARY[product]}", f"Home-page: {readmes.REPO}", "License: MIT",
             f"Project-URL: Repository, {readmes.REPO}",
             "Classifier: License :: OSI Approved :: MIT License"]
    if product == "grund":
        lines += ["Requires-Python: >=3.10",
                  "Classifier: Programming Language :: Python :: Implementation :: CPython"]
    lines.append("Description-Content-Type: text/markdown")
    return ("\n".join(lines) + "\n\n" + readmes.pypi(product, version)).encode("utf-8")


def _digest(data):
    return base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()


def wheel(path, product, version, row, placed):
    """One row's wheel: the placed payloads, the API package for `grund`, and dist-info."""
    dist, pv = plan.DIST[product], plan.pep440(version)
    info = f"{dist}-{pv}.dist-info"
    entries = list(placed)
    if product == "grund":
        entries += [(name.removeprefix("python/"), (matrix.REPO / name).read_bytes(), packing.FILE)
                    for name in sources.tracked("python/grund")]
    tag = f"{plan.WHEEL_ABI[product]}-{matrix.wheel_tag(row)}"
    header = f"Wheel-Version: 1.0\nGenerator: grund-candidate\nRoot-Is-Purelib: false\nTag: {tag}\n"
    entries += [(f"{info}/METADATA", metadata(product, version), packing.FILE),
                (f"{info}/WHEEL", header.encode(), packing.FILE),
                (f"{info}/LICENSE", (matrix.REPO / "LICENSE").read_bytes(), packing.FILE)]
    record = "".join(f"{name},sha256={_digest(data)},{len(data)}\n"
                     for name, data, _ in sorted(entries)) + f"{info}/RECORD,,\n"
    entries.append((f"{info}/RECORD", record.encode(), packing.FILE))
    packing.zip_file(path, entries)


PYPROJECT = """[build-system]
requires = ["maturin>=1.9,<2"]
build-backend = "{backend}"
{backend_path}
[project]
name = "{name}"
version = "{version}"
description = "{summary}"
readme = "README.md"
license = "MIT"
{requires}
[project.urls]
Repository = "{repository}"

[tool.maturin]
manifest-path = "crates/{crate}/Cargo.toml"
locked = true
{maturin}"""
MATURIN = {"grund": 'python-source = "python"\nmodule-name = "grund._native"\n'
                    'features = ["extension-module"]\ndata = "build_backend/data"\n',
           "grund-lsp": 'bindings = "bin"\n'}


def sdist(path, product, version, sha, work):
    """§FS-distribution-candidate.4.2: everything the build reads, and nothing it does not."""
    dist, pv = plan.DIST[product], plan.pep440(version)
    tree = work / f"sdist-{dist}"
    shutil.rmtree(tree, ignore_errors=True)
    sources.closure(CRATES[product], tree, sha)
    grund = product == "grund"
    if grund:
        for name in sources.tracked("python/grund"):
            (tree / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(matrix.REPO / name, tree / name)
        (tree / "build_backend").mkdir()
        shutil.copyfile(HERE / "grund_build.py", tree / "build_backend" / "grund_build.py")
    (tree / "pyproject.toml").write_text(PYPROJECT.format(
        backend="grund_build" if grund else "maturin",
        backend_path='backend-path = ["build_backend"]\n' if grund else "",
        name=product, version=pv, summary=SUMMARY[product], repository=readmes.REPO,
        requires='requires-python = ">=3.10"\n' if grund else "",
        crate=CRATES[product][-1], maturin=MATURIN[product]), encoding="utf-8")
    (tree / "README.md").write_text(readmes.pypi(product, version), encoding="utf-8")
    (tree / "PKG-INFO").write_bytes(metadata(product, version))
    packing.tgz(path, packing.tree(tree, f"{dist}-{pv}/"))
