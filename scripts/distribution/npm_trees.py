"""Every npm package's file tree, before its payloads are placed.

§FS-distribution-candidate.2.3: `grund-cli`'s manifest is the binding's committed
fragment, `crates/grund-node/package-api.json`, plus only what assembly adds;
§FS-distribution-candidate.2.2 the selectors and exact-version optional packages;
§FS-distribution-candidate.3.1 the launchers and each platform package's `index.cjs`;
§FS-distribution-candidate.2.4 the READMEs and the licence. `build` fills in the
payloads and, with `bundle`, the locked closure `build:source` reads
(§FS-distribution-candidate.4.1).
"""

import json
import shutil
from pathlib import Path

import matrix
import readmes
import sources

NODE = matrix.REPO / "crates" / "grund-node"
HERE = Path(__file__).resolve().parent / "npm"
REPOSITORY = {"type": "git", "url": "git+https://github.com/agent-grounds/grund.git"}
HOMEPAGE = "https://github.com/agent-grounds/grund"
COMMAND = {"grund-cli": "grund", "grund-lsp": "grund-lsp"}
CLOSURE = {"grund-cli": ("grund-core", "grund-node", "grund-cli"),
           "grund-lsp": ("grund-core", "grund-lsp")}
CRATE = {"grund-cli": "grund", "grund-lsp": "grund-lsp"}


def _write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _common(name, version, description):
    return {"name": name, "version": version, "description": description, "license": "MIT",
            "repository": REPOSITORY, "homepage": HOMEPAGE}


def umbrella(family, version):
    """The `package.json` of `grund-cli` or `grund-lsp`."""
    rows = matrix.registry_rows()
    platforms = {r["rust_target"]: f"@{family}/{r['npm_suffix']}" for r in rows}
    if family == "grund-cli":
        package = _common(family, version, "The grund CLI and its Node API: check and read "
                                           "ID-based citations across docs and code")
        package["bin"] = {"grund": "launcher.cjs"}
        fragment = json.loads((NODE / "package-api.json").read_text(encoding="utf-8"))
        for key, value in fragment.items():
            package[key] = value
        package["files"] = [*fragment["files"], "launcher.cjs"]
        package["grund"] = {**fragment["grund"], "platformPackages": platforms}
    else:
        package = _common(family, version, "The grund language server, alone: diagnostics, "
                                           "hover and completion for ID-based citations")
        package["bin"] = {"grund-lsp": "launcher.cjs"}
        package["engines"] = {"node": "^22.0.0 || ^24.0.0"}
        package["files"] = ["launcher.cjs", "README.md", "LICENSE", "native", "build"]
        package["scripts"] = {"build:source": "node build/source.mjs"}
        package["grund"] = {"platformPackages": platforms}
    package["optionalDependencies"] = {name: version for name in platforms.values()}
    return package


def platform(family, version, row):
    """The `package.json` of one platform package (§FS-distribution-candidate.2.2)."""
    name = f"@{family}/{row['npm_suffix']}"
    holds = "the grund CLI and Node addon" if family == "grund-cli" else "the grund language server"
    package = _common(name, version, f"{holds} for {row['npm_suffix']}; installed by {family}")
    package.update(matrix.npm_selectors(row["npm_suffix"]))
    package["main"] = "index.cjs"
    payloads = ["bin", "grund.node"] if family == "grund-cli" else ["bin"]
    package["files"] = ["index.cjs", *payloads, "README.md", "LICENSE"]
    return package


def index(family, version, row):
    """§FS-distribution-candidate.3.1: what the loader and the launcher read."""
    binary = matrix.exe(COMMAND[family], row["row"])
    metadata = {"engineVersion": version, "packageVersion": version, "target": row["rust_target"]}
    if family == "grund-cli":
        metadata = {"apiSchemaVersion": 1, **metadata, "napiVersion": 8}
    lines = ["// §FS-distribution-candidate.3.1: where this row's payloads sit.",
             "'use strict';", "const path = require('node:path');"]
    if family == "grund-cli":
        lines.append("exports.addonPath = path.join(__dirname, 'grund.node');")
    lines += [f"exports.executablePath = path.join(__dirname, 'bin', '{binary}');",
              f"exports.metadata = Object.freeze({json.dumps(metadata)});", ""]
    return "\n".join(lines)


def write(out, version, sha=None, bundle=False):
    """Every npm package tree under `out`, one directory per package name."""
    if bundle and version != sources.workspace_version():
        raise SystemExit(f"error: sources are {sources.workspace_version()}, not {version}")
    licence = (matrix.REPO / "LICENSE").read_bytes()
    for family in ("grund-cli", "grund-lsp"):
        tree = out / family
        _write_json(tree / "package.json", umbrella(family, version))
        shutil.copyfile(HERE / "launcher.cjs", tree / "launcher.cjs")
        (tree / "LICENSE").write_bytes(licence)
        (tree / "README.md").write_text(readmes.npm_umbrella(family, version), encoding="utf-8")
        (tree / "build").mkdir(exist_ok=True)
        shutil.copyfile(HERE / "source-build.mjs", tree / "build" / "source.mjs")
        if family == "grund-cli":
            for item in sorted((NODE / "js").iterdir()):
                shutil.copyfile(item, tree / item.name)
            shutil.copyfile(NODE / "source.mjs", tree / "build" / "addon-source.mjs")
        if bundle:
            sources.closure(CLOSURE[family], tree / "build" / "sources", sha)
        for row in matrix.registry_rows():
            tree = out / f"@{family}" / row["npm_suffix"]
            _write_json(tree / "package.json", platform(family, version, row))
            (tree / "index.cjs").write_text(index(family, version, row), encoding="utf-8")
            (tree / "LICENSE").write_bytes(licence)
            (tree / "README.md").write_text(readmes.npm_platform(family, version, row),
                                            encoding="utf-8")
