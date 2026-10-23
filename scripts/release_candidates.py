"""Everything a release helper does from its candidates on (§FS-distribution.4.4):
the rounds that build and test them while the merge queue moves `main`, the push
to `main`, the publisher's dispatch and the `-dev` advance. `release-minor.yml`
and `auto-bump.yml` both call it, after deciding once what is decided once - the
version, the substantive-change filter and CI on the dispatch tip.

    release_candidates.py --helper <workflow> --next <version> [--pinned <sha>]

A round (§FS-distribution.4.4.1.1) builds both candidates on a tip of `main` with
`scripts/release_round.sh`, pushes them to two branches of its own, dispatches
the three runs and watches them with `scripts/candidate_runs.py`. When `main`
moves under it, the round is given up - its runs cancelled, its branches
deleted - and, once `main` has held still, the next round builds on the new
tip. Only a round whose three runs passed reaches `main`, and only its commits
do. A run pinned to a tip (`--pinned`, the dispatch's `expected_main_sha`)
follows no move (§FS-distribution.4.4.2).

Its waits, in seconds, come from the environment so a test does not wait them
out (§AR-ci.11): GRUND_RELEASE_SETTLE, GRUND_RELEASE_BUDGET,
GRUND_RELEASE_ROUND_MIN and GRUND_RELEASE_RESERVE.

Exits 0 once the release commit is on `main`, `release.yml` is dispatched on it
and the advance has followed; 1, naming what stopped it, otherwise."""

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RUNS = HERE / "candidate_runs.py"
# The watch's exits (§AR-ci.11), as `candidate_runs.py` defines them.
PASSED, FAILED, MOVED, DEADLINE = 0, 1, 3, 4
# §FS-distribution.4.4.1.2: at most six rounds.
ROUNDS = 6


def seconds(name: str, default: float) -> float:
    return float(os.environ.get(name, default))


# How long `main` must hold still before the next round builds on it.
SETTLE = seconds("GRUND_RELEASE_SETTLE", 180)
# §FS-distribution.4.4.1.2: the job's 300 minutes, less ten for the steps before this one.
BUDGET = seconds("GRUND_RELEASE_BUDGET", 290 * 60)
# A round starts only with 90 minutes left.
ROUND_MIN = seconds("GRUND_RELEASE_ROUND_MIN", 90 * 60)
# What the watch's deadline leaves for the push, the publisher's dispatch and the advance.
RESERVE = seconds("GRUND_RELEASE_RESERVE", 15 * 60)


class Stop(Exception):
    """The helper ends here, publishing nothing more; the message says why."""


