//! What one `grund init` run feeds the templates (§FS-init.2.3.8): the effective
//! config it will leave governing the target, and the one managed-block section
//! that reads the tree rather than the config.
//!
//! The split this file is the writers' half of. What a block *says* is a
//! function of config alone and is `templates/` (§AR-system.2.11); everything
//! here is a decision about this run — which config governs, and a walk-up that
//! must happen exactly once per invocation (§AR-core-module-layout.1).

use anyhow::Result;
use std::path::Path;

use super::init_workspace_members::render_workspace_members_section_with_run_warnings;
use crate::config::{Config, config_file_in, load_config};
use crate::model::Finding;
#[cfg(test)]
use crate::templates::{
    ConversationSurface, render_agents_append_block, render_agents_md_from_block,
};

/// The `### Workspace members` section one `init` run renders (§FS-init.2.3.4.15),
/// built here rather than inside the block substitutions so it is built
/// **once**. It does not vary by `ConversationSurface`, and a run that writes
/// both `AGENTS.md` and a `CLAUDE.md` companion renders two blocks from one
/// invocation — so a per-block build would repeat the walk-up's I/O and, worse,
/// ask every block in the workspace twice whether its members swallowed its scan,
/// against §FS-check.4.8's once per block per run.
///
/// Canonical target identity omits self regardless of whether this run selected
/// the canonical `AGENTS.md` or only a companion.
pub(super) fn agents_workspace_members_section(
    name: &str,
    config: &Config,
    target: &Path,
    canonical_agent_entrypoint_selected: bool,
) -> (String, Vec<Finding>) {
    render_workspace_members_section_with_run_warnings(
        target,
        Some(name),
        // Collect effective pending metadata. The renderer omits self (and its
        // description); the pending name still participates in alias validation
        // (§FS-init.2.3.4.15.4).
        config.project_description.as_deref(),
        config.marker.as_str(),
        canonical_agent_entrypoint_selected,
    )
}

/// The config that `grund init` will leave governing `target`, which the generated
/// `AGENTS.md` must describe (§FS-init.2.3.8): `target`'s existing config in either
/// discovery form if there is one (§FS-config.1), otherwise the defaults plus the
/// *pending* `project_name` and `project_description` that `init` is about to
/// write into `target/grund.toml` (§FS-init.2.4.7). The `pending` in the name flags
/// that the returned `Config` may carry values that are not yet on disk —
/// callers must not treat it as reflecting persisted state. We do **not** walk
/// up to an ancestor's config here — `init` always writes a config *in*
/// `target` when one is absent.
///
/// A config that fails to load is an error, not a fallback to defaults
/// (§FS-init.2.3.8): the block is rendered *from* this config, so silently
/// substituting defaults writes agent instructions that describe a repository
/// the user does not have — an invalid `[reference] conversation`, marker, or
/// kind set would drop the guidance it selects while `init` still reported
/// success. `grund check` rejects the same file with exit `2`.
pub(super) fn init_pending_effective_config(
    target: &Path,
    name: &str,
    description: Option<&str>,
) -> Result<Config> {
    if config_file_in(target).is_some() {
        load_config(target)
    } else {
        let mut config = Config::default_for(target.to_path_buf());
        config.project_name = Some(name.to_string());
        config.project_description = description.map(str::to_string);
        Ok(config)
    }
}

/// `templates::render_agents_append_block` with the §FS-init.2.3.4.15 section
/// rendered for it — the shape `command_init` had before the section was hoisted
/// out of the substitutions, kept for the tests that render one block from a
/// target and have no second surface for the walk-up to be repeated by.
#[cfg(test)]
pub(crate) fn render_agents_append_block_at(
    name: &str,
    config: &Config,
    target: &Path,
    canonical_agent_entrypoint_selected: bool,
    surface: ConversationSurface,
) -> String {
    let (workspace_members, _) =
        agents_workspace_members_section(name, config, target, canonical_agent_entrypoint_selected);
    render_agents_append_block(name, config, &workspace_members, surface)
}

/// The full generated `AGENTS.md` for a fresh repo — the H1 scaffolding line
/// followed by the managed block (§FS-init.2.3.10). The H1 is *unmanaged* — `init`
/// owns the block, not the title. Deterministic: same `grund` version, same
/// `--name`, same effective config, same workspace state ⇒ byte-identical
/// output (§FS-non-goals.13).
#[cfg(test)]
pub(crate) fn render_agents_md(
    name: &str,
    config: &Config,
    target: &Path,
    canonical_agent_entrypoint_selected: bool,
) -> String {
    let block = render_agents_append_block_at(
        name,
        config,
        target,
        canonical_agent_entrypoint_selected,
        ConversationSurface::Plain,
    );
    render_agents_md_from_block(name, &block)
}
