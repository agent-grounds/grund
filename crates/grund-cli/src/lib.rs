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
    AGENT_SETUP_INSTRUCTIONS, ApiScanError, BatchShowQuery, CheckOpts, CitationDisjunction,
    CitationLevel, CitationRules, CitationTarget, CompleteIdsOpts, Config, CoverCitation,
    CoverOpts, FetchFailureKind, Finding, FindingSite, FmtOpts, FmtScanAbort, IdOpts, IdProposal,
    IdProposalOutcome, InitAgentEntrypointSelection, InitNext, InitOpts, InitOutput, ListEntry,
    ListOpts, ListSizeEntry, ListSizeOpts, NamespaceMatch, PointSizeUnit,
    REFS_QUERY_FAILURE_WARNING, RefHit, RefsOpts, RefsQueryFailure, Report, ShowFormat, ShowMode,
    ShowOpts, ShowQueryError, canonical_template_text, check_with_run_warnings,
    complete_ids_with_run_warnings, config_run_warnings, config_warnings, cover, effective_config,
    fetch_snapshot_with_run_warnings, format_references, init, list_sizes, list_with_run_warnings,
    names_member_id_candidate, propose_id_with_run_warnings, refs_query_failure_is_exit_one,
    refs_with_metadata, render_finding_sites_json, show_batch_with_scope, show_with_scope,
    validate_config,
};
use grund_core::{CHECK_FINDING_CODES, CheckFindingSelection};

// §FS-integrations.1.3: what `grund integrations` is assembled from. The engine
// answers with data — the client set, the detection, the agent surfaces and the
// managed writes — and this crate prints every byte of it (§AR-bindings.3).
use grund_core::{
    BlockOutcome, ConversationRendering, ConversationTarget, GLOBAL_AGENT_INSTRUCTION_TARGETS,
    GRUND_OPEN_RESOLVER, INTEGRATIONS_BLOCK_VERSION, InstallKind, IntegrationClient,
    RESOLVER_TARGET, USER_CONFIG_TARGET, VSCODE_EXTENSION_JS, VSCODE_PACKAGE_JSON,
    WEZTERM_APPLY_CALL, agent_override_table, block_outcome_verb, detect_clients, expand_target,
    install_agent_guidance_block, install_managed_block, install_reference_key,
    integration_is_current, known_agent, known_agents_list, known_clients_line, merge_outcomes,
    needs_wezterm_wiring, read_optional_text, scan_user_config, user_grund_config_path,
    vscode_integration_is_current, write_resolver_script,
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
    "agent-setup-instructions",
    "completions",
    "integrations",
];

include!("cli_help.rs");
include!("cli_help_check.rs");
include!("cli_help_show.rs");
include!("cli.rs");
include!("cli_check.rs");
include!("cli_show.rs");
include!("cli_show_batch.rs");
include!("cli_list.rs");
include!("cli_refs.rs");
include!("cli_cover.rs");
include!("cli_fmt.rs");
include!("cli_fetch.rs");
include!("cli_id.rs");
include!("cli_init.rs");
include!("cli_config.rs");
include!("cli_complete.rs");
include!("cli_integrations.rs");
include!("cli_integrations_write.rs");
include!("tests_integrations.rs");
