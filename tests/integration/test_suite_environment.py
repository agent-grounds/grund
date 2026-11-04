"""§AR-ci-suite-environment — a suite wrapper hands its tests no repository location.

A `git commit` from a linked worktree gives its hooks that worktree's `GIT_DIR` and an
absolute `GIT_INDEX_FILE`. Each wrapper runs here with that environment, on a fixture of a
main repository and a linked worktree on `feature`, over a suite of its own whose test does
what a fixture does (§AR-ci-suite-environment.2): `git init`, `git add -A` and `git commit`
with only a working directory, the shape of `ScanCommitsTests.make_repo`. The Python gate
runs its own `inputs()` and `run_tests`; the cargo wrapper runs a crate of its own, as
`test_ci_output.py` drives it, with a build script and a `harness = false` target so that its
build, its listing and its suite each report what they saw (§AR-ci-suite-environment.4).

The test's own git never reaches a real checkout: `git` here removes what git lists, and
`GIT_DIR` is set only on the child under test and only to the fixture.
"""

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

from test_python_gate_inputs import NOT_LOCAL

REPO = Path(__file__).resolve().parents[2]
SCRIPTS = REPO / "scripts"
CARGO_WRAPPER = SCRIPTS / "run_cargo_suite.py"
# What a fixture runs, with only a working directory.
FIXTURE_GIT = (("init", "--quiet", "--initial-branch", "main"),
               ("config", "user.email", "test@example.com"),
               ("config", "user.name", "Test"),
               ("add", "-A"),
               ("commit", "--quiet", "--no-verify", "-m", "probe commit"))
AUTHOR = "Hook Author"

GATE_CHILD = """
import sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate as gate
sys.exit(gate.run_tests(sys.argv[2], gate.inputs()))
"""
# cargo answers in-process, so nothing is built; the rest of inputs() is the wrapper's own.
INPUTS_CHILD = """
import json, os, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import run_python_gate as gate

real = subprocess.run
builds = []

def run(command, *args, **kwargs):
    if command[0] != "cargo":
        return real(command, *args, **kwargs)
    environment = kwargs.get("env")
    environment = os.environ if environment is None else environment
    builds.append({name: value for name, value in environment.items()
                   if name.startswith("GIT_") or name == "GRUND_BINDINGS_SOURCE_SHA"})
    suffix = ".exe" if os.name == "nt" else ""
    target = Path(command[command.index("--target-dir") + 1])
    oracle = target / "debug" / "examples" / ("grund-binding-oracle" + suffix)
    oracle.parent.mkdir(parents=True, exist_ok=True)
    oracle.touch()
    return subprocess.CompletedProcess(command, 0)

subprocess.run = run
gate.inputs()
print(json.dumps(builds))
"""
NO_GIT_CHILD = """
import json, os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import suite_environment
print(json.dumps(suite_environment.without_repository_variables(os.environ)))
"""
PYTHON_PROBE = f"""
import os
import subprocess
import unittest
from pathlib import Path


class Probe(unittest.TestCase):
    def test_does_what_a_fixture_does(self):
        seen = "".join(f"{{name}}={{value}}\\n" for name, value in os.environ.items()
                       if name.startswith("GIT_"))
        Path(os.environ["PROBE_SEEN"], "test").write_text(seen, encoding="utf-8")
        if "PROBE_REPO" not in os.environ:
            return
        repo = Path(os.environ["PROBE_REPO"])
        repo.mkdir()
        (repo / "file.txt").write_text("probe", encoding="utf-8")
        for command in {FIXTURE_GIT!r}:
            subprocess.run(["git", *command], cwd=repo, check=True, capture_output=True)
"""
RUST_SEEN = """
fn seen(phase: &str) {
    let mut text = String::new();
    for (name, value) in std::env::vars_os() {
        let name = name.to_string_lossy();
        if name.starts_with("GIT_") {
            text += &format!("{name}={}\\n", value.to_string_lossy());
        }
    }
    let dir = std::env::var_os("PROBE_SEEN").expect("PROBE_SEEN");
    std::fs::write(std::path::Path::new(&dir).join(phase), text).unwrap();
}
"""
RUST_BUILD = RUST_SEEN + """
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    seen("build");
}
"""
RUST_COMMANDS = ", ".join("&" + json.dumps(command) for command in FIXTURE_GIT)
RUST_PROBE = RUST_SEEN + f"""
const FIXTURE_GIT: &[&[&str]] = &[{RUST_COMMANDS}];

fn main() {{
    if std::env::args().any(|arg| arg == "--list") {{
        seen("listing");
        println!("probe: test");
        return;
    }}
    seen("suite");
    if let Some(repo) = std::env::var_os("PROBE_REPO") {{
        std::fs::create_dir(&repo).unwrap();
        std::fs::write(std::path::Path::new(&repo).join("file.txt"), "probe").unwrap();
        for command in FIXTURE_GIT {{
            let output = std::process::Command::new("git")
                .args(*command)
                .current_dir(&repo)
                .output()
                .unwrap();
            assert!(output.status.success(), "git {{command:?}}: {{output:?}}");
        }}
    }}
    println!("test probe ... ok");
    println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s");
}}
"""
CARGO_MANIFEST = """
[package]
name = "suite_environment_fixture"
version = "0.0.0"
edition = "2021"
publish = false
build = "build.rs"

[workspace]

[[test]]
name = "probe"
path = "tests/probe.rs"
harness = false
"""


