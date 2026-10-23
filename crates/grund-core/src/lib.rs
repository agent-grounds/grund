//! The engine behind every grund frontend (§FS-distribution.3.1). Grund 1.0
//! freezes the root names §FS-distribution.3.1.2 lists, the visible ones here;
//! a name hidden from these docs is a frontend's seam and outside the promise.
//! An option record is built from `Default` with its fields assigned, and an output record or a refusal is read, never built.
//! Building a promised record with a struct literal, or matching a promised enum without a wildcard arm, became an error in 0.21.0.
//!
//! Since 0.21.0 each block below that is marked not to compile stops at the error its
//! fence names, and the block after it, which differs in one line, is what compiles
//! instead (§FS-distribution.3.1.2, §FS-distribution.3.1.3).
//!
//! The functions the CLI once assembled `grund integrations --write` from are gone, and
//! `integrations` answers what each of them did:
//!
//! ```compile_fail,E0425
//! let detected = grund_core::detect_clients();
//! assert!(detected.len() <= grund_core::IntegrationClient::ALL.len());
//! # Ok::<(), grund_core::IntegrationsError>(())
//! ```
//!
//! ```no_run
//! let detected = grund_core::integrations(grund_core::IntegrationsOpts::default())?.detected;
//! assert!(detected.len() <= grund_core::IntegrationClient::ALL.len());
//! # Ok::<(), grund_core::IntegrationsError>(())
//! ```
//!
//! So are the methods of the kept vocabularies beyond `ALL`, `name` and `from_name`:
//!
//! ```compile_fail,E0599
//! let output = grund_core::integrations(grund_core::IntegrationsOpts::default())?;
//! let status = &output.clients[0];
//! let target = status.client.config_target();
//! assert!(!target.is_empty(), "{} names no target", status.client.name());
//! # Ok::<(), grund_core::IntegrationsError>(())
//! ```
//!
//! ```no_run
//! let output = grund_core::integrations(grund_core::IntegrationsOpts::default())?;
//! let status = &output.clients[0];
//! let target = status.config_target;
//! assert!(!target.is_empty(), "{} names no target", status.client.name());
//! # Ok::<(), grund_core::IntegrationsError>(())
//! ```
//!
//! A promised record is `#[non_exhaustive]`, so it is built from `Default`:
//!
//! ```compile_fail,E0639
//! use grund_core::{ShowMode, ShowOpts};
//! let mut opts = ShowOpts { ..ShowOpts::default() };
//! opts.mode = ShowMode::Toc;
//! assert!(opts.mode == ShowMode::Toc && opts.section.is_none());
//! ```
//!
//! ```
//! use grund_core::{ShowMode, ShowOpts};
//! let mut opts = ShowOpts::default();
//! opts.mode = ShowMode::Toc;
//! assert!(opts.mode == ShowMode::Toc && opts.section.is_none());
//! ```
//!
//! A promised enum is `#[non_exhaustive]` too, so a `match` on one ends in a wildcard arm:
//!
//! ```compile_fail,E0004
//! use grund_core::ShowMode;
//! fn label(mode: ShowMode) -> &'static str {
//!     match mode {
//!         ShowMode::Brief => "brief",
//!         ShowMode::Lead => "lead",
//!         ShowMode::Toc => "toc",
//!         ShowMode::Full => "full",
//!     }
//! }
//! assert_eq!(label(ShowMode::Toc), "toc");
//! ```
//!
//! ```
//! use grund_core::ShowMode;
//! fn label(mode: ShowMode) -> &'static str {
//!     match mode {
//!         ShowMode::Brief => "brief",
//!         ShowMode::Lead => "lead",
//!         ShowMode::Toc => "toc",
//!         ShowMode::Full => "full",
//!         _ => "a mode this release does not name",
//!     }
//! }
//! assert_eq!(label(ShowMode::Toc), "toc");
//! ```

