// §AR-system.2.2: the model component is one Rust module, so its records are
// declared in `model/` and what crosses the boundary is what `model/mod.rs`
// re-exports (§AR-core-module-layout.1).
mod model;
// §AR-system.2.1: the grammar component is one Rust module too, so the lexical
// facts are declared in `grammar/` and what crosses the boundary is what
// `grammar/mod.rs` re-exports (§AR-core-module-layout.1).
mod grammar;
// §AR-system.2.3: config is one Rust module too, so the effective `Config` and
// the reader that fills it are declared in `config/` and what crosses the
// boundary is what `config/mod.rs` re-exports (§AR-core-module-layout.1).
mod config;
// §AR-system.2.4: workspace is one Rust module too, so member expansion, the
// claims and the loaded project set are declared in `workspace/` and what
// crosses the boundary is what `workspace/mod.rs` re-exports.
mod workspace;
// §AR-system.2.5: the scanner is one Rust module too, so the walk, the per-file
// pass and everything they record are declared in `scanner/` and what crosses
// the boundary is what `scanner/mod.rs` re-exports (§AR-scanner).
mod scanner;
// §AR-system.2.6: the checker is one Rust module too, so every rule and the
// report they fill are declared in `checker/` and what crosses the boundary is
// what `checker/mod.rs` re-exports (§AR-checker).
mod checker;
// §AR-system.2.7: the queries are one Rust module too, so every answer read
// from `Findings` is declared in `queries/` and what crosses the boundary is
// what `queries/mod.rs` re-exports (§FS-show, §FS-list, §FS-lsp).
mod queries;
// §AR-system.2.8: the writers are one Rust module too — the formatter, the ID
// proposal, the init scaffold (§FS-init) and the integrations artifacts
// (§FS-integrations) — and `writers/mod.rs` says what crosses the boundary.
mod writers;
// §AR-system.2.9: the embedding surface is one Rust module too, so the public
// contract and the adapters that fill it are declared in `api/` and what an
// embedder reaches is what `api/mod.rs` re-exports (§AR-bindings.2).
mod api;
// §AR-system.2.9: and its one exception, the deprecated `main_entry()` path,
// which is the only directory here that parses argv, writes to a stream or
// returns an `ExitCode`. It may read anything; nothing may read it.
mod compat;

// The crate's public surface, listed rather than globbed
// (§AR-core-module-layout.1): every name below is reachable as
// `grund_core::<name>` exactly as it was when the crate root was flat.

// A component's own items are private to it; what crosses a boundary is what
// its `mod.rs` re-exports `pub(crate)`, and what an embedder reaches is this
// list (§AR-system.4, §AR-bindings.2).

// §AR-system.2.2 model: the records every component passes along — the findings
// a walk produces, the published report, and the value bindings (§FS-values.2).
pub use model::{
    Citation, Declaration, DeclarationSource, DocCommentBlock, E2eCase, E2eSpecRef,
    EmbeddedValueRoot, FileHeading, FileStructure, Finding, FindingSite, Findings, Id,
    InlineCitationSite, InvalidValueSite, NearMissHeading, Report,
    SectionHeadingOutsideDeclaration, SectionInfo, ShowOutput, ShowSection, UnmarkedHeading,
    ValueBinding, ValueComponent, ValueComponentKind, canonical_snapshot_path,
};

// §AR-system.2.1 grammar: the compiled ID grammar, the one lexical fact an
// embedder names (§FS-config.3.2).
pub use grammar::Grammar;

// §AR-system.2.3 config: the validated `Config` and the `grund.toml` records it
// is read from (§FS-config).
pub use config::{
    AbsentOptionalNamespace, CitationDisjunction, CitationLevel, CitationRules, CitationTarget,
    Config, ConfigLocation, KindCitationRules, KindConfig, KindIndex, KindResolution,
    LeadSizeWarning, NamespaceMatch, PointSizeUnit, ShorthandPolicy,
};

// §AR-system.2.4 workspace: the one refusal shape a frontend asks about
// (§FS-workspace.8.1.1).
pub use workspace::names_member_id_candidate;

// §AR-system.2.5 scanner: the published form of what the walk raises
// (§FS-check.2).
pub use scanner::ApiScanError;

// §AR-system.2.6 checker: the finding-code selection `grund-cli` parses
// `--only` and `--skip` into. `#[doc(hidden)]`, so it is not one of the 132
// documented names, but a frontend reads it (§FS-check.5).
pub use checker::{CHECK_FINDING_CODES, CheckFindingSelection};

// §AR-system.2.7 queries: the answers read straight from `Findings` — the
// editor's snapshot, hover and on-type edits, the show options and their typed
// refusal, the batch show, and the size catalog (§FS-show, §FS-list, §FS-lsp).
pub use queries::{
    BatchShowFailure, BatchShowQuery, BatchShowRecord, DeclaredId, LineEdit, ListSizeEntry,
    ListSizeMeasurement, ListSizeOpts, ListSizeOutput, LspCitation, LspDeclaration,
    LspFindingRange, LspSnapshot, LspSnapshotOpts, LspStub, LspUsage, ShowFormat, ShowMode,
    ShowOpts, ShowQueryError, can_replace_trigger_at, citation_under_title, list_sizes,
    lsp_title_hover_body, on_type_line_edits, show_batch_with_scope,
};

// §AR-system.2.8 writers: the init scaffold with its events and refusals, the
// external fact snapshot, and the formatter's scan abort (§FS-init, §FS-fetch,
// §FS-fmt.3).
pub use writers::{
    AGENT_SETUP_INSTRUCTIONS, FetchFailure, FetchFailureKind, FmtScanAbort,
    InitAgentEntrypointSelection, InitError, InitEvent, InitFsHome, InitNext, InitOpts, InitOutput,
    canonical_template_text, fetch_snapshot, init,
};

