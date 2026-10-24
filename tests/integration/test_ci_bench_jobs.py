"""§AR-ci.5 — the benchmark jobs' pinned environment and their one-harness comparison
(§AR-ci.5.1, §AR-benchmarks.5): a build job, a shard matrix and a record job.

Read as text with the layout test's reader (§AR-ci.13.5), and run against a fixture job
too, so a check that can no longer fail is caught."""

import re
import unittest

from test_ci_language_jobs import (BENCH, BENCH_BUILD, BENCH_ENV, BENCH_IMAGE, BENCH_RECORD,
                                   BENCH_SHARD, CI, commands, header, job_env, with_local_actions)
from test_release_workflow import jobs, squash, steps

SNAPSHOT = re.compile(r"https://snapshot\.debian\.org/archive/debian/\d{8}T\d{6}Z\b")


def bench_pin_problems(text):
    """§AR-ci.5: the build job and the shards run in one environment named by digest, and
    only they; the shards, which run Valgrind, install it pinned."""
    found = jobs(text)
    missing = [f"no `{name}` job" for name in (BENCH_BUILD, BENCH_SHARD) if name not in found]
    if missing:
        return missing
    problems = []
    for name in (BENCH_BUILD, BENCH_SHARD):
        problems += [f"{name}: {problem}" for problem in environment_problems(found[name])]
    body = found[BENCH_SHARD]
    if not re.search(r"\bvalgrind=\S+", body):
        problems.append("the bench job installs Valgrind without an exact version")
    if not re.search(r"\blibc6-dbg=\S+", body):
        problems.append("the bench job installs libc6-dbg without an exact version")
    if not SNAPSHOT.search(body) or re.search(r"deb\.debian\.org", body):
        problems.append("the bench job's packages do not come from one snapshot.debian.org timestamp")
    if "iai-callgrind-runner --version 0.16.1" not in body:
        problems.append("the bench job does not pin iai-callgrind-runner 0.16.1")
    return problems


def environment_problems(body):
    """§AR-ci.5: one job's digest-pinned image, its env, and cache keys that carry the digest."""
    head, job_steps = header(body), steps(body)
    problems = []
    image = BENCH_IMAGE.search(head)
    if not image:
        problems.append("the job names no `rust:<release>-bookworm@sha256:<digest>` container")
    if any("rust-toolchain" in step for step in job_steps):
        problems.append("the job installs a toolchain instead of using the image's compiler")
    env = job_env(head)
    for key, value in BENCH_ENV.items():
        if env.get(key) != value:
            problems.append(f"the job sets {key}={env.get(key)!r}, not {value!r}")
    if image and env.get("BENCH_IMAGE") != "sha256:" + image.group(1):
        problems.append(f"BENCH_IMAGE {env.get('BENCH_IMAGE')!r} is not the container's digest")
    keys = [key for step in job_steps if "actions/cache" in step
            for key in re.findall(r"(?m)^\s*key:\s*(.+)$", step)]
    if image and (not keys or any(image.group(1) not in key for key in keys)):
        problems.append(f"the job's cache keys {keys} do not carry the image digest")
    return problems


def bench_comparison_problems(text):
    """§AR-ci.5.1, §AR-benchmarks.5: the build job builds the base binary, one harness in
    each shard measures both binaries, the record job merges, and main checks jitter."""
    found = jobs(text)
    missing = [f"no `{name}` job" for name in (BENCH_BUILD, BENCH_SHARD, BENCH_RECORD)
               if name not in found]
    if missing:
        return missing
    job_steps = steps(found[BENCH_SHARD])
    problems = []
    base = [step for step in steps(found[BENCH_BUILD])
            if "github.base_ref" in step or "pull_request.base.sha" in step]
    if not base:
        problems.append("no step takes the base commit")
    for step in base:
        if "cargo bench" in step:
            problems.append("the base commit's step runs its own harness: " + squash(step)[:120])
        if "cargo build" not in step:
            problems.append("the base commit's step does not build the base binary")
    saving = [step for step in job_steps if "--save-baseline=main" in step]
    if len(saving) != 1:
        problems.append(f"{len(saving)} steps save the baseline, not one")
    for step in saving:
        if "GRUND_BENCH_BIN" not in step:
            problems.append("the baseline run does not measure the base binary via GRUND_BENCH_BIN")
        if "git checkout" in step or "github.base_ref" in step:
            problems.append("the baseline run checks out the base: it is not the PR's harness")
    for step in (step for step in job_steps if BENCH in commands(step)):
        if "GRUND_BENCH_BIN" in step:
            problems.append("the comparing run measures the base binary, not the PR's own")
    if any("git checkout" in step and "base" in step for step in job_steps):
        problems.append("a shard checks out the base: only the build job takes it")
    flat = squash(found[BENCH_SHARD])
    if "bench_record.py record" not in flat:
        problems.append("the shards do not run `bench_record.py record`")
    flat = squash(found[BENCH_RECORD])
    for wanted in ("bench_record.py merge", "bench_record.py compare", "GITHUB_STEP_SUMMARY"):
        if wanted not in flat:
            problems.append(f"the record job does not run `{wanted}`")
    jitter = [step for step in job_steps if "bench_record.py jitter" in step]
    if not jitter:
        problems.append("a push to main does not check the same-binary bound")
    for step in jitter:
        if "refs/heads/main" not in step:
            problems.append("the jitter check is not limited to a push to main")
    return problems


