"""§FS-distribution-candidate.5.3, §FS-distribution.2 — every installed `grund` of this
row answers the read-only e2e corpus byte for byte.

The cases are the checked ones under `tests/e2e/cases`, read the way the e2e runner
reads them: `command.args` split on the shell's quotes, `{repo}` the case's tree
relative to the repository root, `command.cwd` either that root or the tree, and
standard input from `command.stdin`. A case that writes, copies its tree, links,
or runs an external program is not read-only and is left to the e2e suite. Each
install — Cargo, npm, wheel and archive — is held to every golden, and then all four
to each other on a copy of a tree under a directory named with spaces and non-ASCII.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import os
import shlex
import shutil
import subprocess
import unittest

from distribution_support import REPO
from rehearsal_support import CASES, acquire, installed, keep

POSIX = os.name == "posix"


def read(path):
    return path.read_text(encoding="utf-8") if path.exists() else None


def golden(path):
    text = path.read_bytes().replace(b"\r\n", b"\n")
    return b"" if text == b"\n" else text


def read_only_cases():
    """Every case the corpus runs without writing, copying, linking or leaving grund."""
    found = []
    for case in sorted(p for p in CASES.iterdir() if (p / "repo").is_dir()):
        args = read(case / "command.args") or "check {repo}"
        cwd = (read(case / "command.cwd") or "").strip()
        if (case / "command.external").exists() or (case / "symlinks").exists() \
                or (case / "expected.repo").exists() or "--write" in args \
                or "{repo_copy}" in args or cwd == "{repo_copy}":
            continue
        if (case / "requires-unix-shell").exists() and not POSIX:
            continue
        found.append(case)
    return found


def argv(case):
    relative = (case / "repo").relative_to(REPO).as_posix()
    words = shlex.split(read(case / "command.args") or "check {repo}")
    expanded = []
    for word in words:
        if word == "{repo}":
            word = relative
        elif word.startswith("{repo}/"):
            word = relative + word[len("{repo}"):]
        expanded.append(word)
    return expanded


def run_case(binary, case):
    cwd = case / "repo" if (read(case / "command.cwd") or "").strip() == "{repo}" else REPO
    stdin = (case / "command.stdin").read_bytes() if (case / "command.stdin").exists() else None
    ran = subprocess.run([str(binary), *argv(case)], cwd=cwd, input=stdin,
                         capture_output=True, timeout=120)
    return ran.returncode, ran.stdout, ran.stderr


class CorpusTests(unittest.TestCase):
    """Every install, every read-only case, every surface."""

    def setUp(self):
        acquire()
        self.cases = read_only_cases()

    def test_the_corpus_covers_what_the_spec_names(self):
        names = {case.name for case in self.cases}
        for case in ("basic-markdown-valid", "basic-markdown-dangling", "json-report"):
            self.assertIn(case, names)
        args = {case.name: " ".join(argv(case)) for case in self.cases}
        self.assertTrue(any("--only" in a or "--ignore" in a for a in args.values()), "selectors")
        self.assertTrue(any(a.startswith("show ") and int(golden(CASES / n / "expected.exit")) != 0
                            for n, a in args.items()), "a failed ID query")
        self.assertTrue(any((read(c / "command.cwd") or "").strip() == "{repo}" for c in self.cases),
                        "a case run from inside its tree")

    def test_every_install_answers_every_golden(self):
        for install, binary in installed("grund").items():
            with self.subTest(install=install):
                mismatched = []
                for case in self.cases:
                    status, stdout, stderr = run_case(binary, case)
                    expected = (int(golden(case / "expected.exit").strip()),
                                golden(case / "expected.stdout"), golden(case / "expected.stderr"))
                    if (status, stdout, stderr) != expected:
                        mismatched.append(case.name)
                self.assertEqual([], mismatched[:20], f"{len(mismatched)} cases differ under {install}")


class UnicodePathTests(unittest.TestCase):
    """A copy of a tree under spaces and non-ASCII answers the same from every install."""

    def setUp(self):
        acquire()
        self.parent = keep("grund-parity-") / "dír with space"
        shutil.copytree(CASES / "json-report" / "repo", self.parent / "répo-雪")

    def test_every_install_agrees_on_the_copied_tree(self):
        installs = installed("grund")
        for args in (["check", "répo-雪"], ["check", "répo-雪", "--format", "json"],
                     ["show", "FS-001-alpha", "répo-雪"], ["refs", "FS-002-beta", "répo-雪"]):
            with self.subTest(args=args):
                answers = {name: subprocess.run([str(b), *args], cwd=self.parent, capture_output=True,
                                                timeout=120)
                           for name, b in installs.items()}
                reference = answers.pop("cargo")
                self.assertNotEqual(b"", reference.stdout + reference.stderr)
                for name, answer in answers.items():
                    self.assertEqual((reference.returncode, reference.stdout, reference.stderr),
                                     (answer.returncode, answer.stdout, answer.stderr), name)


if __name__ == "__main__":
    unittest.main()
