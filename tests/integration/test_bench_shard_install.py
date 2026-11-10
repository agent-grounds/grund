"""§AR-ci.5.6 — a benchmark shard's install is bounded and tried again, so a mirror that answers
503 or stalls costs a retry rather than a red shard or a 90-minute queue timeout (grund#1191).

Each shard job's `timeout-minutes` is read as text (§AR-ci.13.5). The install is judged by what
it does: every step of the job that runs one of the four network commands, the composite
action's steps read in place of its `uses:`, runs under `bash --noprofile --norc -eo pipefail`,
as `shell: bash` runs it, with `apt-get`, `cargo`, `timeout` and `sleep` replaced by stand-ins
that answer, fail or stall on demand and apt's directories moved under a scratch root. The
fixture checks run the same checks against altered copies, so a check that can no longer fail
is caught."""

import os
import re
import shutil
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

from ci_workflow import CI, field, header, with_local_actions
from test_release_workflow import WORKFLOWS, jobs, steps

NIGHTLY = WORKFLOWS / "bench-nightly.yml"
ACTION = WORKFLOWS.parent / "actions" / "bench-shard"
SHARD = "bench"
# §AR-ci.requirements.2: the most a shard job runs.
JOB_MINUTES = 20
# §AR-ci.5.6: one command's tries fail the step at least this many seconds inside the job's bound.
ROOM = 300
# §AR-ci.5.6: the slowest passing fetch, in seconds, that a per-attempt bound must outlast.
SLOWEST = {"apt-get update": 186, "apt-get install": 186, "cargo install": 50, "cargo fetch": 50}
FAILS_WITH = {"apt-get": 100, "cargo": 101}
# A pause shorter than this is no pause: a 503 does not clear in a second.
PAUSE = 10
MOST_TRIES = 5
NETWORK = re.compile(r"\b(?:apt-get\b[^;&|\n]*\b(?:update|install)\b|cargo\s+(?:install|fetch)\b)")
APT_DIRS = ("/etc/apt/", "/var/lib/apt/", "/var/cache/apt/")
WRITES = re.compile(r"(?:>>?|\btee\s+(?:-a\s+)?)\s*\"?(/[^\s\"]*)")
SENTINEL = re.compile(r"\[stub\] (?:apt-get|cargo) \w+")
BASH = shutil.which("bash")
RED = ("red until #1191: bench-shard's install runs each network command under a per-attempt "
       "bound and retries it, and both shard jobs carry timeout-minutes; implement removes this")

# The stand-ins log one tab-separated event per line to $STUB_DIR/log, and print what they are.
NETWORK_STANDIN = r"""#!/usr/bin/env bash
# apt-get or cargo: answers, fails like a 503 from the mirror, or stalls, as the plan says.
prog=${0##*/} key=
if [ "$prog" = apt-get ]; then
  for arg in "$@"; do case $arg in update|install) key="apt-get $arg"; break;; esac; done
else
  for arg in "$@"; do case $arg in -*|+*) ;; install|fetch) key="cargo $arg"; break;; *) break;; esac; done
fi
[ -n "$key" ] || exit 0
file="$STUB_DIR/${key/ /-}" n=0 i=0 outcome=
[ -f "$file.count" ] && read -r n < "$file.count"
n=$((n + 1)); echo "$n" > "$file.count"
[ -f "$file.plan" ] && while read -r line; do
  i=$((i + 1)); [ "$i" -eq "$n" ] && { outcome=$line; break; }
done < "$file.plan"
printf 'call\t%s\t%s\t%s\t%s\n' "$key" "${STUB_BOUND_ID:--}" "${STUB_BOUND:--}" "${STUB_KILL:--}" \
  >> "$STUB_DIR/log"
echo "[stub] $key"
case $outcome in
  fail) echo "E: [stub] Failed to fetch: 503  No healthy backends" >&2
        [ "$prog" = apt-get ] && exit 100; exit 101;;
  hang) [ -n "$STUB_BOUND_ID" ] && exit 199
        echo "[stub] $key would hang here: nothing bounds it" >&2; exit 125;;
esac
"""
TIMEOUT_STANDIN = r"""#!/usr/bin/env bash
# timeout: runs the command with its bound in the environment; a stall it ended exits 124.
kill=-
while [ $# -gt 0 ]; do
  case $1 in
    --) shift; break;;
    -k|--kill-after) kill=$2; shift 2;;
    --kill-after=*) kill=${1#*=}; shift;;
    -k*) kill=${1#-k}; shift;;
    -s|--signal) shift 2;;
    --signal=*|-s*|--foreground|--preserve-status|-v|--verbose) shift;;
    *) break;;
  esac
done
bound=$1; shift; id=0
[ -f "$STUB_DIR/timeout.count" ] && read -r id < "$STUB_DIR/timeout.count"
id=$((id + 1)); echo "$id" > "$STUB_DIR/timeout.count"
STUB_BOUND_ID=$id STUB_BOUND=$bound STUB_KILL=$kill "$@"
status=$?
[ "$status" -eq 199 ] && exit 124
exit "$status"
"""
STANDINS = {
    "apt-get": NETWORK_STANDIN, "cargo": NETWORK_STANDIN, "timeout": TIMEOUT_STANDIN,
    "sleep": "#!/usr/bin/env bash\nprintf 'sleep\\t%s\\n' \"$*\" >> \"$STUB_DIR/log\"\n",
    "git": "#!/usr/bin/env bash\nexit 0\n",
    "dpkg-query": "#!/usr/bin/env bash\nprintf %s 2.36-9+deb12u14\n",
}


