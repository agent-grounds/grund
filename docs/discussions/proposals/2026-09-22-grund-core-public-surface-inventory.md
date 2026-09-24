# The `grund-core` public root surface, name by name

The evidence [§DISC-grund-core-public-surface](2026-09-22-grund-core-public-surface.md#disc-grund-core-public-surface-what-grund-cores-public-root-surface-is-and-what-it-should-be) argues from. It classifies nothing on its own authority: the counting convention is [§DISC-grund-core-public-surface.3](2026-09-22-grund-core-public-surface.md#3-what-counts-as-a-public-root-name), the columns are [§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries), and `tests/integration/test_public_surface_inventory.py` holds the first column equal to the crate root's export list ([§DISC-grund-core-public-surface.5](2026-09-22-grund-core-public-surface.md#5-what-is-checked)).

Read on the baseline [§DISC-grund-core-public-surface.2](2026-09-22-grund-core-public-surface.md#2-the-audited-baseline) records — `7283e23bb0`, workspace `0.14.2-dev`, `v0.14.1-26-g7283e23bb0`. **182 rows, one per public root name.** If the audit is re-taken on a later commit, that table and every row below move together.

## How a cell reads

Paths are relative to `crates/`, and an evidence location is one site, not every site: the import or call that establishes the consumer, so a reader can start somewhere rather than grep.

**Owner** — the component whose `mod.rs` the item crosses on its way to the root ([§AR-system.4](../../architecture/README.md#4-dependency-direction)).

**Rustdoc** — `visible`, or `#[doc(hidden)]` read from the attribute at the definition. Nothing more: hiding is discoverability, and a hidden symbol still links.

**Spec** — how [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) reaches the name. `named`: its text or its example writes the symbol. `related API`: a Rustdoc-visible function of the `api` component, which [§AR-system.2.9](../../architecture/README.md#29-api) makes the embedding surface and which that point's "and related APIs" reaches. `data type of X`: it appears in the signature of a `named` or `related API` entry point, transitively — one such path is named, not all of them. `no`: outside that closure, so the specification supports embedding it nowhere.

**Consumers** — a textual `use grund_core::…` or `grund_core::…` in `grund-cli`, `grund-lsp` or their test crates. `none found` is the result of searching **this repository** and never a claim about embedders outside it ([§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries)).

**Structural** — for a name no frontend spells: the chain from something one does call, so `via check_with_run_warnings → CheckOutput` means the CLI receives the type without naming it. `—` where the consumer column already answers. `none found` in both columns means no repository consumer of either kind was found.

**Disposition** — the target [§DISC-grund-core-public-surface.6](2026-09-22-grund-core-public-surface.md#6-what-this-discussion-still-has-to-settle) argues for. `keep` changes nothing. `keep, hidden` is an existing `#[doc(hidden)]` seam left as it is. `keep, name in the spec` is a documented data-returning entry point the specification does not yet reach — the gap is the specification's, not the surface's. `hide` proposes `#[doc(hidden)]` with the frontend that reads it named beside it, which retires no symbol. `facade, then retire` is one of the 34 integrations items a coarse entry point would replace, and it is the only disposition that ends in a removal, so it is the only one that owes [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) a notice release.

## What the columns add up to

| Reading | Count |
| --- | --- |
| Public root names | 182 |
| Rustdoc-visible | 169 |
| `#[doc(hidden)]` | 13 |
| Named, a related API, or a data type of one | 104 |
| Outside that closure | 78 |
| Named by a frontend or its tests | 111 |
| Reached structurally only | 33 |
| No repository consumer of either kind | 38 |
| Disposition keep | 106 |
| Disposition keep, hidden | 13 |
| Disposition keep, name in the spec | 16 |
| Disposition hide | 13 |
| Disposition facade, then retire | 34 |

The two count columns do not line up, and they are not meant to: 38 names have no repository consumer while 78 sit outside what the specification reaches, and the two sets overlap only in part. A name can be specification-supported and unused here — that is most of what `scan` returns — or used by a frontend and supported nowhere, which is what a seam is.

## The inventory

| Name | Owner | Rustdoc | Spec | Consumers | Structural | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| `AbsentOptionalNamespace` | config | visible | data type of `validate_config` | none found | via Config → AbsentOptionalNamespace `grund-cli/src/lib.rs:13` | keep |
| `agent_override_table` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `AGENT_SETUP_INSTRUCTIONS` | templates | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | hide |
| `ApiScanError` | scanner | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `BatchShowFailure` | queries | `#[doc(hidden)]` | no | none found | via show_batch_with_scope → BatchShowRecord → BatchShowFailure `grund-cli/src/cli_show_batch.rs:63` | keep, hidden |
| `BatchShowQuery` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `BatchShowRecord` | queries | `#[doc(hidden)]` | no | none found | via show_batch_with_scope → BatchShowRecord `grund-cli/src/cli_show_batch.rs:63` | keep, hidden |
| `block_outcome_verb` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `BlockOutcome` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `can_replace_trigger_at` | queries | visible | no | none found | none found | hide |
| `canonical_snapshot_path` | model | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `canonical_template_text` | templates | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | hide |
| `check` | api | visible | named | grund-lsp tests `grund-lsp/tests/chapter_values.rs:40` | — | keep |
| `CHECK_FINDING_CODES` | checker | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:27` | — | keep, hidden |
| `check_with_opts` | api | visible | related API | none found | none found | keep |
| `check_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `CheckFindingSelection` | checker | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:27` | — | keep, hidden |
| `CheckOpts` | api | visible | data type of `check_with_opts` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CheckOutput` | api | visible | data type of `check_with_opts` | none found | via check_with_run_warnings → CheckOutput `grund-cli/src/cli_check.rs:111` | keep |
| `Citation` | model | visible | data type of `Findings` | none found | none found | keep |
| `citation_under_title` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `CitationDisjunction` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CitationLevel` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CitationRules` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CitationTarget` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `complete_ids` | api | visible | related API | none found | none found | keep |
| `complete_ids_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `CompleteIdsOpts` | api | visible | data type of `complete_ids` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `Config` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `config_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `config_warnings` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `ConfigLocation` | config | visible | data type of `validate_config` | none found | via Config → ConfigLocation `grund-cli/src/lib.rs:13` | keep |
| `ConversationRendering` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `ConversationTarget` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `cover` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `cover_text` | api | visible | related API | none found | none found | keep |
| `CoverCitation` | api | visible | data type of `cover` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CoverEntry` | api | visible | data type of `cover` | grund-cli `grund-cli/src/cli_cover.rs:106` | — | keep |
| `CoverOpts` | api | visible | data type of `cover` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `CoverOutput` | api | visible | data type of `cover` | none found | via cover → CoverOutput `grund-cli/src/cli_cover.rs:56` | keep |
| `CoverTextCitation` | api | visible | data type of `cover_text` | none found | none found | keep |
| `CoverTextEntry` | api | visible | data type of `cover_text` | none found | none found | keep |
| `CoverTextOutput` | api | visible | data type of `cover_text` | none found | none found | keep |
| `Declaration` | model | visible | data type of `Findings` | none found | none found | keep |
| `DeclarationSource` | model | visible | data type of `Findings` | none found | none found | keep |
| `DeclaredId` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `detect_clients` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `DocCommentBlock` | model | visible | data type of `Findings` | none found | none found | keep |
| `E2eCase` | model | visible | data type of `Findings` | none found | none found | keep |
| `E2eSpecRef` | model | visible | data type of `Findings` | none found | none found | keep |
| `effective_config` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/integrations.rs:103` | — | keep |
| `EmbeddedValueRoot` | model | visible | data type of `Findings` | none found | none found | keep |
| `expand_target` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `fetch_snapshot` | writers | visible | no | none found | none found | keep, name in the spec |
| `fetch_snapshot_with_run_warnings` | writers | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `FetchFailure` | writers | visible | no | none found | via fetch_snapshot_with_run_warnings → FetchFailure `grund-cli/src/cli_fetch.rs:8` | keep, name in the spec |
| `FetchFailureKind` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `FileHeading` | model | visible | data type of `Findings` | none found | none found | keep |
| `FileStructure` | model | visible | data type of `Findings` | none found | none found | keep |
| `Finding` | model | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `Findings` | model | visible | named | none found | none found | keep |
| `FindingSite` | model | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `FmtChange` | api | visible | data type of `format_references` | none found | via format_references → FmtOutput → FmtChange `grund-cli/src/cli_fmt.rs:35` | keep |
| `FmtOpts` | api | visible | data type of `format_references` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `FmtOutput` | api | visible | data type of `format_references` | none found | via format_references → FmtOutput `grund-cli/src/cli_fmt.rs:35` | keep |
| `FmtScanAbort` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `format_references` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `GLOBAL_AGENT_INSTRUCTION_TARGETS` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `GlobalAgentTarget` | writers | visible | no | none found | via GLOBAL_AGENT_INSTRUCTION_TARGETS → GlobalAgentTarget `grund-cli/src/lib.rs:32` | facade, then retire |
| `Grammar` | grammar | visible | data type of `validate_config` | none found | via Config → Grammar `grund-cli/src/lib.rs:13` | keep |
| `GRUND_OPEN_RESOLVER` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `Id` | model | visible | data type of `Findings` | none found | none found | keep |
| `IdOpts` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `IdProposal` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `IdProposalOutcome` | api | visible | data type of `propose_id` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `init` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `InitAgentEntrypointSelection` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `InitError` | writers | visible | no | none found | via init → InitError `grund-cli/src/cli_init.rs:64` | keep, name in the spec |
| `InitEvent` | writers | visible | no | none found | via InitOutput → InitEvent `grund-cli/src/lib.rs:13` | keep, name in the spec |
| `InitFsHome` | writers | visible | no | none found | via InitNext → InitFsHome `grund-cli/src/lib.rs:13` | keep, name in the spec |
| `InitNext` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `InitOpts` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `InitOutput` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `InlineCitationSite` | model | visible | data type of `Findings` | none found | none found | keep |
| `install_agent_guidance_block` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `install_managed_block` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `install_reference_key` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `InstallKind` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `integration_is_current` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `IntegrationClient` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `INTEGRATIONS_BLOCK_VERSION` | grammar | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `InvalidValueSite` | model | visible | data type of `Findings` | none found | none found | keep |
| `KindCitationRules` | config | visible | data type of `validate_config` | none found | via CitationRules → KindCitationRules `grund-cli/src/lib.rs:13` | keep |
| `KindConfig` | config | visible | data type of `validate_config` | none found | via Config → KindConfig `grund-cli/src/lib.rs:13` | keep |
| `KindIndex` | config | visible | data type of `validate_config` | none found | via Config → KindConfig → KindIndex `grund-cli/src/lib.rs:13` | keep |
| `KindResolution` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/cli_config.rs:204` | — | keep |
| `known_agent` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `known_agents_list` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `known_clients_line` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `LeadSizeWarning` | config | visible | data type of `validate_config` | none found | via Config → LeadSizeWarning `grund-cli/src/lib.rs:13` | keep |
| `LineEdit` | queries | visible | no | none found | via on_type_line_edits → LineEdit `grund-lsp/src/lib.rs:1329` | hide |
| `LinkSupport` | writers | visible | no | none found | via GLOBAL_AGENT_INSTRUCTION_TARGETS → GlobalAgentTarget → LinkSupport `grund-cli/src/lib.rs:32` | facade, then retire |
| `list` | api | visible | related API | none found | none found | keep |
| `list_sizes` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `list_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `ListEntry` | api | visible | data type of `list` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `ListOpts` | api | visible | data type of `list` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `ListOutput` | api | visible | data type of `list` | none found | via list_with_run_warnings → ListOutput `grund-cli/src/cli_list.rs:204` | keep |
| `ListSizeEntry` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `ListSizeMeasurement` | queries | visible | no | none found | via ListSizeEntry → ListSizeMeasurement `grund-cli/src/lib.rs:13` | keep, name in the spec |
| `ListSizeOpts` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, name in the spec |
| `ListSizeOutput` | queries | visible | no | none found | via list_sizes → ListSizeOutput `grund-cli/src/cli_list.rs:170` | keep, name in the spec |
| `ListSummary` | api | visible | data type of `list` | grund-cli `grund-cli/src/cli_list.rs:238` | — | keep |
| `ListValueRoot` | api | visible | data type of `list` | none found | via ListEntry → ListValueRoot `grund-cli/src/lib.rs:13` | keep |
| `lsp_hover_with_kind_title` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `lsp_snapshot` | api | visible | related API | none found | none found | keep |
| `lsp_snapshot_with_metadata` | api | visible | related API | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `lsp_title_hover_body` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `LspCitation` | queries | visible | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspDeclaration` | queries | visible | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspFindingRange` | queries | visible | data type of `lsp_snapshot` | none found | via LspSnapshot → LspFindingRange `grund-lsp/src/lib.rs:4` | keep |
| `LspSnapshot` | queries | visible | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspSnapshotOpts` | queries | visible | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspSnapshotWithMetadata` | queries | visible | data type of `lsp_snapshot_with_metadata` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspStub` | queries | visible | data type of `lsp_snapshot` | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `LspUsage` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `merge_outcomes` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `names_member_id_candidate` | resolver | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | hide |
| `NamespaceMatch` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `NearMissHeading` | model | visible | data type of `Findings` | none found | none found | keep |
| `needs_wezterm_wiring` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `on_type_line_edits` | queries | visible | no | grund-lsp `grund-lsp/src/lib.rs:4` | — | hide |
| `PointSizeUnit` | config | visible | data type of `validate_config` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `propose_id` | api | visible | related API | none found | none found | keep |
| `propose_id_with_run_warnings` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `read_optional_text` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `reference_style` | api | visible | related API | none found | none found | keep |
| `ReferenceStyle` | api | visible | data type of `reference_style` | none found | none found | keep |
| `RefHit` | api | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `refs` | api | visible | related API | none found | none found | keep |
| `refs_outcome` | api | visible | related API | none found | none found | keep |
| `refs_query_failure_is_exit_one` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `REFS_QUERY_FAILURE_WARNING` | api | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | hide |
| `refs_with_metadata` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `RefsOpts` | api | visible | data type of `refs` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `RefsOutcome` | api | visible | data type of `refs_outcome` | none found | via refs_with_metadata → RefsWithMetadata → RefsOutcome `grund-cli/src/cli_refs.rs:58` | keep |
| `RefsOutput` | api | visible | data type of `refs` | none found | via refs_with_metadata → RefsWithMetadata → RefsOutcome → RefsOutput `grund-cli/src/cli_refs.rs:58` | keep |
| `RefsQueryFailure` | api | visible | data type of `refs_outcome` | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `RefsQueryFailureKind` | api | visible | data type of `refs_outcome` | none found | via RefsQueryFailure → RefsQueryFailureKind `grund-cli/src/lib.rs:13` | keep |
| `RefsWithMetadata` | api | visible | data type of `refs_with_metadata` | none found | via refs_with_metadata → RefsWithMetadata `grund-cli/src/cli_refs.rs:58` | keep |
| `render_finding_sites_json` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `Report` | model | visible | named | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `RESOLVER_TARGET` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `scan` | api | visible | named | none found | none found | keep |
| `scan_user_config` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `SectionHeadingOutsideDeclaration` | model | visible | data type of `Findings` | none found | none found | keep |
| `SectionInfo` | model | visible | data type of `Findings` | none found | none found | keep |
| `ShorthandPolicy` | config | visible | data type of `validate_config` | none found | via Config → ShorthandPolicy `grund-cli/src/lib.rs:13` | keep |
| `show` | api | visible | named | none found | none found | keep |
| `show_batch_with_scope` | queries | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `show_with_overlays` | api | visible | related API | grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `show_with_scope` | api | `#[doc(hidden)]` | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep, hidden |
| `ShowFormat` | queries | visible | data type of `ShowOpts` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `ShowMode` | queries | visible | data type of `ShowOpts` | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `ShowOpts` | queries | visible | named | grund-cli `grund-cli/src/lib.rs:13`; grund-lsp `grund-lsp/src/lib.rs:4` | — | keep |
| `ShowOutput` | model | visible | data type of `show` | none found | via show_with_scope → ShowOutput `grund-cli/src/cli_show.rs:166` | keep |
| `ShowQueryError` | queries | visible | no | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `ShowSection` | model | visible | data type of `show` | none found | via show_with_scope → ShowOutput → ShowSection `grund-cli/src/cli_show.rs:166` | keep |
| `UnmarkedHeading` | model | visible | data type of `Findings` | none found | none found | keep |
| `USER_CONFIG_TARGET` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `user_grund_config_path` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `UserConfigScan` | writers | visible | no | none found | via scan_user_config → UserConfigScan `grund-cli/src/cli_integrations_write.rs:230` | facade, then retire |
| `validate_config` | api | visible | related API | grund-cli `grund-cli/src/lib.rs:13` | — | keep |
| `ValueBinding` | model | visible | data type of `Findings` | none found | none found | keep |
| `ValueComponent` | model | visible | data type of `Findings` | none found | none found | keep |
| `ValueComponentKind` | model | visible | data type of `Findings` | none found | none found | keep |
| `ValueRootOrigin` | model | visible | data type of `Findings` | none found | none found | keep |
| `VSCODE_EXTENSION_JS` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `vscode_integration_is_current` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `VSCODE_PACKAGE_JSON` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `WEZTERM_APPLY_CALL` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
| `write_resolver_script` | writers | visible | no | grund-cli `grund-cli/src/lib.rs:32` | — | facade, then retire |
