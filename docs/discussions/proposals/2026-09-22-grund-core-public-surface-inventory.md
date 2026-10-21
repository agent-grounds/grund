# The `grund-core` public root surface, name by name

The evidence [§DISC-grund-core-public-surface](2026-09-22-grund-core-public-surface.md#disc-grund-core-public-surface-what-grund-cores-public-root-surface-is-and-what-it-should-be) argues from. It classifies nothing on its own authority: the counting convention is [§DISC-grund-core-public-surface.3](2026-09-22-grund-core-public-surface.md#3-what-counts-as-a-public-root-name), the columns are [§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries), and `tests/integration/test_public_surface_inventory.py` holds the first column equal to the crate root's export list ([§DISC-grund-core-public-surface.5](2026-09-22-grund-core-public-surface.md#5-what-is-checked)).

The original 182-name audit uses the baseline [§DISC-grund-core-public-surface.2](2026-09-22-grund-core-public-surface.md#2-the-audited-baseline) records — `7283e23bb0`, workspace `0.14.2-dev`, `v0.14.1-26-g7283e23bb0`. Four additive completion exports were read at `abba38146e` and seven additive Python-binding exports at `fa57931b17`, both workspace `0.16.2-dev`, three hidden, data-only watch seams of [§AR-bindings.3](../../architecture/AR-bindings.md#3-cratesgrund-cli-the-cli-binary) extend it, and eight additive `cover --lines` exports of [§FS-cover.6](../../functional-spec/FS-cover.md#6-line-ownership) were read at `1f129faef7`, and two hidden run-flag seams of [§FS-cli.3.4](../../functional-spec/FS-cli.md#34---path-base--where-report-paths-are-spelled-from) carry the `--path-base` a frontend parses, and eight additive Node-binding exports of [§FS-distribution.3.2](../../functional-spec/FS-distribution.md#32-node-grund-cli-npm-package) were read at `92bd7ff28d`; the concern records `Catalog`, `Project`, `Run` and `Compiled` of [§AR-config.1](../../architecture/AR-config.md#1-the-records), with `effective_project`, followed, and `Config`, `KindConfig` and `CitationRules` became the deprecated aliases that ramp removed in 0.19.0 with `Findings` (see [Removed names](#removed-names)), the hidden schema view of [§AR-config.6.3](../../architecture/AR-config.md#63-the-schema-view-is-lowered-from-a-project-on-demand), `schema_view` and the twelve record types `grund schema` prints from, followed, and `show_from_snapshot` with its `LspShowContext` took the place of `show_with_overlays` when the editor's citation hover began answering from its snapshot ([§FS-lsp.responsiveness.3](../../functional-spec/FS-lsp.md#responsiveness3-a-citation-hover-reads-nothing-the-snapshot-did-not)), and the hidden `grund config migrate` seam of [§FS-config-migrate](../../functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does), `config_migrate` with its `MigrateOpts` and the ten record types its report is built from, was read at `17e31c4b4a`, and the four hidden seams a frontend reads coordinate globs through followed: `check_coordinate_globs` and `reads_as_coordinate_glob` for the launch refusal and the PATH hint of [§FS-coordinate-globs.errors](../../functional-spec/FS-coordinate-globs.md#errors-refusals-notes-and-hints), and `show_batch_with_globs` with its `BatchShowRun` for [§FS-show.1.9](../../functional-spec/FS-show.md#19---batch---all-every-coordinate-in-scope), which took `show_batch_with_scope`'s place in the CLI, and seven hidden citation-site seams of [§FS-refs.globs](../../functional-spec/FS-refs.md#globs---glob-lists-who-cites-a-set), [§FS-refs.from](../../functional-spec/FS-refs.md#from---from-lists-what-one-point-cites) and [§FS-cover.globs](../../functional-spec/FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches) followed them: `refs_sites_with_run_warnings` with its `RefsSitesOpts` and `RefsSites`, `cover_with_globs`, `glob_beside_section` for the pattern a refusal offers, `reads_as_coordinate` for the PATH hint and `cites_clause` for a `--from` total; the existing rows retain their original evidence. The two writer-fault exports exist only with `test-binding-writes`. The transition of [§FS-distribution.3.1.3](../../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point) added fourteen names — `integrations`, its eleven records, and root names for `RelativeHint` and `RelativeSectionParent`, which `Catalog` already reached — read at the commit that added them, and refreshed the consumer evidence of the thirty integrations names it gave notice to. The conflict refusal of [§FS-fmt.3.2](../../functional-spec/FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing) added `FmtConflictAbort`, the third typed refusal of [§FS-distribution.3.1.2](../../functional-spec/FS-distribution.md#312-what-10-freezes), beside `FmtScanAbort`. The hidden `ArmedChecks` of [§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so), the record of what a run armed that `check_with_run_warnings` hands a frontend's `--only`, followed, and the hidden `explain_vocabulary` of [§FS-config-explain](../../functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on), the tables, keys and codes the coverage test asks `grund config explain` about, after it, and the hidden `Code` of [§AR-system.2.2](../../architecture/README.md#22-model), the check code handles rendered from the catalog, took `CHECK_FINDING_CODES`' row, and the hidden `config_explain` that answers `grund config explain`, with `ExplainAsk` and the ten record types its answer is built from, the catalog's four among them, followed. **293 rows, one per public root name**, including feature-gated exports. The counts below include every addition; the Python and Node frontends now supply consumer evidence alongside CLI and LSP.

The two hidden frontend bridges `query_format_hint` and `refs_with_metadata_and_hint`, added for [§FS-show.3.5.1](../../functional-spec/FS-show.md#351-an-id-the-grammar-rejects), were read at `70db4f12de`; their rows record current CLI and regression-test consumers.

The three hidden setup-guide exports `AGENT_SETUP_SECTIONS`, `SetupPart` and `setup_instructions`, added for [§FS-init.5.5](../../functional-spec/FS-init.md#55-the-core-and-its-sections), were read on the `fix/issue-830` branch above `9d78aaaf38`. The CLI reads the setup guide through them since, so `AGENT_SETUP_INSTRUCTIONS` and `canonical_template_text` have no reader outside grund-core.

The four hidden guide exports `GuideSection`, `SETUP_CORE_SECTION`, `guide_sections` and `setup_fresh_path`, added for [§FS-init.5.6](../../functional-spec/FS-init.md#56-the-front-page), were read on the `fix/issue-893` branch above `22f89f353e`. `grund guide` reads the section names through `guide_sections` since, so `AGENT_SETUP_SECTIONS` has no reader outside grund-core.

The two additive hidden frontend seams `refs_with_text_annotations` and `refs_sites_with_text_annotations` carry the index-entry annotations of [§FS-refs.3.1.1](../../functional-spec/FS-refs.md#311-label-the-excluded-index-sites) and [§FS-refs.3.3.1](../../functional-spec/FS-refs.md#331-identify-the-excluded-sites-within-a-summary). They sit outside the embedding closure and frozen promise of [§FS-distribution.3.1.2](../../functional-spec/FS-distribution.md#312-what-10-freezes). The CLI now reads refs through these seams: `refs_sites_with_run_warnings` has no reader outside grund-core, and `refs_with_metadata_and_hint` retains its regression-test consumer.

## How a cell reads

Paths are relative to `crates/`, and an evidence location is one site, not every site: the import or call that establishes the consumer, so a reader can start somewhere rather than grep.

**Owner** — the component whose `mod.rs` the item crosses on its way to the root ([§AR-system.4](../../architecture/README.md#4-dependency-direction)).

**Rustdoc** — `visible`, `visible, #[deprecated]`, or `#[doc(hidden)]` read from the `#[doc(hidden)] pub use` group in `crates/grund-core/src/lib.rs` that exports the name. Nothing more: hiding is discoverability, and a hidden symbol still links.

**Spec** — how [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) reaches the name. `named`: its text or its example writes the symbol. `related API`: a Rustdoc-visible function of the `api` component, which [§AR-system.2.9](../../architecture/README.md#29-api) makes the embedding surface and which that point's "and related APIs" reaches. `data type of X`: it appears in the signature of a `named` or `related API` entry point, transitively — one such path is named, not all of them. `no`: outside that closure, so the specification supports embedding it nowhere.

**Consumers** — a textual `use grund_core::…` or `grund_core::…` in `grund-cli`, `grund-lsp`, `grund-py`, `grund-node` or their test crates, or in the shared Rust oracle example `grund-core/examples/grund-binding-oracle.rs`. `none found` is the result of searching **this repository** and never a claim about embedders outside it ([§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries)).

**Structural** — for a name no frontend spells: the chain from something one does call, so `via check_with_run_warnings → CheckOutput` means the CLI receives the type without naming it. `—` where the consumer column already answers. `none found` in both columns means no repository consumer of either kind was found.

**Disposition** — what the name is in `0.19.0`, by the rule [§FS-distribution.3.1.2](../../functional-spec/FS-distribution.md#312-what-10-freezes) states. `frozen at 1.0` is an entry point that point lists, or a type their signatures reach, transitively; the Spec column names one path that reaches it. `hidden` is `#[doc(hidden)]` and outside the promise: it still links and still works, and the Consumers column names the frontend that reads it, or says it has no reader outside grund-core. `is removed in 0.21.0` is one of the thirty integrations names `integrations` answers for, deprecated since `0.19.0` with a one-line note naming that release ([§FS-distribution.3.1.3](../../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

## What the columns add up to

| Reading | Count |
| --- | --- |
| Public root names | 293 |
| Rustdoc-visible | 113 |
| Visible and deprecated | 30 |
| `#[doc(hidden)]` | 150 |
| Named, a related API, or a data type of one | 140 |
| Outside that closure | 153 |
| Named by a frontend or its tests | 159 |
| Reached structurally only | 47 |
| No repository consumer of either kind | 87 |
| Disposition frozen at 1.0 | 113 |
| Disposition hidden | 150 |
| Disposition is removed in 0.21.0 | 30 |

The two count columns do not line up, and they are not meant to: 87 names have no repository consumer while 153 sit outside what the specification reaches, and the two sets overlap only in part. A name can be specification-supported and unused here — that is most of what `scan` returns — or used by a frontend and supported nowhere, which is what a seam is.

## The inventory

| Name | Owner | Rustdoc | Spec | Consumers | Structural | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| `AbsentOptionalNamespace` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → AbsentOptionalNamespace `grund-cli/src/lib.rs:13` | hidden |
| `agent_override_table` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `AGENT_SETUP_INSTRUCTIONS` | templates | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `AGENT_SETUP_SECTIONS` | templates | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `ApiFailure` | api | `#[doc(hidden)]` | no | none found | via ApiOutcome → ApiFailure `grund-node/src/lib.rs:165` | hidden |
| `ApiFailureSite` | api | `#[doc(hidden)]` | no | none found | via ApiOutcome → ApiFailure → ApiFailureSite `grund-node/src/lib.rs:165` | hidden |
| `ApiOutcome` | api | `#[doc(hidden)]` | no | grund-node `grund-node/src/lib.rs:165` | — | hidden |
| `ApiPartialOutput` | api | `#[doc(hidden)]` | no | none found | via ApiOutcome → ApiFailure → ApiPartialOutput `grund-node/src/lib.rs:165` | hidden |
| `ApiScanError` | scanner | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `BadgeRung` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/tests_schema.rs:8` | — | hidden |
| `BatchShowFailure` | queries | `#[doc(hidden)]` | no | none found | via show_batch_with_globs → BatchShowRun → BatchShowRecord → BatchShowFailure `grund-cli/src/cli_show_batch.rs:104` | hidden |
| `BatchShowQuery` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `BatchShowRecord` | queries | `#[doc(hidden)]` | no | none found | via show_batch_with_globs → BatchShowRun → BatchShowRecord `grund-cli/src/cli_show_batch.rs:85` | hidden |
| `BatchShowRun` | queries | `#[doc(hidden)]` | no | none found | via show_batch_with_globs → BatchShowRun `grund-cli/src/cli_show_batch.rs:70` | hidden |
| `binding_write_fault` | writers | `#[doc(hidden)]` | no | grund-node test seams `grund-node/src/test_runtime.rs:142` | — | hidden |
| `BindingWriteFaultGuard` | writers | `#[doc(hidden)]` | no | grund-node test seams `grund-node/src/test_runtime.rs:135` | — | hidden |
| `block_outcome_verb` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `BlockOutcome` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `can_replace_trigger_at` | queries | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `canonical_snapshot_path` | model | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `canonical_template_text` | templates | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `Catalog` | model | visible | named | none found | none found | frozen at 1.0 |
| `check` | api | visible | named | grund-lsp tests `grund-lsp/tests/chapter_values.rs:40` | — | frozen at 1.0 |
| `check_coordinate_globs` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:23` | — | hidden |
| `check_with_opts` | api | visible | related API | none found | none found | frozen at 1.0 |
| `check_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `CheckFindingSelection` | checker | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:27` | — | hidden |
| `ArmedChecks` | checker | `#[doc(hidden)]` | no | none found | via check_with_run_warnings → ArmedChecks `grund-cli/src/cli_check.rs:169` | hidden |
| `CheckInput` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_watch_inputs.rs:10` | — | hidden |
| `CheckInputObserver` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_watch.rs:78` | — | hidden |
| `CheckOpts` | api | visible | data type of `check_with_opts` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `CheckOutput` | api | visible | data type of `check_with_opts` | none found | via check_with_run_warnings → CheckOutput `grund-cli/src/cli_check.rs:111` | frozen at 1.0 |
| `Citation` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `citation_under_title` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `CitationCompletion` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot_with_completion` | none found | via lsp_snapshot_with_completion → LspSnapshotWithCompletion → LspCompletionContext::complete → CitationCompletion `grund-lsp/src/completion.rs:26` | hidden |
| `CitationDisjunction` | config | `#[doc(hidden)]` | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `CitationLevel` | config | `#[doc(hidden)]` | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `CitationTarget` | config | `#[doc(hidden)]` | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `cites_clause` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:21` | — | hidden |
| `ClauseBy` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `Code` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:29`; grund-py `grund-py/src/lib.rs:35` | — | hidden |
| `CodeClause` | model | `#[doc(hidden)]` | no | none found | via config_explain → Explained → ExplainAnswer → ExplainCode → CodeEnabled → CodeClause `grund-cli/src/cli_config_explain.rs:242` | hidden |
| `CodeEnabled` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `CodeSeverity` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `Compiled` | config | `#[doc(hidden)]` | no | none found | none found | hidden |
| `complete_ids` | api | visible | related API | none found | none found | frozen at 1.0 |
| `complete_ids_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `CompleteIdsOpts` | api | visible | data type of `complete_ids` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `config_explain` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:48` | — | hidden |
| `config_migrate` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:42` | — | hidden |
| `config_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `config_warnings` | api | `#[doc(hidden)]` | related API | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `ConfigLocation` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → ConfigLocation `grund-cli/src/lib.rs:13` | hidden |
| `ConversationRendering` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `ConversationTarget` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `cover` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `cover_lines` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `cover_lines_with_run_warnings` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `cover_text` | api | visible | related API | none found | none found | frozen at 1.0 |
| `cover_text_with_run_warnings` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `cover_with_globs` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:22` | — | hidden |
| `cover_with_run_warnings` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `CoverCitation` | api | visible | data type of `cover` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `CoverEntry` | api | visible | data type of `cover` | grund-cli `grund-cli/src/cli_cover.rs:106` | — | frozen at 1.0 |
| `CoverLineOwner` | api | visible | data type of `cover_lines` | none found | via cover_lines → CoverLinesOutput → CoverLines → CoverLineOwner `grund-cli/src/cli_cover.rs:193` | frozen at 1.0 |
| `CoverLines` | api | visible | data type of `cover_lines` | grund-cli `grund-cli/src/cli_cover.rs:193` | — | frozen at 1.0 |
| `CoverLineSection` | api | visible | data type of `cover_lines` | none found | via cover_lines → CoverLinesOutput → CoverLines → CoverLineOwner → CoverLineSection `grund-cli/src/cli_cover.rs:193` | frozen at 1.0 |
| `CoverLinesOpts` | api | visible | data type of `cover_lines` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `CoverLinesOutput` | api | visible | data type of `cover_lines` | none found | via cover_lines → CoverLinesOutput `grund-cli/src/cli_cover.rs:168` | frozen at 1.0 |
| `CoverOpts` | api | visible | data type of `cover` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `CoverOutput` | api | visible | data type of `cover` | none found | via cover → CoverOutput `grund-cli/src/cli_cover.rs:56` | frozen at 1.0 |
| `CoverTextCitation` | api | visible | data type of `cover_text` | none found | none found | frozen at 1.0 |
| `CoverTextEntry` | api | visible | data type of `cover_text` | none found | none found | frozen at 1.0 |
| `CoverTextOutput` | api | visible | data type of `cover_text` | none found | none found | frozen at 1.0 |
| `Declaration` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `DeclarationSource` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `DeclaredId` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `detect_clients` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `DocCommentBlock` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `E2eCase` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `E2eSpecRef` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `EdgeTarget` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `effective_config` | api | `#[doc(hidden)]` | related API | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/integrations.rs:103` | — | hidden |
| `effective_project` | api | `#[doc(hidden)]` | related API | none found | none found | hidden |
| `EmbeddedValueRoot` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `embedding_call` | api | `#[doc(hidden)]` | related API | grund-py `grund-py/src/lib.rs:3`; oracle `grund-core/examples/grund-binding-oracle.rs:8` | — | hidden |
| `EmbeddingRequest` | api | `#[doc(hidden)]` | data type of `embedding_call` | grund-py `grund-py/src/lib.rs:3`; grund-node `grund-node/src/lib.rs:7`; oracle `grund-core/examples/grund-binding-oracle.rs:8` | — | hidden |
| `explain_vocabulary` | api | `#[doc(hidden)]` | no | grund-cli tests `grund-cli/tests/explain_coverage.rs:177` | — | hidden |
| `ExplainAnswer` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `ExplainAsk` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `ExplainCode` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `Explained` | config | `#[doc(hidden)]` | no | none found | via config_explain → Explained `grund-cli/src/cli_config_explain.rs:7` | hidden |
| `ExplainKey` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:47` | — | hidden |
| `ExplainTable` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:48` | — | hidden |
| `ExplainUnknown` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:48` | — | hidden |
| `expand_target` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `Extent` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `fetch_snapshot` | writers | visible | no | none found | none found | frozen at 1.0 |
| `fetch_snapshot_with_run_warnings` | writers | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `FetchFailure` | writers | visible | no | none found | via fetch_snapshot_with_run_warnings → FetchFailure `grund-cli/src/cli_fetch.rs:8` | frozen at 1.0 |
| `FetchFailureKind` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `FieldContent` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `FileHeading` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `FileStructure` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `Finding` | model | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | frozen at 1.0 |
| `FindingSite` | model | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `FmtChange` | api | visible | data type of `format_references` | none found | via format_references → FmtOutput → FmtChange `grund-cli/src/cli_fmt.rs:35` | frozen at 1.0 |
| `FmtConflictAbort` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `FmtOpts` | api | visible | data type of `format_references` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `FmtOutput` | api | visible | data type of `format_references` | none found | via format_references → FmtOutput `grund-cli/src/cli_fmt.rs:35` | frozen at 1.0 |
| `FmtScanAbort` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `format_references` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `format_references_with_run_warnings` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `Gate` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `GateResult` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `glob_beside_section` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:23` | — | hidden |
| `GLOBAL_AGENT_INSTRUCTION_TARGETS` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `GlobalAgentTarget` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `Grammar` | grammar | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → Grammar `grund-cli/src/lib.rs:13` | hidden |
| `GRUND_OPEN_RESOLVER` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `guide_sections` | templates | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `GuideSection` | templates | `#[doc(hidden)]` | no | none found | via guide_sections → GuideSection `grund-cli/src/lib.rs:13` | hidden |
| `Id` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `IdOpts` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `IdProposal` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `IdProposalOutcome` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `init` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `InitAgentEntrypointSelection` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `InitError` | writers | visible | no | none found | via init → InitError `grund-cli/src/cli_init.rs:64` | frozen at 1.0 |
| `InitEvent` | writers | visible | no | none found | via InitOutput → InitEvent `grund-cli/src/lib.rs:13` | frozen at 1.0 |
| `InitFsHome` | writers | visible | no | none found | via InitNext → InitFsHome `grund-cli/src/lib.rs:13` | frozen at 1.0 |
| `InitNext` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `InitOpts` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `InitOutput` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `InlineCitationSite` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `install_agent_guidance_block` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `install_managed_block` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `install_reference_key` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `InstallKind` | writers | visible | data type of `integrations` | none found | via integrations → IntegrationClientStatus → InstallKind `grund-cli/src/cli_integrations.rs:211` | frozen at 1.0 |
| `integration_is_current` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `IntegrationAgent` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationArtifact` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationClient` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationClientStatus` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationEvent` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationManualStep` | writers | visible | data type of `integrations` | none found | via integrations → IntegrationsOutput → IntegrationManualStep `grund-cli/src/cli_integrations.rs:211` | frozen at 1.0 |
| `IntegrationNote` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationOutcome` | writers | visible | data type of `integrations` | none found | via integrations → IntegrationsOutput → IntegrationEvent → IntegrationOutcome `grund-cli/src/cli_integrations.rs:211` | frozen at 1.0 |
| `integrations` | writers | visible | named | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `INTEGRATIONS_BLOCK_VERSION` | grammar | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `IntegrationsError` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationsFailure` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationsOpts` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `IntegrationsOutput` | writers | visible | data type of `integrations` | grund-cli `grund-cli/src/lib.rs:45` | — | frozen at 1.0 |
| `InvalidValueSite` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `Junction` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/tests_schema.rs:8` | — | hidden |
| `KindCitationRules` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via CitationRules → KindCitationRules `grund-cli/src/lib.rs:13` | hidden |
| `KindIndex` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → KindConfig → KindIndex `grund-cli/src/lib.rs:13` | hidden |
| `KindResolution` | config | `#[doc(hidden)]` | data type of `validate_config` | grund-cli `grund-cli/src/cli_config.rs:204` | — | hidden |
| `known_agent` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `known_agents_list` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `known_clients_line` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `LeadSizeWarning` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → LeadSizeWarning `grund-cli/src/lib.rs:13` | hidden |
| `LineEdit` | queries | `#[doc(hidden)]` | no | none found | via on_type_line_edits → LineEdit `grund-lsp/src/lib.rs:1329` | hidden |
| `LinkSupport` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `link_hover_citations` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:7` | — | hidden |
| `list` | api | visible | related API | none found | none found | frozen at 1.0 |
| `list_sizes` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `list_sizes_with_run_warnings` | queries | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `list_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `ListedComment` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `ListEntry` | api | visible | data type of `list` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `ListOpts` | api | visible | data type of `list` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `ListOutput` | api | visible | data type of `list` | none found | via list_with_run_warnings → ListOutput `grund-cli/src/cli_list.rs:204` | frozen at 1.0 |
| `ListSizeEntry` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `ListSizeMeasurement` | queries | visible | no | none found | via ListSizeEntry → ListSizeMeasurement `grund-cli/src/lib.rs:13` | frozen at 1.0 |
| `ListSizeOpts` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `ListSizeOutput` | queries | visible | no | none found | via list_sizes → ListSizeOutput `grund-cli/src/cli_list.rs:170` | frozen at 1.0 |
| `ListSummary` | api | visible | data type of `list` | grund-cli `grund-cli/src/cli_list.rs:238` | — | frozen at 1.0 |
| `ListValueRoot` | api | visible | data type of `list` | none found | via ListEntry → ListValueRoot `grund-cli/src/lib.rs:13` | frozen at 1.0 |
| `lsp_hover_with_kind_title` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `lsp_snapshot` | api | `#[doc(hidden)]` | related API | none found | none found | hidden |
| `lsp_snapshot_with_completion` | api | `#[doc(hidden)]` | related API | grund-lsp `grund-lsp/src/lib.rs:8` | — | hidden |
| `lsp_snapshot_with_metadata` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `lsp_title_hover_body` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspCitation` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspCompletionContext` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot_with_completion` | grund-lsp `grund-lsp/src/workspace.rs:19` | — | hidden |
| `LspDeclaration` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspFindingRange` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | none found | via LspSnapshot → LspFindingRange `grund-lsp/src/lib.rs:4` | hidden |
| `LspShowContext` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot_with_completion` | grund-lsp `grund-lsp/src/workspace.rs:23` | — | hidden |
| `LspSnapshot` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspSnapshotOpts` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspSnapshotWithCompletion` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot_with_completion` | grund-lsp `grund-lsp/src/workspace.rs:63` | — | hidden |
| `LspSnapshotWithMetadata` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot_with_metadata` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspStub` | queries | `#[doc(hidden)]` | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `LspUsage` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | via usage_over_paths → LspUsage `grund-cli/src/cli_refs.rs:150` | hidden |
| `MappedKey` | config | `#[doc(hidden)]` | no | none found | via config_migrate → MigrateOutcome → MigrateReport → MappedKey `grund-cli/src/cli_config_migrate.rs:113` | hidden |
| `merge_outcomes` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `MigrateFailure` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `MigrateNotice` | config | `#[doc(hidden)]` | no | none found | via config_migrate → MigrateOutcome → MigrateReport → MigrateNotice `grund-cli/src/cli_config_migrate.rs:108` | hidden |
| `MigrateOpts` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `MigrateOutcome` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `MigrateRefusal` | config | `#[doc(hidden)]` | no | none found | via config_migrate → MigrateOutcome → MigrateReport → MigrateRefusal `grund-cli/src/cli_config_migrate.rs:89` | hidden |
| `MigrateReport` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:41` | — | hidden |
| `names_member_id_candidate` | resolver | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `NamespaceMatch` | config | `#[doc(hidden)]` | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `NearMissHeading` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `needs_wezterm_wiring` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `node_embedding_call` | api | `#[doc(hidden)]` | related API | grund-node `grund-node/src/lib.rs:7`; oracle `grund-core/examples/grund-binding-oracle.rs:8` | — | hidden |
| `node_request` | api | `#[doc(hidden)]` | related API | oracle `grund-core/examples/grund-binding-oracle.rs:8` | — | hidden |
| `NodeClass` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `on_type_line_edits` | queries | `#[doc(hidden)]` | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `OperationDiagnostic` | model | `#[doc(hidden)]` | no | none found | none found | hidden |
| `parse_cover_line_range` | api | `#[doc(hidden)]` | related API | no reader outside grund-core | — | hidden |
| `PathBase` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli.rs:58` | — | hidden |
| `Placement` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `PointSizeUnit` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `Project` | config | `#[doc(hidden)]` | named | none found | none found | hidden |
| `propose_id` | api | visible | related API | none found | none found | frozen at 1.0 |
| `propose_id_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `query_format_hint` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_show.rs:339`; tests `grund-cli/tests/config_guidance_sections.rs:143` | — | hidden |
| `read_optional_text` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `reads_as_coordinate` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:25` | — | hidden |
| `reads_as_coordinate_glob` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:27` | — | hidden |
| `reference_style` | api | `#[doc(hidden)]` | related API | none found | none found | hidden |
| `ReferenceStyle` | api | `#[doc(hidden)]` | data type of `reference_style` | none found | none found | hidden |
| `RefHit` | api | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `refs` | api | visible | related API | none found | none found | frozen at 1.0 |
| `refs_outcome` | api | visible | related API | none found | none found | frozen at 1.0 |
| `refs_sites_with_run_warnings` | api | `#[doc(hidden)]` | no | no reader outside grund-core | — | hidden |
| `refs_sites_with_text_annotations` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_refs_sites.rs:59` | — | hidden |
| `refs_with_metadata` | api | `#[doc(hidden)]` | related API | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `refs_with_metadata_and_hint` | api | `#[doc(hidden)]` | no | tests `grund-cli/tests/config_guidance_sections.rs:153` | — | hidden |
| `refs_with_text_annotations` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_refs.rs:124` | — | hidden |
| `RefsOpts` | api | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `RefsOutcome` | api | visible | data type of `refs_outcome` | none found | via refs_with_metadata → RefsWithMetadata → RefsOutcome `grund-cli/src/cli_refs.rs:58` | frozen at 1.0 |
| `RefsOutput` | api | visible | data type of `refs` | none found | via refs_with_metadata → RefsWithMetadata → RefsOutcome → RefsOutput `grund-cli/src/cli_refs.rs:58` | frozen at 1.0 |
| `RefsQueryFailure` | api | visible | data type of `refs_outcome` | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `RefsQueryFailureKind` | api | visible | data type of `refs_outcome` | none found | via RefsQueryFailure → RefsQueryFailureKind `grund-cli/src/lib.rs:13` | frozen at 1.0 |
| `RefsSites` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:19` | — | hidden |
| `RefsSitesOpts` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:19` | — | hidden |
| `RefsWithMetadata` | api | `#[doc(hidden)]` | data type of `refs_with_metadata` | none found | via refs_with_text_annotations → RefsWithMetadata `grund-cli/src/cli_refs.rs:124` | hidden |
| `RefusalAnchor` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:42` | — | hidden |
| `RelativeHint` | model | visible | data type of `scan` | none found | none found | frozen at 1.0 |
| `RelativeSectionParent` | model | visible | data type of `scan` | none found | none found | frozen at 1.0 |
| `render_finding_sites_json` | api | `#[doc(hidden)]` | related API | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `Report` | model | visible | named | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `RESOLVER_TARGET` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `Run` | config | `#[doc(hidden)]` | no | none found | none found | hidden |
| `scan` | api | visible | named | none found | none found | frozen at 1.0 |
| `scan_user_config` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `schema_view` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:36` | — | hidden |
| `SchemaEdge` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `SchemaNode` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `SchemaSlot` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `SchemaView` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:35` | — | hidden |
| `SectionHeadingOutsideDeclaration` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `SectionInfo` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `SETUP_CORE_SECTION` | templates | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `setup_fresh_path` | templates | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `setup_instructions` | templates | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `SetupPart` | templates | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `ShorthandPolicy` | config | `#[doc(hidden)]` | data type of `validate_config` | none found | via Config → ShorthandPolicy `grund-cli/src/lib.rs:13` | hidden |
| `show` | api | visible | named | none found | none found | frozen at 1.0 |
| `show_batch_with_globs` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:28` | — | hidden |
| `show_batch_with_scope` | queries | `#[doc(hidden)]` | no | none found | none found | hidden |
| `show_from_snapshot` | api | `#[doc(hidden)]` | related API | grund-lsp `grund-lsp/src/lib.rs:4` | — | hidden |
| `show_with_scope` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `ShowFormat` | queries | visible | data type of `ShowOpts` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | frozen at 1.0 |
| `ShowMode` | queries | visible | data type of `ShowOpts` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | frozen at 1.0 |
| `ShowOpts` | queries | visible | named | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | frozen at 1.0 |
| `ShowOutput` | model | visible | data type of `show` | none found | via show_with_scope → ShowOutput `grund-cli/src/cli_show.rs:166` | frozen at 1.0 |
| `ShowQueryError` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | frozen at 1.0 |
| `ShowSection` | model | visible | data type of `show` | none found | via show_with_scope → ShowOutput → ShowSection `grund-cli/src/cli_show.rs:166` | frozen at 1.0 |
| `Strength` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/tests_schema.rs:8` | — | hidden |
| `UnmarkedHeading` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `usage_clause` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | via lsp_title_hover_body → usage_clause `grund-lsp/src/lib.rs:4` | hidden |
| `usage_over_paths` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | via title_usage → usage_over_paths `grund-lsp/src/lib.rs:4` | hidden |
| `USER_CONFIG_TARGET` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `user_grund_config_path` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `UserConfigScan` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `validate_config` | api | `#[doc(hidden)]` | related API | grund-cli `grund-cli/src/lib.rs:13` | — | hidden |
| `ValueBinding` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `ValueComponent` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `ValueComponentKind` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `ValueRootOrigin` | model | visible | data type of `Catalog` | none found | none found | frozen at 1.0 |
| `VSCODE_EXTENSION_JS` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `vscode_integration_is_current` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `VSCODE_PACKAGE_JSON` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `WEZTERM_APPLY_CALL` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |
| `with_check_input_observer` | model | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli_watch.rs:117` | — | hidden |
| `with_report_path_base` | config | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/cli.rs:74` | — | hidden |
| `write_resolver_script` | writers | visible, `#[deprecated]` | no | none found | none found | is removed in 0.21.0 |

## Removed names

The four aliases noticed in `0.17.0` were removed in `0.19.0`, the release their notes named ([§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), so they have no row above. Each leaves one line in `crates/grund-core/src/lib.rs` that says so.

- `Findings`: read `Catalog`, the record it named.
- `Config`: `effective_project` returns the `Project` it was built from.
- `KindConfig`: a `Project` holds the kinds as `schema` rows.
- `CitationRules`: a `Project` holds them as `rules.citations`.