class Unjudgeable(Exception):
    """The tree has a shape this checker was not written for: a person judges it."""


def seconds(text):
    found = re.fullmatch(r"(\d+(?:\.\d+)?)([smhd]?)", text.strip())
    if not found:
        return None
    return float(found.group(1)) * {"": 1, "s": 1, "m": 60, "h": 3600, "d": 86400}[found.group(2)]


def run_script(step):
    """A step's `run:` as bash reads it, its block scalar dedented."""
    lines = step.splitlines()
    for index, line in enumerate(lines):
        found = re.match(r"(\s*)(- )?run:\s*(.*)$", line)
        if not found:
            continue
        value = found.group(3).strip()
        if value and value[0] not in "|>":
            return value + "\n"
        indent, block = len(found.group(1)) + len(found.group(2) or ""), []
        for rest in lines[index + 1:]:
            if rest.strip() and len(rest) - len(rest.lstrip()) <= indent:
                break
            block.append(rest)
        return textwrap.dedent("\n".join(block)).strip("\n") + "\n"
    return None


def network_scripts(job_steps):
    """The scripts of the steps that reach the network, in order."""
    found = [script for script in map(run_script, job_steps) if script and NETWORK.search(script)]
    if not any(re.search(r"\bapt-get\b[^\n]*\binstall\b", script) for script in found):
        raise Unjudgeable("the shard installs nothing with apt-get any more, so the mirror may be "
                          "off its path; judge the new shape by hand")
    for script in found:
        outside = [path for path in WRITES.findall(script)
                   if not path.startswith(APT_DIRS + ("/dev/",))]
        if "${{" in script or outside:
            raise Unjudgeable("the install uses an expression or writes outside apt's directories "
                              f"({outside[:1]}), so it cannot run under the stand-ins")
    return found


class Run:
    """One run of the install under the stand-ins: its status, what it printed, what it called."""

    def __init__(self, status, output, log):
        self.status, self.lines = status, output.splitlines()
        self.events = [line.split("\t") for line in log.splitlines() if line]

    def calls(self, key):
        return [event for event in self.events if event[:2] == ["call", key]]

    def bound(self, call):
        return None if call[2] == "-" else (seconds(call[3]), seconds(call[4]) if call[4] != "-" else None)

    def paused(self, key=None, after=1, before=None):
        """Seconds slept after the `after`-th call of `key`, up to its `before`-th or the end;
        with no `key`, every second slept."""
        seen, total = (0, 0.0) if key else (after, 0.0)
        for event in self.events:
            if event[:2] == ["call", key]:
                seen += 1
                if seen == before:
                    break
            elif event[0] == "sleep" and seen >= after:
                total += sum(seconds(arg) or 0 for arg in event[1].split())
        return total

    def around(self, key, attempt):
        """The install's own lines just before the attempt-th call of `key`, back to the call
        before it, and just after it, on to the call after it."""
        marks = [index for index, line in enumerate(self.lines) if SENTINEL.fullmatch(line)]
        mine = [index for index in marks if self.lines[index] == f"[stub] {key}"]
        if len(mine) < attempt:
            return [], []
        at = mine[attempt - 1]
        before = max([index for index in marks if index < at], default=-1)
        after = min([index for index in marks if index > at], default=len(self.lines))
        own = lambda part: [line for line in part if "[stub]" not in line]
        return own(self.lines[before + 1:at]), own(self.lines[at + 1:after])