class HookedWorktree:
    """A main repository, a linked worktree, and the cases that hold a wrapper to them."""

    maxDiff = None
    # What reports what it saw under `PROBE_SEEN`, one file each; each subclass names its own.
    phases = ()

    def run_wrapper(self, environment):
        raise NotImplementedError

    @classmethod
    def setUpClass(cls):
        # git answers this whatever the variables hold, and it reads no repository.
        cls.listed = subprocess.run(["git", "rev-parse", "--local-env-vars"], capture_output=True,
                                    text=True, check=True).stdout.split()

    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name).resolve()
        self.seen_dir = self.root / "seen"
        self.seen_dir.mkdir()
        self.repo = self.root / "probe-repository"
        self.global_config = self.root / "global-gitconfig"
        self.global_config.write_text("", encoding="utf-8")
        self.common = self.root / "main" / ".git"
        self.gitdir = self.common / "worktrees" / "feature"
        main = self.root / "main"
        main.mkdir()
        self.git("init", "--quiet", "--initial-branch", "main", cwd=main)
        # §AR-ci.10.4: no maintenance writer outlives the fixture.
        for key, value in (("gc.auto", "0"), ("maintenance.auto", "false"),
                           ("user.email", "fixture@example.com"), ("user.name", "Fixture")):
            self.git("config", key, value, cwd=main)
        (main / "tracked.txt").write_text("tracked\n", encoding="utf-8")
        self.git("add", "-A", cwd=main)
        self.git("commit", "--quiet", "--no-verify", "-m", "fixture base", cwd=main)
        self.git("worktree", "add", "--quiet", "-b", "feature", str(self.root / "feature"), cwd=main)
        self.assertTrue(self.gitdir.is_dir(), self.gitdir)

    def unhooked(self):
        """This process's environment, whatever started it, without what git lists."""
        return {name: value for name, value in os.environ.items() if name not in self.listed}

    def git(self, *args, cwd=None):
        environment = dict(self.unhooked(), GIT_CONFIG_GLOBAL=str(self.global_config),
                           GIT_CONFIG_NOSYSTEM="1")
        return subprocess.run(["git", *args], cwd=cwd, env=environment, capture_output=True,
                              text=True, check=True).stdout

    def hook_environment(self, **extra):
        """What `git commit` gives a hook in the fixture worktree, with who and how beside it."""
        return dict(self.unhooked(), GIT_DIR=str(self.gitdir),
                    GIT_INDEX_FILE=str(self.gitdir / "index"), GIT_PREFIX="",
                    GIT_AUTHOR_NAME=AUTHOR, GIT_CONFIG_GLOBAL=str(self.global_config),
                    GIT_CONFIG_NOSYSTEM="1", PROBE_SEEN=str(self.seen_dir), **extra)

    def state(self):
        """The hooked worktree's branch, reflogs, index bytes and the config it shares."""
        def text(path):
            return path.read_text(encoding="utf-8", errors="replace")
        return {"refs/heads/feature": self.git("--git-dir", str(self.common), "rev-parse",
                                               "refs/heads/feature").strip(),
                "branch reflog": text(self.common / "logs" / "refs" / "heads" / "feature"),
                "worktree reflog": text(self.gitdir / "logs" / "HEAD"),
                "index sha256": hashlib.sha256((self.gitdir / "index").read_bytes()).hexdigest(),
                "shared config": text(self.common / "config")}

    def seen(self):
        """Each phase's `GIT_*` variables, as it reported them."""
        return {path.name: dict(line.split("=", 1) for line in path.read_text("utf-8").splitlines())
                for path in self.seen_dir.iterdir()}

    def test_a_fixtures_git_leaves_the_hooked_worktree_alone(self):
        before = self.state()
        result = self.run_wrapper(self.hook_environment(PROBE_REPO=str(self.repo)))
        self.assertEqual(before, self.state(),
                         "a fixture's git acted on the worktree the hook runs for\n"
                         + result.stdout + result.stderr)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertTrue((self.repo / ".git").is_dir(), "the fixture's own repository was not made")
        self.assertEqual("probe commit\n", self.git("--git-dir", str(self.repo / ".git"), "log",
                                                    "--format=%s", "HEAD"))

    def test_each_child_sees_no_repository_variable_and_keeps_the_rest(self):
        environment = self.hook_environment()
        for name in self.listed:
            environment.setdefault(name, str(self.gitdir))
        environment.update(GIT_CONFIG_COUNT="0", GIT_CONFIG_PARAMETERS="'probe.key=value'")
        result = self.run_wrapper(environment)
        output = result.stdout + result.stderr
        self.assertEqual(0, result.returncode, output)
        seen = self.seen()
        self.assertEqual(sorted(self.phases), sorted(seen), output)
        for phase, variables in seen.items():
            with self.subTest(phase=phase):
                self.assertEqual([], sorted(set(variables) & set(self.listed)),
                                 f"{phase} inherited these repository variables")
                self.assertEqual(str(self.global_config), variables.get("GIT_CONFIG_GLOBAL"))
                self.assertEqual(AUTHOR, variables.get("GIT_AUTHOR_NAME"))