// §AR-system.2.2: the model component is one Rust module, so its records are
// declared in `model/` and what crosses the boundary is what `model/mod.rs`
// re-exports (§AR-core-module-layout.1.1).
mod model;
// §AR-system.2.13: sources is one Rust module, the one reader of the disk, so
// what crosses its boundary is what `sources/mod.rs` re-exports
// (§AR-core-module-layout.1.1).
mod sources;
// §AR-system.2.1: the grammar component is one Rust module too, so the lexical
// facts are declared in `grammar/` and what crosses the boundary is what
// `grammar/mod.rs` re-exports (§AR-core-module-layout.1.1).
mod grammar;
// §AR-system.2.3: config is one Rust module too, so the effective `Config` and
// the reader that fills it are declared in `config/` and what crosses the
// boundary is what `config/mod.rs` re-exports (§AR-core-module-layout.1.1).
mod config;
// §AR-system.2.4: workspace is one Rust module too, so member expansion, the
// claims and the scope they narrow to are declared in `workspace/` and what
// crosses the boundary is what `workspace/mod.rs` re-exports.
mod workspace;
// §AR-system.2.11: templates is one Rust module too, so what a managed block
// should say as a function of config — the payload, the substitutions and the
// two generated sections — is declared in `templates/` (§FS-init.2.3).
mod templates;
// §AR-system.2.5: the scanner is one Rust module too, so the walk, the per-file
// pass and everything they record are declared in `scanner/` and what crosses
// the boundary is what `scanner/mod.rs` re-exports (§AR-scanner).
mod scanner;
// §AR-system.2.10: the resolver is one Rust module too, so the loaded project
// set, the citation-to-target function and the two answers read off a recorded
// span are declared in `resolver/` (§AR-resolver).
mod resolver;
// §AR-rules.placement / §FS-rules.11: controlled-English parsing and
// producer-neutral fact evaluation meet only inside this production component.
mod rules;
// §AR-system.2.6: the checker is one Rust module too, so every rule and the
// report they fill are declared in `checker/` and what crosses the boundary is
// what `checker/mod.rs` re-exports (§AR-checker).
mod checker;
// §AR-system.2.7: the queries are one Rust module too, so every answer read
// from `Catalog` is declared in `queries/` and what crosses the boundary is
// what `queries/mod.rs` re-exports (§FS-show, §FS-list, §FS-lsp).
mod queries;
// §AR-system.2.8: the writers are one Rust module too — the formatter, the ID
// proposal, the init scaffold (§FS-init) and the integrations artifacts
// (§FS-integrations) — and `writers/mod.rs` says what crosses the boundary.
mod writers;
// §AR-system.2.9: the embedding surface is one module; its public contract and
// adapters live in `api/` and cross through `api/mod.rs`. No process frontend
// lives beside it (§AR-system.2.9.1, §AR-bindings.2).
mod api;

// The fixtures every test module shares — not a component's, because every
// component's tests read them, and imported as `use crate::testing::{…}`
// (§AR-core-module-layout.1.2).
#[cfg(test)]
pub(crate) mod testing;

// The crate's public surface, listed rather than globbed
// (§AR-core-module-layout.1.2): every name below is reachable as `grund_core::<name>`.

// A component's own items are private to it; what crosses a boundary is what
// its `mod.rs` re-exports `pub(crate)`, and what an embedder reaches is this
// list (§AR-system.4, §AR-bindings.2).

// §FS-distribution.3.1.2: each component's names come in two groups. The plain
// `pub use` is the 1.0 promise: the entry points and every type they reach.

// The `#[doc(hidden)]` group is the seams a frontend reads, which link and work
// but are no part of it (§DISC-grund-core-public-surface.6.3).

// §AR-system.2.2 model: the records every component passes along — the catalog
// a walk produces, the published report, and the value bindings (§FS-values.2).
pub use model::{
    Catalog, Citation, Declaration, DeclarationSource, DocCommentBlock, E2eCase, E2eSpecRef,
    EmbeddedValueRoot, FileHeading, FileStructure, Finding, FindingSite, Id, InlineCitationSite,
    InvalidValueSite, NearMissHeading, RelativeHint, RelativeSectionParent, Report,
    SectionHeadingOutsideDeclaration, SectionInfo, ShowOutput, ShowSection, UnmarkedHeading,
    ValueBinding, ValueComponent, ValueComponentKind, ValueRootOrigin,
};
// The host packages' refusal record (§FS-distribution.3.3).
#[doc(hidden)]
pub use model::OperationDiagnostic;
// The check code handles rendered from the §FS-errors.5.5 catalog, whose sorted
// list is the selector vocabulary `grund check --help` and the bindings print.
#[doc(hidden)]
pub use model::Code;

// §AR-system.2.13 sources: the one reader of the disk, whose input observer an
// embedder scopes to a check (§AR-bindings.3) and whose snapshot path an
// editor's request is matched by (§AR-lsp.5.1).
#[doc(hidden)]
pub use sources::{
    CheckInput, CheckInputObserver, canonical_snapshot_path, with_check_input_observer,
};