def say(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def git(*arguments: str) -> str:
    result = subprocess.run(["git", *arguments], capture_output=True, encoding="utf-8", errors="replace")
    if result.returncode != 0:
        raise Stop(f"`git {' '.join(arguments)}` exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout.strip()


def main_tip() -> str | None:
    result = subprocess.run(["git", "ls-remote", "origin", "refs/heads/main"], capture_output=True,
                            encoding="utf-8", errors="replace")
    fields = result.stdout.split()
    return fields[0] if result.returncode == 0 and fields else None


def advance(released: str, dev: str, branch: str, sha: str) -> int:
    """§FS-distribution.4.4.1: the tested advance, through its one implementation."""
    return subprocess.run(["bash", str(HERE / "advance_dev.sh"), released, dev, branch, sha]).returncode


def dev_version(released: str) -> str:
    major, minor, patch = released.split(".")
    return f"{major}.{minor}.{int(patch) + 1}-dev"


class Helper:
    def __init__(self, workflow: str, released: str, pinned: str | None):
        self.workflow, self.next, self.pinned = workflow, released, pinned or None
        self.dev, self.tag = dev_version(released), f"v{released}"
        self.ends = time.time() + BUDGET
        self.run_name = f"{os.environ.get('GITHUB_RUN_ID', 'local')}-{os.environ.get('GITHUB_RUN_ATTEMPT', '1')}"
        self.versions = self.version_sources("HEAD")

    def left(self) -> float:
        return self.ends - time.time()

    def branches(self, round_: int) -> tuple[str, str]:
        """One path component under `release-candidate/`, never reused (§FS-distribution.4.4.1.1)."""
        branch = f"release-candidate/v{self.next}-{self.run_name}-r{round_}"
        return branch, f"{branch}-dev"

    def version_sources(self, ref: str) -> str:
        result = subprocess.run([sys.executable, "scripts/distribution/candidate.py", "versions", "--ref", ref],
                                capture_output=True, encoding="utf-8", errors="replace")
        if result.returncode != 0:
            raise Stop(f"the version sources on {ref} cannot be read: {result.stderr.strip()}")
        return result.stdout.strip()

    def out_of_time(self) -> Stop:
        return Stop(f"out of time: {self.left():.0f}s of the helper's budget remain and a round needs "
                    f"{ROUND_MIN:.0f}s, so v{self.next} was not published; dispatch {self.workflow} again "
                    "(§FS-distribution.4.4.1.2)")

    # -- a round ---------------------------------------------------------------------------

    def run(self) -> int:
        base = git("rev-parse", "HEAD")
        for round_ in range(1, ROUNDS + 1):
            if round_ > 1:
                base = self.settle()
            if self.left() < ROUND_MIN:
                raise self.out_of_time()
            moved = self.round(round_, base)
            if moved is None:
                return 0
            # §FS-distribution.4.4.2: a pinned run follows no move.
            if self.pinned:
                raise Stop(f"main moved from {base} to {moved}, and a run given expected_main_sha follows no "
                           f"move: its runs were cancelled and its branches deleted. To release the new tip, "
                           f"dispatch {self.workflow} again with expected_main_sha={moved}")
            say(f"round {round_} given up: main moved from {base} to {moved}")
        raise Stop(f"out of rounds: {ROUNDS} rounds were given up to a moving main, so v{self.next} was not "
                   f"published; dispatch {self.workflow} again (§FS-distribution.4.4.1.2)")

    def settle(self) -> str:
        """Wait until `main` has held still for SETTLE seconds, then fetch it (§FS-distribution.4.4.1.1)."""
        tip, since = main_tip(), time.time()
        while time.time() - since < SETTLE:
            if self.left() < ROUND_MIN:
                raise self.out_of_time()
            time.sleep(max(0.2, min(15.0, SETTLE / 6)))
            now = main_tip()
            if now != tip:
                tip, since = now, time.time()
        git("fetch", "-q", "origin", "main", "--tags")
        return git("rev-parse", "refs/remotes/origin/main")

    def check(self, base: str) -> None:
        """What a merge can change, checked before the round pushes anything (§FS-distribution.4.4.1.1)."""
        if subprocess.run(["git", "rev-parse", "-q", "--verify", f"refs/tags/{self.tag}"],
                          capture_output=True).returncode == 0:
            raise Stop(f"the tag {self.tag} now exists, so v{self.next} is no longer this helper's to release; "
                       "nothing was pushed")
        found = self.version_sources(base)
        if found != self.versions:
            raise Stop(f"the version sources on main {base} read {found}, not {self.versions} as when the "
                       f"helper started, so v{self.next} is no longer the release they follow; nothing was pushed")

    def round(self, round_: int, base: str) -> str | None:
        """Build, push and run one round on `base`; the tip `main` moved to, or None once published."""
        say(f"round {round_}: building v{self.next} and {self.dev} on {base}")
        git("checkout", "-q", "-f", "--detach", base)
        self.check(base)
        # §FS-distribution.4.4.1.4: a refused rebuild stops before any branch or run.
        if subprocess.run(["bash", "scripts/release_round.sh", self.next, self.dev]).returncode != 0:
            raise Stop(f"the preparation of v{self.next} on {base} was refused; no branch of round {round_} "
                       "was pushed and no run started")
        release, dev = git("rev-parse", "HEAD^"), git("rev-parse", "HEAD")
        branch, dev_branch = self.branches(round_)
        git("push", "-q", "--atomic", "origin", f"{release}:refs/heads/{branch}", f"{dev}:refs/heads/{dev_branch}")
        kept = f"{branch} and {dev_branch} are left for a person to read, and main is untouched"

        ids = []
        for workflow, on, sha, fields in (
                ("release.yml", branch, release,
                 [f"version={self.next}", "publish_crates=false", "create_github_release=false"]),
                ("ci.yml", branch, release, []), ("ci.yml", dev_branch, dev, [])):
            started = subprocess.run([sys.executable, str(RUNS), "dispatch", workflow, on, sha, *fields],
                                     stdout=subprocess.PIPE, encoding="utf-8")
            if started.returncode != 0:
                for run_id in ids:
                    subprocess.run(["gh", "run", "cancel", run_id], capture_output=True)
                raise Stop(f"round {round_}'s runs could not be started; {kept}")
            ids.append(started.stdout.strip())

        watched = subprocess.run([sys.executable, str(RUNS), "watch", "--main", base,
                                  "--deadline", str(self.ends - RESERVE), *ids]).returncode
        if watched == MOVED:
            self.delete(branch, dev_branch)
            return main_tip() or "an unreadable tip"
        if watched == DEADLINE:
            self.delete(branch, dev_branch)
            raise Stop(f"out of time: round {round_}'s runs had not finished by the watch's deadline and were "
                       f"cancelled, so v{self.next} was not published; dispatch {self.workflow} again "
                       "(§FS-distribution.4.4.1.2)")
        if watched != PASSED:
            # §FS-distribution.4.4.1.4: only a failure leaves branches.
            raise Stop(f"a candidate run of round {round_} did not pass; {kept}")

        moved = self.push(release, base, kept)
        if moved:
            self.delete(branch, dev_branch)
            return moved
        self.publish(branch, release, dev_branch, dev)
        return None

    # -- publication -----------------------------------------------------------------------

    def push(self, release: str, base: str, kept: str) -> str | None:
        """§FS-distribution.4.4.1.3: a plain push, read back when git refuses it."""
        pushed = subprocess.run(["git", "push", "origin", f"{release}:refs/heads/main"], capture_output=True,
                                encoding="utf-8", errors="replace")
        sys.stderr.write(pushed.stderr)
        if pushed.returncode == 0:
            return None
        tip = main_tip()
        if tip == release:
            say(f"git reported the push failed, but main is the v{self.next} release commit {release}")
            return None
        if tip in (base, None):
            raise Stop(f"could not push the v{self.next} release commit {release} to main, which is "
                       f"{'still ' + base if tip else 'unreadable'}; git's error:\n{pushed.stderr.strip()}\n{kept}")
        return tip

    def publish(self, branch: str, release: str, dev_branch: str, dev: str) -> None:
        """§FS-distribution.4.4.1.3: dispatch the publisher on the commit the runs passed, then advance."""
        dispatched = subprocess.run([sys.executable, str(RUNS), "dispatch", "release.yml", branch, release,
                                     f"version={self.next}", "publish_crates=true", "create_github_release=true",
                                     f"expected_release_sha={release}"],
                                    stdout=subprocess.PIPE, encoding="utf-8")
        if dispatched.returncode == 0:
            say(f"release.yml run {dispatched.stdout.strip()} publishes v{self.next} from {branch}")
        advanced = advance(self.next, self.dev, dev_branch, dev)
        if dispatched.returncode != 0:
            raise Stop(f"the v{self.next} release commit {release} is on main, but release.yml could not be "
                       f"dispatched to publish it. Do not start the release over: dispatch release.yml on {branch} "
                       f"with version={self.next} and expected_release_sha={release}")
        if advanced != 0:
            raise Stop(f"v{self.next} is on main and release.yml was dispatched, but main was not advanced "
                       f"to {self.dev}")

    def delete(self, *branches: str) -> None:
        result = subprocess.run(["git", "push", "-q", "origin", "--delete", *branches], capture_output=True,
                                encoding="utf-8", errors="replace")
        if result.returncode != 0:
            say(f"warning: could not delete {', '.join(branches)}: {result.stderr.strip()}")


def main(arguments: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("--helper", required=True, help="the workflow file, named in messages")
    parser.add_argument("--next", required=True, help="the version to release")
    parser.add_argument("--pinned", default="", help="expected_main_sha: follow no move of main")
    options = parser.parse_args(arguments)
    try:
        return Helper(options.helper, options.next, options.pinned).run()
    except Stop as stop:
        say(f"error: {stop}")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
