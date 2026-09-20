"""§FS-distribution.4.2.7, §FS-distribution.4.3, §FS-distribution.4.8,
§FS-distribution.4.9, §FS-distribution.4.10 — what the release workflows must
still do for a release to mean what the spec says it means: run the release
guard on every publication path, publish a commit that already carries its
version, build on one old glibc baseline with one pinned toolchain,
profile-guided-optimize every distributed binary, and publish crates.io in
dependency order with the artifacts last.

Read as text, like `test_ci_precommit_parity.py`: the CI Python has no YAML
parser. What is asserted is job and step *shape* — which step precedes which,
what a condition names, what a budget multiplies out to — never the formatting,
so a reflow of these files does not fail this module."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
RELEASE = WORKFLOWS / "release.yml"
PGO_SCRIPT = REPO_ROOT / "scripts" / "pgo-build.sh"
BENCHES = REPO_ROOT / "crates" / "grund-cli" / "benches" / "instructions.rs"
GUARD = "scripts/check_release_ramps.py"


def squash(text):
    return re.sub(r"\s+", " ", text).strip()


def jobs(text):
    """The body of each job in a workflow, keyed by job id.

    A job id is the one key indented two spaces under `jobs:`; everything up to
    the next such key is that job's body."""
    lines = text.splitlines()
    body, current = {}, None
    for line in lines[lines.index("jobs:") + 1:]:
        if line.strip() and not line.startswith(" "):
            break
        header = re.fullmatch(r"  ([A-Za-z0-9_-]+):", line.rstrip())
        if header:
            current = header.group(1)
            body[current] = []
            continue
        if current is not None:
            body[current].append(line)
    return {name: "\n".join(lines) for name, lines in body.items()}


def steps(job_body):
    """One string per step of a job, in order.

    A step opens at the first indentation a `- ` appears at below `steps:`, so
    deeper list items (a `with: path:` block) stay inside the step they belong
    to and the job's own `needs:`/`strategy:` lists never reach here."""
    if "steps:" not in job_body:
        return []
    found, current, indent = [], None, None
    for line in job_body.split("steps:", 1)[1].splitlines():
        stripped = line.strip()
        if not stripped:
            if current is not None:
                current.append(line)
            continue
        column = len(line) - len(line.lstrip())
        if stripped.startswith("- ") and indent in (None, column):
            indent = column
            current = [line]
            found.append(current)
        elif current is not None:
            current.append(line)
    return ["\n".join(step) for step in found]


def step_name(step):
    named = re.search(r"(?:^|\n)\s*-?\s*name:\s*(.+)", step)
    return named.group(1).strip() if named else ""


def training_invocations(script):
    """The instrumented-binary calls of `pgo-build.sh`'s training loop.

    Bounded by the `set +e` / `set -e` pair the loop runs under, so the
    script's own closing `--version` smoke call is not mistaken for training
    input."""
    block = script.split("\nset +e\n", 1)[1].split("\nset -e\n", 1)[0]
    return [line.strip() for line in block.splitlines() if line.strip().startswith('"$grund" ')]


def step_index(job_steps, wanted):
    """The position of the step with exactly this name, or -1.

    Exact, not a substring: `Publish grund` and `Publish grund-core` are two
    steps whose order is the subject of §FS-distribution.4.10."""
    for index, step in enumerate(job_steps):
        if step_name(step) == wanted:
            return index
    return -1


