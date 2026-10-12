# AR-ci: CI mirrors the local pre-commit gate

The CI workflow is the remote form of the local pre-commit gate. Anything that can abort a local commit must also abort CI, so a contributor cannot bypass the repository's local checks by skipping hooks or editing through a web UI. This supports [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) and keeps the link-checking boundary from [§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation) enforced alongside `grund check`.

## placement: What CI measures

```text
pre-commit hook list ─► [ CI ] ─► one verdict per push, pull request or merge-queue group
                         │
                         └─ runs the tree's own grund through cargo run; outside the pipeline
```

Not a component: CI is the remote form of the local gate over the whole tree, and it measures the system rather than sits in it ([§AR-system.5](README.md#5-what-holds-the-shape)). It takes the pre-commit hook list and gives one verdict per push, pull request or merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)). The only `grund` it runs is the one the tree builds, through `cargo run`, so the gate never depends on a released binary; it knows nothing of the pipeline ([§AR-system.1](README.md#1-the-system)) beyond the commands the hooks spell (section 1).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (ID, home, citable, section, lead),
[§FS-terms.terms.2](../functional-spec/FS-terms.md#terms2-citations) (citation, citation site), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan),
[§FS-terms.terms.5](../functional-spec/FS-terms.md#terms5-findings) (verdict), and [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

## 1. Pre-commit is the source of truth

The hook list lives in `.pre-commit-config.yaml`. CI must invoke that list directly with `pre-commit run --all-files`, rather than hand-copying each hook into separate workflow steps. The workflow may install hook prerequisites first, but the set of checks is defined by the pre-commit config. It may also skip, through pre-commit's own `SKIP`, exactly the hooks whose command another job of the workflow already runs ([§AR-ci.13.3](AR-ci.md#133-every-hook-runs-once)): the config still decides what is checked, and the workflow decides only where each check runs, never whether.

When a new pre-commit hook is added, the same change must ensure CI can run it. If the hook needs an external binary, the CI workflow installs that binary before the pre-commit step. If the hook is intentionally local-only, it does not belong in `.pre-commit-config.yaml`; put it in a developer-local hook instead.

### 1.1 Stages without a file list

`pre-commit run --all-files` reproduces only the stages whose input is a file list. A hook bound to a stage with different input — `commit-msg`, whose argument is a message file, or `pre-push`, whose arguments are the refs being pushed — is silently absent from that run, so "CI runs the config" is not by itself parity. Every hook bound *only* to such a stage needs an explicit CI counterpart that feeds it the input the stage would have supplied **and reaches the same verdict on the same tree**: [§AR-ci.8](AR-ci.md#8-commit-message-attribution-gate) is the `commit-msg` one, and no hook is bound only to `pre-push` any more ([§AR-ci.7](AR-ci.md#7-no-changelog-gate)). A hook that also runs at `pre-commit` — `cargo-test`, the one hook `pre-push` still runs — is reproduced by the all-files run and is not implicated. The verdict half is the half that bites: a counterpart fed different input is worse than a missing one, because it passes locally and then fails remotely on a tree the contributor cannot change.

### 1.2 The parity test

All of this is checked rather than remembered: `tests/integration/test_ci_precommit_parity.py` reads the hook list, workflow and Python wrapper and fails when CI stops invoking the hook list, a hook's binary has no pinned install step before that gate, a `commit-msg` hook has no range-scanning counterpart, the Rust hooks and the workflow's own steps spell different commands, [§AR-ci.3](AR-ci.md#3-current-hooks) does not spell a command a Rust hook runs, warnings are denied on one side only, the Python gates invoke different wrappers or discover tests from the wrong directory, a hook bound only to a stage without a file list has no CI step naming the same `scripts/` script, or the pre-commit step's `SKIP` names a hook no other job runs, or leaves out one that a job does ([§AR-ci.13.3](AR-ci.md#133-every-hook-runs-once)). What it holds is the *existence* of a counterpart, which is line-shaped; the *same verdict* half of [§AR-ci.1.1](AR-ci.md#11-stages-without-a-file-list) is held by each gate's own test, which runs both invocations against one fixture tree and asserts they agree — for the attribution gate, `tests/integration/test_check_no_claude_attribution.py`.

## 2. Platform scope

The full Rust build and test matrix still runs on every configured operating system. The pre-commit gate may run on one representative CI platform when the hooks are platform-independent, because its job is policy parity with local commits, not cross-platform behavior coverage. Platform-specific behavior belongs in the build and test jobs.

CI dependency and build caches are performance optimizations only. A cache restore/save failure must not abort the matrix before the actual checks run; the job should continue cold and let `fmt`, Python tests, pre-commit, build, self-check, tests, or benchmarks decide pass/fail.

## 3. Current hooks

The current pre-commit gate runs the same Rust format/build/test commands that development CI runs: `cargo fmt --all -- --check`, `cargo build --quiet --workspace --all-targets --locked` with warnings denied, and `python scripts/run_cargo_suite.py --workspace --all-targets --locked --features grund/test-workspace-load-count`, which runs `cargo test` with those arguments and prints what [§AR-ci.requirements.5.1](AR-ci.md#requirements51-a-suite-prints-its-failures-its-check-ins-and-one-summary) asks rather than a line per test. A plain `cargo test --workspace --all-targets --locked` is green on the same tree but runs less: it reports every test that reads the workspace-load observer as `ignored`, naming the feature that test needs ([§AR-ci.3.3](AR-ci.md#33-test-only-observers)). The test hook also runs at `pre-push`, so a contributor who commits while a test is transiently broken still gets the same local stop before sending the branch. It is the one hook `pre-push` runs: no hook asks a change for a changelog entry at any stage, because no change is gated on the changelog ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)).

The local Python gate runs `python scripts/run_python_gate.py`, and CI runs that
same command as its two selections, one job each ([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-node-acceptance)). The wrapper prepares
the built binary and released Grund 0.16.1 for the co-change compatibility cases
([§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)) and the same-source binding oracle for the
Node parity tests ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)), and the shared build target of the Node
acceptance, the inputs of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs), then runs unittest discovery of `tests/integration`
with the pattern `test_*.py` and that environment from the repository root, timing
each test ([§AR-ci.3.4.2](AR-ci.md#342-per-test-durations)). The parity test checks the shared wrapper invocation,
the two selections CI passes it, and its discovery directory and pattern.

### 3.1 Citations and links

The gate also runs `grund check --full`, including the grounding floor from [§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in), `grund fmt --write` for canonical citation links, and `lychee` for Markdown links. Running both in CI preserves the boundary [§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation) draws: `grund` owns ID citations across docs and source, `lychee` regular Markdown links and URLs.

A canonical `https://github.com/agent-grounds/grund/blob/main/<path>` link in
the checked tree is a same-repository link. Before the network pass, the link
gate maps it to `<path>` in the current checkout and asks `lychee` to validate
that local file and any fragment. Only the exact URLs that pass this local
check are excluded from the network pass; missing paths, bad fragments,
malformed self URLs, and links to every other repository or host remain
failures of the local or network check. This makes a durable default-branch URL
checkable on the branch that first adds its target without accepting a 404 or
pinning a shipped address to that branch.

`--full` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)) is here because a citation in a file outside `grund`'s own `[scan] include` and every kind home ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned)) is invisible to the plain run rather than merely unchecked — the drift this repository asks its users to guard against is one it can suffer too. The flag is purely additive, so the gate still asserts everything it asserted before.

#### 3.1.1 Bounded external-link tolerance

The repository's `scripts/check_links.py` wrapper and `lychee.toml` serve both
local pre-commit and Ubuntu CI, realizing
[§FS-non-goals.1.1](../functional-spec/FS-non-goals.md#11-this-repositorys-external-link-gate).
The existing network pass uses lychee 0.23.0 with explicit `timeout = 30`
(seconds for the whole request), `max_retries = 4` (after the initial attempt),
and `retry_wait_time = 1` (seconds for the exponential-backoff base).
Five attempts have nominal waits of 1, 2, 4, and 8 seconds. A persistent
whole-request timeout therefore consumes a nominal 165 seconds, plus overhead
and possible host-specific pacing, before a nonzero exit. HTTP 404 responses
remain failures, and unresolved timeouts are never accepted as successes.

The pinned binary retains a separate fixed 10-second connection timeout; the
30-second setting does not raise that ceiling. Both timeout phases enter
lychee's shared retry loop, so the extra retry offers either phase another
recovery opportunity. This does not identify which phase stalled at a public
host or guarantee immunity to arbitrary outages.

Real lychee tests against loopback HTTP must obtain a successful response after
a 21-second response-header delay, and after four retryable timeout failures.
They also retain permanent-timeout and HTTP 404 rejection, valid local
fragments, and invalid self-links stopping before any network request. Timing
may be scaled in supplementary fixtures only with an assertion of the exact
production policy and evidence exercising the unscaled policy. A subprocess
watchdog exceeds the selected finite budget; expiration is an infrastructure
failure, not evidence that lychee rejected a persistent link. The wrapper's
routing, exact self-link exclusions, hook invocation, checked documents, and
lychee version stay as described above; no outer whole-tree retry is added.

### 3.2 The managed block's text

Last of the `grund` hooks, and immediately after `grund fmt --write` because that hook rewrites the Markdown this one measures, comes `grund init --check` ([§FS-init.4](../functional-spec/FS-init.md#4-exit-codes)). It catches the one drift no other hook here sees: a managed agent-entrypoint block whose rendered text went stale outside its two generated sections while its `(vN)` heading stayed current, which `grund check` passes because it verifies that version rather than the bytes ([§FS-check.3.5](../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)). Stale text inside `### Citation directions` or `### Clickable citations` already fails the `grund check --full` hook that runs before it ([§FS-init.2.3.5](../functional-spec/FS-init.md#235-citation-directions), [§FS-init.2.3.6](../functional-spec/FS-init.md#236-clickable-citations)). The hook writes nothing and fails only on a path `init` would have written; the fix is the same `init` run without the flag. This repository runs on its own users' behalf here — [§REQ-agents-md.2](../requirements/REQ-agents-md.md#2-the-managed-block-stays-current) requires its own managed block to stay current, which `grund check` enforces by its version, and until this hook existed nothing could say when the rest of its text had gone stale under that version.

### 3.3 Test-only observers

Some tests count what the binary did rather than read what it printed, and the binary counts only when a test-only feature compiles the counter in. `test-workspace-load-count` is the one today: the workspace-load observer of [§AR-resolver.3.1](AR-resolver.md#31-grund-show---batch-loads-once), reached from the `grund` package as `grund/test-workspace-load-count`, and absent from every other build for the reason [§AR-ci.5.3](AR-ci.md#53-the-bench-feature) keeps the `bench` harness out of the regular matrix. The gate's test command enables every such feature, so the gate observes everything. A plain `cargo test` does not, and it is the run a contributor without the hooks types; its reader is the one who must not be sent the wrong way ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)). Without the feature the observer's log is never written, so a count read from it is an observed zero rather than an absent observation: a test expecting one load fails naming a file, a line and two integers, which sends the search into the code under test instead of the command line, and a test expecting none passes having observed nothing.

Stated so a test can hold a test file to it: **a test that reads an observation compiled only under a feature is ignored in a build without that feature, and its ignore reason names the feature as the gate passes it.** It never asserts on the observation there, so a default run neither fails nor passes on something it could not have seen; the same case's other assertions — exit code, stdout, stderr — sit in a test that runs in every build. `tests/integration/test_feature_gated_observers.py` holds each test file that reads an observer to this, and holds the gate's test command to naming every feature such a file needs, because the gate itself, running with every feature on, cannot see a test that breaks the rule.

### 3.4 The Python gate's inputs

The wrapper hands its discovery run four inputs, each an absolute path in the
environment, and treats the four alike: a variable already set is used as
given, and an unset one is prepared from this checkout. `GRUND_BUILT` is the
tree's own `grund`, built by `cargo build -p grund --locked`. `GRUND_RELEASED`
is released Grund 0.16.1, installed once into the wrapper's scratch and held to
its `--version`. `GRUND_BINDINGS_ORACLE` is the same-source Rust oracle the Node
adapter is compared against ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)): the `grund-binding-oracle`
example of `grund-core`, built with `GRUND_BINDINGS_SOURCE_SHA` set to
`git rev-parse HEAD`, so that its `--metadata` reply names the commit under test
and protocol version 1. That is how the workflow's own step builds it. The
wrapper builds it with the plain `cargo` its other build uses, because the
workflow's toolchain pin is CI's, and a local gate must not fail for want of it.
It prepares the oracle on every run that is not given one, and the build is
incremental, so a run after a new commit meets an oracle of that commit rather
than a stale one.

Preparing an input never turns into skipping the tests that need it. A build
that fails, or an executable missing where its build put it, fails the gate:
[§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance) asks this of the compatibility run, and the Node
parity tests ask the same of theirs. The workflow sets the oracle before the
wrapper runs, as [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth) lets it install a hook's prerequisites first, and the
wrapper then uses that one. Nothing a local run lacks may stand between it and
the verdict CI reaches on the same tree.

`tests/integration/test_python_gate_inputs.py` holds the wrapper to this. With
the oracle unset and no CI in its environment, the oracle the wrapper returns
must answer `--metadata` with this checkout's `HEAD` and protocol version 1. An
oracle it is given must come back as that same file's absolute path.

#### 3.4.1 The Node acceptance's shared build target

`GRUND_NODE_TARGET` is the fourth input: one Cargo target directory that every
build of the Node acceptance ([§AR-bindings.5](AR-bindings.md#5-grund-node-the-napi-rs-binding)) shares. Unset, the wrapper prepares
`~/ag/tmp/grund-node-target`, outside the checkout and kept between runs, and CI
restores that directory from a cache step of its own, so a job starts with the
dependency tree already compiled. The acceptance's own helper reads the
same variable and falls back to the same directory, so `tests/bindings/node/run.py`
run on its own shares it too. Every build site names this target or a directory
under it, never a fresh temporary one: the source package and its bundled
fallback rebuild, the documented native builder and its test-seams build, and
the CLI. The CLI the acceptance compares against is built once per run and
reused by every test that needs it. Cargo's own lock makes the sharing safe.

That cache is not the cargo cache's. Its key is
`<os>-node-target-v2-1.95.0-<Cargo.lock hash>`: the runner's OS, a version to
bump when what it holds must be thrown away, the binding toolchain, and the lock
file, with the same key less its hash as the restore prefix. A key of its own is
what makes it save at all: `actions/cache` saves nothing on an exact hit of its
primary key, and the cargo cache's key already hits whenever `Cargo.lock` has not
changed, so a path added to it would never be written. It holds the release
binding's and the dev builds' compiled dependencies and no incremental state;
`v2` is the bump that came with the dev builds of [§AR-ci.3.4.1.1](AR-ci.md#3411-one-release-build-and-every-build-on-one-line), whose
dependencies an entry saved before them does not hold and, on an exact hit,
would never save.
It cannot leave that state out, because `actions/cache` archives a listed
directory whole, so the builds never write it: the CLI builds with
`CARGO_INCREMENTAL=0`, and so do the dev builds of the binding, the release
build is not incremental, and a fresh
checkout's workspace crates could not reuse the state anyway. After one run on
Linux the target is about 0.7 GB before compression, three caches in all,
against a repository limit of 10 GB the other caches already crowd; the
tarball's own crates are not worth caching, because they rebuild at their fresh
paths every run anyway.

Sharing a target changes what is cached, not what is proven. The builds keep
`--locked`, the bundled-sources closure, and the fallback
rebuild after the native artifacts are removed from the installed tarball, so
[§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership) holds as written: the tarball's own workspace crates
still compile from its own sources, at their own paths, and generated files
still stay outside the checkout. What a warm target no longer repeats is the
third-party dependency tree, which that point already lets a source build take
from a populated cache.

`tests/integration/test_node_shared_target.py` holds this: with the variable set,
every `--target-dir` the source package and the CLI build name lies under it and
the CLI is built once; every other build site names this target, the rehearsal's,
or a directory under one of them; unset, the wrapper and the helper agree on one
persistent directory outside the checkout, which a CI cache step of its own
restores, under a key that is not the cargo cache's.

##### 3.4.1.1 One release build, and every build on one line

A warm target saves only the dependency tree, and under the release profile
that is the cheap part: `lto = true` and one codegen unit make the optimisation
and link of the tarball's own cdylib about 48 s of a 50 s build on a
developer's machine, and two to four times that on a runner, with or without a
warm target. So the acceptance pays for that profile once. Exactly one build of
the binding uses `--profile release`: the documented native builder command of
[§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership), in `tests/e2e/test_node_binding.py`, which is the
acceptance's proof that release is ordinary LTO. Every other build of the
binding passes `--profile dev`: the test-seams build, the source package's
`npm run build:source`, and the bundled-fallback rebuild in an unpacked
tarball, the last two through `build:source`'s pass-through to the builder.
What each of those proves does not depend on the optimiser: the test-seams build
proves the runtime seams, worker panics, the writer lease and teardown, and the
source package and its fallback prove the bundled `--locked` source closure, its
ESM, CJS and TypeScript surface, and that the fallback compiles from the
tarball's own sources without reaching the checkout. [§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership)
names a profile for the native builder only, so it holds as written. A build
that only refuses before Cargo runs, or reaches a probe that stands in for it,
compiles nothing and is not counted.

So that CI can attribute the acceptance's time without the captured output of
cargo, every build the acceptance runs prints one line to its standard error
as it finishes: the profile, the target directory and the wall seconds, in the
form `node build: <profile> <target-dir> <seconds>s`. It is one line per build
and none for anything else the harness runs. The gate reaches the acceptance
through `tests/integration/test_node_binding_acceptance.py`, which captures its
output to show only on failure; it passes these lines, and nothing else, on to
its own standard error on every run, so they reach the job log of a green leg.

`tests/integration/test_node_shared_target.py` holds this too: across the
support module's source package and every other build site, exactly one build
of the binding resolves to the release profile, and it is the documented native
builder's; a build the harness runs prints its one line naming its profile,
target and seconds; the wrapper shows those lines, and only those, on a green
run; and the CI cache key carries the `v2` this point bumped.

#### 3.4.2 Per-test durations

The wrapper times every test it runs and ends with a short table of the slowest
ten, one `<seconds>s <test id>` line each under a `Slowest tests` heading, so where
the gate's time goes is measured rather than inferred from the timestamps of
`unittest`'s dot lines. A test's time runs from the previous test's end to its
own, so the class and module fixtures run before it, `setUpClass` and
`setUpModule` and the previous class's teardown, are charged to it, and the last
test carries the run's final teardown; a cargo build in a fixture shows up
against the first test of its class rather than nowhere. A discovery that finds
no tests exits 5, as `python -m unittest` does from Python 3.12. It needs no
Python newer than the gate's own floor, which is why it does not use
`unittest --durations`, new in Python 3.12. It adds those
ten lines and no line per test, because a passing test is not worth a line of
the log. `run_python_gate.run_tests(start, environment)` is the one entry that
runs a discovery and prints the table; the wrapper calls it on `tests/integration`
and returns its exit status.
`tests/integration/test_python_gate_durations.py` runs it on a small suite of its
own.

#### 3.4.3 Selecting the Node acceptance

The wrapper takes at most one argument, which selects what its discovery runs.
`--without-node-acceptance` runs every test it discovers except the Node
acceptance, the tests of the module `test_node_binding_acceptance`;
`--only-node-acceptance` runs those alone. No argument runs everything, and that
is what the local `python-test` hook runs, so the local verdict is the one the
two selections reach together, and CI runs them as two jobs ([§AR-ci.13.1](AR-ci.md#131-one-job-per-language))
because the Node acceptance is the longest test of the gate by minutes. Any other
argument, or both at once, exits 2 with a usage line before any input is
prepared, so a mistyped selection costs nothing and is never silently the whole
suite.

The cut is by module. A module that fails to import is still in the selection
its name puts it in, so a Node acceptance that no longer imports fails the Node
job rather than vanishing from both. Each selection still ends with the slowest
table and still exits 5 when it finds nothing ([§AR-ci.3.4.2](AR-ci.md#342-per-test-durations)), so a Node
selection that matches no test fails rather than passing on nothing run.
`run_python_gate.select(suite, selection)` is the one function that makes the
cut: given a discovered `unittest` suite and `None`, `'without-node-acceptance'`
or `'only-node-acceptance'`, it returns a suite of the tests that selection runs.
`run_python_gate.run_tests(start, environment, selection=None)` hands the selection to
the one timed discovery, so a selection is cut where the tests are found.
`tests/integration/test_python_gate_selection.py` holds it to this without
running the Node acceptance itself.

## 4. Performance smoke guard

CI carries a cheap floor on [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) that does not depend on any benchmarking toolchain: the `grund check . --full` self-check step runs the already-built binary under a generous wall-clock `timeout` — long enough never to flake on a loaded runner, short enough to fail the build on a catastrophic regression such as an accidental quadratic walk or a second read pass over every file. It is not the budget itself (the budget is tens of milliseconds; the ceiling is tens of seconds) — it is the difference between "we'd notice eventually" and "the build is red on the commit that did it". The precise per-commit meter is the [§AR-ci.5](AR-ci.md#5-benchmark-job) benchmark job, and this timeout stays as its catastrophic backstop.

## 5. Benchmark job

A separate `bench` job runs the instruction-counting harness ([§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands)). Because the harness runs `grund` under Callgrind, the job first installs `valgrind` and `cargo install`s `iai-callgrind-runner` pinned to the same version as the `iai-callgrind` dev-dependency. It runs on one representative platform (the harness measures instruction counts, not cross-platform behavior — [§AR-ci.2](AR-ci.md#2-platform-scope)'s reasoning applies) and on every push and pull request, so the numbers are recorded per commit.

### 5.1 Pull requests and pushes

On pull requests, the job checks out the PR base branch and runs `cargo bench -p grund --features bench --locked --quiet --bench instructions -- --save-baseline=main --save-summary=json`; then it checks out the PR commit and runs `cargo bench -p grund --features bench --locked --quiet --bench instructions -- --baseline=main --save-summary=json`. The named baseline comes from the current base branch rather than a stale generated artifact, while the committed baseline figures in `docs/benchmarks.md` remain the human-readable reference point. On pushes to `main`, the job records the current counts and saves JSON summaries without comparing against itself. It does not run on a merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)): the pull request already compared, and the push the queue makes to `main` records.

### 5.2 Regression limits

The intended harness limits are 5% `Ir` for the generated 10k-file fixture and 12.5% `Ir` for the other hot commands, which scan the smaller generated fixtures rather than this repository's own tree ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)). Those limits are not wired up today: the PR rerun passes no limit flag and the harness sets no `RegressionConfig`, so a regression is recorded in the summaries and compared against the baseline but does not fail the build; growth beyond those limits is the behavior the gate is intended to enforce for the [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) targets.

### 5.3 The `bench` feature

The harness body is gated behind the `bench` Cargo feature, so the regular build and test matrix compiles only a no-op bench target and stays free of the Valgrind dependency — only this job needs it; that gate is also why [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s "CI installs hook prerequisites" pattern is followed here as a dedicated job rather than folded into the test matrix.

## 6. PGO stays out of development CI

Development CI does **not** run the profile-guided-optimization pipeline. `scripts/pgo-build.sh` does an instrumented release build, runs the [§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) self-repo hot command list as the training run, merges profiles with `llvm-profdata`, and rebuilds the optimized release artifact; that cost and toolchain belong to release packaging ([§FS-distribution.4](../functional-spec/FS-distribution.md#4-release-process), [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)) and explicit benchmark work, not ordinary push/PR feedback. The manual **Pre-release checks** workflow runs the PGO script and self-checks the resulting binary before publish, and that job gates the platform it builds on; a platform whose hosted-runner PGO training is broken, which the release matrix marks `pgo_required: false`, publishes a self-checked LTO-only binary instead of blocking the release ([§FS-distribution.4.9](../functional-spec/FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)). The cross-registry candidate is a manual lane of the same kind: `candidate-rehearsal.yml` runs only on `workflow_dispatch`, and its PGO builds and its installed rows on each OS's runner never run on push or pull requests ([§FS-distribution-candidate.5.1](../functional-spec/FS-distribution-candidate.md#51-the-rehearsal-holds-no-credential), [§FS-distribution-candidate.5.6](../functional-spec/FS-distribution-candidate.md#56-a-row-runs-on-its-own-runner-and-a-skipped-row-is-named)). What push/PR CI does carry of it is cheap and offline — the inventory, manifest, launcher, mocked-publisher and workflow-isolation tests in `tests/integration/`, collected by the ordinary Python gate. The dev loop remains `fmt`, Python tests, pre-commit hooks, build, self-check, tests, the instruction-counting benchmark job ([§AR-ci.5](AR-ci.md#5-benchmark-job)), and the `commit-messages` job ([§AR-ci.8.1](AR-ci.md#81-ci-is-the-enforcing-layer)).

## 7. No changelog gate

Neither a hook nor CI asks a change for a changelog entry: no change is gated on the changelog, and the release builds the record itself, from the pull requests merged since the previous tag ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)). A gate stood here — a `pre-push` hook and a pull-request job asking one question of local git, whether the branch added an entry — and released records cite this section for it, which is why the number stays. It went because the record moved: an entry per change cost every change a file, a number and a conflict on every rebase, and the hand-written release section that replaced it cost every release a pull request and held the scheduled one until somebody wrote it.

What the gate asked that the release still needs is answered elsewhere. Nothing waits in the tree for a release ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so there is no pending entry to check and no hold: neither helper workflow numbers anything before the rotation, and the scheduled one no longer waits for a section to be written ([§FS-distribution.4.4](../functional-spec/FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)). That nothing merged ships without its line is the release's own list to answer, since it asks the forge about every commit in the range and refuses rather than write a partial one ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)). The one record a change still writes for a release, a verdict correction's `release-note` section ([§FS-distribution.4.6.4](../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), is asked of the tree by the Python suite — the backward-compatibility meter, `tests/integration/test_backwards_compatibility_routes.py` — which both `pre-commit run --all-files` and the test matrix run, so a correction without its notice fails on the pull request that makes it. The same suite holds every decision record's `release-note` section to the one bullet the bump publishes, reading it with the bump's own `release_notices` functions rather than a copy of them, in `tests/integration/test_release_note_shape.py` ([§FS-distribution.4.6.4.1](../functional-spec/FS-distribution.md#4641-the-shape-is-held-on-the-pull-request-that-writes-it)), so a notice the next release would refuse fails on the pull request that writes it rather than at that release.

## 8. Commit-message attribution gate

No AI-tool attribution boilerplate — a co-author trailer naming an assistant, a "generated with" marker, a session-link trailer, an assistant's no-reply address — may land in this repository's files or in its commit messages. `scripts/check_no_claude_attribution.py` owns the pattern set and scans three surfaces from one definition: staged file contents at `pre-commit`, a single message at `commit-msg`, and a revision range in CI. The patterns are narrow by design, because the target is machine-appended boilerplate rather than the subject matter: prose that discusses Claude is ordinary repository content and stays legal.

### 8.1 CI is the enforcing layer

The two local stages are advisory, and both failure modes are real. `pre-commit install --hook-type commit-msg` is a per-clone step a fresh clone has not run, and `--no-verify` skips whatever is installed; a repository history is the evidence that neither is hypothetical. CI is therefore the enforcing layer, and it runs as a dedicated `commit-messages` job rather than a step in the test matrix: the check needs the full history (`fetch-depth: 0`) that the matrix deliberately does not fetch, needs no Rust toolchain, and should report in seconds rather than behind a build.

### 8.2 The scanned range

The job scans what the event contributes, not the whole history — a pull request's `base..head`, a merge-queue group's `base_sha..head_sha` ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), a push's `before..sha`. An unresolvable base is not a failure: the first push of a branch reports the all-zero SHA and a force-push reports a SHA the runner cannot reach, so the script narrows to the tip commit and says so. A gate on messages catches an offending commit only while it is still cheap to amend; once merged, the same trailer costs a history rewrite and a force-push of every descendant branch and tag, which is why this runs on the pull request rather than on the release.

## 9. File-size budget gate

Source files are read by agents far more often than they are written, and a file that has outgrown its subject is charged to every one of those reads. [§AR-core-module-layout.3](AR-core-module-layout.md#3-file-size) has stated the rule for implementation files since the core/CLI split — below 500 lines of code — but stated it with nothing to enforce it, and several files reached eight times that figure. Reviewers see diffs, not totals, so nothing pushes back at the moment a file crosses the line.

[`fissile`](https://github.com/agent-grounds/fissile) is that feedback loop, configured in `.agent-grounds/fissile.toml`. It measures and reports only; it never edits code, so how to split a flagged file stays a human decision.

### 9.1 Where the config lives

The config sits in `.agent-grounds/` rather than under `.agents/` because the two directories are different kinds of thing: `.agents/` holds agent *instructions*, and a managed agent sandbox may mount it read-only — a budget kept there is one the agent working in the checkout cannot repair. A config left at the old `.agents/` path is still read, as a deprecated fallback, so nothing breaks for a repository that has not moved — but the move here is only half a change on its own: discovery of the new home arrives in `fissile` 0.8.3, and any pin below it finds no config at all and falls back to built-in defaults. That is why [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s install step moves in the same commit as the file.

### 9.2 What a budget counts

Budgets are per file type, because the number that is right for an implementation module is wrong for a test file, a golden fixture, or a CI workflow. Line budgets count code lines only: this repository's method puts citations and rationale in doc comments ([§AR-core-module-layout.4](AR-core-module-layout.md#4-citation-placement)), and a budget that taxed comments would pay contributors to delete the grounding the project exists to keep. A byte budget is the backstop for a file that is expensive to read for some other reason.

### 9.3 Citable Markdown

Markdown carries two budgets, because `grund` decides who pays for a document's length. A spec is reached by citation and read as a lead, a section, or a table of contents ([§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id)), so [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) — the right amount of spec, not the whole file — is already answered at the read rather than at the file, and a ceiling in the hundreds of lines bought a reader nothing. It did have a price: a document whose numbered sections are live citation targets cannot be split without relocating those IDs and sweeping every citing site in the same change, so the budget was pushing a heavier move than the overage justified and collecting deferred exceptions instead.

The citable tree under `docs/` therefore gets soft 750 / hard 2000. The soft tier is about twice the code rules — `core-source` is 350/500 and `tests` 450/700 — and what it names there is the defect that still costs something: a document that has become two subjects. The hard tier is four times `core-source`'s ceiling, and that distance is deliberate: it is the tier with no override, so it has to sit far enough above the natural size of the documents it measures that no ordinary edit can reach it.

### 9.4 Entrypoints

Read-whole files, under the `entrypoints` rule, keep the tighter **soft** budget of 250, because nothing addresses `README.md`, `CLAUDE.md`, `AGENTS.md`, or a `SKILL.md` by citation: they are read whole, the agent entrypoints into every session's context, and every line is charged to every one of those reads. Their **hard** ceiling is raised, from 400 to 500. `skills/grund-init/SKILL.md` measured 387 lines and cannot be split at all — it is a shipped artifact whose bytes a test asserts ([§FS-init.5](../functional-spec/FS-init.md#5-agent-setup-instructions)) — so the old ceiling stood 13 lines from blocking a file no contributor is allowed to shorten, and the tier with no override is the wrong tier to park a shipped artifact against. Today, `fissile measure` reports 539 lines for it — 522 when that entry was written, then 533 and 539 as two more promises landed — so the raised ceiling is behind it too and the file is no longer a standing soft finding: it is carried by a **structural** hard entry in `docs/file-size-human-exceptions.toml` with no soft twin, which is the shape [§AR-ci.9.7](AR-ci.md#97-structural-and-deferred-entries) and [§AR-ci.9.8](AR-ci.md#98-soft-twins) prescribe for a file whose split is illegal rather than owed. What moves as the tree changes is that entry's `max_accepted`, raised by a human each time grund promises to say more, so re-measure the file whenever the instructions it ships grow.

### 9.5 One policy across the foundation projects

The four foundation projects state both budgets as one shared policy, which is the argument of [§AR-ci.9.3](AR-ci.md#93-citable-markdown) and [§AR-ci.9.4](AR-ci.md#94-entrypoints) generalized, and add that a changelog is an append-only record that is not line-measured at all. Adopting it moved the `docs` ceiling from 1500 to 2000, for the same reason the entrypoint ceiling moved from 400 to 500: `rhei`'s largest spec measures 1482 lines, so 1500 would have left it 18 lines — one ordinary edit from a block with no override, against documents that cannot be split cheaply ([§AR-ci.9.3](AR-ci.md#93-citable-markdown)). 2000 is not a new opinion either; it is the ceiling `rhei`'s own `AR-source-file-size.1` already declares for any file. The largest document today is `docs/functional-spec/FS-workspace.md`, at 1115 non-blank lines — 365 over the 750-line soft budget and 885 below the 2000-line hard ceiling — so re-measure it as the tree changes. When the limit was raised, no entry in either registry accepted a file against the `docs` rule, so raising it stranded none of them; the soft registry has since accepted this file against that rule.

### 9.6 Severities and registries

**soft** warns and exits 0 — the nudge to split next time you are in the file. **hard** fails the commit, and there is no override flag.

Each severity has its own registry, and an accepted file is recorded in the one matching the finding it silences: `docs/file-size-agent-exceptions.toml` for soft, `docs/file-size-human-exceptions.toml` for hard. The split is a permissions boundary, not bookkeeping — an agent may record soft debt on its own, while a hard entry is a human's to add. Every entry carries a reason, an `until` trigger that retires it, and the size it accepts. That last field is what makes this a ratchet rather than an amnesty: an accepted file is frozen at its recorded size, and one line past that ceiling brings the finding back at full severity. Adoption against an already-oversized tree is therefore honest in both directions — nothing is blocked today, and nothing can quietly get worse.

### 9.7 Structural and deferred entries

An entry's `kind` fixes what its reason has to establish, because "this file must not be split" and "this file has not been split yet" are different claims and only one of them is debt. A **structural** entry names the constraint that makes splitting illegal or harmful — a shipped artifact whose bytes are asserted, a type registry with no internal seam — and never expires, so its `until` is `indefinite`. A **deferred** entry names the boundary that is missing and what has to exist before the split can happen, and its `until` is that condition. A size threshold is not a condition: `until` may not say "revisit if it passes 500", because a bigger number retires nothing and the entry would outlive everyone who understood it. Keeping the two kinds apart is the point — a file that is merely large should not be able to borrow the language of one that must stay whole — and `fissile audit` reports them as separate counts so debt cannot hide inside necessity.

### 9.8 Soft twins

The kind also decides how far one entry reaches. A file over the hard budget is over the soft one too, and `fissile` leaves that soft finding standing under a **deferred** hard entry — by the tool's reasoning the split is still owed, so its nudge keeps arriving until something silences it. That is why this repository records such a file in both registries, the normal steady state for a file awaiting a split: the hard entry keeps the commit unblocked, and the soft entry stops the same known fact from being reprinted on every unrelated commit until someone stops reading the output. A warning nobody can act on today is noise, and noise is what teaches a reader to skip the one finding that mattered. A **structural** hard entry needs no second entry, because it silences both tiers itself: splitting the file is illegal, so a soft warning would name work nobody may do and no amount of work could clear.

The argument still lives in exactly one place. The hard entry carries it; the soft twin records the same ceiling, points at the entry that reasons, and retires when it does. Copying the prose across would give one file two arguments free to disagree, and the disagreement would surface as a reader trusting the stale half. The twin points by naming the hard registry, because an entry has no name of its own: what identifies it is the registry it lives in and the path it accepts, which is the same pair `fissile` leads its diagnostics with. An `EX-NNN` slug on top of that pair would be a second name for a fact the tool already computes, and the second name is the one that can go stale.

### 9.9 The registry test

Neither of those rules is visible to `fissile`, which validates the sizes but not what a reason may claim, so `tests/integration/test_file_size_exceptions.py` enforces them: kinds against their `until`, `until` against line-count triggers, twins against their originals — present for a deferred entry, absent for a structural one — and every recorded path against the tree, so an entry left behind by a rename fails the commit that renamed it. It also pins the registry schema version, so upgrading `fissile` without migrating the registries fails by name rather than as a parse error at commit time.

### 9.10 Remote enforcement

Per [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth) the hook is in `.pre-commit-config.yaml` rather than a developer-local hook, so CI installs `lychee` beside `pre-commit` on the one leg that runs it, and `pre-commit run --all-files` enforces the same budgets remotely. The `fissile` binary is installed on every leg instead, ahead of the Python tests, because the ownership regression of [§AR-core-module-layout.1.5](AR-core-module-layout.md#15-split-ownership) measures with it. The hook takes its file list from pre-commit, so a commit measures what it touches while the CI run measures the whole tree; `[scan].exclude` applies either way, keeping generated asset mirrors and the e2e fixture trees out of both.

## 10. A test fixture owns the tree it scans

A test that writes a temporary repository and scans it must be the only thing writing, reading and removing that tree. Where two tests can name one root, the suite fails on a file it wrote itself, on one platform, and passes on the rerun — and a check that goes green on a rerun teaches everyone to rerun, which is how a genuine platform-specific regression gets waved through. That is [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) failing at the gate instead of at a release, and the gate cannot tell the two apart from the outside, so the rule is on the fixture: its root is unique by construction, and it holds that root exclusively.

### 10.1 Identity, not the clock

A fixture root's name is derived from what distinguishes the fixture — the process id together with the thread that builds it, or a process-wide atomic serial — and never rests on a reading of the wall clock. `libtest` runs a test binary's cases as threads of one process, so inside a binary the process id distinguishes nothing and a timestamp is the whole of such an identity; two cases whose readings land in one tick of the platform's clock resolution then name one directory. That resolution is the platform's to choose — macOS reports `clock_getres(CLOCK_REALTIME)` as 1000ns where Linux reports 1ns — so a clock-derived name is unique on a maintainer's machine and not on the runner's, which is the asymmetry [§AR-ci.2](AR-ci.md#2-platform-scope) leaves to the build-and-test jobs and the one no one is positioned to debug.

Stated so a test can hold a fixture to it, with no control over the clock: **two roots named from one clock reading still differ.** A thread id alone does not satisfy that, because two fixtures built on one thread share it; the process-wide atomic serial of the four `crates/grund-cli/tests` fixtures does, and so does a thread id carrying one. `crates/grund-core/src/testing.rs` (`test_root`) takes a caller-supplied name beside its thread id for the same reason.

### 10.2 The root is claimed, not adopted

A fixture creates its root exclusively — `create_dir` on the root itself, so a name already taken is an `AlreadyExists` the suite reports — rather than `create_dir_all` over whatever is already there. This is the half that closes the class rather than one instance of it: a unique name makes today's collision impossible, while an exclusive claim makes any later one a named failure at the moment it happens instead of a file vanishing mid-scan in an unrelated case.

Two things follow. A fixture removes only a tree it created, because a `Drop` that removes the root by name whether or not this fixture created it is what turns sharing into a deleted tree. And a refused claim leaves what it found untouched: clearing a root in order to take it moves the deletion earlier rather than removing it, so pre-clearing is not a substitute for refusing, however reasonable it looks as leftover hygiene.

### 10.3 A fixture root inherits no config from above it

Discovery climbs from the path it is given to the filesystem root, probing both config names at every level ([§FS-config.1](../functional-spec/FS-config.md#1-file-location-and-discovery)), and a fixture root is no exception: a `grund.toml` anywhere above it, bare or under `.agents/`, becomes that fixture's config. A root built under the system temp directory therefore belongs to whatever tree `TMPDIR` happens to sit in, and a scratch directory inside a grounded project is an ordinary place for it to sit. A case asserting the zero-config fallback ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)) then fails as though discovery had regressed, when what moved is the environment, and a contributor reads it as a regression in whatever change is under test. The walk is right and stays as it is; the rule is on the fixture: the root a fixture helper hands out is one that no config covers, under either name, at the root or through any of its ancestors.

The helper takes its base from a short candidate list, the system temp directory first and, on unix, `/tmp` where that differs, and uses the first candidate no config covers. It asks through discovery's own probe, so the guard reads exactly the names discovery reads. When every candidate is covered, the suite fails at the helper, naming `TMPDIR`, each candidate and the config that covers it, rather than handing out a covered root and letting a zero-config case invert into a failure about the fixture. This is the version-control guard `fixture_root` in `crates/grund-cli/tests/init_refused_targets.rs` already holds, applied to config files. The check belongs to the helper and not to the cases: no case sets `TMPDIR`, because the process environment is shared by every case running beside it.

Stated so a test can hold `crates/grund-core/src/testing.rs` (`test_root`) to it: **no config under either name sits at or above a root the helper returns.** A zero-config case that does find one names the absolute path it found, so the failure says where the config came from.

### 10.4 Git descendants stay inside the fixture lifetime

Waiting for a Git command to return does not wait for its detached maintenance
descendants. A temporary Git fixture must prevent automatic GC and maintenance
from leaving a repository writer alive when strict cleanup starts, including
geometric repacking on newer Git versions. This supplies the lifetime half of
[§FS-distribution.4.5.1](../functional-spec/FS-distribution.md#451-release-helper-verification-owns-its-temporary-history-through-cleanup).

The changelog suites establish `gc.auto=0` and `maintenance.auto=false` as local
repository configuration immediately after initialization and before the first
commit. Shared initialization applies that policy to release listing, compatibility
notices and archive rotation; direct dated commits and rebases inherit it from the repository,
so command wrappers cannot be the only enforcement point. Explicit synchronous
Git work remains available to tests that need it.

`TemporaryDirectory.cleanup` stays strict. Ignoring cleanup errors, retrying removal
or sleeping until a detached writer finishes does not meet this contract. A
regression forces a real maintenance threshold and checks both the returned-command
boundary and strict removal; reading configuration alone is not evidence that the
tree has no outliving writer.

## 11. A release candidate is dispatched the same CI

Beside push, pull request and merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), the workflow runs on `workflow_dispatch`, and that trigger is the release helpers': each dispatches it on its release commit and on the `-dev` advance built over it, and pushes neither to `main` until both have passed ([§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). A dispatched run is the same CI a push to `main` gets, not a narrower one, because the commit it judges is about to become `main`. It behaves as a push does: the benchmark job records counts without comparing ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and the commit-message job, given no `before` to scan from, narrows to the tip commit ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)). `tests/integration/test_release_candidate_runs.py` holds the trigger to the helpers that use it.

## 12. A merge queue lands every pull request

`main` takes a pull request only through GitHub's merge queue. The queue tests each pull request on `main` together with every pull request queued ahead of it, under the checks a pull request must pass — the three `cargo test` legs, each the aggregate of its operating system's jobs ([§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate)) — and lands exactly the commit it tested, so nothing reaches `main` that CI has not passed, as [§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed) holds the release helpers to. That is why no pull request has to be current with `main` itself, and the up-to-date requirement is off: a pull request that went green on an older `main` is not rebased and run again for every merge ahead of it. The requirement is not merely dropped, because in this repository two pull requests that are each green often fail together — one moves a point the other still cites, and only their combination is red. The queue meets that combination on its own ref, before `main`; the requirement switched off without a queue would let it land, and `main` would go red on a commit no run had judged ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

The workflow runs on `merge_group` for the queue. The test matrix runs there as it does on a push, and the commit-message job scans the group's `base_sha..head_sha` ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)). The benchmark job does not run: on a pull request it compares against the base branch, and on a push to `main` it records counts ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and the queue's push to `main` still runs it. That push run stays the record of `main`. `tests/integration/test_merge_queue_runs.py` holds the trigger and both jobs to this.

## 13. Each operating system's suites run as parallel jobs

A leg that runs its suites one after another takes as long as their sum, and the
slowest leg is how long every pull request and every merge-queue group waits
([§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-ten-minutes--directional)). So the suites of a leg run as jobs of their own, side by
side, and the leg takes as long as its longest one. The jobs are written out
rather than generated by YAML anchors, because the tests that hold this layout
read the workflow as text: the CI Python has no YAML parser.

### 13.1 One job per language

On `ubuntu-latest` and `windows-latest` each leg is three jobs. The **Rust** job
runs `cargo fmt --all -- --check`, the native watch contract, `cargo build
--quiet --workspace --all-targets --locked`, the grund self-check of §AR-ci.4 and
`python scripts/run_cargo_suite.py --workspace --all-targets --locked --features
grund/test-workspace-load-count` ([§AR-ci.3](AR-ci.md#3-current-hooks)). The **Python** job runs `python
scripts/run_python_gate.py --without-node-acceptance`, and the **Node** job
runs `python scripts/run_python_gate.py --only-node-acceptance`
([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-node-acceptance)). No job on those two runs more than one of the three suites.

`macos-latest` may keep them in one job. It is the shortest leg, and the free
plan's five concurrent macOS jobs are shared by every run, the merge queue's
concurrent groups included ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), so splitting it buys little and can
starve the queue.

On every operating system each of those commands runs exactly once: none is
dropped by the split and none repeated by it. The plain `python
scripts/run_python_gate.py` runs in no job, because it would run the Node
acceptance a second time.

### 13.2 The required check is an aggregate

Each operating system keeps one job named `cargo test (<os>)`, the name the
ruleset requires and the merge queue waits for, so neither changes. It runs no
suite of its own. It `needs:` every job that runs a command of [§AR-ci.13.1](AR-ci.md#131-one-job-per-language) on
its operating system — on Ubuntu, the pre-commit job too — and no job of
another. That is why the language jobs are a job per operating system rather
than a matrix: a matrix job's result, as a job needing it reads it, is all its
legs' results together, so an aggregate needing one would judge every operating
system's suites and wait for all of them.

It passes only when every job it needs succeeded. Its `if:` is `always()`,
because a job whose need failed is otherwise skipped, and a skipped required
check counts as passed; its step then fails on any result other than
`success`, so a need that failed, was cancelled or was skipped fails it. It runs
on `ubuntu-latest` whichever operating system it judges, so it takes no macOS
slot.

### 13.3 Every hook runs once

The pre-commit step runs on Ubuntu with `SKIP` naming exactly the hooks whose
command another job already runs: `cargo-fmt-check`, `cargo-build` and
`cargo-test`, whose commands the Rust job's steps spell, and `python-test`,
whose command the Python and Node jobs run between them as its two selections.
Every other hook runs there and only there. So each hook's command runs once on
Ubuntu, a skipped hook is one a named job runs, and the list in
`.pre-commit-config.yaml` stays what is checked ([§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth));
`tests/integration/test_ci_precommit_parity.py` computes the set it must skip
from the workflow rather than from a list of its own ([§AR-ci.1.2](AR-ci.md#12-the-parity-test)).

### 13.4 Cache keys are per job

Every job still restores the cargo cache and builds what it needs. No two jobs
on one operating system share a cache key: `actions/cache` saves nothing on an
exact hit of its key, so the first job to save a shared key would fix what every
other one restores, and theirs would never be saved. A key names its job, as
`${{ github.job }}` or a literal does; a restore prefix may still fall back to
another job's entries, because reading one overwrites nothing. The Node
target's cache of [§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target) is the job's that runs the Node acceptance.

### 13.5 The layout test

`tests/integration/test_ci_language_jobs.py` reads `ci.yml` as text, expands
each `matrix.os` list, and holds the workflow to [§AR-ci.13.1](AR-ci.md#131-one-job-per-language), [§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate) and
[§AR-ci.13.4](AR-ci.md#134-cache-keys-are-per-job); it runs its checks against fixture workflows too, so a check that
can no longer fail is caught.

## requirements: What CI's wall clock and its log hold to

Requirements on how long the gate takes (requirements.1–4) and on what it prints (requirements.5), each carrying a **mark**, as [§FS-config.requirements](../functional-spec/FS-config.md#requirements-what-the-config-contract-holds-to) marks its own: *realized* where CI behaves this way today and a section here says so; *directional* where it binds what is written from here on rather than describing what runs today; *deferred* where a named follow-up owes it. Every pull request waits for the slowest gate, and so does every merge, which the merge queue of [§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request) lands only after a full run on its group, so a slow gate is the loop [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) says gets routed around. A **gate** here is one job of `ci.yml` as the forge names it, and each leg of a matrix job is a gate of its own: `cargo test (ubuntu-latest)` and `cargo test (windows-latest)` are two gates. The average is the target and the maximum is the hard limit: a gate that drifts past the average is a report, and a gate past the maximum is a failure.

### requirements.1: Every gate averages at most ten minutes — directional

Each gate's mean duration over the tracking window of [§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized) is at most the threshold [§AR-ci.requirements.4](AR-ci.md#requirements4-the-threshold-and-the-window-are-configuration--realized) configures, ten minutes unless it says otherwise. Directional: when this was written the three test legs averaged about 43, 42 and 25 minutes, so it binds every change from here on not to add to a gate's time without paying for it elsewhere, and it becomes realized once every gate is under it. Serves [§GOAL-fast-feedback.3](../goals.md#3-measurable).

### requirements.2: No run of a gate passes twenty minutes — deferred

Every job of `ci.yml` carries `timeout-minutes: 20`, so a run past it fails like any other red check rather than holding a pull request for an hour. This bounds the job, and the self-check's own `timeout` of [§AR-ci.4](AR-ci.md#4-performance-smoke-guard) stays inside it as the catastrophic backstop for that one step; the two do not replace each other. It binds a dispatched release candidate as it binds a push, because a candidate runs the same `ci.yml` ([§AR-ci.11](AR-ci.md#11-a-release-candidate-is-dispatched-the-same-ci)). Deferred: a twenty-minute ceiling today would fail every pull request, so it lands after the Windows Python gate (agent-grounds/grund#585) and the Ubuntu `pre-commit` step's link check come under it, and the change that adds the timeouts flips this mark.

### requirements.3: The average is measured, and drift is reported — realized

The average is measured, not assumed. `scripts/ci_duration.py` reads the last *window* successful runs of `ci.yml` triggered by a push to `main`, newest first, and averages each gate's duration — a job's `completed_at` less its `started_at` — over the runs it appears in; a skipped job is not a sample. It averages each step of each gate the same way, which is where the time went. When any gate's average exceeds the threshold, it writes one report naming each such gate, its average, the window, and that gate's slowest steps, and puts it on one open issue: it opens the issue when no open issue's body begins with the marker line `<!-- ci-duration-drift -->`, and edits the body of the one that does instead of opening a second; an issue that only quotes the marker further down is not the drift issue. A query that returns no run measures nothing, so the tracker says so and exits 1 rather than reporting every gate under the threshold. A run in which every gate is under the threshold writes nothing; closing the issue is a person's call. The workflow `.github/workflows/ci-duration.yml` runs it on a schedule and on `workflow_dispatch`, never on a push or a pull request, with `actions: read` and `issues: write`. It stays out of the pre-commit hook list and out of `grund check`: it reads the forge, which [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service) keeps out of any verdict and [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s parity keeps out of the local gate. Its averaging, its report and its open-or-edit decision are functions of the forge's JSON, so `tests/integration/test_ci_duration.py` holds them to this offline, against canned runs. This is requirements.1's meter in the sense of [§AR-goal-measurement](AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals).

### requirements.4: The threshold and the window are configuration — realized

The threshold and the window live in `.github/ci-duration.json`, which the tracker reads on every run and in which it finds no literal it would otherwise fall back to: `threshold_minutes`, the average a gate may not exceed (`10`), and `window`, how many runs it averages over (`8`), a whole number from 1 to 100 because the forge lists at most 100 runs a page. A missing file or a missing key is an error naming it, never a silent default. They are repository CI configuration and not a `grund.toml` key: `grund` never reads them, and a key it never reads would cross [§FS-config.requirements.8](../functional-spec/FS-config.md#requirements8-a-projects-meaning-is-self-contained--realized). The maximum of requirements.2 is not here, because the forge reads it from `ci.yml` and from nowhere else.

### requirements.5: A green step prints a screenful, a red one its failures — realized

What a CI step prints is part of how CI is built. Whoever reads a red job, a person or an agent running `gh run view --log-failed`, reads every line before the failure, so a line that says only that something passed is paid for on every pull request and every push to `main`: the cost [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) says a read should not carry, on the loop [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) keeps short. Realized: before it, the workspace `cargo test` step printed about 2200 lines on Ubuntu, 1640 of them `test … ok`, the `test-watch` contract test about 270, and `pip install pre-commit` 442; every step below now prints what requirements.5.1–5.4 ask, and a step added from here on is held to the same.

#### requirements.5.1: A suite prints its failures, its check-ins and one summary

A suite step is the workspace `cargo test` on all three legs, which runs the e2e runner `crates/grund-cli/tests/e2e.rs` among its targets, the `test-watch` contract test of the `native watch contract` step, and the Python gate of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs). It prints only these, and exits with the suite's own status:

- each failure, under the failing test's name, with its full output: what it wrote to stdout and stderr, its own and its child processes', and the panic or assertion message;
- a check-in each time another 5% of the suite has finished, so at most twenty, one line each: `<done>/<total> tests, <failed> failed, <elapsed>s elapsed, slowest: <test> <seconds>s, …`, naming up to three of the slowest tests so far, slowest first, and the last check-in reads `<total>/<total>`;
- the slowest-ten table of [§AR-ci.3.4.2](AR-ci.md#342-per-test-durations), where the suite keeps one;
- one summary as its last line: `<total> tests: <passed> passed, <failed> failed, <ignored> ignored, <elapsed>s`, where *ignored* counts libtest's ignored tests and `unittest`'s skipped ones. A red summary still adds up: before the elapsed time it adds `, <n> not run` for the tests that never reported, and `, <n> errors` for each failure no test owns — a test binary that crashed or a `harness = false` target that failed, a `unittest` class or module fixture that raised — which the check-ins count among the failures.

Nothing else: no line per passing test, no `unittest` dot stream, no output a passing test or its child process wrote, no build progress. On success that is about twenty lines whatever the suite's size, and the slowest-tests figure is what attributes a gate's time per test for requirements.3. `tests/integration/test_ci_output.py` holds both kinds of suite to this on suites of its own.

#### requirements.5.2: The cargo suites go through one wrapper

`scripts/run_cargo_suite.py` takes `cargo test`'s arguments, runs `cargo test` with them over libtest's stable output, and prints what requirements.5.1 asks. Both `ci.yml` steps that run a cargo suite, `native watch contract` and `cargo test`, call it, and so does the `cargo-test` hook, so the hook and the workflow still spell one command ([§AR-ci.1.2](AR-ci.md#12-the-parity-test)). It needs Python's standard library and the toolchain the repository already uses and nothing else: no test runner a contributor must install. It builds the tests before it runs them, as quietly as requirements.5.4 asks, so a compile error is printed in full and a green build prints nothing.

#### requirements.5.3: A hook prints its findings or one line — realized

A pre-commit hook prints its findings when it fails and one line when it passes. Realized: `pre-commit` captures each hook's output, prints it when the hook fails, and prints one `Passed` line otherwise.

#### requirements.5.4: Builds and installs are quiet unless they fail

Every `cargo build`, `cargo run`, `cargo install`, `cargo bench` and `pip install` in `ci.yml` passes `--quiet` (`-q` for pip), the binding oracle's `cargo +1.95.0 build` in `scripts/prepare_binding_oracle.py` included, and so does the `cargo-build` hook, so the hook and the workflow still spell one command. `--quiet` drops the progress lines — `Compiling`, `Downloading`, `Collecting` — and keeps every warning and error, so a failure still shows in full in the step that failed; cargo does not pass it on to a `harness = false` bench, so the benchmark job still prints its report. A suite's own builds and the `git` its fixtures run are held to the same: a green Python gate prints no `Compiling` line and no `LF will be replaced by CRLF` warning.
