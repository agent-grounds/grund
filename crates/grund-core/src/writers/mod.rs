//! The writers component (§AR-system.2.8): the edits `grund` produces — citation
//! normalization and the cross-reference links of §FS-fmt, the proposed ID of
//! §FS-id, the init scaffold with its managed agent-entrypoint block (§FS-init),
//! the external fact snapshot of §FS-fetch, and the clickable-citation client
//! artifacts of §FS-integrations. They consume `Findings` and the tree, are the
//! only components that write to it, and each writes only what its spec names
//! (§REQ-no-data-loss).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). Each file keeps the name of the writer it belongs
//! to rather than dropping a prefix, because this component holds five: the
//! `fmt_*` files are the formatter — the rewrite walk, the two suppressed
//! scopes, the link pass and its target, the value-binding protection, the
//! workspace pass, the completeness proof and the one refusal it carries out —
//! `id.rs` is the ID proposal, the `fetch*` pair is the snapshot, the `init_*`
//! files are the scaffold (the run and its report, the entrypoint table, the
//! plan one run makes, the block splice, the templates and the two generated
//! sections, the workspace-members walk-up, the notes, and what the `<path>`
//! argument is), and the `integrations_*` files are the clickable-citation
//! artifacts (the closed client set, detection, the agent instruction surfaces,
//! the installs and their byte-current probes, and the user configuration).
//!
//! Two halves of this component are `compat/init.rs` and
//! `compat/integrations*.rs`: the deprecated `main_entry()` adapters, which
//! render inside the engine (§AR-system.2.9) and nothing here may import. The
//! line is the one `tests/integration/test_engine_boundary.py` measures — a
//! function that writes to a stream or returns an `ExitCode` is a renderer and
//! went there, so no file under this directory prints. `run_integrations` is the
//! one exception the published CLI still forces: it is `pub` and imported by
//! `grund-cli`, so `lib.rs` re-exports it until the CLI carries its own copy.
//!
//! What the component does **not** hold any more. Four lexical items went down
//! into `grammar/` (§AR-system.2.1), because a component below was reading each
//! of them upward: the anchor derivation of §DF-github-anchor-fidelity, whole,
//! as `grammar/anchors.rs`; the ID renderer with its qualified form, into
//! `grammar/ids.rs`; the record of one marked citation on a Markdown line,
//! beside them; and the managed-block protocol — finding the block and reading
//! its version — as `grammar/managed_block.rs`, which the checker's
//! agent-entrypoint rule (§FS-check.3.5) now reads downward and which this
//! component had implemented twice, once for §FS-init.2.3 and once for
//! §FS-integrations.4.1. The `[fmt] exclude` glob grammar of §FS-config.3.10 and
//! the TOML basic-string escaper went down into `config/`, and `plural` down
//! into `checker/`, each to the lowest component that reads it. What came the
//! other way is the §FS-fmt.6.6 auto-enable pair, out of the deprecated `fmt`
//! adapter: neither of its two callers is the command (§AR-system.2.9).

mod fetch;
mod fetch_write;
mod fmt_complete_findings;
mod fmt_error;
mod fmt_link_targets;
mod fmt_links;
mod fmt_rewrite;
mod fmt_shorthand_links;
mod fmt_suppress;
mod fmt_value_bindings;
mod fmt_workspace;
mod id;
mod init;
mod init_block;
mod init_citation_directions;
mod init_entrypoints;
mod init_notes;
mod init_plan;
mod init_target;
mod init_templates;
mod init_workspace_members;
mod integrations_agents;
mod integrations_clients;
mod integrations_detect;
mod integrations_install;
mod integrations_user_config;

