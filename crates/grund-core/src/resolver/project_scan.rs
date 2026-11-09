//! One project of a run, loaded and scanned, or only loaded where the run asks which
//! project an alias path names (§AR-resolver.3, §FS-imports.writes.answer).

use anyhow::Result;

use super::context::WorkspaceProject;
use crate::config::ProjectRecords;
use crate::model::Catalog;
use crate::scanner::{ScanError, scan_tree_with_workspace};
use crate::workspace::WorkspaceCitationTarget;

/// One project's scan, or nothing where the run asks only which project an alias
/// path names (§FS-imports.writes.answer).
pub(super) fn scan_project(
    config: &ProjectRecords,
    targets: &[WorkspaceCitationTarget],
    scan: bool,
) -> Result<(Catalog, Vec<ScanError>)> {
    if !scan {
        return Ok(Default::default());
    }
    scan_tree_with_workspace(
        config.schema(),
        config.frame(),
        Some(config.frame().root()),
        true,
        targets,
    )
}

pub(super) fn load_workspace_project(
    alias: String,
    config: ProjectRecords,
    targets: &[WorkspaceCitationTarget],
    scan: bool,
) -> Result<WorkspaceProject> {
    let (findings, scan_errors) = scan_project(&config, targets, scan)?;
    Ok(WorkspaceProject {
        alias,
        config,
        findings,
        scan_errors,
        mount: None,
    })
}