class ReleaseGuardTests(unittest.TestCase):
    """§FS-distribution.4.2.7 — every publication path runs the release guard,
    on the version it is about to publish or compute rather than on a literal
    somebody has to remember to change."""

    def test_the_verify_job_runs_the_guard_on_the_resolved_version(self):
        verify = squash(jobs(RELEASE.read_text(encoding="utf-8"))["verify"])
        self.assertIn(f"run: python3 {GUARD} \"${{{{ steps.version.outputs.version }}}}\"", verify)

    def test_both_bump_helpers_run_the_guard_on_the_version_they_compute(self):
        for name in ("auto-bump.yml", "release-minor.yml"):
            with self.subTest(workflow=name):
                text = squash((WORKFLOWS / name).read_text(encoding="utf-8"))
                self.assertIn(f"python3 {GUARD} \"${{{{ steps.versions.outputs.next }}}}\"", text)

    def test_no_publication_path_runs_the_guard_on_a_literal_version(self):
        for name in ("release.yml", "auto-bump.yml", "release-minor.yml"):
            with self.subTest(workflow=name):
                text = (WORKFLOWS / name).read_text(encoding="utf-8")
                calls = re.findall(rf"{re.escape(GUARD)}\s+(\S+)", text)
                self.assertTrue(calls, f"{name} does not run the release guard at all")
                for call in calls:
                    self.assertIn("${{", call, f"{name} runs the guard on a hard-coded version")


class VerifyJobTests(unittest.TestCase):
    """§FS-distribution.4.3 — `release.yml` publishes a commit that already
    carries its version: it verifies the requested version against the Cargo
    packages it publishes, re-runs the package-name guard, and bumps nothing."""

    @classmethod
    def setUpClass(cls):
        cls.text = RELEASE.read_text(encoding="utf-8")
        cls.jobs = jobs(cls.text)
        cls.verify = squash(cls.jobs["verify"])

    def test_the_requested_version_is_checked_against_the_published_packages(self):
        """Two manifest reads cover all three packages only because two of them
        inherit the workspace version; a package that took a literal version of
        its own would leave this job checking a version nobody publishes."""
        self.assertIn("workspace_version()", self.verify)
        self.assertIn("manifest_version \"$manifest_ref\" crates/grund-core/Cargo.toml", self.verify)
        for package in ("grund-cli", "grund-lsp"):
            with self.subTest(package=package):
                manifest = (REPO_ROOT / "crates" / package / "Cargo.toml").read_text(encoding="utf-8")
                inherits = re.search(r"^version\.workspace\s*=\s*true$", manifest, re.M)
                self.assertIsNotNone(inherits, f"{package} no longer inherits the workspace version")

    def test_a_mismatched_package_version_fails_the_job(self):
        for label in ("root package version is", "grund-core version is"):
            with self.subTest(check=label):
                self.assertIn(f"echo \"error: ${{manifest_ref}}: {label}", self.verify)

    def test_the_package_name_guard_is_re_run_before_anything_is_built(self):
        self.assertIn("run: bash scripts/check-registry-names.sh", self.verify)
        names = step_index(steps(self.jobs["verify"]), "Check registry names")
        self.assertNotEqual(names, -1)

    def test_the_crates_io_token_preflight_runs_when_publishing_is_enabled(self):
        preflight = steps(self.jobs["verify"])[step_index(steps(self.jobs["verify"]), "Preflight crates.io token")]
        self.assertIn("steps.mode.outputs.publish_crates == 'true'", squash(preflight))

    def test_the_release_workflow_bumps_no_version(self):
        """The bump belongs to the helpers, and the difference is the point of
        §FS-distribution.4.3 — so the search term is shown to be a real one by
        finding it in the workflow that *does* bump."""
        bumper = (WORKFLOWS / "auto-bump.yml").read_text(encoding="utf-8")
        self.assertIn("cargo set-version", bumper)
        self.assertNotIn("cargo set-version", self.text)
        self.assertNotIn("cargo-edit", self.text)

    def test_every_advertised_target_is_built(self):
        targets = ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu")
        runners = ("macos-15-intel", "macos-15", "windows-latest", "windows-11-arm")
        squashed = squash(self.text)
        for target in targets:
            with self.subTest(target=target):
                self.assertIn(f"triple: {target}", squashed)
        for runner in runners:
            with self.subTest(runner=runner):
                self.assertIn(f"os: {runner}", squashed)