class PythonGateTests(HookedWorktree, unittest.TestCase):
    """§AR-ci.3.4: the gate's own `inputs()` and `run_tests`, on a `unittest` tree of its own."""

    phases = ("test",)

    def setUp(self):
        super().setUp()
        self.suite = self.root / "suite"
        self.suite.mkdir()
        (self.suite / "test_probe.py").write_text(PYTHON_PROBE, encoding="utf-8")

    def stand_in(self, name, line):
        """An executable printing `line`: a shebang means nothing on Windows, a .cmd does."""
        if os.name == "nt":
            path = self.root / (name + ".cmd")
            path.write_text(f"@echo {line}\n", encoding="utf-8")
        else:
            path = self.root / name
            path.write_text(f"#!/bin/sh\necho '{line}'\n", encoding="utf-8")
            path.chmod(0o755)
        return path

    def inputs(self, **given):
        return dict(GRUND_RELEASED=str(self.stand_in("grund-released", "grund 0.16.1")),
                    GRUND_NODE_TARGET=str(self.root / "node-target"), **given)

    def run_wrapper(self, environment):
        environment.update(self.inputs(GRUND_BUILT=str(self.root / "grund-built-unused"),
                                       GRUND_BINDINGS_ORACLE=str(self.root / "oracle-unused")))
        return subprocess.run([sys.executable, "-c", GATE_CHILD, str(SCRIPTS), str(self.suite)],
                              cwd=self.root, env=environment, capture_output=True,
                              encoding="utf-8", errors="replace", timeout=600)

    def test_its_own_builds_and_the_oracles_commit_are_this_checkouts(self):
        environment = self.hook_environment(**self.inputs(
            CARGO_TARGET_DIR=str(self.root / "target")))
        # `git rev-parse HEAD` reads the real checkout here, so it keeps the user's own config.
        del environment["GIT_CONFIG_GLOBAL"]
        for name in (*NOT_LOCAL, "GRUND_BUILT"):
            environment.pop(name, None)
        result = subprocess.run([sys.executable, "-c", INPUTS_CHILD, str(SCRIPTS)],
                                cwd=self.root, env=environment, capture_output=True,
                                encoding="utf-8", errors="replace", timeout=600)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        builds = json.loads(result.stdout.splitlines()[-1])
        self.assertEqual(2, len(builds), builds)
        head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, env=self.unhooked(),
                              capture_output=True, text=True, check=True).stdout.strip()
        self.assertEqual(head, builds[-1].get("GRUND_BINDINGS_SOURCE_SHA"),
                         "the oracle names the hooked repository's commit, not this checkout's")
        for build in builds:
            self.assertEqual([], sorted(set(build) & set(self.listed)),
                             "a build inputs() starts inherited these repository variables")


class CargoSuiteTests(HookedWorktree, unittest.TestCase):
    """§AR-ci.requirements.5.2: `scripts/run_cargo_suite.py` on a crate of its own."""

    phases = ("build", "listing", "suite")

    def setUp(self):
        super().setUp()
        self.crate = self.root / "crate"
        (self.crate / "src").mkdir(parents=True)
        (self.crate / "tests").mkdir()
        (self.crate / "Cargo.toml").write_text(textwrap.dedent(CARGO_MANIFEST), encoding="utf-8")
        (self.crate / "build.rs").write_text(RUST_BUILD, encoding="utf-8")
        (self.crate / "src" / "lib.rs").write_text("", encoding="utf-8")
        (self.crate / "tests" / "probe.rs").write_text(RUST_PROBE, encoding="utf-8")

    def run_wrapper(self, environment):
        environment["CARGO_TARGET_DIR"] = str(self.crate / "target")
        return subprocess.run([sys.executable, str(CARGO_WRAPPER), "--manifest-path",
                               str(self.crate / "Cargo.toml"), "--test", "probe"],
                              cwd=REPO, env=environment, capture_output=True,
                              encoding="utf-8", errors="replace", timeout=600)


class NoGitTests(unittest.TestCase):
    """§AR-ci-suite-environment.1: with no `git` to start, nothing is removed and nothing raises."""

    def test_with_no_git_to_start_the_environment_stands_and_nothing_raises(self):
        empty = tempfile.TemporaryDirectory()
        self.addCleanup(empty.cleanup)
        # Only the child loses git: this process keeps its own PATH.
        given = str(Path(empty.name) / "hooked")
        environment = dict(os.environ, PATH=empty.name, GIT_DIR=given)
        result = subprocess.run([sys.executable, "-c", NO_GIT_CHILD, str(SCRIPTS)], cwd=empty.name,
                                env=environment, capture_output=True, text=True, timeout=60)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(given, json.loads(result.stdout).get("GIT_DIR"), result.stdout)


if __name__ == "__main__":
    unittest.main()