pub use fetch::{FetchFailure, FetchFailureKind, fetch_snapshot};
pub use fmt_error::FmtScanAbort;
pub use init::{InitError, InitEvent, InitFsHome, InitNext, InitOpts, InitOutput, init};
pub use init_plan::InitAgentEntrypointSelection;
pub use init_templates::{AGENT_SETUP_INSTRUCTIONS, canonical_template_text};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use fmt_link_targets::markdown_link_target;
pub(crate) use fmt_links::flatten_cross_ref_links;
pub(crate) use fmt_rewrite::{FmtRunOpts, auto_cross_refs_for_scope, fmt_tree};
pub(crate) use fmt_suppress::{FmtDirectives, FmtExcluded};
pub(crate) use fmt_workspace::fmt_workspace_projects;
pub(crate) use id::{format_id, slugify_title};
pub(crate) use init_citation_directions::citation_directions_section;
pub(crate) use init_entrypoints::companion_agent_entrypoints;
pub(crate) use init_templates::{ConversationSurface, clickable_citations_section};

// What the deprecated `compat/init.rs` and `compat/integrations*.rs` renderers read,
// through the crate root because nothing here may import them
// (§AR-system.2.9). This block is what the compat task retires.
pub(crate) use integrations_agents::{
    ConversationRendering, ConversationTarget, GLOBAL_AGENT_INSTRUCTION_TARGETS,
    agent_override_table, known_agent, known_agents_list,
};
pub(crate) use integrations_clients::{
    GRUND_OPEN_RESOLVER, InstallKind, IntegrationClient, RESOLVER_TARGET, VSCODE_EXTENSION_JS,
    VSCODE_PACKAGE_JSON, expand_target, known_clients_line,
};
pub(crate) use integrations_detect::detect_clients;
pub(crate) use integrations_install::{
    BlockOutcome, WEZTERM_APPLY_CALL, block_outcome_verb, install_agent_guidance_block,
    install_managed_block, integration_is_current, merge_outcomes, needs_wezterm_wiring,
    vscode_integration_is_current, write_resolver_script,
};
pub(crate) use integrations_user_config::{
    USER_CONFIG_TARGET, install_reference_key, read_optional_text, scan_user_config,
    user_grund_config_path,
};

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// per-line rewrite with its options, which the shorthand-rewrite cases drive
// directly, and the wrapper pass without §FS-fmt.2.4's prebuilt indexes.
#[cfg(test)]
pub(crate) use fmt_links::wrap_markdown_links;
#[cfg(test)]
pub(crate) use fmt_rewrite::{FmtLineOpts, fmt_line};
#[cfg(test)]
pub(crate) use init::{docs_scaffold, init_fs_home};
#[cfg(test)]
pub(crate) use init_block::{AgentsUpdateResult, update_agents_text};
#[cfg(test)]
pub(crate) use init_citation_directions::CITATION_LEVEL_LEGEND;
#[cfg(test)]
pub(crate) use init_entrypoints::{
    AgentEntrypoint, CanonicalSurfaceReach, InitCompanionAgentEntrypoint,
    agents_with_own_entrypoint,
};
#[cfg(test)]
pub(crate) use init_notes::shadowed_claude_entrypoint_note;
#[cfg(test)]
pub(crate) use init_plan::{
    requested_init_companion_agent_entrypoints, workspace_init_companion_agent_entrypoints,
};
#[cfg(test)]
pub(crate) use init_target::{refuse_init_global_instruction_paths, refuse_init_target};
#[cfg(test)]
pub(crate) use init_templates::{
    inline_citation_style_sentence, render_agents_append_block_at, render_agents_md,
    render_grund_toml,
};
#[cfg(test)]
pub(crate) use init_workspace_members::render_workspace_members_section;
#[cfg(test)]
pub(crate) use integrations_agents::LinkSupport;
#[cfg(test)]
pub(crate) use integrations_clients::{ITERM2_SNIPPET, KITTY_SNIPPET, WEZTERM_SNIPPET};
#[cfg(test)]
pub(crate) use integrations_detect::value_names_codium;
#[cfg(test)]
pub(crate) use integrations_install::set_executable;
#[cfg(test)]
pub(crate) use integrations_user_config::{
    conversation_preference, conversation_target_preference, install_conversation_preference,
};
