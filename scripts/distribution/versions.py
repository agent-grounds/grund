"""Every version source, read and written by one tool.

§FS-distribution-candidate.6.2: the Cargo manifests, `Cargo.lock` and `pyproject.toml`
carry one version, Python's spelled in PEP 440. `check` reads them, from the working
tree or from a ref (§FS-distribution.4.3: the verify job reads the release source,
not the checkout), and fails naming each one that disagrees. `set_version` writes them
all, and both bump helpers use it (§FS-distribution.4.5).

`set_version` rewrites only the lines that hold a version, so on the Cargo files it
writes exactly what `cargo set-version --workspace` did before it: the workspace
version, a crate's own literal version, every path dependency's requirement on a
member, and each member's entry in the lock. Line endings and every other byte stay.
"""

import re
import subprocess
import tomllib
from pathlib import Path

from plan import pep440

PYPROJECT = "pyproject.toml"


class Source:
    """The version files of one tree, read from disk or from one git ref."""

    def __init__(self, root, ref=None):
        self.root, self.ref = Path(root), ref

    def read(self, name):
        if self.ref is None:
            path = self.root / name
            return path.read_text(encoding="utf-8") if path.is_file() else None
        shown = subprocess.run(["git", "-C", str(self.root), "show", f"{self.ref}:{name}"],
                               capture_output=True)
        return shown.stdout.decode("utf-8") if shown.returncode == 0 else None


def _members(source):
    """Each workspace member's manifest path and package name. A member whose
    manifest is absent from the tree carries no version, so it is not read."""
    text = source.read("Cargo.toml")
    if text is None:
        raise SystemExit(f"error: no Cargo.toml at {source.ref or source.root}")
    found = {}
    for member in tomllib.loads(text)["workspace"]["members"]:
        manifest = source.read(f"{member}/Cargo.toml")
        if manifest is not None:
            found[f"{member}/Cargo.toml"] = tomllib.loads(manifest)["package"]["name"]
    return text, found


def readings(source):
    """(file, what, version) for every place a version is written down."""
    root, members = _members(source)
    workspace = tomllib.loads(root)["workspace"]["package"]["version"]
    found = [("Cargo.toml", "workspace.package.version", workspace)]
    names = set(members.values())
    for path, name in members.items():
        manifest = tomllib.loads(source.read(path))
        own = manifest["package"].get("version")
        if isinstance(own, str):
            found.append((path, f"package {name}", own))
        for table in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dep, spec in manifest.get(table, {}).items():
                if dep in names and isinstance(spec, dict) and "version" in spec:
                    found.append((path, f"{table}.{dep}", spec["version"]))
    lock = source.read("Cargo.lock")
    if lock is not None:
        for package in tomllib.loads(lock).get("package", []):
            if package["name"] in names and "source" not in package:
                found.append(("Cargo.lock", f"package {package['name']}", package["version"]))
    pyproject = source.read(PYPROJECT)
    if pyproject is not None:
        found.append((PYPROJECT, "project.version",
                      tomllib.loads(pyproject)["project"]["version"]))
    return workspace, found


def check(root, ref=None, expect=None):
    """§FS-distribution-candidate.6.2: one version, or an error naming each disagreement."""
    version, found = readings(Source(root, ref))
    where = f" at {ref}" if ref else ""
    problems = []
    for name, what, value in found:
        wanted = pep440(version) if name == PYPROJECT else version
        if value != wanted:
            problems.append(f"{name}{where} ({what}) is {value}, not {wanted}")
    if expect is not None and version != expect:
        problems.append(f"the version{where} is {version}, not the expected {expect}")
    if problems:
        raise SystemExit("".join(f"error: {p}\n" for p in problems).rstrip("\n"))
    return version


def _rewrite(path, change):
    with open(path, encoding="utf-8", newline="") as handle:
        text = handle.read()
    updated = change(text)
    if updated != text:
        with open(path, "w", encoding="utf-8", newline="") as handle:
            handle.write(updated)


def _in_table(text, header, pattern, value):
    """Replace `key = "..."` inside one `[header]` table, and nowhere else."""
    lines = text.splitlines(keepends=True)
    inside = False
    for at, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("["):
            inside = stripped == f"[{header}]"
        elif inside and re.match(pattern, line):
            lines[at] = re.sub(r'"[^"]*"', f'"{value}"', line, count=1)
            return "".join(lines)
    raise SystemExit(f"error: no {pattern} line in [{header}]")


def set_version(root, version):
    """Write `version` into every source `readings` reads (§FS-distribution.4.5)."""
    root = Path(root)
    _, members = _members(Source(root))
    names = set(members.values())
    _rewrite(root / "Cargo.toml",
             lambda t: _in_table(t, "workspace.package", r'version\s*=\s*"', version))
    dependency = re.compile(r'(?m)^((?:%s)\s*=\s*\{[^}\n]*\bversion\s*=\s*)"[^"]*"'
                            % "|".join(re.escape(n) for n in sorted(names)))
    for path in members:
        def change(text):
            if re.search(r'(?m)^version\s*=\s*"', text):
                text = _in_table(text, "package", r'version\s*=\s*"', version)
            return dependency.sub(lambda m: f'{m.group(1)}"{version}"', text)
        _rewrite(root / path, change)
    lock = re.compile(r'(?m)^(name = "(%s)"\r?\nversion = )"[^"]*"(?=\r?\n(?!source))'
                      % "|".join(re.escape(n) for n in sorted(names)))
    if (root / "Cargo.lock").is_file():
        _rewrite(root / "Cargo.lock", lambda t: lock.sub(lambda m: f'{m.group(1)}"{version}"', t))
    if (root / PYPROJECT).is_file():
        _rewrite(root / PYPROJECT,
                 lambda t: _in_table(t, "project", r'version\s*=\s*"', pep440(version)))