// §AR-system.2.1 grammar: the compiled ID grammar a frontend names (§FS-config.3.2).
#[doc(hidden)]
pub use grammar::Grammar;

// §AR-system.2.3 config: the measurement unit a promised size answer is in, and,
// hidden, the concern records a `grund.toml` lowers to (§AR-config.1), which
// still move (§FS-schema.13).
pub use config::PointSizeUnit;
// §FS-show.3.5.1 / §FS-refs.3.1.1: frontend-only prose and annotations,
// outside the embedding result records.
#[doc(hidden)]
pub use api::{
    query_format_hint, refs_sites_with_text_annotations, refs_with_metadata_and_hint,
    refs_with_text_annotations,
};
#[doc(hidden)]
pub use config::{
    AbsentOptionalNamespace, CitationDisjunction, CitationLevel, CitationTarget, Compiled,
    ConfigLocation, KindCitationRules, KindIndex, KindResolution, LeadSizeWarning, NamespaceMatch,
    Project, Run, ShorthandPolicy,
};
// §FS-cli.3.4: the run's `--path-base`, which a frontend parses and scopes.
#[doc(hidden)]
pub use config::{PathBase, with_report_path_base};
// §AR-config.6.3: the schema view `grund schema` prints, and its parts, hidden
// while the record still moves (§FS-schema.13).
#[doc(hidden)]
pub use api::schema_view;
#[doc(hidden)]
pub use config::{
    BadgeRung, EdgeTarget, Extent, FieldContent, Junction, NodeClass, Placement, SchemaEdge,
    SchemaNode, SchemaSlot, SchemaView, Strength,
};
// §FS-config-migrate: `grund config migrate --to 2` and the one record it returns.
#[doc(hidden)]
pub use api::{MigrateOpts, config_migrate};
// §FS-config-explain: `grund config explain`, the answer it returns, and the
// tables, keys and codes it answers for.
#[doc(hidden)]
pub use api::{config_explain, explain_vocabulary};
#[doc(hidden)]
pub use config::explain::{
    ExplainAnswer, ExplainAsk, ExplainCode, ExplainKey, ExplainTable, ExplainUnknown, Explained,
};
#[doc(hidden)]
pub use config::migrate::{
    Gate, GateResult, ListedComment, MappedKey, MigrateFailure, MigrateNotice, MigrateOutcome,
    MigrateRefusal, MigrateReport, RefusalAnchor,
};
#[doc(hidden)]
pub use model::{ClauseBy, CodeClause, CodeEnabled, CodeSeverity};

// §FS-distribution.3.1, §REQ-backwards-compatibility.2: the four root aliases
// noticed in 0.17.0, each gone on the release its note named.

// `Findings` was removed in 0.19.0; read `Catalog`, the record it named.
// `Config` was removed in 0.19.0; `effective_project` returns the `Project` it was built from.
// `KindConfig` was removed in 0.19.0; a `Project` holds the kinds as `schema` rows.
// `CitationRules` was removed in 0.19.0; a `Project` holds them as `rules.citations`.

// §FS-distribution.3.1.3, §REQ-backwards-compatibility.2: the thirty names the CLI
// once assembled `grund integrations --write` from, and nine methods of the kept
// vocabularies, noticed in 0.19.0 and gone on the release their notes named.

