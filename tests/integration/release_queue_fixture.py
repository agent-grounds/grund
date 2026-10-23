"""The fixture a release helper's whole job runs on while the merge queue moves
`main` (§FS-distribution.4.4.1.1, §AR-ci.11).

`Case` builds a local bare origin that stands for agent-grounds/grund - the
version sources, changelog and `scripts/` of this checkout, at `0.19.1-dev` over
a `v0.19.0` tag, one merged pull request `A` on `main`, and the commits the
queue can land held under `refs/queue/` where no fetch looks - and runs every
`run:` step of the helper's one job, from the first, the way GitHub runs it:
under `bash --noprofile --norc -eo pipefail`, `${{ }}` filled in, `if:` read,
in a clone of that origin. `gh` is `release_queue_forge.py`, and `cargo` a stub
that accepts only `cargo update --workspace`, so nothing reaches the network.

The workflow is read as text, as `test_release_workflow.py` reads it, because
the Python gate has no YAML parser. It is not named `test_*`, so `unittest
discover` imports it only from the modules that use it."""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from changelog_gate_fixture import REPO_ROOT, environment
from test_release_workflow import WORKFLOWS, jobs, steps

FORGE = Path(__file__).resolve().parent / "release_queue_forge.py"
RUN_ID, PREV, NEXT, DEV = "39962684811", "0.19.0", "0.20.0", "0.20.1-dev"
STEP_TIMEOUT = 600
# The tree the helper reads: version sources, changelog, the ledger and the scripts.
SKELETON = ("Cargo.toml", "Cargo.lock", "crates/grund-core/Cargo.toml", "crates/grund-cli/Cargo.toml",
            "crates/grund-lsp/Cargo.toml", "crates/grund-py/pyproject.toml",
            "tests/e2e/cases/cli-version/expected.stdout",
            "tests/integration/command_agreement_facts/cli-version.json", "docs/changelog.md",
            "docs/changelog", "docs/discussions/proposals/2026-09-30-core-concerns.md", "scripts")
# Commits the queue can land, each over the one before it unless it names its parent.
QUEUE = (("m1", None, "Speed up the scanner's queue path (#975)", {"queued.rs": "// 1\n"}),
         ("m2", "m1", "Name the queue's second group (#976)", {"queued.rs": "// 2\n"}),
         ("m3", "m2", "Name the queue's third group (#977)", {"queued.rs": "// 3\n"}),
         ("m4", "m3", "Name the queue's fourth group (#978)", {"queued.rs": "// 4\n"}),
         ("m5", "m4", "Name the queue's fifth group (#979)", {"queued.rs": "// 5\n"}),
         ("m6", "m5", "Name the queue's sixth group (#980)", {"queued.rs": "// 6\n"}),
         # A ramp landed for a later release refuses either helper (§FS-distribution.4.2.2).
         ("ramp", None, "Warn on the old spelling (#981)",
          {"ramp.rs": 'pub const W: &str = "the old spelling became an error in 0.21.0";\n'}))
_TEMPLATE = None


class CannotRun(AssertionError):
    """The fixture has no answer: a call or an expression it does not model."""


