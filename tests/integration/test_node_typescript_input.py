"""§AR-ci.3.4.1.3 — the Node acceptance's TypeScript compiler is a prepared input.

The helper is run in a child whose `subprocess.run` stands in for npm, so nothing here
reaches the registry: each install attempt plays the next outcome of a script, and a
successful one lays the compiler where npm would. The workflows are read as text.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from ci_workflow import ci_text

REPO = Path(__file__).resolve().parents[2]
NODE_HARNESS = REPO / "tests" / "bindings" / "node"
WORKFLOWS = REPO / ".github" / "workflows"
NOT_LOCAL = ("GRUND_NODE_TARGET", "CI", "GITHUB_ACTIONS", "GITHUB_ENV")

NETWORK = ("ENOTFOUND", "EAI_AGAIN", "ETIMEDOUT", "ECONNRESET", "ECONNREFUSED")

PREPARE = """
import json, os, subprocess, sys, time
from pathlib import Path
sys.dont_write_bytecode = True
time.sleep = lambda seconds: None
sys.path.insert(0, sys.argv[1])
outcomes, log = json.loads(sys.argv[2]), Path(sys.argv[3])
import launch, support

class Done:
    def __init__(self, returncode, stderr=""):
        self.stdout, self.stderr, self.returncode = "", stderr, returncode

def npm(argv, cwd=None, env=None, **options):
    argv = [str(arg) for arg in argv]
    if "install" not in argv:
        return Done(0)
    where = Path(argv[argv.index("--prefix") + 1] if "--prefix" in argv else cwd)
    with log.open("a") as out:
        out.write(json.dumps({"argv": argv, "dir": str(where.resolve())}) + "\\n")
    outcome = outcomes.pop(0) if outcomes else "ok"
    package = where / "node_modules" / "typescript"
    (package / "bin").mkdir(parents=True, exist_ok=True)
    (package / "package.json").write_text(json.dumps({"version": support.TYPESCRIPT}))
    if outcome != "ok":
        # A failed install may leave part of its tree behind; it must never count as ready.
        return Done(1, f"npm error code {outcome}\\nnpm error network request failed\\n")
    (package / "bin" / "tsc").write_text("#!/usr/bin/env node\\n")
    return Done(0)

launch.subprocess.run = npm
subprocess.run = npm
try:
    print(json.dumps({"tsc": str(support.typescript_compiler()), "error": None}))
except BaseException as error:
    print(json.dumps({"tsc": None, "error": f"{type(error).__name__}: {error}"}))
"""


def prepare(root, outcomes, log):
    environment = {key: value for key, value in os.environ.items() if key not in NOT_LOCAL}
    environment["GRUND_NODE_TARGET"] = str(root)
    result = subprocess.run([sys.executable, "-c", PREPARE, str(NODE_HARNESS),
                             json.dumps(outcomes), str(log)],
                            cwd=root.parent, env=environment, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    return json.loads(result.stdout.splitlines()[-1])


def pinned():
    """The version the harness pins, asked of a child so no `support` leaks into this run."""
    result = subprocess.run([sys.executable, "-c", "import sys; sys.dont_write_bytecode = True; "
                             f"sys.path.insert(0, {str(NODE_HARNESS)!r}); import support; "
                             "print(support.TYPESCRIPT)"],
                            capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout.strip()


def jobs(workflow):
    """Each job of a workflow, a path or its text, as (name, text), split at the two-space
    job keys; `<caller>/<job>` is a called leg's job as the layout test reads it."""
    text = workflow if isinstance(workflow, str) else workflow.read_text(encoding="utf-8")
    body = text.split("\njobs:\n", 1)[1]
    parts = re.split(r"^  ([A-Za-z0-9_/-]+):\s*$", body, flags=re.M)
    return list(zip(parts[1::2], parts[2::2]))


def steps(job):
    return job.split("\n      - ")


def keys(step):
    return [line.strip()[len("key:"):].strip() for line in step.splitlines()
            if line.strip().startswith("key:")]


class NodeTypescriptInputTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.temp = Path(temp.name)
        self.version = pinned()
        self.use("shared")

    def use(self, name):
        """A fresh shared root, with the log of every install attempt beside it."""
        self.root = self.temp / name
        self.root.mkdir()
        self.log = self.temp / f"{name}.npm.jsonl"
        self.home = self.root / "tools" / f"typescript-{self.version}"
        self.tsc = self.home / "node_modules" / "typescript" / "bin" / "tsc"

    def installs(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_a_prepared_compiler_is_returned_and_nothing_runs(self):
        self.tsc.parent.mkdir(parents=True)
        self.tsc.write_text("#!/usr/bin/env node\n")
        (self.tsc.parents[1] / "package.json").write_text(json.dumps({"version": self.version}))
        result = prepare(self.root, ["ENOTFOUND"] * 5, self.log)
        self.assertIsNone(result["error"], result)
        self.assertEqual(self.tsc.resolve(), Path(result["tsc"]).resolve())
        self.assertEqual([], self.installs(), "a prepared compiler must not reach npm")

    def test_a_cold_root_installs_once_into_a_sibling_and_renames_it(self):
        result = prepare(self.root, ["ok"], self.log)
        self.assertIsNone(result["error"], result)
        self.assertEqual(self.tsc.resolve(), Path(result["tsc"]).resolve())
        self.assertTrue(self.tsc.is_file(), "the compiler is in place after the install")
        installs = self.installs()
        self.assertEqual(1, len(installs), installs)
        self.assertIn(f"typescript@{self.version}", installs[0]["argv"],
                      "the install names the pinned version exactly")
        where = Path(installs[0]["dir"])
        self.assertNotEqual(self.home.resolve(), where,
                            "the install must not write the directory it is read from")
        self.assertEqual(self.home.parent.resolve(), where.parent,
                         "the install goes to a sibling, renamed into place")
        again = prepare(self.root, ["E404"], self.log)
        self.assertIsNone(again["error"], again)
        self.assertEqual(1, len(self.installs()), "a second run reuses the prepared compiler")

    def test_a_network_error_is_retried(self):
        for code in NETWORK:
            with self.subTest(code=code):
                self.use(code)
                result = prepare(self.root, [code, code, "ok"], self.log)
                self.assertIsNone(result["error"], result)
                self.assertEqual(3, len(self.installs()), "two network failures, then success")
                self.assertTrue(self.tsc.is_file())

    def test_a_persistent_network_error_fails_after_three_attempts(self):
        result = prepare(self.root, ["ENOTFOUND"] * 10, self.log)
        self.assertIsNotNone(result["error"], "an unreachable registry fails the test")
        self.assertIn("ENOTFOUND", result["error"], "npm's own output reaches the failure")
        self.assertEqual(3, len(self.installs()), "three attempts in all")
        self.assertFalse(self.home.exists(), "a failed install never leaves a ready directory")

    def test_any_other_error_is_not_retried_and_leaves_nothing_ready(self):
        result = prepare(self.root, ["E404"], self.log)
        self.assertIsNotNone(result["error"], result)
        self.assertIn("E404", result["error"])
        self.assertEqual(1, len(self.installs()), "only a network-class error is retried")
        self.assertFalse(self.home.exists(), "a failed install never leaves a ready directory")
        result = prepare(self.root, ["ok"], self.log)
        self.assertIsNone(result["error"], "what a failed install left behind blocks nothing")
        self.assertTrue(self.tsc.is_file())

    def test_every_job_running_the_node_acceptance_caches_the_compiler(self):
        cached = f"~/ag/tmp/grund-node-target/tools/typescript-{self.version}"
        found = 0
        # An exact hit saves nothing, so a key another cache uses never saves this one.
        every_key = [(workflow.name, name, step, key)
                     for workflow in sorted(WORKFLOWS.glob("*.yml"))
                     for name, job in jobs(workflow) for step in steps(job)
                     if "actions/cache" in step for key in keys(step)]
        # ci.yml with its leg's jobs in place of their calls, one per system (§AR-ci.13.5).
        for workflow, text in (("ci.yml", ci_text()),
                               ("macos-suites.yml", WORKFLOWS / "macos-suites.yml")):
            for name, job in jobs(text):
                if "--only-node-acceptance" not in job:
                    continue
                found += 1
                where = f"{workflow} job {name}"
                listed = steps(job)
                runs = next(index for index, step in enumerate(listed)
                            if "--only-node-acceptance" in step)
                restore = [index for index, step in enumerate(listed)
                           if "actions/cache/restore" in step and cached in step]
                save = [index for index, step in enumerate(listed)
                        if "actions/cache/save" in step and cached in step]
                self.assertTrue(restore and min(restore) < runs,
                                f"{where}: no cache step restores {cached} before the acceptance")
                self.assertTrue(save and max(save) > runs,
                                f"{where}: no cache step saves {cached} after the acceptance")
                restored = {key for index in restore for key in keys(listed[index])}
                saved = {key for index in save for key in keys(listed[index])}
                self.assertEqual(1, len(restored), restored)
                self.assertEqual(restored, saved, f"{where}: restore and save keys differ")
                key = next(iter(restored))
                self.assertIn(f"typescript-{self.version}", key,
                              f"{where}: the key must name the pinned version")
                self.assertNotIn("node-target", key, f"{where}: an exact hit never saves")
                others = [(file, job_name) for file, job_name, step, other in every_key
                          if other == key and cached not in step]
                self.assertEqual([], others, f"{where}: {key} is shared with another cache")
        self.assertEqual(3, found, "the Node acceptance runs on ubuntu, windows and macOS")


if __name__ == "__main__":
    unittest.main()
