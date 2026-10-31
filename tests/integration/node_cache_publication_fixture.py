"""Real Cargo payload fixture for §AR-ci.3.4.1.5.

Promotes #1122's triage witness: execute publication commands in a disposable
checkout and inspect the files selected by the actual workflow. This adapter
does not filter artifacts or prescribe a production snapshot representation.
"""

import glob
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import textwrap

from test_ci_cache_saves import cache_action, condition, with_value
from test_release_workflow import REPO_ROOT, jobs, step_name, steps


WORKSPACE = ("grund", "grund-core", "grund-node", "fixture-helper")
DEPENDENCIES = ("vendor", "grund-vendor")


def files(root):
    return {p: p.read_bytes() for p in root.rglob("*") if p.is_file()}


class PublicationFixture:
    def __init__(self, base, workflow):
        self.base = base
        self.home = base / "home"
        self.root = self.home / "ag/tmp/grund-node-target"
        self.checkout = base / "checkout"
        self.checkout.mkdir()
        self.environment = dict(
            os.environ, HOME=str(self.home), USERPROFILE=str(self.home),
            CARGO_HOME=os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
            RUSTUP_HOME=os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")),
            CARGO_INCREMENTAL="0", CARGO_NET_OFFLINE="true", RUSTFLAGS="-Cdebuginfo=0",
            GRUND_NODE_TARGET=str(self.root), CARGO_TARGET_DIR=str(self.checkout / "target"),
            GITHUB_WORKSPACE=str(self.checkout), PYTHONDONTWRITEBYTECODE="1",
            RUNNER_TEMP=str(base / "runner-temp"),
        )
        for relative in ("scripts", "tests/bindings"):
            shutil.copytree(REPO_ROOT / relative, self.checkout / relative,
                            ignore=shutil.ignore_patterns("__pycache__"))
        bodies = jobs((REPO_ROOT / ".github/workflows" / workflow).read_text())
        self.steps = next(steps(body) for body in bodies.values()
                          if any(step_name(s) == "Save the Node acceptance's build target"
                                 for s in steps(body)))
        self.restore = self.index("Restore the Node acceptance's build target")
        self.save = self.index("Save the Node acceptance's build target")
        self.acceptance = self.index("node acceptance")
        self.artifacts = {"workspace": set(), "dependency": set()}
        self.targets = []
        self.owners = []
        self._build_checkouts()

    def index(self, name):
        return next(i for i, step in enumerate(self.steps) if step_name(step) == name)

    def cargo(self, checkout, target, profile):
        result = subprocess.run(
            ["cargo", "build", "--offline", "--workspace", "--all-targets",
             "--profile", profile, "--target-dir", str(target), "--message-format=json"],
            cwd=checkout, env=self.environment, capture_output=True, text=True,
        )
        if result.returncode:
            raise AssertionError(f"fixture Cargo build failed:\n{result.stderr}")
        return [json.loads(line) for line in result.stdout.splitlines()
                if json.loads(line).get("reason") == "compiler-artifact"]

    def _workspace(self, checkout):
        checkout.mkdir(exist_ok=True)
        (checkout / "Cargo.toml").write_text(
            f"[workspace]\nmembers={json.dumps(WORKSPACE)}\n"
            f"exclude={json.dumps(DEPENDENCIES)}\nresolver=\"2\"\n")
        for index, name in enumerate(WORKSPACE + DEPENDENCIES):
            crate = checkout / name
            (crate / "src").mkdir(parents=True)
            manifest = f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
            if name == "fixture-helper":
                manifest += '[lib]\nname="custom_workspace_output"\ncrate-type=["rlib", "cdylib"]\n'
            if name == "grund":
                manifest += "[dependencies]\n" + "".join(
                    f'{dep}={{path="../{dep}"}}\n' for dep in WORKSPACE[1:] + DEPENDENCIES)
            (crate / "Cargo.toml").write_text(manifest)
            (crate / "src/lib.rs").write_text(f"pub fn answer()->usize {{ {index + 42} }}\n")
            # Both workspace and third-party build-script artifacts must be classified.
            (crate / "build.rs").write_text(
                'fn main() { println!("cargo:rerun-if-changed=build.rs"); '
                f'println!("cargo:rustc-env=FIXTURE_PACKAGE={name}"); }}\n')
        main = checkout / "grund"
        (main / "src/main.rs").write_text('fn main() { println!("{}", vendor::answer()); }\n')
        (main / "examples").mkdir()
        (main / "examples/probe.rs").write_text("fn main() {}\n")

    def _build_checkouts(self):
        spec = importlib.util.spec_from_file_location(
            "publication_binding_support", REPO_ROOT / "tests/bindings/support.py")
        support = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(support)
        for checkout in (self.checkout, self.base / "sibling-checkout"):
            self._workspace(checkout)
            support.REPO = checkout.resolve()
            target = support.checkout_target(self.root)
            self.owners.append(target / "checkout.json")
            for build_target in (target, target / "native-builder"):
                self.targets.append((checkout, build_target))
                for profile in ("dev", "release"):
                    for message in self.cargo(checkout, build_target, profile):
                        package = Path(message["manifest_path"]).parent.name
                        kind = "dependency" if package in DEPENDENCIES else "workspace"
                        self.artifacts[kind].update(Path(p) for p in message["filenames"])
                    directory = "debug" if profile == "dev" else profile
                    incremental = build_target / directory / "incremental/probe"
                    incremental.mkdir(parents=True)
                    (incremental / "state").write_bytes(b"incremental output must not be published")
        tools = self.root / "tools/typescript-5.9.3"
        tools.mkdir(parents=True)
        (tools / "compiler").write_bytes(b"separately cached TypeScript input")

    def expand(self, text):
        for key, value in self.environment.items():
            text = text.replace("${{ env." + key + " }}", value)
        text = text.replace("${{ github.workspace }}", str(self.checkout))
        text = text.replace("~/", str(self.home) + "/")
        if "${{" in text:
            raise AssertionError(f"fixture cannot evaluate workflow expression: {text}")
        return text

    def run_steps(self, selected, cargo_hit):
        for step in selected:
            match = re.search(r"(?m)^\s*(?:- )?run:\s*(.*)$", step)
            if not match:
                continue
            guard = condition(step)
            # Simulate main publication with a Node prefix hit, independently of Cargo.
            if cargo_hit and "steps.cargo.outputs.cache-hit != 'true'" in guard:
                continue
            command = match.group(1).strip()
            if command in ("|", ">"):
                command = textwrap.dedent(step[match.end():]).strip()
            result = subprocess.run(["bash", "-c", self.expand(command)], cwd=self.checkout,
                                    env=self.environment, capture_output=True, text=True)
            if result.returncode:
                raise AssertionError(f"publication command failed: {command}\n"
                                     f"{result.stdout}\n{result.stderr}")

    def prepare(self, cargo_hit):
        # macOS has later test/build commands: execute only guarded publication work,
        # never its real gate against the tiny fixture. No cache action is executed.
        selected = [s for s in self.steps[self.acceptance + 1:self.save]
                    if condition(s) and cache_action(s) is None]
        self.run_steps(selected, cargo_hit)

    def payload(self):
        included = set()
        for expression in with_value(self.steps[self.save], "path").splitlines():
            exclude = expression.startswith("!")
            expression = self.expand(expression.lstrip("!"))
            if not os.path.isabs(expression):
                expression = str(self.checkout / expression)
            found = set()
            for name in glob.glob(expression, recursive=True):
                path = Path(name)
                found.update(files(path) if path.is_dir() else [path])
            if exclude:
                included.difference_update(found)
            else:
                included.update(found)
        self.payload_times = {p: p.stat().st_mtime_ns for p in included if p.is_file()}
        return {p: p.read_bytes() for p in self.payload_times}

    def restore_payload(self, payload):
        shutil.rmtree(self.root)
        # Emulate unpacking at the paths the real cache action selected.
        for path, data in payload.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            os.utime(path, ns=(self.payload_times[path], self.payload_times[path]))
        end = next((i for i in range(self.restore + 1, self.acceptance)
                    if cache_action(self.steps[i]) == "restore"), self.acceptance)
        self.run_steps(self.steps[self.restore + 1:end], cargo_hit=True)


def digest(data):
    return hashlib.sha256(data).hexdigest()