PINNED_BENCH = """jobs:
  bench-build:
    if: github.event_name != 'merge_group'
    runs-on: ubuntu-latest
    container: rust:1.90.0-bookworm@sha256:{digest}
    env:
      BENCH_IMAGE: sha256:{digest}
      LC_ALL: C.UTF-8
      TZ: UTC
      RAYON_NUM_THREADS: 4
      CARGO_INCREMENTAL: 0
      RUSTFLAGS: ""
    steps:
      - uses: actions/checkout@v6
      - uses: actions/cache/restore@v5
        with:
          key: bench-{digest}-${{{{ hashFiles('**/Cargo.lock') }}}}
      - if: github.event_name == 'pull_request'
        run: |
          git checkout --force "origin/${{{{ github.base_ref }}}}"
          cargo build --profile bench -p grund --bin grund --locked
          git checkout --force "${{{{ github.sha }}}}"
      - run: cargo bench -p grund --features bench --locked --quiet --bench instructions --no-run
      - uses: actions/upload-artifact@v4
  bench:
    name: cargo bench (shard ${{{{ matrix.shard }}}})
    needs: [bench-build]
    if: github.event_name != 'merge_group'
    runs-on: ubuntu-latest
    container: rust:1.90.0-bookworm@sha256:{digest}
    env:
      BENCH_IMAGE: sha256:{digest}
      LC_ALL: C.UTF-8
      TZ: UTC
      RAYON_NUM_THREADS: 4
      CARGO_INCREMENTAL: 0
      RUSTFLAGS: ""
    steps:
      - uses: actions/checkout@v6
      - uses: actions/cache/restore@v5
        with:
          key: bench-registry-{digest}-${{{{ hashFiles('**/Cargo.lock') }}}}
      - run: |
          echo "deb https://snapshot.debian.org/archive/debian/20261001T000000Z bookworm main" > /etc/apt/sources.list
          apt-get install -y valgrind=1:3.19.0-1 libc6-dbg=2.36-9+deb12u14
          cargo install iai-callgrind-runner --version 0.16.1 --locked --quiet
      - uses: actions/download-artifact@v5
      - if: github.event_name == 'pull_request'
        env:
          GRUND_BENCH_BIN: ../grund-base
        run: cargo bench -p grund --features bench --locked --quiet --bench instructions -- --save-baseline=main --save-summary=json
      - if: github.event_name == 'pull_request'
        run: {bench}
      - run: python3 scripts/bench_record.py record --out head-1.json
      - if: github.event_name == 'push' && github.ref == 'refs/heads/main'
        run: python3 scripts/bench_record.py jitter first.json second.json
  bench-record:
    needs: [bench]
    if: github.event_name != 'merge_group'
    runs-on: ubuntu-latest
    steps:
      - run: |
          python3 scripts/bench_record.py merge --out head.json head-*.json
          python3 scripts/bench_record.py compare base.json head.json >> "$GITHUB_STEP_SUMMARY"
""".format(digest="0" * 64, bench=BENCH)


class BenchJobTests(unittest.TestCase):
    """§AR-ci.5 and §AR-ci.5.1 on `ci.yml`."""

    @classmethod
    def setUpClass(cls):
        cls.text = with_local_actions(CI.read_text(encoding="utf-8"))

    def test_the_bench_job_runs_in_a_pinned_environment(self):
        # §AR-ci.5
        self.assertEqual([], bench_pin_problems(self.text))

    def test_one_harness_measures_both_binaries_and_main_checks_jitter(self):
        # §AR-ci.5.1, §AR-benchmarks.5, §AR-benchmarks.7.3
        self.assertEqual([], bench_comparison_problems(self.text))


class BenchJobCheckTests(unittest.TestCase):
    """The bench checks pass the job §AR-ci.5 asks for and fail each way off it."""

    def test_the_job_asked_for_passes(self):
        self.assertEqual([], bench_pin_problems(PINNED_BENCH))
        self.assertEqual([], bench_comparison_problems(PINNED_BENCH))

    def test_a_floating_environment_is_caught(self):
        for old, new in (("@sha256:" + "0" * 64, ""), ("valgrind=1:3.19.0-1", "valgrind"),
                         ("      TZ: UTC\n", ""), ("bench-" + "0" * 64, "bench"),
                         ("libc6-dbg=2.36-9+deb12u14", "libc6-dbg"),
                         ("snapshot.debian.org/archive/debian/20261001T000000Z", "deb.debian.org/debian"),
                         ("BENCH_IMAGE: sha256:" + "0" * 64, "BENCH_IMAGE: sha256:" + "1" * 64)):
            with self.subTest(drop=old):
                self.assertTrue(bench_pin_problems(PINNED_BENCH.replace(old, new)))
        toolchain = PINNED_BENCH.replace(
            "      - uses: actions/checkout@v6\n",
            "      - uses: actions/checkout@v6\n      - uses: dtolnay/rust-toolchain@stable\n")
        self.assertTrue(bench_pin_problems(toolchain))

    def test_a_base_harness_is_caught(self):
        base_harness = PINNED_BENCH.replace(
            "cargo build --profile bench -p grund --bin grund --locked",
            "cargo bench -p grund --features bench --bench instructions -- --save-baseline=main")
        self.assertTrue(bench_comparison_problems(base_harness))
        self.assertTrue(bench_comparison_problems(PINNED_BENCH.replace("GRUND_BENCH_BIN", "GRUND_BIN")))

    def test_a_missing_jitter_check_is_caught(self):
        self.assertTrue(bench_comparison_problems(
            PINNED_BENCH.replace("bench_record.py jitter", "bench_record.py record")))


if __name__ == "__main__":
    unittest.main()
