# AR-benchmarks: instruction-counting benchmarks for the hot CLI commands

How `grund` turns the [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) targets into a number CI records on every commit. The goal-meter map points here from [§AR-goal-measurement.2](AR-goal-measurement.md#2-goal-meters); this spec pins what is measured, with what tool, and why the meter counts CPU instructions rather than wall-clock seconds. The synthetic large fixture, the committed baseline and the CI benchmark job shipped in 0.4.1 (`docs/changelog/0.4.1.md`); the job's regression limits fail the build ([§AR-ci.5.2](AR-ci.md#52-regression-limits)); the decision that picks instruction counting over a wall-clock harness is [§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds).

## placement: What the benchmarks measure

```text
generated fixture, base-branch baseline ─► [ benchmarks ] ─► instruction count per hot command ─► CI job (AR-ci.5)
                                              │
                                              └─ runs the built grund binary as a subprocess: the CLI, end to end
```

Not a component: a meter on the pipeline ([§AR-system.1](README.md#1-the-system)), taken through the CLI frontend so that it counts what a user's invocation counts ([§AR-system.3](README.md#3-frontends), [§AR-system.5](README.md#5-what-holds-the-shape)). It takes a generated fixture and a baseline saved from the base branch ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)) and gives one instruction count per hot command, recorded by the CI job of [§AR-ci.5](AR-ci.md#5-benchmark-job). It knows nothing of the engine's internals: each benchmark runs the built `grund` binary as a subprocess (section 1).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, lead),
[§FS-terms.terms.2](../functional-spec/FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan), [§FS-terms.terms.6](../functional-spec/FS-terms.md#terms6-rules-and-directions)
(direction), and [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

## 1. What is benched

A `cargo bench` harness at `crates/grund-cli/benches/instructions.rs`, gated behind the `grund` package's `bench` Cargo feature so a plain `cargo test --all-targets` compiles only a no-op bench target and never tries to run the Callgrind body (it needs Valgrind). Each benchmark runs the **freshly built `grund` binary** as a subprocess under Callgrind — not a library call — because the figure we care about is the cost of an invocation an agent or a CI step actually makes, including process start-up, argument parsing, the walk, and output formatting.

### 1.1 Generated fixtures, never this repository

Every benchmark scans a **generated conformant fixture**, never this repository's own tree. Benchmarking the live repo conflates the two things the regression gate must keep apart: a genuine code slowdown, and the repo simply doing more work because its own docs or `grund.toml` evolved (the case in point: enabling `[citations]` direction checks on itself makes `grund check` validate more, which is not a code regression). A fixed, generated input isolates the first — the only thing the gate should fail on — so the baseline is stable across commits regardless of how the repo's content evolves.

`scripts/generate_large_benchmark_fixture.py` regenerates every fixture under `target/bench-fixtures/` before each benchmark command is built, so a cached `target/` cannot hide generator changes:

- a **canonical** 1.5k-file fixture for the per-command suite — representative, but small enough to keep the suite quick;
- a **10k-file** fixture for the large-repo budget benchmark, backing the large-repo promise ([§GOAL-small-and-large.2](../goals.md#2-large-repo-promise)) and fast-feedback target ([§GOAL-fast-feedback.1](../goals.md#1-performance-targets));
- a **citations-enabled** fixture (`--citations`) so one benchmark exercises the citing-side classification and direction passes ([§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)), which the other fixtures (no `[citations]`) deliberately skip.

Every generated fixture carries a `manifest.json` at its root, outside the scanned `docs/` tree, holding the generator's `schema_version` and a SHA-256 `digest` over the fixture's sorted relative paths and their contents, the manifest itself excluded. Equal generator arguments give an equal digest, and one changed byte gives another. The schema version changes whenever the generator's output changes shape. Base and head are measured on one fixture bundle, so their digests must match; the run record carries them ([§AR-benchmarks.7.1](AR-benchmarks.md#71-what-a-record-holds)), and a changed digest starts a new series rather than reading as a code change.

### 1.2 The benched commands

The benched subcommands are the ones the agent loop and the CI/pre-commit gate run most. They are the rows of one declarative **scenario catalog**, `crates/grund-cli/benches/scenarios.json`, and the harness builds its benchmark group from that file rather than from a hand-written list of `#[binary_benchmark]` functions. Python 3.6 reads JSON with no new dependency, which is why the catalog is JSON (section 1.5). The catalog has two parts:

- `fixtures` names each generated fixture of [§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository) by its directory under the repository root, its file count, whether it declares `[citations]`, and, for the canonical fixture, the `subject` ID its single-ID scenarios read (`FS-00001-feature-00001`).
- `scenarios` is the ordered list of rows. Each row gives:
  - `name`: the benchmark's id. iai-callgrind and the run record of [§AR-benchmarks.7.1](AR-benchmarks.md#71-what-a-record-holds) both key on it, so a row keeps its name for as long as it exists;
  - `args`: the arguments before the fixture path, with `{id}` standing for the subject;
  - `fixture`: a key of `fixtures`. The fixture's path is the last argument;
  - `setup`: optional arguments the harness runs once outside Callgrind, on the binary under test and on the same fixture, before it measures the row (section 1.3);
  - `limit_pct`: the relative `Ir` limit of [§AR-ci.5.2](AR-ci.md#52-regression-limits);
  - `ceiling_ir` and `ceiling_from`: the absolute `Ir` ceiling, and the CI count it was set from ([§AR-ci.5.2](AR-ci.md#52-regression-limits));
  - `pgo`: `false`, or the row's place in PGO training (section 1.5).

Adding a row adds a scenario, and the harness needs no other change. Each row is measured with exactly the invocation the table below gives for it. The table is the catalog's rendering, row for row, and `tests/integration/test_bench_scenarios.py` holds the two to each other:

| Benchmark | Invocation | Who runs it, how often |
|---|---|---|
| `check` | `grund check <fixture>` | Every save (LSP), every commit (pre-commit), every push (CI). The headline operation [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) exists for. |
| `check_large_10k` | `grund check <10k-file fixture>` | The large-repo budget input from [§GOAL-fast-feedback.1](../goals.md#1-performance-targets). |
| `check_citations` | `grund check <citations-enabled fixture>` | The citation-direction code path ([§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)); its delta against `check` (same file count) is the direction-checking overhead. |
| `list` | `grund list <fixture>` | An agent discovering the right `<ID>` before it cites one. |
| `show_brief` | `grund show <ID> --brief <fixture>` | An agent or LSP hover checking the title plus first paragraph. |
| `show` | `grund show <ID> <fixture>` | An agent grounding itself on a bare citation — the lead-default read AGENTS.md / CLAUDE.md tell it to do first, before editing the code that realizes a cited ID. |
| `show_full` | `grund show <ID> --full <fixture>` | An agent escalating to the full declaration body when the narrower slices are not enough. |
| `refs` | `grund refs <ID> <fixture>` | An agent checking a declaration's blast radius before changing or moving it. |
| `cover` | `grund cover <fixture>` | What the diff-aware co-change recipe ([§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits)) and CI consume. |
| `fmt_check` | `grund fmt --check <fixture>` | The pre-commit / CI normalization gate. |

### 1.3 `fmt --check` measures a canonical fixture

The `fmt --check` benchmark setup first runs `fmt --write` on its generated
fixture outside Callgrind. This keeps the measured input canonical under the
revision being benchmarked, so the command exits successfully while its
instruction count still covers the complete dry-run scan.

### 1.4 Two cost shapes, and a clean exit

`check`, `check_large_10k`, `check_citations`, `list`, `cover`, and `fmt --check` walk a whole scanned tree; `show --brief`, `show`, `show --full`, and `refs` do the same walk plus a per-ID extraction or filter — so the set spans the engine's two cost shapes (full report vs. single-ID query), and the show ladder spans the short hover preview, the cheap lead default, and the full recursive body over realistic inputs.

Each command exits 0 on its conformant input (which keeps iai-callgrind's default `ExitWith::Success` happy); on a broken tree `check` / `fmt --check` exit non-zero and the bench fails — intentionally, since a baseline recorded against a broken tree is worthless.

### 1.5 The same command list trains PGO

The hot command list is also the training run for the profile-guided-optimization build of the distributed release binary (`scripts/pgo-build.sh`, [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)): the everyday operations we benchmark are the ones we profile against, so "the operations that matter" has one definition. The PGO run trains on this repository's own tree (a representative real workload, where profiling input *should* reflect reality), while the benchmarks run the same commands against the generated fixtures of [§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository) (where the gate needs a *stable* input).

The command list is defined once, as the catalog of [§AR-benchmarks.1.2](AR-benchmarks.md#12-the-benched-commands). The rows whose `pgo` is not `false` are its **`pgo` subset**. The subset, taken in catalog order, is the `grund` executable's training list. Each row in it gives an `id` that replaces `{id}`, because training reads this repository's own declarations (`FS-check`, `GOAL-fast-feedback`) where the benchmark reads the fixture's. It also gives an `api` label, which names the native call that runs the operation for the Node addon and the Python extension. `scripts/distribution/train.py` is the one PGO reader of the catalog. It reads the catalog with nothing but the standard library. `describe grund` prints the training list as `"grund <args>"` strings, `describe node-addon` and `describe python-extension` print the `api` labels, and both training runs drive the subset rather than a list of their own. `scripts/pgo-build.sh` hands every product, `grund` included, to `train.py`, so no copy of the list remains for a comment to keep in sync.

The training evidence a release records ([§FS-distribution-candidate.7.3](../functional-spec/FS-distribution-candidate.md#73-the-packaged-payload-is-the-profile-use-build)) is `describe`'s output. It stays byte for byte what the hand-written loop produced: `["grund check", "grund list", "grund show FS-check --brief", "grund show FS-check", "grund show FS-check --full", "grund refs GOAL-fast-feedback", "grund cover", "grund fmt --check"]`. If it changed, `training_sha256` and the profile key would move with no change to the training. `check_large_10k` and `check_citations` are deliberately excluded from PGO training: the first is a CI budget input, and the second is a fixture configuration, so neither is a representative release invocation. The list trains the `grund` executable alone. The language server trains on a workload of its own, and the Node addon and the Python extension train on the subset's operations through their APIs ([§FS-distribution-candidate.7.1](../functional-spec/FS-distribution-candidate.md#71-each-product-trains-on-its-own-workload)), so the list stays the one definition of the hot operations without becoming another product's profile. PGO stays out of the development build and push/PR CI loop ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)): the release pipeline runs it, and so does explicit benchmark work comparing the optimized release artifact.

## 2. Why instruction counts, not wall-clock

Callgrind reports the number of CPU instructions executed (plus cache-miss estimates), and that number is **deterministic** for a given binary and input — it does not move when the CI runner is loaded, when a neighbour VM is noisy, or when the kernel reschedules. A wall-clock harness (criterion and friends) measures the thing the [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) budget is literally written in (milliseconds), but its variance on shared CI is large enough that "fail on regression" either flakes (tight threshold) or misses real regressions (loose threshold). Instruction count is the proxy that makes the [§AR-ci.5](AR-ci.md#5-benchmark-job) regression gate implementable: a quadratic walk or an accidental second read pass moves the instruction count by a margin no scheduling noise can mask. The full rationale, and the alternatives weighed, is [§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds).

### 2.1 The wall-clock guard backstops the budget

The millisecond budget itself is still backstopped by the cheap wall-clock guard in [§AR-ci.4](AR-ci.md#4-performance-smoke-guard) (the built `grund check . --full` self-check under a generous `timeout`); the instruction-count harness is the precise regression meter, the timeout is the catastrophic-blowup floor. The two are complementary, not redundant.

## 3. How it runs

- Locally: `cargo bench -p grund --features bench --bench instructions` (requires `valgrind`, `python3`, and `iai-callgrind-runner` on `PATH`). With no prior run, iai-callgrind records the counts and exits clean; on a later run in the same `target/` it prints the delta against the previous run. Every fixture is regenerated on each run ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)). `cargo bench` writes no run record; `python3 scripts/bench_record.py record --out target/bench-record.json`, run after it, does. That record says `environment: local`, a series of its own ([§AR-benchmarks.7.4](AR-benchmarks.md#74-a-local-run-is-a-series-of-its-own)).
- In CI: the dedicated `bench` job of [§AR-ci.5](AR-ci.md#5-benchmark-job) runs the harness in a pinned environment. A pull request measures the base binary and its own under its own harness ([§AR-benchmarks.5](AR-benchmarks.md#5-comparing-two-revisions), [§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and growth past the limits of [§AR-ci.5.2](AR-ci.md#52-regression-limits) fails the job. A push to `main` runs the suite twice on one binary and checks the same-binary bound ([§AR-benchmarks.7.3](AR-benchmarks.md#73-the-same-binary-bound)).
- Every run is written down as one run record by `bench_record.py record`, which the CI job runs after each `cargo bench` and a local run runs by hand, and two records are compared only within one series ([§AR-benchmarks.7](AR-benchmarks.md#7-the-run-record-and-its-series)).

## 4. Relationship to the rest of the tree

- [§AR-goal-measurement.2](AR-goal-measurement.md#2-goal-meters) — this harness is the fast-feedback meter; [§GOAL-fast-feedback.1](../goals.md#1-performance-targets)'s targets are recorded per commit and, on a pull request, compared against a baseline saved from the base branch ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)); growth past the limits fails the build ([§AR-ci.5.2](AR-ci.md#52-regression-limits)).
- [§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds) — the decision record for choosing Callgrind/iai-callgrind over a wall-clock criterion harness ([§AR-benchmarks.2](AR-benchmarks.md#2-why-instruction-counts-not-wall-clock)), and for benchmarking the binary rather than the library ([§AR-benchmarks.1](AR-benchmarks.md#1-what-is-benched)).
- [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms) / [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine) — the harness is deliberately established before the workspace split and bindings work move the engine around, so any slowdown shows up as a diff against a known-good instruction count rather than going unnoticed.
- [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload) — the PGO training corpus of [§AR-benchmarks.1.5](AR-benchmarks.md#15-the-same-command-list-trains-pgo); the release pipeline that runs `scripts/pgo-build.sh` is release packaging ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)), whose contract is [§FS-distribution.4](../functional-spec/FS-distribution.md#4-release-process).

## 5. Comparing two revisions

The counts are deterministic for a given **binary**, not for a given source tree, and two checkouts have their own `target/` directories — one of which may hold a differently-built binary. A PGO build from `scripts/pgo-build.sh` ([§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)) is the case to watch, because it is trained on this very workload and reads several percent faster on it.

So benching a branch in one checkout against `main` in another measures the build as much as the change, and it fails by producing a plausible number rather than an error. Take both numbers in **one worktree**, switching the source under a single `target/`, and against the branch's **merge base** rather than whatever `main` has since become.

Two revisions are compared by **one harness on one fixture bundle**, and only the binaries differ. The harness takes the binary under test from `GRUND_BENCH_BIN` when it is set, and its own freshly built `grund` otherwise. Measuring the base revision means building the base's `grund` binary alone, then running the newer revision's harness with `GRUND_BENCH_BIN` naming it and `--save-baseline`, and the same harness on its own binary with `--baseline`. The base revision's harness and fixture generator are never run, so a change to either cannot read as a change in the code.

### 5.1 A scenario the base binary refuses

A scenario the base binary cannot run — it names a flag or a mode the base does not know — is not compared, and it does not fail the comparison. Before measuring a scenario under `GRUND_BENCH_BIN`, the harness runs its invocation once outside Callgrind; when that exits non-zero, the harness leaves the scenario out of the base run and the base's run record lists it as refused. A scenario's setup that runs the binary, such as `fmt_check`'s `fmt --write`, is part of that probe: a base binary that fails the setup refuses the scenario rather than stopping the harness. Under the harness's own binary a failed setup still fails the run. The head run still measures it. The comparison reports it as `no base, not compared`. Only a scenario both binaries ran can fail its relative limit. Every scenario the head measures is still held to its ceiling ([§AR-benchmarks.5.2](AR-benchmarks.md#52-a-scenario-with-no-base)).

### 5.2 A scenario with no base

A scenario with no stored count to compare against does not fail for that. This happens when a pull request adds a row that the base binary refuses (section 5.1), or on the first local run in a fresh `target/`. A row a pull request adds that the base binary accepts is measured on both binaries by the pull request's own harness, so it is compared from that same pull request on. A run with no base records the scenario's count, which becomes the baseline the next comparison reads, and from then on the relative limit gates it. Its absolute ceiling ([§AR-ci.5.2](AR-ci.md#52-regression-limits)) does not need a base, so it holds from the scenario's first CI run.

iai-callgrind 0.16.1 already behaves this way: a soft limit is checked only when both counts exist, and a hard limit is checked on the new count alone. So `--baseline=main` with no stored `main` count prints the count against `N/A` and exits 0, and a count over a hard limit exits 3, which fails `cargo bench`. The harness and the bench job keep that behaviour and add no step that treats a missing base as an error.

## 6. Scope

This is a build-and-CI tool, not a CLI surface: it adds no subcommand, no flag, no output shape, so the `tests/e2e/cases/*` corpus is unchanged and nothing here is covered by the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) user-visible-surface contract. The benched commands are specified by their functional specs ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-citation-in-a-repo), [§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](../functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](../functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](../functional-spec/FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk), [§FS-cli](../functional-spec/FS-cli.md#fs-cli-grunds-command-line-surface-conventions)) rather than re-specified here.

## 7. The run record and its series

Every run of the harness is normalized into one compact JSON **run record** by `python3 scripts/bench_record.py record`, run after `cargo bench` (the CI job runs it after each run, and a local run runs it by hand), and two records are compared only when they belong to one **series**. The record is what makes a count readable apart from the job that took it: it says what was measured, with what, and on which input.

### 7.1 What a record holds

A record holds the `commit` and the SHA-256 of the binary measured (`binary_sha256`), the `ir` count per scenario that ran, and the scenarios the binary `refused` ([§AR-benchmarks.5.1](AR-benchmarks.md#51-a-scenario-the-base-binary-refuses)). Beside them it holds its identity fields:

- `record_version`, the version of this record format;
- `environment`, `ci` or `local`;
- `image`, the container image's digest, or `local`;
- `rustc`, `valgrind` and `runner`, the versions of the compiler, of Valgrind and of `iai-callgrind-runner`;
- `build_flags`, the `RUSTFLAGS`, `CARGO_INCREMENTAL` and Cargo profile the binary was built with;
- `locale` and `timezone`, the `LC_ALL` and `TZ` of the run;
- `threads`, the `RAYON_NUM_THREADS` of the run;
- `fixtures`, each fixture's `schema_version` and `digest` ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)).

A record missing an identity field is refused when it is written.

### 7.2 The series key

The series key is the SHA-256 of the identity fields, and of nothing else: the commit and the binary digest are what a comparison compares, so they never change the key, while a change to any identity field does. `bench_record.py compare` refuses two records whose keys differ, exits non-zero, and names each identity field that differs, so a fixture, toolchain or environment change cannot be read as a code change. The first record after such a change starts a new series; records before it are never compared with it.

### 7.3 The same-binary bound

The same-binary bound is **1%** `Ir`: two runs of one binary on one fixture bundle, in one series, give each scenario counts that differ by at most that much. A push to `main` checks it ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)) with `bench_record.py jitter`, which fails naming each scenario past the bound. If that check ever fails on an unchanged binary, it is reported as a flaky test, with the labels `bug` and `flaky-test`. Every regression limit stays at least five times this bound ([§AR-ci.5.2](AR-ci.md#52-regression-limits)).

### 7.4 A local run is a series of its own

A local `cargo bench` runs outside the pinned image, so the record `bench_record.py record` writes after it says `environment: local` and `image: local`, and it is a series of its own by construction. A run of the pinned image on another machine records `local` too: the host's processor is not in the series key, so its counts must not join CI's series. That is the honest reading of the gap between the same commit's counts locally and in CI. `docs/benchmarks.md` gives the one `docker run` command that reproduces CI's environment.
