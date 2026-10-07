"""Deterministic packing: the same files always give the same bytes.

§FS-distribution-candidate.6.1: an artifact's digest is part of the candidate's
identity, so nothing about the moment or the machine of packing reaches it — every
entry has one fixed time and owner, entries are sorted, and gzip records no time or
name. Every entry is `(name, bytes, mode)`; an executable payload is `0755`.
"""

import gzip
import io
import tarfile
import zipfile
from pathlib import Path

# npm's own fixed tarball time (1985-10-26); zip cannot go below 1980.
MTIME = 499162500
ZIP_TIME = (1980, 1, 1, 0, 0, 0)
FILE, EXECUTABLE = 0o644, 0o755


def tree(root, prefix, executables=()):
    """Every file under `root`, named `prefix` + its relative path."""
    root = Path(root)
    found = []
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        relative = path.relative_to(root).as_posix()
        mode = EXECUTABLE if relative in executables else FILE
        found.append((prefix + relative, path.read_bytes(), mode))
    return found


def tgz(path, entries):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w", format=tarfile.PAX_FORMAT) as tar:
        for name, data, mode in sorted(entries):
            info = tarfile.TarInfo(name)
            info.size, info.mode, info.mtime = len(data), mode, MTIME
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            tar.addfile(info, io.BytesIO(data))
    packed = io.BytesIO()
    with gzip.GzipFile(filename="", fileobj=packed, mode="wb", compresslevel=9, mtime=0) as out:
        out.write(buffer.getvalue())
    _write(path, packed.getvalue())


def zip_file(path, entries):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data, mode in sorted(entries):
            info = zipfile.ZipInfo(name, date_time=ZIP_TIME)
            info.external_attr = (0o100000 | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    _write(path, buffer.getvalue())


def _write(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