class GlibcBaselineAndToolchainTests(unittest.TestCase):
    """§FS-distribution.4.8 — one old glibc baseline, one pinned toolchain."""

    @classmethod
    def setUpClass(cls):
        cls.text = RELEASE.read_text(encoding="utf-8")
        cls.jobs = jobs(cls.text)

    def test_both_linux_binaries_are_built_in_a_digest_pinned_manylinux_image(self):
        images = re.findall(r"image:\s*\"?([^\"\s]+)\"?", self.jobs["build-linux-gnu-pgo"])
        self.assertEqual(2, len(images), images)
        seen = set()
        for image in images:
            with self.subTest(image=image):
                pinned = re.fullmatch(r"\S+/manylinux2014_(\w+)@sha256:[0-9a-f]{64}", image)
                self.assertIsNotNone(pinned, "a release image must be pinned by digest, not by tag")
                seen.add(pinned.group(1))
        self.assertEqual({"x86_64", "aarch64"}, seen)

    def test_the_pinned_toolchain_is_a_single_exact_version(self):
        pin = re.search(r"RUST_TOOLCHAIN:\s*\"?(\d+\.\d+\.\d+)\"?", self.text)
        self.assertIsNotNone(pin, "release.yml pins no exact Rust toolchain")

    def test_every_job_that_installs_rust_installs_the_pinned_toolchain(self):
        installs = 0
        for name, body in self.jobs.items():
            for step in steps(body):
                flat = squash(step)
                if "dtolnay/rust-toolchain" in flat:
                    installs += 1
                    with self.subTest(job=name, step=step_name(step)):
                        self.assertIn("toolchain: ${{ env.RUST_TOOLCHAIN }}", flat)
                if "sh.rustup.rs" in flat:
                    installs += 1
                    with self.subTest(job=name, step=step_name(step)):
                        self.assertIn("--default-toolchain \"$RUST_TOOLCHAIN\"", flat)
                        self.assertIn("-e RUST_TOOLCHAIN", flat)
        self.assertGreaterEqual(installs, 4, "expected host-runner and in-container installs")


class ProfileGuidedOptimizationTests(unittest.TestCase):
    """§FS-distribution.4.9 — the distributed binaries are PGO'd, the training
    corpus is this repository's own tree under the benchmark hot command list,
    and nothing is packaged that has not self-checked — the LTO-only fallback
    included."""

    @classmethod
    def setUpClass(cls):
        cls.jobs = jobs(RELEASE.read_text(encoding="utf-8"))
        cls.script = PGO_SCRIPT.read_text(encoding="utf-8")

    def test_every_platform_build_runs_the_pgo_script(self):
        for name in ("build-linux-gnu-pgo", "build-native-pgo"):
            with self.subTest(job=name):
                self.assertIn("bash scripts/pgo-build.sh", squash(self.jobs[name]))

    def test_no_binary_is_packaged_before_it_self_checks(self):
        for name, check in (
            ("build-linux-gnu-pgo", "Self-check PGO release binary"),
            ("build-native-pgo", "Self-check release binary"),
        ):
            with self.subTest(job=name):
                job_steps = steps(self.jobs[name])
                self_check = step_index(job_steps, check)
                self.assertNotEqual(self_check, -1)
                self.assertLess(self_check, step_index(job_steps, "Package binary"))
                self.assertLess(self_check, step_index(job_steps, "Upload binary artifact"))
                self.assertIn("check . --format json", squash(job_steps[self_check]))
                self.assertNotIn("if:", squash(job_steps[self_check]))

    def test_the_lto_fallback_is_only_for_a_platform_whose_pgo_is_not_required(self):
        job_steps = steps(self.jobs["build-native-pgo"])
        fallback = step_index(job_steps, "Build LTO release binary fallback")
        self.assertNotEqual(fallback, -1)
        condition = squash(job_steps[fallback])
        self.assertIn("! matrix.pgo_required", condition)
        self.assertIn("steps.pgo.outcome == 'failure'", condition)
        self.assertGreater(fallback, step_index(job_steps, "Build PGO release binary"))
        self.assertLess(fallback, step_index(job_steps, "Self-check release binary"))

    def test_exactly_one_platform_may_fall_back(self):
        matrix = self.jobs["build-native-pgo"].split("steps:", 1)[0]
        self.assertEqual(1, len(re.findall(r"pgo_required:\s*false", matrix)))
        self.assertEqual(3, len(re.findall(r"pgo_required:\s*true", matrix)))

    def test_the_training_run_uses_this_repositorys_own_tree(self):
        trainers = training_invocations(self.script)
        self.assertGreaterEqual(len(trainers), 6, trainers)
        for invocation in trainers:
            with self.subTest(invocation=invocation):
                self.assertIn('"$repo"', invocation)

    def test_the_training_commands_are_the_benchmark_hot_command_list(self):
        """§AR-benchmarks' list and the training corpus are one list by
        contract, so the two files are compared rather than each pinned to a
        hand-written copy of it."""
        trained = {invocation.split()[1] for invocation in training_invocations(self.script)}
        benched = set(re.findall(r'\.args?\(\[?"([a-z]+)"', BENCHES.read_text(encoding="utf-8")))
        self.assertEqual(benched, trained)
        for depth in ("show FS-check --brief", "show FS-check", "show FS-check --full", "fmt --check"):
            with self.subTest(command=depth):
                self.assertIn(f'"$grund" {depth}', squash(self.script))

    def test_an_empty_training_run_fails_the_build(self):
        self.assertIn("PGO training produced no .profraw files", self.script)


