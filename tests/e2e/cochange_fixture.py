"""Synthetic Git repositories for §FS-cochange-recipe.snapshots and evidence.

Fixture construction and black-box assertions only; no recipe evaluation lives here.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
RECIPE = ROOT / "examples/cochange/cochange.py"
MARK = chr(167)
CONFIG = '''grund_config_version = 1
project_name = "fixture"
[id]
format = "{kind}-{slug}"
[reference]
strict = true
[[kinds]]
kind = "FS"
folder = "docs"
[scan]
include = ["src", "tests"]
extensions = ["md", "py"]
'''


def trailer(paths=("src/lib.py",), missing=("spec", "test"), reason="Refactor only"):
    return "\n\nGrund-Cochange: " + json.dumps(
        {"paths": list(paths), "missing": list(missing), "reason": reason})


class Fixture:
    def __init__(self, root):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.policy = self.root.parent / (self.root.name + "-policy.json")
        self.policy.write_text(json.dumps({"config_root": ".",
            "source_paths": ["src/"], "test_paths": ["tests/"],
            "eligible_kinds": ["FS"]}))
        self.git("init", "-q", "--initial-branch=main")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "core.autocrlf", "false")
        self.git("config", "core.filemode", "true")
        self.write("grund.toml", CONFIG)
        for name in ("alpha", "beta"):
            self.write(f"docs/FS-{name}.md",
                       f"# FS-{name}: {name}\n\n## 1. Behavior\n\nBefore.\n")
        self.write("docs/README.md", f"- [{MARK}FS-alpha](FS-alpha.md)\n- [{MARK}FS-beta](FS-beta.md)\n")
        self.write("src/lib.py", self.source())
        self.write("tests/test_lib.py", self.source())
        self.base = self.commit("Baseline")

    @staticmethod
    def source(targets=("FS-alpha.1",), value=1):
        return "".join(f"# {MARK}{target}\n" for target in targets) + f"value = {value}\n"

    def git(self, *args, check=True):
        result = subprocess.run(["git", "-C", str(self.root), *args],
            capture_output=True, env={**os.environ, "GIT_CONFIG_NOSYSTEM": "1",
                "GIT_CONFIG_GLOBAL": os.devnull})
        if check and result.returncode:
            raise AssertionError(f"git {args}: {result.stderr!r}")
        return result.stdout.decode("utf-8").strip()

    def write(self, path, text):
        out = self.root / path
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")

    def edit(self, path):
        with (self.root / path).open("a", encoding="utf-8") as out:
            out.write("\n# Related edit\n")

    def change(self, spec=False, test=False, targets=("FS-alpha.1",)):
        self.write("src/lib.py", self.source(targets, 2))
        if spec:
            self.edit("docs/FS-alpha.md")
        if test:
            self.edit("tests/test_lib.py")

    def commit(self, message, stage=True):
        if stage:
            self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")

    def invoke(self, mode="ci", message="Change", base=None, grund=None, **kwargs):
        executable = grund or os.environ.get("GRUND_BUILT") or shutil.which("grund")
        common = [sys.executable, str(RECIPE), "--repo", str(self.root),
                  "--policy", str(self.policy), "--grund", str(executable)]
        if mode == "commit-msg":
            msg = self.root.parent / (self.root.name + "-message")
            msg.write_text(message, encoding="utf-8")
            args = [mode, "--base", base or self.base, "--message", str(msg)]
        else:
            args = [mode, "--target", base or self.base, "--head", "HEAD"]
        return subprocess.run(common + args, capture_output=True, text=True,
                              cwd=self.root, **kwargs)


class RecipeCase(unittest.TestCase):
    def setUp(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="cochange-contract-", dir=scratch)
        self.addCleanup(self.temp.cleanup)
        self.fixture = Fixture(Path(self.temp.name) / "repo")

    def report(self, result, code):
        self.assertTrue(result.stdout.strip(),
            f"recipe emitted no JSON report (exit {result.returncode}): {result.stderr.strip()}")
        report = json.loads(result.stdout)
        self.assertEqual(code, result.returncode, report)
        self.assertEqual("", result.stderr)
        self.assertIn("file-level related edits", report["limitation"])
        self.assertEqual(json.loads(self.fixture.policy.read_text()), report["policy"])
        self.assertEqual(sorted(row["path"] for row in report["sources"]),
                         [row["path"] for row in report["sources"]])
        return report

    def source_row(self, report, path="src/lib.py"):
        return next(row for row in report["sources"] if row["path"] == path)

    def error(self, report, code, path=None):
        matches = [row for row in report["errors"] if row["code"] == code]
        self.assertTrue(matches, report)
        if path:
            self.assertIn(path, [row["path"] for row in matches])
        self.assertTrue(all(row["message"].strip() for row in matches))

    def evaluate(self, code, message="Change"):
        self.fixture.commit(message)
        return self.report(self.fixture.invoke(), code)
