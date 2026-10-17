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

The benched subcommands are the ones the agent loop and the CI/pre-commit gate run most. They are the rows of one declarative **scenario catalog**, `crates/grund-cli/benches/scenarios.json`, and the harness builds its benchmark group from that file rather than from a hand-written list of `#[binary_benchmark]` functions. Python 3.6 reads JSON with no new dependency, which is why the catalog is JSON (section 1.5). The catalog has three parts:

- `shards` is the number of shards the pull-request tier is split into (section 1.2.2).
- `fixtures` names each generated fixture of [§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository) by its directory under the repository root, its file count, whether it declares `[citations]`, and, where its rows read one, the `subject` ID its single-ID scenarios read (`FS-00001-feature-00001` for the canonical fixture). A fixture that exercises a feature also names it as `feature`, one of the generator's `--feature` values (`rules`, `named-sections`, `values`, `external-facts`, `cross-refs`), and a workspace names its member count as `members`, which splits the file count among that many member projects that cite into each other.
- `scenarios` is the ordered list of rows. Each row gives:
  - `name`: the benchmark's id. iai-callgrind and the run record of [§AR-benchmarks.7.1](AR-benchmarks.md#71-what-a-record-holds) both key on it, so a row keeps its name for as long as it exists;
  - `args`: the arguments before the fixture path, with `{id}` standing for the subject. A `frontend: lsp` row's `args` is `["session"]`, the editor session of section 1.7, and a `frontend: python` row's `args` name the API call the way a `grund` row names its command, `["show", "{id}", "--full"]` for `show(id, mode="full")`;
  - `fixture`: a key of `fixtures`. The fixture's path is the last argument;
  - `path`: optional, a file under the fixture named instead of the fixture's root, for a command that reads one file, as `cover --lines` does;
  - `stdin`: optional, the lines the harness feeds the row on stdin, with `{id}` standing for the subject, for a command that reads its queries there, as `show --batch` does. The harness writes them to a file under `target/` outside Callgrind;
  - `setup`: optional arguments the harness runs once outside Callgrind, on the binary under test and on the same fixture, before it measures the row (section 1.3);
  - `limit_pct`: the relative `Ir` limit of [§AR-ci.5.2](AR-ci.md#52-regression-limits);
  - `ceiling_ir` and `ceiling_from`: the absolute `Ir` ceiling, and the CI count it was set from ([§AR-ci.5.2](AR-ci.md#52-regression-limits));
  - `pgo`: `false`, or the row's place in PGO training (section 1.5). A row added after the ten below says `false`;
  - `tier`: `pr` or `nightly`, the tier that runs it (section 1.2.1);
  - `group`: on a `nightly` row, the feature group section 1.2.1 renders it under;
  - `frontend`: absent for a row that runs `grund`, or `lsp` or `python` for a row that measures a frontend (section 1.7).

A fixture is generated by the one generator of [§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository), and an entry may carry options the generator takes for the feature it exercises: workspace members, committed rule declarations, named sections, values, a fetched snapshot. Each feature #699 names is isolated in at least one row, and only the scale-sensitive paths (the scan, `check`, `list`) are measured at several sizes; no feature is crossed with every size.

Adding a row adds a scenario, and the harness needs no other change. Each `pr` row is measured with exactly the invocation the table below gives for it. The table renders the `pr` tier, row for row in catalog order, and `tests/integration/test_bench_scenarios.py` holds the two to each other; a frontend row's invocation names its frontend's process rather than `grund`:

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
| `check_seq_255` | `grund check <255-file fixture>` | The sequential scan, one file under the parallel threshold (section 1.8); every small repository takes this path. |
| `check_par_256` | `grund check <256-file fixture>` | The parallel scan at exactly the threshold (section 1.8); its delta against `check_seq_255` is the cost of going parallel. |
| `check_json` | `grund check --format json <fixture>` | CI and editors reading findings as JSON. |
| `check_full` | `grund check --full <fixture>` | CI's self-check, which walks past `[scan] include` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)). |
| `check_require_grounding` | `grund check --require-grounding <fixture>` | A repository that requires every source file to cite a declared ID. |
| `check_rule` | `grund check --rule Each FS should have at most 1 Terms chapter. <fixture>` | An author trying a controlled-English rule before committing it. |
| `check_suggestions` | `grund check --suggestions <citations-enabled fixture>` | An author asking for the `should` findings as well. |
| `check_workspace` | `grund check <5-member workspace>` | A workspace root checking its member projects together (`workspace` mode). |
| `check_rules` | `grund check <rules fixture>` | Committed rule declarations evaluated on every check (`rules` mode). |
| `check_named_sections` | `grund check <named-sections fixture>` | Named section paths and the section lock (`named-sections` mode). |
| `check_values` | `grund check <values fixture>` | Value bindings compared against their declared values (`values` mode). |
| `check_external_facts` | `grund check <external-facts fixture>` | A fetched external snapshot scanned and cited like any declaration (`external-facts` mode). |
| `list_json` | `grund list --format json <fixture>` | Tools reading the catalog as NDJSON. |
| `list_selector` | `grund list --selector FS <fixture>` | An agent narrowing the catalog to one kind or chapter. |
| `list_size` | `grund list --size <fixture>` | An agent finding heavy leads to split before reading them. |
| `list_unused` | `grund list --unused <fixture>` | An author finding declarations nothing cites yet. |
| `list_workspace` | `grund list <5-member workspace>` | Listing a workspace's members at once. |
| `list_large_10k` | `grund list <10k-file fixture>` | Listing the large repository. |
| `show_toc` | `grund show <ID> --toc <named-sections fixture>` | An agent choosing which subsection to read next. |
| `show_section` | `grund show <ID> --section format <named-sections fixture>` | An agent reading one named section. |
| `show_json` | `grund show <ID> --format json <fixture>` | Tools reading a lead as JSON. |
| `show_batch` | `grund show --batch --format json <fixture> < queries` | An agent grounding several citations in one process. |
| `show_batch_all` | `grund show --batch --all --format json <fixture>` | Exporting every declaration and section at once. |
| `refs_descendants` | `grund refs <ID>.format --descendants <named-sections fixture>` | A section's blast radius, its subsections included. |
| `refs_citations` | `grund refs FS-00002-feature-00002 --summary <citations-enabled fixture>` | Citation sites grouped by file on the citations fixture. |
| `cover_lines` | `grund cover --lines 1-40 <fixture>/docs/functional-spec/README.md` | Who owns a range of lines of one file, as the co-change recipe asks. |
| `cover_json` | `grund cover --format json <citations-enabled fixture>` | Coverage as JSON on the citations fixture. |
| `fmt_cross_refs` | `grund fmt --check --cross-refs <cross-refs fixture>` | The normalization gate of a repository that links its citations. |
| `fmt_marker` | `grund fmt --check --marker <cross-refs fixture>` | The normalization gate of a repository that marks bare IDs. |
| `id` | `grund id FS bench feature <fixture>` | An author minting the next free ID. |
| `complete_ids` | `grund complete ids --prefix FS-001 <fixture>` | Shell and editor completion of an ID prefix. |
| `lsp` | `grund-lsp < session on <256-file fixture>` | The language server answering an editor session on stdin (section 1.7). |

#### 1.2.1 Two tiers

A pull request runs the **`pr` tier**, about 40 rows, and the regression gate of [§AR-ci.5.2](AR-ci.md#52-regression-limits) holds every one. The **`nightly` tier** is everything else, so that the catalog holds 100 to 200 rows in all. The nightly run measures every row of both tiers, on `main`, every night and before a release ([§AR-ci.5.4](AR-ci.md#54-the-nightly-tier)), against the absolute ceilings alone. Both tiers write the same run record ([§AR-benchmarks.7](AR-benchmarks.md#7-the-run-record-and-its-series)). The ten rows above were the whole catalog before the tiers, and they are `pr` rows that keep their names, arguments, limits, ceilings and `pgo` entries byte for byte, so the training list of section 1.5 does not move.

At this size a table row per scenario would cost every reader of this point more than it tells them ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). So the `nightly` tier is rendered one line per feature group, with its row count, in the order the groups first appear in the catalog, and `tests/integration/test_bench_scenarios.py` holds this table to the catalog as well:

| Group | Rows | What it isolates |
|---|---|---|
| `scale` | 23 | `check`, `list`, `show --full`, `refs` and `cover` at 100 to 5,000 files, and the citations fixture at 5,000 and 10,000. |
| `check-flags` | 9 | `check`'s flags on the fixtures the `pr` tier does not run them on, and `--only`, `--ignore`, `--only-rule`, `--path-base`. |
| `show-modes` | 9 | Every `show` mode and format, the bare `grund <ID>` form, and `--batch` with `--toc` and at 10,000 files. |
| `list-modes` | 8 | `list --kind`, `--summary`, each `--size` unit and `--top`, and JSON at 10,000 files. |
| `refs-modes` | 6 | `refs --section`, `--summary`, `--total` and JSON. |
| `cover-modes` | 4 | `cover` of one file, several `--lines` ranges, and 10,000 files. |
| `fmt-modes` | 4 | `fmt --check` with `--cross-refs` and `--marker`, together and at 5,000 files. |
| `query` | 6 | `id --width`, `--explain` and JSON, and `complete ids` with and without sections. |
| `workspace` | 6 | A 10-member workspace of 5,000 files, and `refs` across members. |
| `rules` | 4 | Rule declarations at 5,000 files, and a `--rule` on top of them. |
| `named-sections` | 5 | Named sections at 5,000 files, their subsections and their size. |
| `values` | 4 | Value bindings at 5,000 files and the values chapter itself. |
| `external-facts` | 4 | External snapshots at 5,000 files, and `show`, `refs`, `list` of a fact. |
| `frontends` | 10 | The Python extension on the canonical fixture, one call per API entry, and `grund-lsp` on larger fixtures. |

#### 1.2.2 Shards

The `pr` tier is split into `shards` shards, six to eight, which run side by side so the tier's wall clock stays about what the ten rows took serially ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). No row names its shard. The assignment is derived from the catalog by `scripts/bench_shards.py`, and only there: it takes the `pr` rows in descending `ceiling_ir` order, ties by name, and gives each to the shard with the smallest `ceiling_ir` sum so far, ties to the lowest shard number. Every `pr` row lands in exactly one shard. The harness asks the script for its rows: with `GRUND_BENCH_SHARD=<i>/<n>` set it measures the `pr` rows of shard `i`, with `GRUND_BENCH_TIER=nightly` it measures every row of both tiers, and with neither it measures the whole `pr` tier, which is what a local `cargo bench` runs. The script reads the catalog with Python 3.6's standard library, as `train.py` does (section 1.5).

### 1.3 `fmt --check` measures a canonical fixture

The `fmt --check` benchmark setup first runs `fmt --write` on its generated
fixture outside Callgrind. This keeps the measured input canonical under the
revision being benchmarked, so the command exits successfully while its
instruction count still covers the complete dry-run scan. Other rows share the
fixture, and the harness regenerates a fixture that is not the generator's, so
it sets up a row with a `setup` after it has prepared every other row of its run.

### 1.4 Two cost shapes, and a clean exit

`check`, `check_large_10k`, `check_citations`, `list`, `cover`, and `fmt --check` walk a whole scanned tree; `show --brief`, `show`, `show --full`, and `refs` do the same walk plus a per-ID extraction or filter — so the set spans the engine's two cost shapes (full report vs. single-ID query), and the show ladder spans the short hover preview, the cheap lead default, and the full recursive body over realistic inputs.

Each command exits 0 on its conformant input (which keeps iai-callgrind's default `ExitWith::Success` happy); on a broken tree `check` / `fmt --check` exit non-zero and the bench fails — intentionally, since a baseline recorded against a broken tree is worthless.

### 1.5 The same command list trains PGO

The hot command list is also the training run for the profile-guided-optimization build of the distributed release binary (`scripts/pgo-build.sh`, [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)): the everyday operations we benchmark are the ones we profile against, so "the operations that matter" has one definition. The PGO run trains on this repository's own tree (a representative real workload, where profiling input *should* reflect reality), while the benchmarks run the same commands against the generated fixtures of [§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository) (where the gate needs a *stable* input).

The command list is defined once, as the catalog of [§AR-benchmarks.1.2](AR-benchmarks.md#12-the-benched-commands). The rows whose `pgo` is not `false` are its **`pgo` subset**. The subset, taken in catalog order, is the `grund` executable's training list. Each row in it gives an `id` that replaces `{id}`, because training reads this repository's own declarations (`FS-check`, `GOAL-fast-feedback`) where the benchmark reads the fixture's. It also gives an `api` label, which names the native call that runs the operation for the Node addon and the Python extension. `scripts/distribution/train.py` is the one PGO reader of the catalog. It reads the catalog with nothing but the standard library. `describe grund` prints the training list as `"grund <args>"` strings, `describe node-addon` and `describe python-extension` print the `api` labels, and both training runs drive the subset rather than a list of their own. `scripts/pgo-build.sh` hands every product, `grund` included, to `train.py`, so no copy of the list remains for a comment to keep in sync.

The training evidence a release records ([§FS-distribution-candidate.7.3](../functional-spec/FS-distribution-candidate.md#73-the-packaged-payload-is-the-profile-use-build)) is `describe`'s output. It stays byte for byte what the hand-written loop produced: `["grund check", "grund list", "grund show FS-check --brief", "grund show FS-check", "grund show FS-check --full", "grund refs GOAL-fast-feedback", "grund cover", "grund fmt --check"]`. If it changed, `training_sha256` and the profile key would move with no change to the training. `check_large_10k` and `check_citations` are deliberately excluded from PGO training: the first is a CI budget input, and the second is a fixture configuration, so neither is a representative release invocation. The list trains the `grund` executable alone. The language server trains on a workload of its own, and the Node addon and the Python extension train on the subset's operations through their APIs ([§FS-distribution-candidate.7.1](../functional-spec/FS-distribution-candidate.md#71-each-product-trains-on-its-own-workload)), so the list stays the one definition of the hot operations without becoming another product's profile. PGO stays out of the development build and push/PR CI loop ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)): the release pipeline runs it, and so does explicit benchmark work comparing the optimized release artifact.

### 1.6 The performance-surface inventory

Choosing the benched commands by how often they run ([§AR-benchmarks.1.2](AR-benchmarks.md#12-the-benched-commands)) leaves a gap: a command, a flag or a mode can ship with functional tests and no scenario, and nobody has to decide that it needs none. The **performance-surface inventory**, `crates/grund-cli/benches/surface.json`, closes that gap. Every item of grund's public surface either names scenarios from the catalog or carries a waiver that says why it has none. The inventory sits beside the catalog rather than inside it, so the harness and `scripts/distribution/train.py` read the catalog exactly as before. It adds no scenario, and the catalog stays the one list of scenarios.

The inventory covers three kinds of item, and only the third is kept by hand:

- **Commands**: every entry of the `SUBCOMMANDS` table in `crates/grund-cli/src/lib.rs`, the list the CLI dispatches on, and every entry of its `HIDDEN_SUBCOMMANDS` table, which the CLI dispatches too but keeps out of help and completion, such as the shell-completion helper `complete`. The inventory never copies either list. The test reads them. A hidden command has no help page, so it is inventoried as a command with no flags.
- **Flags**: every `--flag` the command's help page prints, read from the help page's source: the arm of `print_subcommand_help` in `crates/grund-cli/src/cli_help.rs` for that command, the `print_*_help` function the arm calls, and every `print_*` helper those call, such as `print_run_flag_help`. Its string literals are the text `grund <command> --help` prints. `--help` itself is left out, because every command takes it and it scans nothing. A positional word such as `config show` or `completions bash` is not a flag.
- **Designated performance-sensitive modes**: ways of running the engine that no command or flag names, so the CLI definition cannot list them. They are the list in section 1.6.1. That list is the one part of the surface written by hand, and adding a mode means adding it there.

`surface.json` holds an object with two keys. `commands` maps each command to its entry, and that entry may carry a `flags` object that maps each of its flags to an entry of its own. `modes` maps each designated mode to its entry. An entry is valid when it has exactly one of two keys:

- `scenarios`: a non-empty list of catalog row names. Each name must be a `name` in `scenarios.json`. For a command, each named row's first argument must be that command. For a flag, each named row's first argument must be the command and its `args` must carry the flag.
- `waiver`: a reason, as a non-empty string, why the item has no scenario. "Not on the save, commit or push path" is a reason. An empty string is not.

A flag is covered in one of three ways. A catalog row of its command carries it in `args`, and then it needs no entry. Or it has an entry of its own under its command's `flags`. Or its command is waived, and the waiver covers the command's flags with it, because a command with no scenario has no flag worth a separate one.

`tests/integration/test_bench_surface.py` holds the inventory to the surface, and it is a static read, so it runs in pre-commit. It fails, naming the item and `surface.json`, in each of these cases:

- a command has no entry;
- a flag is not covered;
- a designated mode has no entry;
- an entry names a scenario the catalog lacks, or one that does not measure its item;
- an entry has neither `scenarios` nor `waiver`, has both, or has an empty reason;
- an entry is stale, because it names a command, a flag or a mode that no longer exists.

Adding a subcommand, a flag or a designated mode with neither a scenario nor a waiver therefore fails the gate, and the person adding it has to make the decision.

#### 1.6.1 The designated performance-sensitive modes

Each line names a mode by its key in `surface.json`'s `modes`:

- `parallel-scan`: the scan of a large file list on several threads, which merges in sorted path order ([§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)).
- `workspace`: a scan from a workspace root over its member projects ([§AR-workspace](AR-workspace.md#ar-workspace-how-the-config-time-workspace-layer-composes-with-the-config-loader-and-the-scanner)).
- `lsp`: the `grund-lsp` language server, which rescans on every save ([§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built)).
- `python`: the Python extension, `grund-py`, which drives the engine through its API ([§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms)).
- `node`: the Node addon, `grund-node`. Waived until its counts are shown repeatable (section 1.7).
- `sequential-scan`: the scan of a file list below the parallel threshold, on one thread (section 1.8).
- `rules`: committed rule declarations evaluated by `grund check` ([§AR-rules](AR-rules.md#ar-rules-sentences-and-facts-meet-only-in-the-rule-engine)).
- `named-sections`: named section paths and the section lock that records cited ones ([§FS-section-lock](../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)).
- `values`: value bindings compared against their declared values ([§FS-values](../functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)).
- `external-facts`: a fetched external snapshot in its kind's home, scanned and cited like any declaration ([§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)).

Every mode but `node` names scenarios. A waiver states a reason that holds today; it never defers to a later issue, so no waiver in `surface.json` names one. `check --watch` keeps its waiver, because a loop whose cost per change is one rerun of a counted `check` has nothing of its own to count.

### 1.7 Frontends

Which frontends are measured, and how, is [§DA-benchmark-frontends](../decisions/architectural/DA-benchmark-frontends.md#da-benchmark-frontends-the-benchmarks-measure-the-language-server-and-the-python-extension-not-the-core-library-or-the-node-addon):

- **No core-library scenario.** Every frontend calls the core, and a `grund` row already counts it together with the start-up an agent pays.
- **`lsp`, in the `pr` tier.** A row with `frontend: lsp` runs the `grund-lsp` binary under Callgrind, driven over stdio by the editor session its PGO training runs: `initialize`, then each document's `didOpen` followed by its `hover`, then `shutdown`. The session is written once, in `scripts/distribution/train.py`; `train.py session --root <fixture>` prints it as framed JSON-RPC, and the harness feeds that to the server's stdin, so Callgrind counts the server and no client. `describe grund-lsp` stays byte for byte what it was.
- **`python`, in the `nightly` tier.** A row with `frontend: python` runs a pinned interpreter on the built extension with Callgrind's collection off at start and toggled on at the extension's entry point, so interpreter start-up does not drown the engine. A toggle holds only on the thread that enters the function, so collection also toggles on in the body of each worker thread the engine starts, and a row counts the same engine work its `grund` row does. It is nightly because building the extension would cost the pull-request wall clock.
- **`node`, waived.** V8's JIT and garbage-collector threads move instruction counts between runs of one build, so a count would not hold the same-binary bound of section 7.3. The inventory records the waiver and this reason until the counts are shown repeatable.

### 1.8 The sequential and parallel scan pair

One tree scan goes parallel at `PARALLEL_SCAN_MIN_FILES` scanned files ([§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)), 256 today, and every fixture of the ten rows lies far above it. So two `pr` rows run `check` on two generated fixtures that straddle it: one scans one file fewer than the threshold and takes the sequential path, and one scans exactly the threshold and takes the parallel one. The inventory names the first under `sequential-scan` and the second under `parallel-scan`. `tests/integration/test_bench_tiers.py` reads the threshold from `crates/grund-core/src/scanner/tree.rs` and generates both fixtures to count their scanned files, so the pair moves when the threshold does.

## 2. Why instruction counts, not wall-clock

Callgrind reports the number of CPU instructions executed (plus cache-miss estimates), and that number is **deterministic** for a given binary and input — it does not move when the CI runner is loaded, when a neighbour VM is noisy, or when the kernel reschedules. A wall-clock harness (criterion and friends) measures the thing the [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) budget is literally written in (milliseconds), but its variance on shared CI is large enough that "fail on regression" either flakes (tight threshold) or misses real regressions (loose threshold). Instruction count is the proxy that makes the [§AR-ci.5](AR-ci.md#5-benchmark-job) regression gate implementable: a quadratic walk or an accidental second read pass moves the instruction count by a margin no scheduling noise can mask. The full rationale, and the alternatives weighed, is [§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds).

### 2.1 The wall-clock guard backstops the budget

The millisecond budget itself is still backstopped by the cheap wall-clock guard in [§AR-ci.4](AR-ci.md#4-performance-smoke-guard) (the built `grund check . --full` self-check under a generous `timeout`); the instruction-count harness is the precise regression meter, the timeout is the catastrophic-blowup floor. The two are complementary, not redundant.

## 3. How it runs

- Locally: `cargo bench -p grund --features bench --bench instructions` (requires `valgrind`, `python3`, and `iai-callgrind-runner` on `PATH`). With no prior run, iai-callgrind records the counts and exits clean; on a later run in the same `target/` it prints the delta against the previous run. Every fixture is regenerated on each run ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)). `cargo bench` writes no run record; `python3 scripts/bench_record.py record --out target/bench-record.json`, run after it, does. That record says `environment: local`, a series of its own ([§AR-benchmarks.7.4](AR-benchmarks.md#74-a-local-run-is-a-series-of-its-own)).
- In CI: one build job of [§AR-ci.5](AR-ci.md#5-benchmark-job) builds the binaries in a pinned environment and hands them to every shard of the `pr` tier (section 1.2.2), each a job of the same environment. On a pull request each shard measures the base binary and the head's under the pull request's harness ([§AR-benchmarks.5](AR-benchmarks.md#5-comparing-two-revisions), [§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and growth past the limits of [§AR-ci.5.2](AR-ci.md#52-regression-limits) fails that shard. On a push to `main` each shard runs its rows twice on one binary and checks the same-binary bound ([§AR-benchmarks.7.3](AR-benchmarks.md#73-the-same-binary-bound)).
- Nightly and before a release: the `nightly` tier of section 1.2.1, every row of both tiers, on `main`, against the ceilings alone ([§AR-ci.5.4](AR-ci.md#54-the-nightly-tier)).
- Every run is written down as one run record per revision ([§AR-benchmarks.7](AR-benchmarks.md#7-the-run-record-and-its-series)): each shard writes a partial record with `bench_record.py record`, and `bench_record.py merge` folds the partials into the record. A local run, one process, writes the record directly. Two records are compared only within one series.

### 3.1 The measured process runs under the runtime pins

iai-callgrind clears the environment before it starts the program Callgrind measures, so a variable the job sets reaches the measured process only when the harness names it. The harness names exactly the three **runtime pins**, `RAYON_NUM_THREADS`, `LC_ALL` and `TZ`, as one list in `crates/grund-cli/benches/instructions.rs`, and hands that list to `pass_through_envs` on the configuration every row is measured with, whatever its frontend; every other variable stays cleared, so nothing else of the caller's environment reaches the count. The runtime pins are the variables the measured process reads: `RAYON_NUM_THREADS` sizes its Rayon pool, whose scheduling is part of the count, and `LC_ALL` and `TZ` are the locale and timezone it runs in. The build-only pins of [§AR-ci.5](AR-ci.md#5-benchmark-job), `CARGO_INCREMENTAL` and `RUSTFLAGS`, shape the binary before it runs and are not forwarded. So the `locale`, `timezone` and `threads` a run record names (section 7.1) are those of the process its counts come from. The native probe of [§AR-benchmarks.5.1](AR-benchmarks.md#51-a-scenario-the-base-binary-refuses) inherits the whole environment and needs no list.

## 4. Relationship to the rest of the tree

- [§AR-goal-measurement.2](AR-goal-measurement.md#2-goal-meters) — this harness is the fast-feedback meter; [§GOAL-fast-feedback.1](../goals.md#1-performance-targets)'s targets are recorded per commit and, on a pull request, compared against a baseline saved from the base branch ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)); growth past the limits fails the build ([§AR-ci.5.2](AR-ci.md#52-regression-limits)).
- [§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds) — the decision record for choosing Callgrind/iai-callgrind over a wall-clock criterion harness ([§AR-benchmarks.2](AR-benchmarks.md#2-why-instruction-counts-not-wall-clock)), and for benchmarking the binary rather than the library ([§AR-benchmarks.1](AR-benchmarks.md#1-what-is-benched)).
- [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms) / [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine) — the harness is deliberately established before the workspace split and bindings work move the engine around, so any slowdown shows up as a diff against a known-good instruction count rather than going unnoticed.
- [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload) — the PGO training corpus of [§AR-benchmarks.1.5](AR-benchmarks.md#15-the-same-command-list-trains-pgo); the release pipeline that runs `scripts/pgo-build.sh` is release packaging ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)), whose contract is [§FS-distribution.4](../functional-spec/FS-distribution.md#4-release-process).

## 5. Comparing two revisions

The counts are deterministic for a given **binary**, not for a given source tree, and two checkouts have their own `target/` directories — one of which may hold a differently-built binary. A PGO build from `scripts/pgo-build.sh` ([§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)) is the case to watch, because it is trained on this very workload and reads several percent faster on it.

So benching a branch in one checkout against `main` in another measures the build as much as the change, and it fails by producing a plausible number rather than an error. Take both numbers in **one worktree**, switching the source under a single `target/`, and against the branch's **merge base** rather than whatever `main` has since become.

Two revisions are compared by **one harness on one fixture bundle**, and only the binaries differ. Under shards each shard compares its own rows this way, base then head on the same runner, so no count is compared with one another runner took. The harness takes the binary under test from `GRUND_BENCH_BIN` when it is set, and its own freshly built `grund` otherwise. Measuring the base revision means building the base's `grund` binary alone, then running the newer revision's harness with `GRUND_BENCH_BIN` naming it and `--save-baseline`, and the same harness on its own binary with `--baseline`. The base revision's harness and fixture generator are never run, so a change to either cannot read as a change in the code.

### 5.1 A scenario the base binary refuses

A scenario the base binary cannot run — it names a flag or a mode the base does not know — is not compared, and it does not fail the comparison. Before measuring a scenario under `GRUND_BENCH_BIN`, the harness runs its invocation once outside Callgrind; when that exits non-zero, the harness leaves the scenario out of the base run and the base's run record lists it as refused. A scenario's setup that runs the binary, such as `fmt_check`'s `fmt --write`, is part of that probe: a base binary that fails the setup refuses the scenario rather than stopping the harness. Under the harness's own binary a failed setup still fails the run. The head run still measures it, and the base run still generates and digests its fixture ([§AR-benchmarks.7.1.1](AR-benchmarks.md#711-one-record-per-revision-merged-from-its-shards)), so a refusal never changes the series. The comparison reports it as `no base, not compared`. Only a scenario both binaries ran can fail its relative limit. Every scenario the head measures is still held to its ceiling ([§AR-benchmarks.5.2](AR-benchmarks.md#52-a-scenario-with-no-base)).

### 5.2 A scenario with no base

A scenario with no stored count to compare against does not fail for that. This happens when a pull request adds a row that the base binary refuses (section 5.1), or on the first local run in a fresh `target/`. A row a pull request adds that the base binary accepts is measured on both binaries by the pull request's own harness, so it is compared from that same pull request on. A run with no base records the scenario's count, which becomes the baseline the next comparison reads, and from then on the relative limit gates it. Its absolute ceiling ([§AR-ci.5.2](AR-ci.md#52-regression-limits)) does not need a base, so it holds from the scenario's first CI run.

iai-callgrind 0.16.1 already behaves this way: a soft limit is checked only when both counts exist, and a hard limit is checked on the new count alone. So `--baseline=main` with no stored `main` count prints the count against `N/A` and exits 0, and a count over a hard limit exits 3, which fails `cargo bench`. The harness and the bench job keep that behaviour and add no step that treats a missing base as an error.

## 6. Scope

This is a build-and-CI tool, not a CLI surface: it adds no subcommand, no flag, no output shape, so the `tests/e2e/cases/*` corpus is unchanged and nothing here is covered by the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) user-visible-surface contract. The performance-surface inventory of [§AR-benchmarks.1.6](AR-benchmarks.md#16-the-performance-surface-inventory) is repository policy in the same way: it reads the CLI's surface and changes none of it. The benched commands are specified by their functional specs ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-citation-in-a-repo), [§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](../functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](../functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](../functional-spec/FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk), [§FS-cli](../functional-spec/FS-cli.md#fs-cli-grunds-command-line-surface-conventions)) rather than re-specified here.

## 7. The run record and its series

Every run of the harness is normalized into one compact JSON **run record** by `python3 scripts/bench_record.py record`, run after `cargo bench` (the CI job runs it after each run, and a local run runs it by hand), and two records are compared only when they belong to one **series**. The record is what makes a count readable apart from the job that took it: it says what was measured, with what, and on which input.

### 7.1 What a record holds

A record holds the `commit` and the SHA-256 of the binary measured (`binary_sha256`), the `ir` count per scenario that ran, the scenarios the binary `refused` ([§AR-benchmarks.5.1](AR-benchmarks.md#51-a-scenario-the-base-binary-refuses)), and the `tier` that ran, `pr` or `nightly` (section 1.2.1). Beside them it holds its identity fields:

- `record_version`, the version of this record format, 3 since the measured process runs under the runtime pins of section 3.1: a record of version 2 names a `locale`, `timezone` and `threads` the process its counts come from did not have, so a record of version 3 never joins its series (section 7.2);
- `environment`, `ci` or `local`;
- `image`, the container image's digest, or `local`;
- `rustc`, `valgrind` and `runner`, the versions of the compiler, of Valgrind and of `iai-callgrind-runner`;
- `build_flags`, the `RUSTFLAGS`, `CARGO_INCREMENTAL` and Cargo profile the binary was built with;
- `locale` and `timezone`, the `LC_ALL` and `TZ` of the run, which reach the measured process (section 3.1);
- `threads`, the `RAYON_NUM_THREADS` of the run, which sizes the measured process's Rayon pool (section 3.1);
- `fixtures`, each fixture's `schema_version` and `digest` ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)).

A record missing an identity field is refused when it is written.

#### 7.1.1 One record per revision, merged from its shards

The two tiers write one format, and a run writes one record per revision however many shards measured it. Each shard writes a partial record of its own rows with `bench_record.py record --tier <tier>`, and `bench_record.py merge --out <record> <partial>…` folds them into one. A run's fixture bundle is the fixtures of every row it selected, whether the binary under test refuses the row or not, so the base and head of one shard digest the same bundle whatever the base refuses, and `compare` holds them to it exactly. Each shard digests only its own rows' fixtures, so `fixtures` is merged as a bundle: the merged record's `fixtures` is the union of the partials' fixture maps, and a fixture two partials digest differently is refused. Every other identity field must match exactly. Merge refuses, exits non-zero and names the cause, when two partials differ in such a field (named as `compare` names it, so a pin that drifted between two legs is caught here), in `commit`, in `binary_sha256` or in `tier`, or when one scenario is in two partials. The merged record holds the union of the counts and of the refusals. A merged record's fixture bundle is therefore the fixtures of the rows its tier selected, so each tier is a series of its own: records of one tier compare, a pull request's base against its head and a nightly run against the nightly runs before it, and a nightly record never joins a pull request's series. Neither the tier nor the shard count is itself in the series key.

### 7.2 The series key

The series key is the SHA-256 of the identity fields, and of nothing else: the commit and the binary digest are what a comparison compares, so they never change the key, while a change to any identity field does. `bench_record.py compare` refuses two records whose keys differ, exits non-zero, and names each identity field that differs, so a fixture, toolchain or environment change cannot be read as a code change. The first record after such a change starts a new series; records before it are never compared with it.

### 7.3 The same-binary bound

The same-binary bound is **1%** `Ir`: two runs of one binary on one fixture bundle, in one series, give each scenario counts that differ by at most that much. A push to `main` checks it ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)) with `bench_record.py jitter`, which fails naming each scenario past the bound. Each shard checks its own rows, both runs on its own runner. If that check ever fails on an unchanged binary, it is reported as a flaky test, with the labels `bug` and `flaky-test`. Every regression limit stays at least five times this bound ([§AR-ci.5.2](AR-ci.md#52-regression-limits)).

### 7.4 A local run is a series of its own

A local `cargo bench` runs outside the pinned image, so the record `bench_record.py record` writes after it says `environment: local` and `image: local`, and it is a series of its own by construction. A run of the pinned image on another machine records `local` too: the host's processor is not in the series key, so its counts must not join CI's series. That is the honest reading of the gap between the same commit's counts locally and in CI. `docs/benchmarks.md` gives the one `docker run` command that reproduces CI's environment.
