"""§AR-ci.1 — CI is the remote form of the local pre-commit gate: the workflow
runs the hook list itself rather than a hand-copy of it, installs every binary a
hook needs before that step, gives every hook bound only to a stage without a
file list the explicit counterpart that stage's input demands (§AR-ci.1.1,
§AR-ci.8), and the Rust hooks spell the commands the workflow's own steps spell,
warnings denied on both sides (§AR-ci.3), and §AR-ci.3's own prose spells them
too, so the architecture hands a reader the command the gate runs. Hook/workflow
files are read as text: the CI Python has no YAML parser. The Python wrapper's
discovery argv is inspected as syntax, without executing its setup or tests.

§AR-ci.1.2 divides the work: this test holds the *existence* of a counterpart,
because that is the half a line-shaped read can see. The other half — that the
counterpart reaches the same verdict on the same tree — is held by each gate's
own test, `tests/integration/test_check_no_claude_attribution.py` for the
attribution gate.

§FS-distribution.4.6 takes one gate away rather than adding one: no hook and no
workflow job asks a change for a changelog entry (§AR-ci.7)."""

import ast
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRE_COMMIT = REPO_ROOT / ".pre-commit-config.yaml"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
CI = WORKFLOWS / "ci.yml"
AR_CI = REPO_ROOT / "docs" / "architecture" / "AR-ci.md"
PYTHON_GATE = REPO_ROOT / "scripts" / "run_python_gate.py"
RUST_HOOKS = ("cargo-fmt-check", "cargo-build", "cargo-test")
FILE_LIST_STAGES = ("pre-commit", "manual")
ENV_PREFIX = "env RUSTFLAGS=-Dwarnings "
# The release helper writes the release section (§FS-distribution.4.6); it gates no change.
RELEASE_HELPER = "scripts/prepare_changelog_release.py"


def _hooks(text):
    hooks, current = [], None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        started = re.match(r"- id:\s*(\S+)$", line)
        if started:
            current = {"id": started.group(1)}
            hooks.append(current)
            continue
        if current is not None:
            key, sep, value = line.partition(":")
            if sep and re.fullmatch(r"[a-z_]+", key):
                current[key] = value.strip()
    return hooks


def _lines(text):
    return [line.strip() for line in text.splitlines() if line.strip() and not line.strip().startswith("#")]


def _run_lines(lines):
    return [line[len("run:"):].strip() for line in lines if line.startswith("run:")]


def _section_lead(text, heading):
    """The code spans of one section's lead: the lines after `heading` up to the next heading."""
    lines = text.splitlines()
    start = lines.index(heading) + 1
    end = next((i for i in range(start, len(lines)) if lines[i].startswith("#")), len(lines))
    lead = " ".join(lines[start:end])
    return [" ".join(span.split()) for span in re.findall(r"`([^`]+)`", lead)]


class CiPreCommitParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pre_commit_text = PRE_COMMIT.read_text(encoding="utf-8")
        cls.hooks = _hooks(cls.pre_commit_text)
        cls.ci_lines = _lines(CI.read_text(encoding="utf-8"))
        cls.ci_runs = _run_lines(cls.ci_lines)
        cls.by_id = {hook["id"]: hook for hook in cls.hooks}

    def test_the_hook_list_was_parsed(self):
        self.assertGreaterEqual(len(self.hooks), 8)
        for hook in self.hooks:
            self.assertIn("entry", hook, hook["id"])
            self.assertIn("stages", hook, hook["id"])

    def test_ci_runs_the_hook_list_itself(self):
        self.assertIn("pre-commit run --all-files", self.ci_runs)

    def test_every_external_binary_is_installed_before_the_hooks_run(self):
        gate = self.ci_lines.index("run: pre-commit run --all-files")
        for hook in self.hooks:
            binary = hook["entry"].split()[0]
            if binary in ("cargo", "python", "env"):
                continue
            with self.subTest(binary=binary):
                installs = [
                    index
                    for index, line in enumerate(self.ci_lines)
                    if re.match(rf"cargo install {re.escape(binary)} --version \S+ --locked$", line)
                ]
                self.assertTrue(installs, f"ci.yml pins no `cargo install {binary} --version`")
                self.assertLess(installs[0], gate, f"{binary} is installed after the pre-commit step")

    def test_rust_hooks_spell_the_commands_ci_runs(self):
        for hook_id in RUST_HOOKS:
            with self.subTest(hook=hook_id):
                command = self.by_id[hook_id]["entry"].removeprefix(ENV_PREFIX)
                self.assertIn(command, self.ci_runs, f"{hook_id} runs a command no CI step runs")

    def test_ar_ci_3_spells_the_commands_the_rust_hooks_run(self):
        # §AR-ci.3 is where a reader looks up what the gate runs; a command it
        # spells short of the hook's is one that fails where the gate passes.
        spans = _section_lead(AR_CI.read_text(encoding="utf-8"), "## 3. Current hooks")
        for hook_id in RUST_HOOKS:
            with self.subTest(hook=hook_id):
                command = self.by_id[hook_id]["entry"].removeprefix(ENV_PREFIX)
                self.assertIn(command, spans, f"§AR-ci.3 does not spell the command {hook_id} runs")

    def test_warnings_are_denied_on_both_sides(self):
        self.assertTrue(self.by_id["cargo-build"]["entry"].startswith(ENV_PREFIX))
        rustflags = [line for line in self.ci_lines if line.startswith("RUSTFLAGS:")]
        self.assertEqual(1, len(rustflags), rustflags)
        self.assertEqual("-Dwarnings", re.sub(r'[\s"]', "", rustflags[0].partition(":")[2]))

    def test_every_commit_msg_hook_has_a_range_scanning_counterpart(self):
        counterparts = 0
        for hook in self.hooks:
            if "commit-msg" not in hook["stages"]:
                continue
            counterparts += 1
            script = next(token for token in hook["entry"].split() if token.startswith("scripts/"))
            with self.subTest(hook=hook["id"]):
                self.assertTrue(
                    any(script in line and "--range" in line for line in self.ci_runs),
                    f"no CI step feeds {script} a commit range",
                )
        self.assertGreaterEqual(counterparts, 1)

    def test_every_hook_without_a_file_list_stage_has_a_ci_step_naming_its_script(self):
        counterparts = 0
        for hook in self.hooks:
            stages = {item.strip() for item in hook["stages"].strip("[]").split(",")}
            if stages & set(FILE_LIST_STAGES):
                continue
            counterparts += 1
            script = next(token for token in hook["entry"].split() if token.startswith("scripts/"))
            with self.subTest(hook=hook["id"]):
                self.assertTrue(
                    any(script in line for line in self.ci_runs),
                    f"{hook['id']} runs at no stage `pre-commit run --all-files` reproduces "
                    f"and no CI step runs {script}",
                )
        self.assertGreaterEqual(counterparts, 1)

    def test_no_hook_asks_a_change_for_a_changelog_entry(self):
        # §FS-distribution.4.6: the changelog is written before a release, so no hook, at any
        # stage, gates a change on it.
        hooks = [hook["id"] for hook in self.hooks if "changelog" in f"{hook['id']} {hook['entry']}".lower()]
        self.assertEqual([], hooks, "a hook still gates a change on the changelog")

    def test_no_workflow_job_asks_a_change_for_a_changelog_entry(self):
        # §FS-distribution.4.6, §AR-ci.7: nor does a job of any workflow; the release helper only
        # releases the entries it finds.
        for workflow in sorted(WORKFLOWS.glob("*.yml")):
            text = workflow.read_text(encoding="utf-8")
            with self.subTest(workflow=workflow.name):
                job_ids = re.findall(r"^  ([A-Za-z0-9_-]+):\s*$", text.split("\njobs:", 1)[1], re.M)
                self.assertEqual([], [job for job in job_ids if "changelog" in job.lower()])
                scripts = set(re.findall(r"scripts/[\w./-]*changelog[\w.-]*", text)) - {RELEASE_HELPER}
                self.assertEqual(set(), scripts, "a workflow still runs a changelog check")

    def assert_python_discovery_parity(self, hook, ci_runs, wrapper):
        """Shared setup must reach this test home (§AR-ci.1.2, §FS-cochange-recipe.examples)."""
        home = Path(__file__).resolve().parent.relative_to(REPO_ROOT).as_posix()
        command = "python scripts/run_python_gate.py"
        self.assertEqual(command, hook)
        ci = [run for run in ci_runs if "run_python_gate.py" in run]
        self.assertEqual(1, len(ci), ci)
        self.assertEqual(hook, ci[0])
        calls = [node for node in ast.walk(ast.parse(wrapper))
                 if isinstance(node, ast.Call) and ast.unparse(node.func) == "subprocess.run"
                 and node.args and isinstance(node.args[0], ast.List)
                 and any(isinstance(arg, ast.Constant) and arg.value == "unittest"
                         for arg in node.args[0].elts)]
        self.assertEqual(1, len(calls))
        argv = calls[0].args[0].elts
        self.assertEqual("sys.executable", ast.unparse(argv[0]))
        self.assertEqual(["-m", "unittest", "discover", "-s", home, "-p", "test_*.py"],
                         [ast.literal_eval(arg) for arg in argv[1:]])
        keywords = {keyword.arg: ast.unparse(keyword.value) for keyword in calls[0].keywords}
        self.assertEqual("ROOT", keywords.get("cwd"))
        self.assertEqual("inputs()", keywords.get("env"))

    def test_python_tests_are_discovered_from_this_home_on_both_sides(self):
        self.assert_python_discovery_parity(self.by_id["python-test"]["entry"], self.ci_runs,
                                            PYTHON_GATE.read_text(encoding="utf-8"))

    def test_python_discovery_parity_rejects_wrong_home_and_mismatched_invocations(self):
        wrapper = PYTHON_GATE.read_text(encoding="utf-8")
        hook = self.by_id["python-test"]["entry"]
        home = Path(__file__).resolve().parent.relative_to(REPO_ROOT).as_posix()
        wrong_home = wrapper.replace(home, "tests/e2e")
        self.assertNotEqual(wrapper, wrong_home)
        variants = ((hook, self.ci_runs, wrong_home),
                    (hook + " --other", self.ci_runs, wrapper),
                    (hook, [run + " --other" if run == hook else run for run in self.ci_runs], wrapper))
        for candidate in variants:
            with self.subTest(hook=candidate[0], ci=candidate[1]):
                with self.assertRaises(AssertionError):
                    self.assert_python_discovery_parity(*candidate)

    def test_ar_ci_3_spells_the_python_wrapper_command(self):
        spans = _section_lead(AR_CI.read_text(encoding="utf-8"), "## 3. Current hooks")
        self.assertIn(self.by_id["python-test"]["entry"], spans)

    def test_a_clone_installs_every_stage_a_hook_uses(self):
        installed = re.search(r"default_install_hook_types:\s*\[(.*)\]", self.pre_commit_text)
        self.assertIsNotNone(installed)
        types = {item.strip() for item in installed.group(1).split(",")}
        used = set()
        for hook in self.hooks:
            used.update(item.strip() for item in hook["stages"].strip("[]").split(","))
        self.assertEqual(set(), used - types, "a hook stage no clone installs is a hook nobody runs")


if __name__ == "__main__":
    unittest.main()
