"""Resolved Python/frontend isolation (§AR-bindings.1, §FS-distribution.3.3.7)."""

import json
import subprocess
import unittest
from support import REPO


class IsolationTests(unittest.TestCase):
    def test_python_is_an_independent_core_frontend(self):
        metadata = json.loads(subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=REPO, capture_output=True, text=True, check=True).stdout)
        names = {p["id"]: p["name"] for p in metadata["packages"]}
        ids = {names[i]: i for i in metadata["workspace_members"]}
        self.assertIn("grund-py", ids, "the missing Python frontend must join the workspace")
        nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
        def deps(identity):
            return {d["pkg"] for d in nodes[identity]["deps"]
                    if any(k["kind"] is None for k in d["dep_kinds"])}
        def closure(name):
            seen, pending = set(), [ids[name]]
            while pending:
                new = deps(pending.pop()) - seen
                seen |= new
                pending.extend(new)
            return {names[i] for i in seen}
        frontends = {"grund", "grund-lsp", "grund-py", "grund-node"} & set(ids)
        self.assertIn("grund-core", {names[i] for i in deps(ids["grund-py"])})
        for frontend in frontends:
            self.assertFalse(closure(frontend) & (frontends - {frontend}))
        self.assertFalse(closure("grund-core") & frontends)
        for name in ("grund", "grund-core", "grund-lsp"):
            self.assertFalse(any(p.startswith(("pyo3", "python", "maturin")) for p in closure(name)))
        # Default Cargo CLI selection must still exclude Python entirely.
        self.assertNotIn(ids["grund-py"], metadata["workspace_default_members"])

    def test_default_cli_build_does_not_need_python(self):
        import os
        environment = dict(os.environ, PYO3_PYTHON="/no/python/available",
                           PYTHON_SYS_EXECUTABLE="/no/python/available")
        subprocess.run(["cargo", "check", "--locked", "-p", "grund"], cwd=REPO,
                       capture_output=True, text=True, check=True, env=environment)
