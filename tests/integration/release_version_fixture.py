"""Offline candidate trees for §FS-distribution.4.5 and §FS-distribution.4.2.2.

Ports issue #888's triage reproducer: execute the checked workflow's version
writes and git-add commands, inspect each index before the dev commit, and run
the existing ledger validator. No forge query, publication or Cargo build runs.
"""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

from test_release_workflow import preparation


REPO = Path(__file__).resolve().parents[2]
CASE = "tests/e2e/cases/cli-version"
FACT = "tests/integration/command_agreement_facts/cli-version.json"
LEDGER = "docs/discussions/proposals/2026-09-30-core-concerns.md"
LEDGER_TEST = ("test_core_concerns_ledger.CoreConcernsLedgerTest."
               "test_every_ledger_row_removes_after_its_notice_and_after_what_shipped")
# Resolve PATH before CreateProcess can select System32's WSL bash stub.
# The Windows lane needs Git Bash, as do the other shell integration helpers.
BASH = shutil.which("bash") or "bash"


def run(root, *args, **kwargs):
    return subprocess.run([str(a) for a in args], cwd=root, capture_output=True,
                          text=True, encoding="utf-8", timeout=60, **kwargs)


def git(root, *args):
    result = run(root, "git", "-c", "user.name=release-test", "-c",
                 "user.email=release-test@example.invalid", "-c", "maintenance.auto=false",
                 "-c", "gc.auto=0", "-c", "core.fsmonitor=false", *args)
    if result.returncode:
        raise AssertionError(f"git {args}: {result.stdout}{result.stderr}")
    return result.stdout


def copy_tree(test):
    scratch = Path.home() / "ag/tmp"
    scratch.mkdir(parents=True, exist_ok=True)
    temp = tempfile.TemporaryDirectory(prefix="grund-888-release-test-", dir=scratch)
    test.addCleanup(temp.cleanup)
    root = Path(temp.name)
    manifest = tomllib.loads((REPO / "Cargo.toml").read_text(encoding="utf-8"))
    files = {"Cargo.toml", "Cargo.lock", "crates/grund-py/pyproject.toml", FACT, LEDGER,
             "docs/changelog.md", "scripts/release-floor.txt"}
    files.update(f"{member}/Cargo.toml" for member in manifest["workspace"]["members"]
                 if (REPO / member / "Cargo.toml").is_file())
    for name in files:
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / name, path)
    for name in ("scripts/distribution", CASE, "docs/changelog"):
        shutil.copytree(REPO / name, root / name, ignore=shutil.ignore_patterns("__pycache__"))
    git(root, "init", "-q")
    git(root, "config", "core.autocrlf", "false")
    git(root, "add", "-A")
    git(root, "commit", "-qm", "Candidate test baseline")
    return root


def read_bytes(root, path, staged=False):
    if staged:
        return subprocess.check_output(["git", "show", ":" + path], cwd=root)
    return (root / path).read_bytes()


def inventory(root, staged=False):
    paths = git(root, "ls-files", "-z", "--", CASE).strip("\0").split("\0")
    return {path.removeprefix(CASE + "/"): {"bytes": list(read_bytes(root, path, staged))}
            for path in paths}


def bump_versions(helper, root=REPO):
    """§FS-distribution.4.5: the release and dev versions a helper writes over root.

    Derived from the tree's workspace version, as the workflows derive theirs
    from the last tag, so every phase changes the inputs on main and on each
    candidate tree (#895): a literal equal to the tree's version writes nothing.
    """
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    core = manifest["workspace"]["package"]["version"].split("-", 1)[0]
    major, minor, patch = (int(part) for part in core.split("."))
    if helper == "release-minor":
        return f"{major}.{minor + 1}.0", f"{major}.{minor + 1}.1-dev"
    if helper == "auto-bump":
        return f"{major}.{minor}.{patch + 1}", f"{major}.{minor}.{patch + 2}-dev"
    raise ValueError(f"unknown release helper {helper}")


def set_version(root, version):
    return run(root, sys.executable, "scripts/distribution/candidate.py",
               "set-version", version)


def workflow_phases(name):
    """Select offline writes/staging from the real steps, as the triage probe did.

    Preserve the legacy printf while present. Cargo lock refresh and changelog
    forge queries are outside this version-input probe; no push/branch command
    is executed. The existing workflow shape tests cover their ordering.
    """
    phases, adds = [], []
    # §FS-distribution.4.4: the helper's round, which its controller builds, holds the writes.
    for raw in preparation(f"{name}.yml").replace("\\\n", " ").splitlines():
        line = raw.strip()
        if line.startswith("python3 scripts/distribution/candidate.py set-version "):
            phases.append([line])
        elif line.startswith("printf ") and CASE + "/expected.stdout" in line:
            phases[-1].append(line)
        elif line.startswith("git add "):
            adds.append(line)
    if len(phases) != 2 or len(adds) != 2:
        raise AssertionError(f"Cannot identify both write/staging phases in {name}")
    return [writes + [add] for writes, add in zip(phases, adds)]


def execute_phase(test, root, commands, version):
    for line in commands:
        line = line.replace("${{ steps.versions.outputs.next }}", "${TEST_VERSION}")
        line = line.replace("$dev", "${TEST_VERSION}").replace("$next", "${TEST_VERSION}")
        result = run(root, BASH, "-euc", line,
                     env={**os.environ, "TEST_VERSION": version})
        test.assertEqual(0, result.returncode, f"{line}: {result.stdout}{result.stderr}")


def release_heading(root, release):
    # Only the release marker matters to the existing ledger test. Do not query
    # the forge or duplicate the ledger's last-shipped/row validation predicate.
    with (root / "docs/changelog.md").open("a", encoding="utf-8") as handle:
        handle.write(f"\n## 0. [{release}] — simulated candidate\n")


def validate_ledger(root):
    # Run the existing validator, including its repository-derived last release,
    # against the candidate copy. Do not copy its row predicate into this probe.
    probe = (
        "import pathlib, sys, unittest; import test_core_concerns_ledger as ledger; "
        "root = pathlib.Path(sys.argv.pop(1)); ledger.REPO_ROOT = root; "
        "ledger.DISCUSSION = root / sys.argv.pop(1); "
        "ledger.CHANGELOG = root / 'docs/changelog.md'; "
        "unittest.main(module=ledger)"
    )
    return run(REPO / "tests/integration", sys.executable, "-c", probe, root, LEDGER,
               LEDGER_TEST.removeprefix("test_core_concerns_ledger."))
