//! What one `grund init` run feeds the templates (§FS-init.2.3.8): the effective
//! config it will leave governing the target, and the workspace guidance
//! that reads the tree rather than the config (§FS-init.2.3.4.16).
//!
//! The split this file is the writers' half of. What a block *says* is a
//! function of config alone and is `templates/` (§AR-system.2.11); everything
//! here is a decision about this run — which config governs, and a walk-up that
//! must happen exactly once per invocation (§AR-core-module-layout.1).

use anyhow::Result;
use std::path::Path;

use super::init_target::derive_default_name;
use super::init_workspace_members::render_workspace_guidance_with_run_warnings;
use crate::config::{
    AbsentOptionalNamespace, Config, Invocation, Project, Selection, config_file_in, pending_config,
};
use crate::model::Finding;
#[cfg(test)]
use crate::templates::{
    ConversationSurface, render_agents_append_block, render_agents_md_from_block,
};
use crate::templates::{Template, WorkspaceGuidance, render_grund_toml};
use crate::workspace::open;

/// Workspace presence (§FS-init.2.3.4.16) and the member section (§FS-init.2.3.4.15),
/// built here rather than inside the block substitutions so it is built
/// **once**. It does not vary by `ConversationSurface`, and a run that writes
/// both `AGENTS.md` and a `CLAUDE.md` companion renders two blocks from one
/// invocation — so a per-block build would repeat the walk-up's I/O and, worse,
/// ask every block in the workspace twice whether its members swallowed its scan,
/// against §FS-check.3.29's once per block per run.
///
/// Canonical target identity omits self regardless of whether this run selected
/// the canonical `AGENTS.md` or only a companion. The same walk names the
/// optional members absent from the tree, which every entrypoint's write reads
/// (§FS-init.2.3.4.15.8).
pub(super) fn agents_workspace_guidance(
    name: &str,
    config: &Config,
    target: &Path,
    canonical_agent_entrypoint_selected: bool,
) -> (
    WorkspaceGuidance,
    Vec<Finding>,
    Vec<AbsentOptionalNamespace>,
) {
    render_workspace_guidance_with_run_warnings(
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

/// The config and generated name that `grund init` will leave governing
/// `target`, which the generated `AGENTS.md` must describe (§FS-init.2.3.8):
/// `target`'s existing config in either discovery form if there is one
/// (§FS-config.1), otherwise the `grund.toml` `template` renders (§FS-init.1.6),
/// read as `init` is about to write it into `target`, with its *pending*
/// `project_name` and `project_description` (§FS-init.2.4.7). The `pending` in
/// the name flags that the returned `Config` may carry values that are not yet on disk — callers must not treat it as
/// reflecting persisted state. We do **not** walk up to an ancestor's config
/// here — `init` always writes a config *in* `target` when one is absent.
///
/// A config that fails to load is an error, not a fallback to defaults
/// (§FS-init.2.3.8.3): the block is rendered *from* this config, so silently
/// substituting defaults writes agent instructions that describe a repository
/// the user does not have — an invalid `[reference] conversation`, marker, or
/// kind set would drop the guidance it selects while `init` still reported
/// success. `grund check` rejects the same file with exit `2`.
pub(super) fn init_pending_effective_config(
    target: &Path,
    template: Template,
    explicit_name: Option<&str>,
    description: Option<&str>,
) -> Result<(Config, String)> {
    if config_file_in(target).is_some() {
        let config = open(&Invocation::new(target, true, Selection::Discovery))?;
        // §FS-init.2.3.8: every canonical renderer consumes one generated identity:
        // the explicit flag, then target-local config, then the target basename.
        let name = match explicit_name.or(config.project_name.as_deref()) {
            Some(name) => name.to_string(),
            None => derive_default_name(target)?,
        };
        return Ok((config, name));
    }
    let name = match explicit_name {
        Some(name) => name.to_string(),
        None => derive_default_name(target)?,
    };
    // §FS-init.2.4.10: the block describes the file `init` writes, read as it will
    // be read, so a row with no built-in default (`REQ`) reaches the map too.
    let text = render_grund_toml(template, &name, description);
    let invocation = Invocation::new(target, true, Selection::Discovery);
    Ok((pending_config(&text, target, &invocation)?, name))
}

/// The project `grund init --scaffold` would leave at `root`, where no config
/// file is: the default template's `grund.toml`, read as init reads it before
/// writing it (§FS-init.2.4.10). The no-config schema note is derived from it
/// (§FS-schema.8.2), so the note follows the template and names nothing of its own.
pub(crate) fn scaffold_project(root: &Path, name: &str) -> Result<Project> {
    let text = render_grund_toml(Template::default(), name, None);
    let invocation = Invocation::new(root, true, Selection::Discovery);
    Ok(pending_config(&text, root, &invocation)?.project().clone())
}

/// `templates::render_agents_append_block` with the §FS-init.2.3.4.16 discovery
/// and §FS-init.2.3.4.15 section
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
    let (workspace, _, _) =
        agents_workspace_guidance(name, config, target, canonical_agent_entrypoint_selected);
    render_agents_append_block(
        name,
        config.project(),
        config.run(),
        config.compiled(),
        &workspace,
        surface,
    )
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