class CratesIoPublishOrderTests(unittest.TestCase):
    """§FS-distribution.4.10 — crates.io publishes in dependency order, the
    dependent publishes wait for resolution, and artifacts upload last."""

    @classmethod
    def setUpClass(cls):
        cls.jobs = jobs(RELEASE.read_text(encoding="utf-8"))
        cls.publish = steps(cls.jobs["publish-crates"])

    def test_the_core_publishes_before_the_crates_that_depend_on_it(self):
        core = step_index(self.publish, "Publish grund-core")
        wait = step_index(self.publish, "Wait for grund-core dependency resolution")
        self.assertNotEqual(core, -1)
        self.assertLess(core, wait)
        for dependent in ("Publish grund-lsp", "Publish grund"):
            with self.subTest(step=dependent):
                self.assertLess(wait, step_index(self.publish, dependent))

    def test_only_the_unpublished_crates_are_published_again(self):
        for step, gate in (
            ("Publish grund-core", "steps.state.outputs.core_published == 'false'"),
            ("Publish grund-lsp", "steps.state.outputs.lsp_published == 'false'"),
        ):
            with self.subTest(step=step):
                self.assertIn(gate, squash(self.publish[step_index(self.publish, step)]))

    def test_the_wait_for_resolution_is_bounded_at_thirty_minutes(self):
        wait = self.publish[step_index(self.publish, "Wait for grund-core dependency resolution")]
        attempts = int(re.search(r"max_attempts=(\d+)", wait).group(1))
        seconds = int(re.search(r"sleep_seconds=(\d+)", wait).group(1))
        self.assertEqual(30 * 60, attempts * seconds)

    def test_the_platform_builds_gate_the_publish(self):
        needs = self.jobs["publish-crates"].split("steps:", 1)[0]
        for job in ("verify", "build-linux-gnu-pgo", "build-native-pgo"):
            with self.subTest(needs=job):
                self.assertRegex(needs, rf"-\s+{re.escape(job)}\b")

    def test_artifacts_upload_only_after_the_builds_and_the_publish(self):
        release = self.jobs["github-release"].split("steps:", 1)[0]
        for job in ("build-linux-gnu-pgo", "build-native-pgo", "publish-crates"):
            with self.subTest(needs=job):
                self.assertRegex(release, rf"-\s+{re.escape(job)}\b")
        condition = squash(release)
        self.assertIn("needs.build-linux-gnu-pgo.result == 'success'", condition)
        self.assertIn("needs.build-native-pgo.result == 'success'", condition)
        self.assertIn(
            "needs.publish-crates.result == 'success' || needs.publish-crates.result == 'skipped'",
            condition,
            "a failed crates.io publish must not still ship the artifacts",
        )


if __name__ == "__main__":
    unittest.main()