def install_run(scripts, plan=None):
    """Run each script as its step would, stopping at the first that fails, with `plan`
    giving each network command's outcome per call: `ok`, `fail` or `hang`."""
    with tempfile.TemporaryDirectory(prefix="bench-shard-install-") as scratch:
        root = Path(scratch)
        stub, bin_dir, work = root / "stub", root / "bin", root / "work"
        for directory in (stub, bin_dir, work, root / "home", root / "etc/apt/sources.list.d",
                          root / "etc/apt/apt.conf.d", root / "var/lib/apt/lists/partial",
                          root / "var/cache/apt/archives/partial"):
            directory.mkdir(parents=True)
        for name, text in STANDINS.items():
            (bin_dir / name).write_text(text, encoding="utf-8")
            (bin_dir / name).chmod(0o755)
        for key, outcomes in (plan or {}).items():
            (stub / f"{key.replace(' ', '-')}.plan").write_text("\n".join(outcomes) + "\n")
        (stub / "log").touch()
        env = {name: value for name, value in os.environ.items()
               if not name.startswith(("STUB_", "GITHUB_"))}
        env.update(PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}", STUB_DIR=str(stub),
                   HOME=str(root / "home"), GITHUB_WORKSPACE=str(work),
                   GITHUB_ENV=str(root / "github-env"), GITHUB_ACTION_PATH=str(ACTION))
        status, output = 0, ""
        for index, script in enumerate(scripts):
            for directory in APT_DIRS:
                script = script.replace(directory, f"{root}{directory}")
            path = root / f"step-{index}.sh"
            path.write_text(script, encoding="utf-8")
            try:
                done = subprocess.run([BASH, "--noprofile", "--norc", "-eo", "pipefail", str(path)],
                                      cwd=work, env=env, stdout=subprocess.PIPE,
                                      stderr=subprocess.STDOUT, text=True, timeout=60)
                status, output = done.returncode, output + done.stdout
            except subprocess.TimeoutExpired:
                status, output = "timed out", output + "the step ran past 60 s under the stand-ins\n"
            if status != 0:
                break
        return Run(status, output, (stub / "log").read_text(encoding="utf-8"))


def says(line, *words):
    """Whether `line` carries each word whole, a number never as part of a version."""
    return all(re.search(rf"(?<![\w.:=+-]){re.escape(str(word))}(?![\w.:+-])", line)
               for word in words)


def command_problems(scripts, key, healthy, budget):
    """§AR-ci.5.6 for one network command: bounded, tried again after a pause, said aloud."""
    program, code = key.split()[0], FAILS_WITH[key.split()[0]]
    calls = healthy.calls(key)
    if len(calls) != 1:
        return [f"runs {len(calls)} times when the mirror answers, not once"]
    problems = []
    if not says((healthy.around(key, 1)[0] or [""])[-1], *key.split(), 1):
        problems.append("no line just before attempt 1 names the command and the attempt")
    bound = healthy.bound(calls[0])
    if bound is None or bound[0] is None:
        problems.append("runs with no per-attempt bound, so a fetch that trickles holds the step "
                        "until the merge queue's 90-minute timeout")
    else:
        if bound[1] is None:
            problems.append("its bound has no --kill-after, so a command that ignores the signal "
                            "outlives it")
        if bound[0] <= SLOWEST[key]:
            problems.append(f"its bound of {bound[0]:g} s ends a fetch no slower than a passing "
                            f"shard's {SLOWEST[key]} s")
    once = install_run(scripts, {key: ["fail"]})
    if once.status != 0:
        return problems + [f"one failure (exit {code}) fails the step: it is not tried again"]
    tries = once.calls(key)
    if len(tries) != 2:
        problems.append(f"is tried {len(tries)} times after one failure, not twice")
    elif tries[0][2] != "-" and tries[0][2] == tries[1][2]:
        problems.append("one bound covers every attempt, so a stalled attempt ends them all")
    if once.paused(key, 1, 2) < PAUSE:
        problems.append(f"is tried again with no pause of {PAUSE} s or more after a failure")
    if not any(says(line, program, key.split()[1], code) for line in once.around(key, 1)[1]):
        problems.append(f"no line after a failed attempt names the command and its exit {code}")
    first, second = once.around(key, 1)[0][-1:], once.around(key, 2)[0][-1:]
    if not second or not says(second[0], *key.split(), 2) or first == second:
        problems.append("no line just before attempt 2 names the command and the attempt")
    if bound and bound[0] is not None:
        stalled = install_run(scripts, {key: ["hang"]})
        if stalled.status != 0 or len(stalled.calls(key)) != 2:
            problems.append("an attempt its bound ended (exit 124) is not tried again")
    down = install_run(scripts, {key: ["fail"] * (MOST_TRIES + 1)})
    tries = down.calls(key)
    if down.status == 0:
        return problems + ["a mirror that stays down does not fail the step"]
    if down.status != code:
        problems.append(f"the step gives up with exit {down.status}, not the command's {code}")
    if len(tries) > MOST_TRIES:
        problems.append(f"is tried {len(tries)} times, more than {MOST_TRIES}")
    bounds = [down.bound(call) for call in tries]
    if all(bound and bound[0] is not None for bound in bounds):
        worst = sum(bound[0] + (bound[1] or 0) for bound in bounds) + down.paused(key)
        if worst > budget:
            problems.append(f"its {len(tries)} tries at their bounds and the pauses between them "
                            f"take {worst:g} s, past the {budget} s that leave the step {ROOM} s "
                            "inside the job's bound")
    if not any(says(line, program, key.split()[1], code) for line in down.around(key, len(tries))[1]):
        problems.append(f"no line after its last attempt names the command and its exit {code}")
    return problems


