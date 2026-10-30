//! Which of a run's loaded projects report their own findings, decided once for
//! `grund check`'s workspace arm (`run.rs`) and the editor's report
//! (`lsp_report.rs`), so a terminal and an editor over one tree cannot disagree
//! about whose lint a run shows (§FS-lsp.4.1). A mounted project, an import, is
//! read for what the run's own citations need, and its own findings are its
//! standalone run's (§FS-imports.findings): it reports no lint, no caution and no
//! member of a block here.

use crate::config::Config;
use crate::resolver::WorkspaceProject;
use crate::workspace::absent_optional_members;
use crate::writers::{ListedProject, listed_projects};

/// The projects whose own findings a run reports, in load order: every project but
/// a mounted one (§FS-imports.findings). Rules that read the whole graph — a
/// qualified citation's target, a scan's completeness — still read every project.
pub(super) fn reporting(
    projects: &[WorkspaceProject],
) -> impl Iterator<Item = &WorkspaceProject> + Clone {
    projects.iter().filter(|project| project.reports_findings())
}

/// §FS-check.3.5.5: the projects a workspace run renders its blocks' workspace
/// sections from, or `None` where it leaves them out — a run at a nested group's
/// root, whose projects carry the group's alias path, and one with an optional
/// member absent anywhere in the tree, the fact `init` keeps the member list on
/// (§FS-init.2.3.4.15.8). The list is the projects already loaded, so the
/// comparison adds no walk. An import is no member, so the block lists none, and a
/// block of imports alone renders as no block (§FS-imports.declaration.table).
pub(super) fn workspace_block_projects(
    root_config: &Config,
    projects: &[WorkspaceProject],
) -> Option<Vec<ListedProject>> {
    let outermost =
        reporting(projects).all(|project| project.config.workspace_scope_path.is_empty());
    let only_imports = root_config.project().workspace.holds_only_imports();
    (outermost && !only_imports && absent_optional_members(root_config).is_empty()).then(|| {
        listed_projects(reporting(projects).map(|project| {
            (
                project.alias.clone(),
                project.config.root.clone(),
                project.config.project_description.clone(),
            )
        }))
    })
}
