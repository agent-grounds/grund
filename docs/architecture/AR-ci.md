# AR-ci: CI mirrors the local pre-commit gate

The CI workflow is the remote form of the local pre-commit gate. Anything that can abort a local commit must also abort CI, so a contributor cannot bypass the repository's local checks by skipping hooks or editing through a web UI. This supports [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) and keeps the link-checking boundary from [§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation) enforced alongside `grund check`.

## placement: What CI measures

```text
pre-commit hook list ─► [ CI ] ─► one verdict per push or pull request
                         │
                         └─ runs the tree's own grund through cargo run; outside the pipeline
```

Not a component: CI is the remote form of the local gate over the whole tree, and it measures the system rather than sits in it ([§AR-system.5](README.md#5-what-holds-the-shape)). It takes the pre-commit hook list and gives one verdict per push or pull request. The only `grund` it runs is the one the tree builds, through `cargo run`, so the gate never depends on a released binary; it knows nothing of the pipeline ([§AR-system.1](README.md#1-the-system)) beyond the commands the hooks spell (section 1).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (ID, home, citable, section, lead),
[§FS-terms.terms.2](../functional-spec/FS-terms.md#terms2-citations) (citation, citation site), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan),
[§FS-terms.terms.5](../functional-spec/FS-terms.md#terms5-findings) (verdict), and [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

## 1. Pre-commit is the source of truth

The hook list lives in `.pre-commit-config.yaml`. CI must invoke that list directly with `pre-commit run --all-files`, rather than hand-copying each hook into separate workflow steps. The workflow may install hook prerequisites first, but the set of checks is defined by the pre-commit config.

When a new pre-commit hook is added, the same change must ensure CI can run it. If the hook needs an external binary, the CI workflow installs that binary before the pre-commit step. If the hook is intentionally local-only, it does not belong in `.pre-commit-config.yaml`; put it in a developer-local hook instead.

### 1.1 Stages without a file list

`pre-commit run --all-files` reproduces only the stages whose input is a file list. A hook bound to a stage with different input — `commit-msg`, whose argument is a message file, or `pre-push`, whose arguments are the refs being pushed — is silently absent from that run, so "CI runs the config" is not by itself parity. Every hook bound *only* to such a stage needs an explicit CI counterpart that feeds it the input the stage would have supplied **and reaches the same verdict on the same tree**: [§AR-ci.8](AR-ci.md#8-commit-message-attribution-gate) is the `commit-msg` one, and no hook is bound only to `pre-push` any more ([§AR-ci.7](AR-ci.md#7-no-changelog-gate)). A hook that also runs at `pre-commit` — `cargo-test`, the one hook `pre-push` still runs — is reproduced by the all-files run and is not implicated. The verdict half is the half that bites: a counterpart fed different input is worse than a missing one, because it passes locally and then fails remotely on a tree the contributor cannot change.

### 1.2 The parity test

All of this is checked rather than remembered: `tests/integration/test_ci_precommit_parity.py` reads the hook list, workflow and Python wrapper and fails when CI stops invoking the hook list, a hook's binary has no pinned install step before that gate, a `commit-msg` hook has no range-scanning counterpart, the Rust hooks and the workflow's own steps spell different commands, [§AR-ci.3](AR-ci.md#3-current-hooks) does not spell a command a Rust hook runs, warnings are denied on one side only, the Python gates invoke different wrappers or discover tests from the wrong directory, or a hook bound only to a stage without a file list has no CI step naming the same `scripts/` script. What it holds is the *existence* of a counterpart, which is line-shaped; the *same verdict* half of [§AR-ci.1.1](AR-ci.md#11-stages-without-a-file-list) is held by each gate's own test, which runs both invocations against one fixture tree and asserts they agree — for the attribution gate, `tests/integration/test_check_no_claude_attribution.py`.

## 2. Platform scope

The full Rust build and test matrix still runs on every configured operating system. The pre-commit gate may run on one representative CI platform when the hooks are platform-independent, because its job is policy parity with local commits, not cross-platform behavior coverage. Platform-specific behavior belongs in the build and test jobs.

CI dependency and build caches are performance optimizations only. A cache restore/save failure must not abort the matrix before the actual checks run; the job should continue cold and let `fmt`, Python tests, pre-commit, build, self-check, tests, or benchmarks decide pass/fail.

## 3. Current hooks

The current pre-commit gate runs the same Rust format/build/test commands that development CI runs: `cargo fmt --all -- --check`, `cargo build --workspace --all-targets --locked` with warnings denied, and `cargo test --workspace --all-targets --locked --features grund/test-workspace-load-count`. A plain `cargo test --workspace --all-targets --locked` is green on the same tree but runs less: it reports every test that reads the workspace-load observer as `ignored`, naming the feature that test needs ([§AR-ci.3.3](AR-ci.md#33-test-only-observers)). The test hook also runs at `pre-push`, so a contributor who commits while a test is transiently broken still gets the same local stop before sending the branch. It is the one hook `pre-push` runs: no hook asks a change for a changelog entry at any stage, because no change is gated on the changelog ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)).

Both Python gates run `python scripts/run_python_gate.py`. The wrapper prepares
the built binary and released Grund 0.16.1 for the co-change compatibility cases
([§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)) and the same-source binding oracle for the
Node parity tests ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)), the three inputs of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs), then runs
`python -m unittest discover -s tests/integration -p test_*.py` with that environment
from the repository root. The parity test checks the shared wrapper invocation
and its discovery directory and pattern.

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

The wrapper hands its discovery run three inputs, each an absolute path in the
environment, and treats the three alike: a variable already set is used as
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

## 4. Performance smoke guard

CI carries a cheap floor on [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) that does not depend on any benchmarking toolchain: the `grund check . --full` self-check step runs the already-built binary under a generous wall-clock `timeout` — long enough never to flake on a loaded runner, short enough to fail the build on a catastrophic regression such as an accidental quadratic walk or a second read pass over every file. It is not the budget itself (the budget is tens of milliseconds; the ceiling is tens of seconds) — it is the difference between "we'd notice eventually" and "the build is red on the commit that did it". The precise per-commit meter is the [§AR-ci.5](AR-ci.md#5-benchmark-job) benchmark job, and this timeout stays as its catastrophic backstop.

## 5. Benchmark job

A separate `bench` job runs the instruction-counting harness ([§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands)). Because the harness runs `grund` under Callgrind, the job first installs `valgrind` and `cargo install`s `iai-callgrind-runner` pinned to the same version as the `iai-callgrind` dev-dependency. It runs on one representative platform (the harness measures instruction counts, not cross-platform behavior — [§AR-ci.2](AR-ci.md#2-platform-scope)'s reasoning applies) and on every push and pull request, so the numbers are recorded per commit.

### 5.1 Pull requests and pushes

On pull requests, the job checks out the PR base branch and runs `cargo bench -p grund --features bench --locked --bench instructions -- --save-baseline=main --save-summary=json`; then it checks out the PR commit and runs `cargo bench -p grund --features bench --locked --bench instructions -- --baseline=main --save-summary=json`. The named baseline comes from the current base branch rather than a stale generated artifact, while the committed baseline figures in `docs/benchmarks.md` remain the human-readable reference point. On pushes to `main`, the job records the current counts and saves JSON summaries without comparing against itself.

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

The job scans what the event contributes, not the whole history — a pull request's `base..head`, a push's `before..sha`. An unresolvable base is not a failure: the first push of a branch reports the all-zero SHA and a force-push reports a SHA the runner cannot reach, so the script narrows to the tip commit and says so. A gate on messages catches an offending commit only while it is still cheap to amend; once merged, the same trailer costs a history rewrite and a force-push of every descendant branch and tag, which is why this runs on the pull request rather than on the release.

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

Beside push and pull request, the workflow runs on `workflow_dispatch`, and that trigger is the release helpers': each dispatches it on its release commit and on the `-dev` advance built over it, and pushes neither to `main` until both have passed ([§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). A dispatched run is the same CI a push to `main` gets, not a narrower one, because the commit it judges is about to become `main`. It behaves as a push does: the benchmark job records counts without comparing ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and the commit-message job, given no `before` to scan from, narrows to the tip commit ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)). `tests/integration/test_release_candidate_runs.py` holds the trigger to the helpers that use it.