def shard_problems(text, name):
    """§AR-ci.5.6 on one workflow's shard job: the job's bound, then each network command."""
    if SHARD not in jobs(text):
        raise Unjudgeable(f"{name}: no `{SHARD}` job; judge the new layout by hand")
    minutes = field(header(jobs(text)[SHARD]), "timeout-minutes")
    problems = []
    if minutes is None:
        problems.append("the job has no `timeout-minutes`, so a stall no command's bound covers "
                        "holds it until the merge queue's 90-minute timeout")
    elif not minutes.isdigit() or not 0 < int(minutes) <= JOB_MINUTES:
        problems.append(f"the job's `timeout-minutes: {minutes}` is no bound of at most "
                        f"{JOB_MINUTES}")
        minutes = None
    scripts = network_scripts(steps(jobs(with_local_actions(text))[SHARD]))
    healthy = install_run(scripts)
    if healthy.status != 0:
        return [f"{name}: the install fails though every command answers: "
                + " / ".join(healthy.lines[-3:])]
    if healthy.paused():
        problems.append("the install pauses though nothing failed")
    budget = int(minutes or JOB_MINUTES) * 60 - ROOM
    for key in SLOWEST:
        problems += [f"{key}: {problem}" for problem in command_problems(scripts, key, healthy, budget)]
    return [f"{name}: {problem}" for problem in problems]


POSIX = unittest.skipIf(os.name == "nt" or BASH is None,
                        "the stand-ins are bash scripts; the shard runs on Linux")


@POSIX
class ShardInstallTests(unittest.TestCase):
    """§AR-ci.5.6 on the repository's own shard jobs."""

    def check(self, path):
        problems = shard_problems(path.read_text(encoding="utf-8"), path.name)
        if problems:
            self.fail("\n" + "\n".join(problems))

    @unittest.skip(RED)
    def test_the_pull_request_shard_install_is_bounded_and_retried(self):
        self.check(CI)

    @unittest.skip(RED)
    def test_the_nightly_shard_install_is_bounded_and_retried(self):
        self.check(NIGHTLY)


TODAY = r"""jobs:
  bench:
    name: cargo bench (shard ${{ matrix.shard }})
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: Install Valgrind and iai-callgrind-runner
        shell: bash
        run: |
          git config --global --add safe.directory "$GITHUB_WORKSPACE"
          rm -f /etc/apt/sources.list.d/* && echo "deb [check-valid-until=no] \
            https://snapshot.debian.org/archive/debian/20261001T000000Z bookworm main" > /etc/apt/sources.list
          apt-get -o Acquire::Retries=5 update -qq
          apt-get -o Acquire::Retries=5 install -y -qq --no-install-recommends valgrind=1:3.19.0-1 \
            libc6-dbg="$(dpkg-query -W -f '${Version}' libc6)"
          cargo install iai-callgrind-runner --version 0.16.1 --locked --quiet
          cargo fetch --locked --quiet
          mkdir -p bench-records && echo "RECORDS=$PWD/bench-records" >> "$GITHUB_ENV"
"""
WRAPPER = r"""          fetch() {
            local attempt status=0
            for attempt in 1 2 3; do
              echo "bench-shard: $* (attempt $attempt of 3)"
              if timeout --kill-after=10 240 "$@"; then return 0; else status=$?; fi
              echo "bench-shard: $* failed with exit $status (attempt $attempt of 3)"
              if [ "$attempt" -lt 3 ]; then sleep 30; fi
            done
            return "$status"
          }
"""