def git(*args, cwd=None, env=None):
    result = subprocess.run(["git", *args], cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode != 0:
        raise CannotRun(f"fixture: `git {' '.join(args)}` failed: {result.stderr.strip()}")
    return result.stdout.strip()


def _isolated(home):
    config = home / "gitconfig"
    config.write_text("[init]\n\tdefaultBranch = main\n[advice]\n\tdetachedHead = false\n"
                      "[gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n")
    return environment({"GIT_CONFIG_GLOBAL": str(config), "GIT_CONFIG_NOSYSTEM": "1",
                        "GIT_TERMINAL_PROMPT": "0"})


def template():
    """The origin every case copies, built once per process: {name: sha} beside it."""
    global _TEMPLATE
    if _TEMPLATE is not None:
        return _TEMPLATE
    root = Path(tempfile.mkdtemp(prefix="grund-983-queue-template-"))
    env = _isolated(root)
    who = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid")
    seed, origin = root / "seed", root / "origin.git"
    git("init", "-q", str(seed), env=env)
    for relative in SKELETON:
        source, target = REPO_ROOT / relative, seed / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(source, target, ignore=shutil.ignore_patterns("__pycache__"))
        else:
            shutil.copy2(source, target)

    def commit(message):
        git("-C", str(seed), "add", "-A", env=env)
        git("-C", str(seed), *who, "commit", "-q", "--allow-empty", "-m", message, env=env)
        return git("-C", str(seed), "rev-parse", "HEAD", env=env)

    def set_version(version):
        subprocess.run([sys.executable, "scripts/distribution/candidate.py", "set-version", version],
                       cwd=seed, env=env, check=True, capture_output=True)

    set_version(PREV)
    commit(f"Release v{PREV}")
    git("-C", str(seed), "tag", f"v{PREV}", env=env)
    set_version("0.19.1-dev")
    commit("Open 0.19.1-dev for development")
    (seed / "crates/grund-core/src").mkdir(parents=True)
    (seed / "crates/grund-core/src/lib.rs").write_text("// the scanner\n")
    shas = {"a": commit("Speed up the scanner (#970)")}
    for name, parent, message, files in QUEUE:
        git("-C", str(seed), "switch", "-q", "--detach", shas[parent or "a"], env=env)
        for file, text in files.items():
            (seed / "crates/grund-core/src" / file).write_text(text)
        shas[name] = commit(message)
    # A pull request that moves every version source (§FS-distribution.4.4.1.2).
    git("-C", str(seed), "switch", "-q", "--detach", shas["a"], env=env)
    set_version("0.19.2-dev")
    shas["versions"] = commit("Move main to 0.19.2-dev by hand (#982)")
    git("init", "-q", "--bare", str(origin), env=env)
    refs = [f"{sha}:refs/queue/{name}" for name, sha in shas.items() if name != "a"]
    git("-C", str(seed), "push", "-q", str(origin), f"{shas['a']}:refs/heads/main", "--tags", *refs, env=env)
    _TEMPLATE = (origin, shas)
    return _TEMPLATE


# Pushes to main as the scenario's `push_main` asks: `refuse` every one; `race` lands the
# next queued commit first and refuses the push; `land` takes it and still reports failure.
PRE_RECEIVE = r"""#!/bin/sh
state="{state}"
while read old new ref; do
  [ "$ref" = refs/heads/main ] || continue
  mode=$(cat "$state/push_main" 2>/dev/null || true)
  clean="env -u GIT_QUARANTINE_PATH -u GIT_OBJECT_DIRECTORY -u GIT_ALTERNATE_OBJECT_DIRECTORIES"
  case "$mode" in
    refuse) echo "simulated rejection: the push to main was refused" >&2; exit 1 ;;
    race) rm -f "$state/push_main"; $clean git update-ref refs/heads/main "$(cat "$state/race")"
          echo "simulated race: the merge queue landed first" >&2; exit 1 ;;
    land) rm -f "$state/push_main"; $clean git update-ref refs/heads/main "$new"
          echo "simulated lost answer: the push landed but its answer did not" >&2; exit 1 ;;
  esac
  [ "$old" = 0000000000000000000000000000000000000000 ] && continue
  git merge-base --is-ancestor "$old" "$new" && continue
  echo "main is protected: $new does not contain $old" >&2; exit 1
done
"""


class Case:
    """One run of one helper's job against its own copy of the origin."""

    def __init__(self, scratch: Path, workflow: str, scenario=None, inputs=None, knobs=None, timeout=STEP_TIMEOUT):
        self.workflow, self.inputs, self.timeout = workflow, dict(inputs or {}), timeout
        self.dir = Path(tempfile.mkdtemp(prefix=f"{workflow.split('.')[0]}-", dir=scratch))
        source, self.shas = template()
        self.origin, self.work, self.state = self.dir / "origin.git", self.dir / "work", self.dir / "forge"
        shutil.copytree(source, self.origin)
        for path in (self.state, self.dir / "bin", self.dir / "gh"):
            path.mkdir()
        scenario = {key: ({r: self.resolve(v) for r, v in value.items()} if isinstance(value, dict) else value)
                    for key, value in (scenario or {}).items()}
        (self.state / "scenario.json").write_text(json.dumps(scenario))
        hooks = self.origin / "hooks"
        (hooks / "pre-receive").write_text(PRE_RECEIVE.replace("{state}", str(self.state)))
        (hooks / "post-receive").write_text(f'#!/bin/sh\ncat >> "{self.state / "pushes"}"\n')
        for hook in ("pre-receive", "post-receive"):
            (hooks / hook).chmod(0o755)
        self.env = _isolated(self.dir)
        self.env.update({
            "PATH": f"{self.dir / 'bin'}{os.pathsep}{os.environ.get('PATH', '')}",
            "GH_TOKEN": "stub", "GH_HOST": "github.invalid", "GH_CONFIG_DIR": str(self.dir / "gh"),
            "GH_PROMPT_DISABLED": "1", "GH_STUB_STATE": str(self.state), "GH_STUB_ORIGIN": str(self.origin),
            "PYTHONUTF8": "1",
            # Every wait the helper's scripts make, zeroed unless the case sets it.
            "CANDIDATE_RUNS_POLL": "0", "CANDIDATE_RUNS_WATCH": "0", "GRUND_ADVANCE_RETRY_DELAY": "0",
            "GRUND_RELEASE_SETTLE": "0",
        })
        self.env.update({key: str(value) for key, value in (knobs or {}).items()})
        for name, body in (("gh", f'exec "{sys.executable}" "{FORGE}" "$@"'),
                           ("cargo", '[ "$*" = "update --workspace" ] && exit 0\n'
                                     'echo "cargo stub: cargo $* is not modelled" >&2; exit 3')):
            (self.dir / "bin" / name).write_text(f"#!/bin/sh\n{body}\n")
            (self.dir / "bin" / name).chmod(0o755)
        git("clone", "-q", str(self.origin), str(self.work), env=self.env)
        (self.state / "pushes").write_text("")

    def resolve(self, value):
        """A queue commit's name in a scenario becomes its sha."""
        if isinstance(value, str) and value in self.shas:
            return self.shas[value]
        return value

    def push_main(self, mode: str, race: str = "m1"):
        (self.state / "push_main").write_text(mode)
        (self.state / "race").write_text(self.shas[race])

    # -- the job -------------------------------------------------------------------------

    def value(self, expr):
        expr = expr.strip()
        found = re.fullmatch(r"steps\.([\w-]+)\.outputs\.([\w-]+)", expr)
        if found:
            return self.outputs.get(found[1], {}).get(found[2], "")
        found = re.fullmatch(r"inputs\.([\w-]+)", expr)
        if found:
            return self.inputs.get(found[1], "")
        github = {"run_id": RUN_ID, "run_attempt": "1", "ref": "refs/heads/main", "ref_name": "main",
                  "sha": self.shas["a"], "token": "stub", "repository": "agent-grounds/grund",
                  "server_url": "https://github.invalid", "event_name": "workflow_dispatch",
                  "workspace": str(self.work)}
        found = re.fullmatch(r"github\.([\w-]+)", expr)
        if found and found[1] in github:
            return github[found[1]]
        if re.fullmatch(r"secrets\.\w+", expr):
            return "stub-secret"
        found = re.fullmatch(r"env\.([\w-]+)", expr)
        if found and found[1] in self.job_env:
            return self.job_env[found[1]]
        raise CannotRun(f"`${{{{ {expr} }}}}` is not an expression this fixture models")

    def fill(self, text):
        return re.sub(r"\$\{\{(.*?)\}\}", lambda m: self.value(m[1]), str(text))

    def condition(self, expr, ok):
        expr = expr.strip()
        if expr.startswith("${{") and expr.endswith("}}"):
            expr = expr[3:-2].strip()
        status = re.search(r"\b(success|failure|always|cancelled)\(\)", expr)
        return Condition(self, expr, ok).read() and (status is not None or ok)

    def run(self):
        """Run the job; return (ok, [(name, exit code, stdout, stderr)])."""
        text = (WORKFLOWS / self.workflow).read_text(encoding="utf-8")
        (body,) = jobs(text).values()
        self.outputs, self.job_env = {}, {}
        self.job_env = {k: self.fill(v) for k, v in {**mapping(text, "env", 0), **mapping(body, "env", 4)}.items()}
        ok, self.ran = True, []
        for n, step in enumerate(map(parse_step, steps(body))):
            name = step.get("name") or step.get("uses") or f"step {n}"
            if "uses" in step or ("if" in step and not self.condition(step["if"], ok)) or (
                    "if" not in step and not ok):
                continue
            script, out = self.dir / f"step-{n}.sh", self.dir / f"step-{n}.output"
            script.write_text(self.fill(step["run"]))
            out.write_text("")
            env = dict(self.env, **self.job_env, **{k: self.fill(v) for k, v in step.get("env", {}).items()})
            env.update(GITHUB_OUTPUT=str(out), GITHUB_RUN_ID=RUN_ID, GITHUB_RUN_ATTEMPT="1",
                       GITHUB_REF="refs/heads/main", GITHUB_REF_NAME="main", GITHUB_REF_TYPE="branch",
                       GITHUB_SHA=self.shas["a"], GITHUB_REPOSITORY="agent-grounds/grund",
                       GITHUB_EVENT_NAME="workflow_dispatch", GITHUB_WORKSPACE=str(self.work),
                       RUNNER_TEMP=str(self.dir))
            try:
                result = subprocess.run(["bash", "--noprofile", "--norc", "-eo", "pipefail", str(script)],
                                        cwd=self.work, env=env, capture_output=True, text=True,
                                        timeout=self.timeout)
            except subprocess.TimeoutExpired:
                # The job's own timeout kills it, its runs still going.
                self.ran.append((name, -1, "", f"killed: the step ran past {self.timeout}s"))
                ok = False
                break
            if step.get("id"):
                self.outputs[step["id"]] = parse_output(out.read_text())
            self.ran.append((name, result.returncode, result.stdout, result.stderr))
            if result.returncode != 0 and step.get("continue-on-error") != "true":
                ok = False
        if (self.state / "unmodelled").exists():
            raise CannotRun("the job made a gh call the stub does not model: "
                            + (self.state / "unmodelled").read_text().strip())
        self.ok = ok
        return self

    # -- what came of it -----------------------------------------------------------------

    def stderr(self):
        return "\n".join(err for _name, _code, _out, err in self.ran)

    def report(self):
        lines = [f"{self.workflow}: job {'passed' if self.ok else 'failed'}"]
        for name, code, _out, err in self.ran:
            lines.append(f"  step {name!r}: exit {code}")
            lines += [f"    | {line}" for line in err.strip().splitlines()[-8:] if code]
        for run in self.runs():
            lines.append(f"  run {run['id']} r{run['round']} {run['workflow']}@{run['sha'][:10]} on "
                         f"{run['branch']} over {run['main'][:10]}: {run['status']}/{run['conclusion']}")
        lines.append(f"  main ends at {self.main()[:10]}; branches {self.branches()}")
        return "\n".join(lines)

    def runs(self, round_=None):
        path = self.state / "runs.json"
        runs = json.loads(path.read_text()) if path.exists() else []
        return [run for run in runs if round_ is None or run["round"] == round_]

    def rounds(self):
        return max([run["round"] for run in self.runs()] or [0])

    def published(self):
        return [run for run in self.runs() if run["workflow"] == "release.yml"
                and run["fields"].get("publish_crates") == "true"]

    def passed(self, sha, workflow, round_):
        return any(run["sha"] == sha and run["workflow"] == workflow and run["round"] == round_
                   and run["conclusion"] == "success" for run in self.runs())

    def main(self):
        return git("--git-dir", str(self.origin), "rev-parse", "refs/heads/main", env=self.env)

    def parent(self, sha):
        return git("--git-dir", str(self.origin), "rev-parse", f"{sha}^", env=self.env)

    def contains(self, sha, ancestor):
        return subprocess.run(["git", "--git-dir", str(self.origin), "merge-base", "--is-ancestor", ancestor,
                               sha], env=self.env).returncode == 0

    def version(self, sha):
        text = git("--git-dir", str(self.origin), "show", f"{sha}:Cargo.toml", env=self.env)
        return re.search(r'(?m)^version = "([^"]+)"', text)[1]

    def created(self):
        """The candidate branches the job created on origin, in order."""
        pushes = [line.split() for line in (self.state / "pushes").read_text().splitlines()]
        return [ref.removeprefix("refs/heads/") for old, _new, ref in pushes
                if set(old) == {"0"} and ref.startswith("refs/heads/release-candidate/")]

    def branches(self):
        out = git("--git-dir", str(self.origin), "for-each-ref", "--format=%(refname)",
                  "refs/heads/release-candidate/", env=self.env)
        return sorted(line.removeprefix("refs/heads/") for line in out.splitlines())


class Condition:
    """An `if:` expression: `&&`, `||`, `!`, parentheses, `==`/`!=` on strings, and
    the status functions, compared case-insensitively as GitHub compares them."""

    TOKEN = re.compile(r"\s*(&&|\|\||==|!=|!|\(|\)|'[^']*'|[\w.-]+\(\)|[\w.-]+)")

    def __init__(self, case, expr, ok):
        self.case, self.ok, self.tokens, at = case, ok, [], 0
        while at < len(expr.rstrip()):
            found = self.TOKEN.match(expr, at)
            if not found:
                raise CannotRun(f"cannot read the condition `if: {expr}`")
            self.tokens.append(found[1])
            at = found.end()

    def read(self):
        value = self.either()
        if self.tokens:
            raise CannotRun(f"trailing {self.tokens} in a condition")
        return bool(value)

    def either(self):
        value = self.both()
        while self.tokens[:1] == ["||"]:
            self.tokens.pop(0)
            right = self.both()
            value = value or right
        return value

    def both(self):
        value = self.compare()
        while self.tokens[:1] == ["&&"]:
            self.tokens.pop(0)
            right = self.compare()
            value = value and right
        return value

    def compare(self):
        value = self.atom()
        if self.tokens[:1] in (["=="], ["!="]):
            op, right = self.tokens.pop(0), self.atom()
            equal = str(value).lower() == str(right).lower()
            return equal if op == "==" else not equal
        return value

    def atom(self):
        token = self.tokens.pop(0)
        if token == "!":
            return not self.atom()
        if token == "(":
            value = self.either()
            self.tokens.pop(0)
            return value
        if token.startswith("'"):
            return token[1:-1]
        statuses = {"success()": self.ok, "failure()": not self.ok, "always()": True, "cancelled()": False}
        if token in statuses:
            return statuses[token]
        if token in ("true", "false"):
            return token
        return self.case.value(token)


def parse_step(text):
    """A step's keys: scalars, a `run: |` block, and the `env:` mapping."""
    lines = text.splitlines()
    lines[0] = re.sub(r"^(\s*)- ", lambda m: m[1] + "  ", lines[0])
    indent = len(lines[0]) - len(lines[0].lstrip())
    step, i = {}, 0
    while i < len(lines):
        line = lines[i]
        found = re.fullmatch(r"([\w-]+):\s*(.*)", line[indent:].rstrip()) if line[:indent].strip() == "" else None
        i += 1
        if not found:
            continue
        key, rest = found[1], found[2]
        block = []
        while i < len(lines) and (not lines[i].strip() or len(lines[i]) - len(lines[i].lstrip()) > indent):
            block.append(lines[i])
            i += 1
        if rest in ("|", "|-", ">"):
            deeper = min(len(b) - len(b.lstrip()) for b in block if b.strip())
            step[key] = "\n".join(b[deeper:] for b in block) + "\n"
        elif rest == "" and key in ("env", "with"):
            step[key] = {k: v for k, v in (re.fullmatch(r"\s*([\w-]+):\s*(.*)", b).groups()
                                           for b in block if b.strip() and not b.strip().startswith("#"))}
            step[key] = {k: unquote(v) for k, v in step[key].items()}
        else:
            step[key] = unquote(re.sub(r"\s+#.*$", "", rest))
    return step


def unquote(value):
    value = value.strip()
    return value[1:-1] if len(value) > 1 and value[0] == value[-1] and value[0] in "'\"" else value


def mapping(text, key, indent):
    """The `key:` mapping at this indentation, before any `steps:`."""
    found, inside = {}, False
    for line in re.split(r"\n\s*steps:" if indent else r"\njobs:", text, maxsplit=1)[0].splitlines():
        depth = len(line) - len(line.lstrip())
        if line.strip() == f"{key}:" and depth == indent:
            inside = True
        elif inside and line.strip() and depth > indent:
            pair = re.fullmatch(r"\s*([\w-]+):\s*(.*)", line)
            if pair:
                found[pair[1]] = unquote(pair[2])
        elif inside and line.strip():
            inside = False
    return found


def parse_output(text):
    """GITHUB_OUTPUT: name=value lines and name<<DELIM blocks."""
    out, lines, i = {}, text.splitlines(), 0
    while i < len(lines):
        line = lines[i]
        if "<<" in line and ("=" not in line or line.index("<<") < line.index("=")):
            key, delim = line.split("<<", 1)
            end = lines.index(delim, i + 1)
            out[key] = "\n".join(lines[i + 1:end])
            i = end + 1
            continue
        if "=" in line:
            key, value = line.split("=", 1)
            out[key] = value
        i += 1
    return out