// `agent_override_table` was removed in 0.21.0; set `IntegrationsOpts::agent`.
// `block_outcome_verb` was removed in 0.21.0; use `IntegrationOutcome::name`.
// `BlockOutcome` was removed in 0.21.0; read `IntegrationOutcome` on `IntegrationEvent`.
// `detect_clients` was removed in 0.21.0; read `IntegrationsOutput::detected`.
// `expand_target` was removed in 0.21.0; read the paths on `IntegrationEvent`.
// `GLOBAL_AGENT_INSTRUCTION_TARGETS` was removed in 0.21.0; the agents are `IntegrationAgent::ALL`.
// `GlobalAgentTarget` was removed in 0.21.0; read `IntegrationNote::Guidance`, which names the agent.
// `GRUND_OPEN_RESOLVER` was removed in 0.21.0; read `IntegrationArtifact::resolver`.
// `install_agent_guidance_block` was removed in 0.21.0; call `integrations` with `write` set.
// `install_managed_block` was removed in 0.21.0; call `integrations` with `write` set.
// `install_reference_key` was removed in 0.21.0; set `IntegrationsOpts::conversation_target`.
// `integration_is_current` was removed in 0.21.0; read `IntegrationClientStatus::installed`.
// `INTEGRATIONS_BLOCK_VERSION` was removed in 0.21.0; `integrations` writes the block at its version.
// `known_agent` was removed in 0.21.0; use `IntegrationAgent::from_name`.
// `known_agents_list` was removed in 0.21.0; list `IntegrationAgent::ALL` by name.
// `known_clients_line` was removed in 0.21.0; list `IntegrationClient::ALL` by name.
// `LinkSupport` was removed in 0.21.0; `integrations` applies the gate.
// `merge_outcomes` was removed in 0.21.0; `integrations` reports one `IntegrationEvent` per target.
// `needs_wezterm_wiring` was removed in 0.21.0; read `IntegrationNote::UnwiredWezterm`.
// `read_optional_text` was removed in 0.21.0; `integrations` reads the user configuration.
// `RESOLVER_TARGET` was removed in 0.21.0; read `IntegrationArtifact::resolver_target`.
// `scan_user_config` was removed in 0.21.0; read `IntegrationsOutput::warnings`.
// `USER_CONFIG_TARGET` was removed in 0.21.0; `integrations` reads the user configuration.
// `user_grund_config_path` was removed in 0.21.0; `integrations` reads the user configuration.
// `UserConfigScan` was removed in 0.21.0; `integrations` reads the user configuration.
// `VSCODE_EXTENSION_JS` was removed in 0.21.0; read `IntegrationArtifact::extension_js`.
// `vscode_integration_is_current` was removed in 0.21.0; read `IntegrationClientStatus::installed`.
// `VSCODE_PACKAGE_JSON` was removed in 0.21.0; read `IntegrationArtifact::package_json`.
// `WEZTERM_APPLY_CALL` was removed in 0.21.0; read `IntegrationNote::UnwiredWezterm`.
// `write_resolver_script` was removed in 0.21.0; call `integrations` with `write` set.
// `IntegrationClient::is_terminal` was removed in 0.21.0; read `IntegrationArtifact::resolver`.
// `IntegrationClient::install_kind` was removed in 0.21.0; read `IntegrationClientStatus::install_kind`.
// `IntegrationClient::snippet` was removed in 0.21.0; read `IntegrationArtifact::snippet`.
// `IntegrationClient::config_target` was removed in 0.21.0; read `IntegrationClientStatus::config_target`.
// `IntegrationClient::install_command` was removed in 0.21.0; the install command is the frontend's own.
// `IntegrationClient::comment_prefix` was removed in 0.21.0; call `integrations` with `write` set.
// `IntegrationClient::prepends_block` was removed in 0.21.0; call `integrations` with `write` set.
// `IntegrationClient::fresh_config_scaffold` was removed in 0.21.0; call `integrations` with `write` set.
// `ConversationTarget::accepted_list` was removed in 0.21.0; list `ConversationTarget::ALL` by name.

// §AR-system.2.5 scanner: the published form of what the walk raises
// (§FS-check.2.4).
pub use scanner::ApiScanError;

// §AR-system.2.10 resolver: the one refusal shape a frontend asks about
// (§FS-workspace.8.1.1.7).
#[doc(hidden)]
pub use resolver::names_member_id_candidate;

// §AR-system.2.6 checker: the finding-code selection `grund-cli` parses
// `--only` and `--ignore` into (§FS-check.1.4), and the record of what the run
// armed that it reads to say a selected check was off (§FS-check.1.4.1).
#[doc(hidden)]
pub use checker::{ArmedChecks, CheckFindingSelection};

// §AR-system.2.7 queries: the answers read straight from `Catalog` — the show
// options and their typed refusal and the size catalog (§FS-show, §FS-list) —
// and, hidden, the editor's snapshot, hover, on-type edits and batch show (§FS-lsp).
#[doc(hidden)]
pub use queries::{
    BatchShowFailure, BatchShowQuery, BatchShowRecord, BatchShowRun, CitationCompletion,
    DeclaredId, LineEdit, LspCitation, LspCompletionContext, LspDeclaration, LspFindingRange,
    LspShowContext, LspSnapshot, LspSnapshotOpts, LspSnapshotWithCompletion,
    LspSnapshotWithMetadata, LspStub, LspUsage, can_replace_trigger_at, citation_under_title,
    cites_clause, link_hover_citations, list_sizes_with_run_warnings, lsp_hover_with_kind_title,
    lsp_title_hover_body, on_type_line_edits, show_batch_with_globs, show_batch_with_scope,
    usage_clause, usage_over_paths,
};
pub use queries::{
    ListSizeEntry, ListSizeMeasurement, ListSizeOpts, ListSizeOutput, ShowFormat, ShowMode,
    ShowOpts, ShowQueryError, list_sizes,
};
// §FS-coordinate-globs.errors, §FS-refs.1.2, §FS-refs.1.3: the launch refusal, the
// pattern a refusal offers and the PATH hints a frontend asks for.
#[doc(hidden)]
pub use queries::{
    check_coordinate_globs, glob_beside_section, reads_as_coordinate, reads_as_coordinate_glob,
};

