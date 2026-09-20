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
//! (§AR-core-module-layout.1.1). Each file keeps the name of the writer it belongs
//! to rather than dropping a prefix, because this component holds five: the
//! `fmt_*` files are the formatter — the rewrite walk, the link pass, the
//! value-binding protection, the workspace pass, the completeness proof and the
//! one refusal it carries out — `id.rs` is the ID proposal, the `fetch*` pair is
//! the snapshot, the `init_*` files are the scaffold (the run and its report,
//! the plan one run makes, the block splice, what this run renders the block
//! from, the workspace-members walk-up, the notes, and what the `<path>`
//! argument is), and the `integrations_*` files are the clickable-citation
//! artifacts (the closed client set, detection, the agent instruction surfaces,
//! the installs and their byte-current probes, and the user configuration).
//!
//! One half of this component is `compat/init.rs`: the deprecated `main_entry()`
//! adapter for the scaffold, which renders inside the engine (§AR-system.2.9.1)
//! and nothing here may import. The line is the one
//! `tests/integration/test_engine_boundary.py` measures — a function that writes
//! to a stream or returns an `ExitCode` is a renderer and went there, so no file
//! under this directory prints. The `integrations` command was the second half
//! and is not here any more: its argv, its bytes and its exit codes are
//! `crates/grund-cli/src/cli_integrations*.rs`, the frontend that owns rendering
//! (§FS-integrations.1.3, §AR-bindings.3), and what it reads of this component is
//! the `pub` block below rather than a renderer `lib.rs` re-exports
//! (§DA-engine-renders-nothing).
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
//! §FS-integrations.4.1. The `[fmt] exclude` glob grammar of §FS-config.3.10.1 and
//! the TOML basic-string escaper went down into `config/`, and `plural` — by way
//! of `checker/` — into `model/text.rs`, each to the lowest component that reads
//! it. What came the other way is the §FS-fmt.6.6 auto-enable pair, out of the
//! deprecated `fmt` adapter: neither of its two callers is the command
//! (§AR-system.2.9.1).
//!
//! `fmt_link_targets.rs` left the same way when §AR-system.2.10 became a
//! component: where a citation's link points is a function of the loaded
//! findings, and the checker's index-entry rule was reading it upward out of
//! this component for the link it compares a page against, so it is
//! `resolver/link_targets.rs` and the link pass reads it downward
//! (§FS-check.3.18, §AR-resolver.placement).
//!
//! Three more left when §AR-system.2.11 became one, all for the same reason: the
//! checker's agent-entrypoint rule (§FS-check.3.5, §AR-checker.2.7) compares an
//! `AGENTS.md` block against a fresh render, and it was reading four items
//! upward out of this component to do it. What a managed block *should say* is
//! a function of config alone, so `init_templates.rs` and
//! `init_citation_directions.rs` are `templates/` (§AR-system.2.11) — the
//! embedded payload, the substitutions and the two generated sections. Which
//! entrypoint files a repository *has* is a probe over the tree, so
//! `init_entrypoints.rs` is `scanner/agent_entrypoints.rs`, one walk the plan
//! here and the checker's companion scan both read. And the two suppressed
//! scopes of §FS-fmt.2.5 went down into `grammar/fmt_suppress.rs` with the
//! cross-reference flattening of `fmt_links.rs`, because recognizing a
//! directive and recognizing a wrapper are lexical and the editor's on-type
//! rule and three readers of a flattened body were reaching sideways for them
//! (§FS-lsp.1.4). What stays here is what decides what a run writes: the plan,
//! the splice, the walk-up, the notes, and the effective config the block is
//! rendered from (`init_render.rs`).

mod fetch;
mod fetch_write;
mod fmt_complete_findings;
mod fmt_error;
mod fmt_links;
mod fmt_local_sections;
mod fmt_rewrite;
mod fmt_shorthand_links;
mod fmt_tree;
mod fmt_value_bindings;
mod fmt_workspace;
mod id;
mod init;
mod init_block;
mod init_guidance;
mod init_notes;
mod init_plan;
mod init_render;
mod init_target;
mod init_workspace_members;
mod integrations_agents;
mod integrations_clients;
mod integrations_detect;
mod integrations_install;
mod integrations_user_config;

pub use fetch::{FetchFailure, FetchFailureKind, fetch_snapshot, fetch_snapshot_with_run_warnings};
pub use fmt_error::FmtScanAbort;
pub use init::{InitError, InitEvent, InitOpts, InitOutput, init};
pub use init_guidance::{InitFsHome, InitNext};
pub use init_plan::InitAgentEntrypointSelection;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use fmt_tree::{FmtRunOpts, auto_cross_refs_for_scope, fmt_tree};
pub(crate) use fmt_workspace::fmt_workspace_projects;
pub(crate) use id::{format_id, slugify_title};

// What `grund integrations` is assembled from: the closed client set with each
// client's artifact, what the environment detects, the agent instruction
// surfaces, the splices a `--write` carries out, and the user configuration.

// The command's argv, its bytes and its exit codes are the CLI's
// (§FS-integrations.1.3, §AR-bindings.3), so what it reads here crosses a crate
// boundary and is public; each name returns data (§FS-distribution.3.1).
pub use integrations_agents::{
    ConversationRendering, ConversationTarget, GLOBAL_AGENT_INSTRUCTION_TARGETS, GlobalAgentTarget,
    LinkSupport, agent_override_table, known_agent, known_agents_list,
};
pub use integrations_clients::{
    GRUND_OPEN_RESOLVER, InstallKind, IntegrationClient, RESOLVER_TARGET, VSCODE_EXTENSION_JS,
    VSCODE_PACKAGE_JSON, expand_target, known_clients_line,
};
pub use integrations_detect::detect_clients;
pub use integrations_install::{
    BlockOutcome, WEZTERM_APPLY_CALL, block_outcome_verb, install_agent_guidance_block,
    install_managed_block, integration_is_current, merge_outcomes, needs_wezterm_wiring,
    vscode_integration_is_current, write_resolver_script,
};
pub use integrations_user_config::{
    USER_CONFIG_TARGET, UserConfigScan, install_reference_key, read_optional_text,
    scan_user_config, user_grund_config_path,
};

// What another component's tests read (§AR-core-module-layout.1.3): the rewrite
// pair, the scaffold, the block render and the terminal snippets. The rest went
// beside their own cases.
#[cfg(test)]
pub(crate) use fmt_links::wrap_markdown_links;
#[cfg(test)]
pub(crate) use fmt_rewrite::{FmtLineOpts, fmt_line};
#[cfg(test)]
pub(crate) use init_guidance::docs_scaffold;
#[cfg(test)]
pub(crate) use init_render::render_agents_append_block_at;
#[cfg(test)]
pub(crate) use integrations_clients::{ITERM2_SNIPPET, KITTY_SNIPPET, WEZTERM_SNIPPET};

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_agent_entrypoints;
#[cfg(test)]
mod tests_id;
#[cfg(test)]
mod tests_init_agents;
#[cfg(test)]
mod tests_init_target;
#[cfg(test)]
mod tests_integrations;
#[cfg(test)]
mod tests_integrations_config;
#[cfg(test)]
mod tests_local_section_citations;
#[cfg(test)]
mod tests_open_resolver;
#[cfg(test)]
mod tests_workspace_members;
