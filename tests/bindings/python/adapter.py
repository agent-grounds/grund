"""The Python side of binding acceptance (§AR-bindings.6, §FS-distribution.3.0.3).

`binding()` admits only this checkout's crate installed from source, `plain` turns a
host record into lossless data, and `python_call` runs one corpus operation through
the API. The oracle side they are compared against is the shared `support.py`.
"""

import dataclasses
import importlib
import importlib.metadata
import json
import os
from pathlib import Path
import sys
from collections.abc import Mapping

from support import REPO, temporary

# §FS-distribution.3.3.7: a checkout installs the binding from its crate directory.
SOURCE = REPO / "crates/grund-py"


def binding():
    module = importlib.import_module("grund")
    location = Path(module.__file__).resolve()
    if location.name != "__init__.py" or location.parent.name != "grund":
        raise AssertionError(f"not the approved mixed Python package: {location}")
    native = importlib.import_module("grund._native")
    if not Path(native.__file__).resolve().is_relative_to(location.parent):
        raise AssertionError("native extension and Python package have different origins")
    direct = importlib.metadata.distribution("grund").read_text("direct_url.json")
    from urllib.parse import urlparse
    from urllib.request import url2pathname
    if not direct:
        raise AssertionError("acceptance requires a local-source install, not a registry package")
    source = Path(url2pathname(urlparse(json.loads(direct)["url"]).path)).resolve()
    expected = Path(os.environ.get("GRUND_ACCEPTANCE_SOURCE", SOURCE)).resolve()
    if source != expected:
        raise AssertionError(f"unrelated installed grund: {source}; expected {expected}")
    return module


def plain(value):
    """Lossless host records: no selected-field or message-only projection."""
    if dataclasses.is_dataclass(value):
        return {f.name: plain(getattr(value, f.name)) for f in dataclasses.fields(value)}
    if isinstance(value, Mapping):
        return {k: plain(v) for k, v in value.items()}
    if isinstance(value, tuple):
        return [plain(v) for v in value]
    if value is None or type(value) in (str, bool, int, float):
        return value
    raise AssertionError(f"untyped/non-contract result: {type(value)!r}")


def python_call(module, operation, root, args=(), options=None):
    # Capture native file descriptors as well as Python writes on every corpus
    # operation. This helper is used serially; concurrency tests call the API directly.
    with temporary() as temp, open(Path(temp) / "streams", "w+b") as streams:
        sys.stdout.flush()
        sys.stderr.flush()
        saved = [os.dup(fd) for fd in (1, 2)]
        cwd = Path.cwd()
        try:
            for fd in (1, 2):
                os.dup2(streams.fileno(), fd)
            result = _python_call(module, operation, root, args, options)
            sys.stdout.flush()
            sys.stderr.flush()
        finally:
            for fd, original in zip((1, 2), saved):
                os.dup2(original, fd)
                os.close(original)
        streams.seek(0)
        output = streams.read()
        if output:
            raise AssertionError(f"{operation} wrote process streams: {output!r}")
        if Path.cwd() != cwd:
            raise AssertionError(f"{operation} changed global cwd")
        return result


def _python_call(module, operation, root, args=(), options=None):
    keywords = dict(options or {})
    if operation not in ("integrations", "agent_setup_instructions"):
        if operation == "scan":
            args = (root,)
        elif operation == "init":
            args = (root,)
        else:
            keywords["root"] = root
    try:
        result = getattr(module, operation)(*args, **keywords)
    except module.GrundError as error:
        failure = plain(error.failure)
        return {"failure": failure, "result": None,
                "run_cautions": failure["run_cautions"]}
    data = plain(result)
    return {"failure": None, "result": data,
            "run_cautions": data.get("run_cautions", [])}