// §AR-system.2.9 api: the embedding surface itself — one data-returning
// function per question, with the option and output records around it
// (§AR-bindings.2, §FS-distribution.3).
pub use api::{
    CheckOpts, CheckOutput, CompleteIdsOpts, CoverCitation, CoverEntry, CoverOpts, CoverOutput,
    CoverTextCitation, CoverTextEntry, CoverTextOutput, FmtChange, FmtOpts, FmtOutput, IdOpts,
    IdProposal, IdProposalOutcome, ListEntry, ListOpts, ListOutput, ListSummary, ListValueRoot,
    REFS_QUERY_FAILURE_WARNING, RefHit, ReferenceStyle, RefsOpts, RefsOutcome, RefsOutput,
    RefsQueryFailure, RefsQueryFailureKind, check, check_with_opts, complete_ids, config_warnings,
    cover, cover_text, effective_config, format_references, list, lsp_snapshot, propose_id,
    reference_style, refs, refs_outcome, refs_query_failure_is_exit_one, render_finding_sites_json,
    scan, show, show_with_overlays, show_with_scope, validate_config,
};

// §AR-system.2.9's one exception, the deprecated `main_entry()` path
// §REQ-backwards-compatibility.2 keeps for 0.4 consumers.
#[allow(deprecated)]
pub use compat::main_entry;

// The `warning:` shape both `config` frontends share (§FS-config.4.2), and the
// one renderer `grund-cli` still imports from the engine, which the
// compat-retirement step replaces with a copy of its own (§AR-system.2.9).
pub use compat::{print_config_warnings, run_integrations};

// The crate's own `tests_*` modules are `include!`d into this root and reach
// their vocabulary through `use super::*`, so the std names their fixtures spell
// are imported here rather than in each of them (§AR-core-module-layout.1).
#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::ExitCode;

// ...and so is the crate itself. The 69 test modules below read it **flat**, the
// way they did when `lib.rs` was one `include!` list, so that moving each into
// the component it tests stays a change of its own (§AR-core-module-layout.1).

// These globs are private and `#[cfg(test)]`, so they exist only in a test
// build: production code reads every component by module path and sees only
// what that component re-exports (§AR-system.4).
#[cfg(test)]
use {
    api::*, checker::*, compat::*, config::*, grammar::*, model::*, queries::*, scanner::*,
    workspace::*, writers::*,
};

// Tests, one module per category (§AR-core-module-layout.1). `tests_support`
// holds the fixtures they share and must come first.
include!("tests_support.rs");
include!("tests_config_discovery.rs");
include!("tests_config_scan.rs");
include!("tests_id_grammar.rs");
include!("tests_check_full.rs");
include!("tests_kind_index.rs");
include!("tests_kind_index_config.rs");
include!("tests_non_citable_kinds.rs");
include!("tests_unwalked_kinds.rs");
include!("tests_kind_index_entry_form.rs");
include!("tests_kind_index_enrollment.rs");
include!("tests_check_full_scope.rs");
include!("tests_nothing_recognized.rs");
include!("tests_declaration_near_miss.rs");
include!("tests_duplicate_sections.rs");
include!("tests_section_body_scope.rs");
include!("tests_section_outside_declaration.rs");
include!("tests_comment_block.rs");
include!("tests_comment_block_position.rs");
include!("tests_grounding_style.rs");
include!("tests_grounding_per_place.rs");
include!("tests_grounding_config.rs");
include!("tests_inline_note_layout.rs");
include!("tests_inline_note_layout_check.rs");
include!("tests_scanner.rs");
include!("tests_unmarked_headings.rs");
include!("tests_values.rs");
include!("tests_embedded_values.rs");
include!("tests_embedded_value_boundaries.rs");
include!("tests_scanner_walk.rs");
include!("tests_scanner_walk_roots.rs");
include!("tests_scanner_walk_errors.rs");
include!("tests_shorthand.rs");
include!("tests_shorthand_rewrite.rs");
include!("tests_shorthand_docstring.rs");
include!("tests_shorthand_surfaces.rs");
include!("tests_shorthand_numeric_run.rs");
include!("tests_fmt_suppression.rs");
include!("tests_fmt_workspace.rs");
include!("tests_citation_directions.rs");
include!("tests_citation_directions_render.rs");
include!("tests_managed_block_drift.rs");
include!("tests_check_finding_selection.rs");
include!("tests_workspace.rs");
include!("tests_workspace_message_paths.rs");
include!("tests_workspace_nested.rs");
include!("tests_workspace_claims.rs");
include!("tests_workspace_claim_answers.rs");
include!("tests_workspace_absorbed_scan.rs");
include!("tests_unlisted_workspace_block.rs");
include!("tests_unread_opted_out_block.rs");
include!("tests_alias_hints.rs");
include!("tests_workspace_members.rs");
include!("tests_workspace_optional_members.rs");
include!("tests_cover_workspace.rs");
include!("tests_init_agents.rs");
include!("tests_init_scan_guidance.rs");
include!("tests_init_target.rs");
include!("tests_integrations.rs");
include!("tests_integrations_config.rs");
include!("tests_resolver.rs");
include!("tests_clickable_citations.rs");
include!("tests_api.rs");
include!("tests_refs_query_failures.rs");
include!("tests_external_facts.rs");
include!("tests_fmt_errors.rs");
include!("tests_lsp_hover.rs");
