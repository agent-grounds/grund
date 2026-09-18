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

// The fixtures every test module shares — not a component's, because every
// component's tests read them, and imported as `use crate::testing::{…}`
// (§AR-core-module-layout.1).
#[cfg(test)]
pub(crate) mod testing;

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
// embedder names (§FS-config.3.2), and the version a managed integrations block
// is written and read at (§FS-integrations.4.2).
pub use grammar::{Grammar, INTEGRATIONS_BLOCK_VERSION};

// §AR-system.2.3 config: the validated `Config` and the `grund.toml` records it
// is read from (§FS-config).
pub use config::{
    AbsentOptionalNamespace, CitationDisjunction, CitationLevel, CitationRules, CitationTarget,
    Config, ConfigLocation, KindCitationRules, KindConfig, KindIndex, KindResolution,
    LeadSizeWarning, NamespaceMatch, PointSizeUnit, ShorthandPolicy,
};

// §AR-system.2.5 scanner: the published form of what the walk raises
// (§FS-check.2).
pub use scanner::ApiScanError;

// §AR-system.2.10 resolver: the one refusal shape a frontend asks about
// (§FS-workspace.8.1.1).
pub use resolver::names_member_id_candidate;

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

// §AR-system.2.11 templates: the setup skill a command prints byte-for-byte and
// the newline canonicalization every template is read through (§FS-init.5,
// §FS-init.2.1).
pub use templates::{AGENT_SETUP_INSTRUCTIONS, canonical_template_text};

// §AR-system.2.8 writers: the init scaffold with its events and refusals, the
// external fact snapshot, and the formatter's scan abort (§FS-init, §FS-fetch,
// §FS-fmt.3).
pub use writers::{
    FetchFailure, FetchFailureKind, FmtScanAbort, InitAgentEntrypointSelection, InitError,
    InitEvent, InitFsHome, InitNext, InitOpts, InitOutput, fetch_snapshot,
    fetch_snapshot_with_run_warnings, init,
};

// §AR-system.2.8 writers, the clickable-citation artifacts of §FS-integrations:
// the client set, the detection, the agent surfaces and the managed writes. The
// command that renders them is the CLI's (§FS-integrations.1, §AR-bindings.3).
pub use writers::{
    BlockOutcome, ConversationRendering, ConversationTarget, GLOBAL_AGENT_INSTRUCTION_TARGETS,
    GRUND_OPEN_RESOLVER, GlobalAgentTarget, InstallKind, IntegrationClient, LinkSupport,
    RESOLVER_TARGET, USER_CONFIG_TARGET, UserConfigScan, VSCODE_EXTENSION_JS, VSCODE_PACKAGE_JSON,
    WEZTERM_APPLY_CALL, agent_override_table, block_outcome_verb, detect_clients, expand_target,
    install_agent_guidance_block, install_managed_block, install_reference_key,
    integration_is_current, known_agent, known_agents_list, known_clients_line, merge_outcomes,
    needs_wezterm_wiring, read_optional_text, scan_user_config, user_grund_config_path,
    vscode_integration_is_current, write_resolver_script,
};

// §AR-system.2.9 api: the embedding surface itself — one data-returning
// function per question, with the option and output records around it
// (§AR-bindings.2, §FS-distribution.3).
pub use api::{
    CheckOpts, CheckOutput, CompleteIdsOpts, CoverCitation, CoverEntry, CoverOpts, CoverOutput,
    CoverTextCitation, CoverTextEntry, CoverTextOutput, FmtChange, FmtOpts, FmtOutput, IdOpts,
    IdProposal, IdProposalOutcome, ListEntry, ListOpts, ListOutput, ListSummary, ListValueRoot,
    REFS_QUERY_FAILURE_WARNING, RefHit, ReferenceStyle, RefsOpts, RefsOutcome, RefsOutput,
    RefsQueryFailure, RefsQueryFailureKind, check, check_with_opts, complete_ids,
    complete_ids_with_run_warnings, config_run_warnings, config_warnings, cover, cover_text,
    effective_config, format_references, list, list_with_run_warnings, lsp_snapshot, propose_id,
    propose_id_with_run_warnings, reference_style, refs, refs_outcome,
    refs_query_failure_is_exit_one, render_finding_sites_json, scan, show, show_with_overlays,
    show_with_scope, validate_config,
};

// §AR-system.2.9's one exception, the deprecated `main_entry()` path
// §REQ-backwards-compatibility.2 keeps for 0.4 consumers.
#[allow(deprecated)]
pub use compat::main_entry;