def fixed(wrapper=WRAPPER):
    text = TODAY.replace("    runs-on: ubuntu-latest\n",
                         "    runs-on: ubuntu-latest\n    timeout-minutes: 20\n")
    text = text.replace("          apt-get -o Acquire::Retries=5 update", wrapper
                        + "          fetch apt-get -o Acquire::Retries=5 update")
    for command in ("apt-get -o Acquire::Retries=5 install", "cargo install", "cargo fetch"):
        text = text.replace("          " + command, "          fetch " + command)
    return text


@POSIX
class CheckTests(unittest.TestCase):
    """The checks fail the install as it was and pass a bounded, retried one."""

    def problems(self, text):
        return shard_problems(text, "fixture")

    def assertEach(self, problems, phrase, keys=tuple(SLOWEST)):
        for key in keys:
            self.assertTrue([p for p in problems if f": {key}: " in p and phrase in p],
                            f"{key}: {phrase!r} missing from {problems}")

    def test_the_install_as_it_was_fails_every_property(self):
        problems = self.problems(TODAY)
        self.assertIn("no `timeout-minutes`", problems[0])
        self.assertEach(problems, "no per-attempt bound")
        self.assertEach(problems, "fails the step: it is not tried again")
        self.assertEach(problems, "no line just before attempt 1")

    def test_a_bounded_retried_install_passes(self):
        self.assertEqual([], self.problems(fixed()))

    def test_a_job_bound_past_twenty_minutes_is_no_bound(self):
        problems = self.problems(fixed().replace("timeout-minutes: 20", "timeout-minutes: 90"))
        self.assertEqual(1, len(problems), problems)
        self.assertIn("`timeout-minutes: 90` is no bound", problems[0])

    def test_cargo_left_unwrapped_is_caught(self):
        problems = self.problems(fixed().replace("fetch cargo", "cargo"))
        self.assertEach(problems, "no per-attempt bound", ("cargo install", "cargo fetch"))
        self.assertEqual([], [p for p in problems if ": apt-get " in p])

    def test_a_retry_that_bash_e_ends_is_caught(self):
        trap = WRAPPER.replace('if timeout --kill-after=10 240 "$@"; then return 0; else status=$?; fi',
                               'timeout --kill-after=10 240 "$@"; status=$?\n'
                               '              [ "$status" -ne 0 ] || return 0')
        problems = self.problems(fixed(trap))
        self.assertEach(problems, "fails the step: it is not tried again")
        self.assertEqual(4, len(problems), problems)

    def test_one_bound_around_every_attempt_is_caught(self):
        shared = fixed().replace(
            "          fetch apt-get -o Acquire::Retries=5 update -qq\n",
            "          timeout --kill-after=10 600 bash -c 'for attempt in 1 2 3; do\n"
            "            echo \"apt-get update (attempt $attempt)\"\n"
            "            apt-get -o Acquire::Retries=5 update -qq && exit 0\n"
            "            echo \"apt-get update failed with exit $? (attempt $attempt)\"; sleep 30\n"
            "          done; exit 100'\n")
        problems = self.problems(shared)
        self.assertEach(problems, "one bound covers every attempt", ("apt-get update",))
        self.assertEqual([], [p for p in problems if ": apt-get update: " not in p])

    def test_a_retry_that_says_nothing_and_never_waits_is_caught(self):
        bare = ('          fetch() { for attempt in 1 2 3; do if timeout 240 "$@"; then return 0; fi; '
                'done; return 1; }\n')
        problems = self.problems(fixed(bare))
        for phrase in ("no --kill-after", "no line just before attempt 1", "no pause",
                       "no line after a failed attempt", "gives up with exit 1"):
            self.assertEach(problems, phrase)

    def test_a_bound_a_passing_fetch_outlasts_is_caught(self):
        problems = self.problems(fixed(WRAPPER.replace(" 240 ", " 120 ")))
        self.assertEach(problems, "ends a fetch no slower than a passing shard's 186 s",
                        ("apt-get update", "apt-get install"))
        self.assertEqual([], [p for p in problems if ": cargo " in p])

    def test_tries_that_leave_no_room_in_the_job_are_caught(self):
        long = (WRAPPER.replace("1 2 3", "1 2 3 4 5").replace("-lt 3", "-lt 5")
                .replace(" 240 ", " 300 ").replace("of 3", "of 5"))
        self.assertEach(self.problems(fixed(long)), f"past the {JOB_MINUTES * 60 - ROOM} s")

    def test_a_job_that_no_longer_uses_apt_is_for_a_person(self):
        gone = "\n".join(line for line in TODAY.splitlines()
                         if "apt-get" not in line and "libc6-dbg" not in line) + "\n"
        with self.assertRaises(Unjudgeable):
            self.problems(gone)


if __name__ == "__main__":
    unittest.main()
