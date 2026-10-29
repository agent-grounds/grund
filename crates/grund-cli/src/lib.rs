//! Top-level dispatch, shared output helpers, and `main_entry`. One file per
//! command follows, in `SUBCOMMANDS` order — the frontend crate is assembled by
//! `include!`, so a command's file is a flat slice of the same crate and needs
//! no `mod`/`use` wiring (§AR-core-module-layout.3). The engine crate is not:
//! `grund-core` is one Rust module per component and splices in nothing
//! (§AR-core-module-layout.1).

// §AR-bindings.3: the `grund` frontend crate owns top-level CLI dispatch.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use grund_core::{
    ApiScanError, BatchShowQuery, CheckOpts, CitationDisjunction, CitationLevel, CitationTarget,
    CompleteIdsOpts, CoverCitation, CoverLinesOpts, CoverOpts, FetchFailureKind, Finding,
    FindingSite, FmtConflictAbort, FmtOpts, FmtScanAbort, IdOpts, IdProposal, IdProposalOutcome,
    InitAgentEntrypointSelection, InitNext, InitOpts, InitOutput, ListEntry, ListOpts,
    ListSizeEntry, ListSizeOpts, NamespaceMatch, PathBase, PointSizeUnit, RefHit, RefsOpts,
    RefsOutput, RefsQueryFailure, RefsSites, RefsSitesOpts, Report, SetupPart, ShowFormat,
    ShowMode, ShowOpts, ShowQueryError, check_coordinate_globs, check_with_run_warnings,
    cites_clause, complete_ids_with_run_warnings, config_run_warnings, config_warnings,
    cover_lines, cover_with_globs, effective_config, effective_project,
    fetch_snapshot_with_run_warnings, format_references, glob_beside_section, guide_sections, init,
    list_sizes, list_with_run_warnings, names_member_id_candidate, propose_id_with_run_warnings,
    reads_as_coordinate, reads_as_coordinate_glob, render_finding_sites_json, setup_fresh_path,
    setup_instructions, show_batch_with_globs, show_with_scope, usage_clause, usage_over_paths,
    validate_config, with_report_path_base,
};
use grund_core::{CheckFindingSelection, Code};

// §AR-config.6.3: the schema view the engine lowers and `grund schema` prints
// (§FS-schema, §DA-engine-renders-nothing).
use grund_core::{
    EdgeTarget, Extent, NodeClass, ScaffoldNote, SchemaEdge, SchemaNode, SchemaSlot, SchemaView,
    schema_text, schema_view,
};

// §FS-config-migrate: the one record `grund config migrate --to 2` returns, which
// this crate renders (§AR-config.8).
use grund_core::{
    Gate, GateResult, ListedComment, MigrateFailure, MigrateOpts, MigrateOutcome, MigrateReport,
    RefusalAnchor, config_migrate,
};

// §FS-config-explain: the one answer `grund config explain` returns, which this
// crate prints (§AR-config.9).
use grund_core::{
    ClauseBy, CodeEnabled, CodeFix, CodeSeverity, ExplainAnswer, ExplainAsk, ExplainCode,
    ExplainKey, ExplainReading, ExplainTable, ExplainUnknown, config_explain,
};

// §FS-integrations.1.3: `grund integrations` is one engine operation
// (§FS-distribution.3.1.3), whose answer this crate prints every byte of
// (§AR-bindings.3).
use grund_core::{
    ConversationRendering, ConversationTarget, IntegrationAgent, IntegrationArtifact,
    IntegrationClient, IntegrationClientStatus, IntegrationEvent, IntegrationNote,
    IntegrationsError, IntegrationsFailure, IntegrationsOpts, IntegrationsOutput, integrations,
};

const SUBCOMMANDS: &[&str] = &[
    "check",
    "show",
    "list",
    "refs",
    "cover",
    "fmt",
    "fetch",
    "id",
    "init",
    "config",
    "schema",
    "guide",
    "completions",
    "integrations",
];

/// Dispatched by `main_entry` but kept out of help and completion: the shell
/// completion helper (§FS-completions.2). `guide`'s old name is gone from grund
/// 0.20.0, so it is no subcommand at all (§FS-init.5.7).
const HIDDEN_SUBCOMMANDS: &[&str] = &["complete"];

// §AR-bindings.3: distributed builds erase private hooks and their arguments.
#[cfg(feature = "test-watch")]
macro_rules! watch_observe {
    ($event:expr) => {
        watch_observe_event($event)
    };
}
#[cfg(not(feature = "test-watch"))]
macro_rules! watch_observe {
    ($event:expr) => {
        ()
    };
}
#[cfg(feature = "test-watch")]
macro_rules! watch_inject_failure {
    ($operation:expr) => {
        watch_inject_failure_event($operation)
    };
}
#[cfg(not(feature = "test-watch"))]
macro_rules! watch_inject_failure {
    ($operation:expr) => {
        Ok::<(), String>(())
    };
}

include!("cli_help.rs");
include!("cli_help_list.rs");
include!("cli_help_refs.rs");
include!("cli_help_check.rs");
include!("cli_help_show.rs");
include!("cli_help_fmt.rs");
include!("cli_help_fetch.rs");
include!("cli_help_config.rs");
include!("cli_help_schema.rs");
include!("cli.rs");
include!("cli_check.rs");
include!("cli_watch.rs");
include!("cli_watch_inputs.rs");
include!("cli_watch_terminal.rs");
include!("cli_watch_signal.rs");
#[cfg(feature = "test-watch")]
include!("cli_watch_observer.rs");
include!("cli_show.rs");
include!("cli_show_batch.rs");
include!("cli_list.rs");
include!("cli_list_render.rs");
include!("cli_refs.rs");
include!("cli_refs_annotations.rs");
include!("cli_refs_sites.rs");
include!("cli_cover.rs");
include!("cli_fmt.rs");
include!("cli_fetch.rs");
include!("cli_id.rs");
include!("cli_init.rs");
include!("cli_config.rs");
include!("cli_config_explain.rs");
include!("cli_config_migrate.rs");
include!("cli_schema.rs");
include!("cli_schema_table.rs");
include!("cli_schema_graph.rs");
include!("cli_complete.rs");
include!("cli_integrations.rs");
include!("cli_integrations_write.rs");
include!("tests_fetch.rs");
include!("tests_integrations.rs");
include!("tests_schema.rs");