// §AR-system.2.11 templates: the setup skill a command prints byte-for-byte, its
// sections and their front page (§FS-init.5.3, §FS-init.5.5, §FS-init.5.6), and the
// newline canonicalization every template is read through (§FS-init.2.1).
#[doc(hidden)]
pub use templates::{
    AGENT_SETUP_INSTRUCTIONS, AGENT_SETUP_SECTIONS, GuideSection, SETUP_CORE_SECTION, SetupPart,
    canonical_template_text, guide_sections, setup_fresh_path, setup_instructions,
};

// §AR-system.2.8 writers: the init scaffold with its events and refusals, the
// external fact snapshot, and the formatter's scan abort (§FS-init, §FS-fetch,
// §FS-fmt.3).
#[doc(hidden)]
pub use writers::fetch_snapshot_with_run_warnings;
pub use writers::{
    FetchFailure, FetchFailureKind, FmtConflictAbort, FmtScanAbort, InitAgentEntrypointSelection,
    InitAllocation, InitAllocationHome, InitError, InitEvent, InitFsHome, InitNext, InitOpts,
    InitOutput, fetch_snapshot, init,
};

// §AR-system.2.8 writers, `grund integrations` as one operation
// (§FS-distribution.3.1.3): its options, answer and refusal, in the closed
// vocabularies. The CLI renders it (§FS-integrations.1.3, §AR-bindings.3).
pub use writers::{
    ConversationRendering, ConversationTarget, InstallKind, IntegrationAgent, IntegrationArtifact,
    IntegrationClient, IntegrationClientStatus, IntegrationEvent, IntegrationManualStep,
    IntegrationNote, IntegrationOutcome, IntegrationsError, IntegrationsFailure, IntegrationsOpts,
    IntegrationsOutput, integrations,
};
// §AR-system.2.9 api: the embedding surface itself — one data-returning
// function per question, with the option and output records around it
// (§AR-bindings.2, §FS-distribution.3).

// Hidden: the host packages' carriers, the editor's snapshot, the configuration
// readers and the CLI's `*_with_run_warnings` twins.
#[doc(hidden)]
pub use api::{
    ApiFailure, ApiFailureSite, ApiOutcome, ApiPartialOutput, EmbeddingRequest, ReferenceStyle,
    RefsSites, RefsSitesOpts, RefsWithMetadata, check_with_run_warnings,
    complete_ids_with_run_warnings, config_run_warnings, config_warnings,
    cover_lines_with_run_warnings, cover_text_with_run_warnings, cover_with_globs,
    cover_with_run_warnings, effective_config, effective_project, embedding_call,
    format_references_with_run_warnings, list_with_run_warnings, lsp_snapshot,
    lsp_snapshot_with_completion, lsp_snapshot_with_metadata, node_embedding_call, node_request,
    parse_cover_line_range, propose_id_with_run_warnings, reference_style,
    refs_sites_with_run_warnings, refs_with_metadata, render_finding_sites_json,
    show_from_snapshot, show_with_scope, validate_config,
};
pub use api::{
    CheckOpts, CheckOutput, CompleteIdsOpts, CoverCitation, CoverEntry, CoverLineOwner,
    CoverLineSection, CoverLines, CoverLinesOpts, CoverLinesOutput, CoverOpts, CoverOutput,
    CoverTextCitation, CoverTextEntry, CoverTextOutput, FmtChange, FmtOpts, FmtOutput, IdOpts,
    IdProposal, IdProposalOutcome, ListEntry, ListOpts, ListOutput, ListSummary, ListValueRoot,
    RefHit, RefsOpts, RefsOutcome, RefsOutput, RefsQueryFailure, RefsQueryFailureKind, check,
    check_with_opts, complete_ids, cover, cover_lines, cover_text, format_references, list,
    propose_id, refs, refs_outcome, scan, show,
};
// The `test-binding-writes` hooks, outside the promise with their feature.
#[cfg(feature = "test-binding-writes")]
#[doc(hidden)]
pub use writers::{BindingWriteFaultGuard, binding_write_fault};
