"""The support matrix, read from the one table that defines it.

§FS-distribution-candidate.1.1: the table in the specification is the one place a
script, a workflow and a test read the rows from, so this module parses it rather
than restating it. The Linux rows' container images are the digests `release.yml`
already pins (§FS-distribution.4.8), read from that file for the same reason.
"""

import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SPEC = REPO / "docs" / "functional-spec" / "FS-distribution-candidate.md"
RELEASE = REPO / ".github" / "workflows" / "release.yml"
RUST_TOOLCHAIN = "1.95.0"


def _cell(text):
    text = text.strip()
    return None if text == "deferred" else text.strip("`")


def _images():
    """Rust target -> pinned manylinux image, from the release's build matrix."""
    text = RELEASE.read_text(encoding="utf-8")
    pairs = re.findall(r'triple: (\S+)\n(?:\s+#.*\n|\s+runner: .*\n)*\s+image: "([^"]+)"', text)
    return dict(pairs)


def rows():
    """§FS-distribution-candidate.1.1, one dict per row, in the table's order."""
    text = SPEC.read_text(encoding="utf-8")
    section = text.split("### 1.1 ", 1)[1].split("\n### ", 1)[0]
    images = _images()
    found = []
    for line in section.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 6 or not cells[0].startswith("`"):
            continue
        row, floor, target, suffix, wheel, runner = cells
        target, suffix = _cell(target), _cell(suffix)
        container = None
        if "manylinux container" in runner:
            container = images.get(target)
            if not container:
                raise SystemExit(f"error: release.yml pins no manylinux image for {target}")
        found.append({
            "row": _cell(row),
            "payload_floor": floor.split("/", 1)[1].strip(),
            "rust_target": target,
            "npm_suffix": suffix,
            "wheel_platform": _cell(wheel),
            "runner": re.search(r"`([^`]+)`", runner).group(1),
            "container": container,
            "registry": suffix is not None,
        })
    if len(found) != 6:
        raise SystemExit(f"error: {SPEC.name} section 1.1 must hold six rows, read {len(found)}")
    return found


def registry_rows():
    return [r for r in rows() if r["registry"]]


def row(name):
    for found in rows():
        if found["row"] == name:
            return found
    raise SystemExit(f"error: no matrix row {name}; rows are "
                     + ", ".join(r["row"] for r in rows()))


def windows(row_name):
    return row_name.startswith("win32")


def exe(name, row_name):
    return name + ".exe" if windows(row_name) else name


def wheel_tag(row_entry):
    """The platform tag a wheel carries on disk: auditwheel's compressed manylinux2014
    spelling on Linux, the table's own elsewhere."""
    platform = row_entry["wheel_platform"]
    if platform.startswith("manylinux2014_"):
        arch = platform[len("manylinux2014_"):]
        return f"manylinux_2_17_{arch}.{platform}"
    return platform


def npm_selectors(suffix):
    """§FS-distribution-candidate.2.2: the `os`, `cpu` and `libc` a suffix stands for."""
    parts = suffix.split("-")
    selectors = {"os": [parts[0]], "cpu": [parts[1]]}
    if parts[0] == "linux":
        selectors["libc"] = ["glibc"]
    return selectors
