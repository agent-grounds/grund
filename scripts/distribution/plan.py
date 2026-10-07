"""What one candidate holds, before anything is built.

§FS-distribution-candidate.2.1 names the thirty-nine artifacts and twenty-two
payloads of a full release, and §FS-distribution-candidate.3.1 where each payload
is placed. This module is the only spelling of either: `build` assembles exactly
this plan, and `verify` holds a candidate to it.
"""

import matrix

PRODUCTS = ("grund", "grund-lsp", "node-addon", "python-extension")
FAMILY = {"grund": "grund-cli", "grund-lsp": "grund-lsp"}
DIST = {"grund": "grund", "grund-lsp": "grund_lsp"}
WHEEL_ABI = {"grund": "cp310-abi3", "grund-lsp": "py3-none"}


def pep440(version):
    """§FS-distribution-candidate.6.2: `X.Y.Z-dev` is `X.Y.Z.dev0` on PyPI."""
    return version[:-4] + ".dev0" if version.endswith("-dev") else version


def is_development(version):
    return version.endswith("-dev") or ".dev" in version


def archive_name(product, version, row):
    suffix = ".zip" if matrix.windows(row["row"]) else ".tar.gz"
    return f"archives/{product}-{version}-{row['rust_target']}{suffix}"


def npm_name(family, version, row=None):
    if row is None:
        return f"npm/{family}-{version}.tgz"
    return f"npm/{family}-{row['npm_suffix']}-{version}.tgz"


def wheel_name(product, version, row):
    return (f"pypi/{DIST[product]}-{pep440(version)}-{WHEEL_ABI[product]}-"
            f"{matrix.wheel_tag(row)}.whl")


def sdist_name(product, version):
    return f"pypi/{DIST[product]}-{pep440(version)}.tar.gz"


def artifacts(version, rows=None):
    """§FS-distribution-candidate.2.1, in a fixed order: crates, npm, PyPI, archives.
    `rows` narrows the plan to those rows' share, plus every row-independent artifact."""
    wanted = {r["row"] for r in (rows or matrix.rows())}
    every, registry = matrix.rows(), matrix.registry_rows()
    found = [{"path": f"cargo/{name}-{version}.crate", "registry": "crates.io",
              "package": name, "kind": "crate", "row": None, "version": version}
             for name in ("grund-core", "grund", "grund-lsp")]
    for family in ("grund-cli", "grund-lsp"):
        found.append({"path": npm_name(family, version), "registry": "npm", "package": family,
                      "kind": "npm", "row": None, "version": version})
        found += [{"path": npm_name(family, version, r), "registry": "npm",
                   "package": f"@{family}/{r['npm_suffix']}", "kind": "npm", "row": r["row"],
                   "version": version} for r in registry]
    for product in ("grund", "grund-lsp"):
        found += [{"path": wheel_name(product, version, r), "registry": "pypi",
                   "package": product, "kind": "wheel", "row": r["row"],
                   "version": pep440(version)} for r in registry]
        found.append({"path": sdist_name(product, version), "registry": "pypi",
                      "package": product, "kind": "sdist", "row": None,
                      "version": pep440(version)})
    for product in ("grund", "grund-lsp"):
        found += [{"path": archive_name(product, version, r), "registry": "github",
                   "package": product, "kind": "archive", "row": r["row"], "version": version}
                  for r in every]
    return [a for a in found if a["row"] is None or a["row"] in wanted]


def placements(product, row, version):
    """§FS-distribution-candidate.3.1: (artifact, path inside it) for one payload."""
    pv, name = pep440(version), row["row"]
    if product in ("grund", "grund-lsp"):
        binary = matrix.exe(product, name)
        found = [(archive_name(product, version, row),
                  f"{product}-{version}-{row['rust_target']}/{binary}")]
        if row["registry"]:
            found.append((npm_name(FAMILY[product], version, row), f"package/bin/{binary}"))
            found.append((wheel_name(product, version, row),
                          f"{DIST[product]}-{pv}.data/scripts/{binary}"))
        return found
    if product == "node-addon":
        return [(npm_name("grund-cli", version, row), "package/grund.node")]
    extension = "_native.pyd" if matrix.windows(name) else "_native.abi3.so"
    return [(wheel_name("grund", version, row), f"grund/{extension}")]


def payloads(version, rows=None):
    """The twenty-two payloads: both executables on every row, the addon and the
    extension on the registry rows only (§FS-distribution-candidate.1.4)."""
    found = []
    for product in PRODUCTS:
        for row in rows or matrix.rows():
            if product not in ("grund", "grund-lsp") and not row["registry"]:
                continue
            found.append({"id": f"{product}-{row['row']}", "product": product,
                          "target": row["rust_target"], "row": row["row"],
                          "placements": [{"artifact": a, "path": p}
                                         for a, p in placements(product, row, version)]})
    return found


def plan(version, sha, rows=None):
    return {"version": version, "source_sha": sha,
            "rows": [r["row"] for r in (rows or matrix.rows())],
            "artifacts": artifacts(version, rows), "payloads": payloads(version, rows)}
