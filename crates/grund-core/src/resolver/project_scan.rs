//! One project of a run, loaded and scanned, or only loaded where the run asks which
//! project an alias path names (§AR-resolver.3, §FS-imports.writes.answer).

use anyhow::Result;

use super::context::WorkspaceProject;
use crate::config::ProjectRecords;
use crate::model::Catalog;
use crate::scanner::{ScanError, scan_tree_with_workspace, unwalked_path_failure};
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

/// The failure at the run's `<path>` that no project's walk raises, because a workspace run walks
/// every project whole and starts at no path (§AR-scanner.1.6), put first among
/// the failures of the project the path lies in, of the deepest one that holds it, else of the
/// first project that reports, so a command that stops at a project's first failure stops at the
/// path, and one that lists them all lists it once.
pub(crate) fn report_unwalked_path(
    projects: &mut [WorkspaceProject],
    root: &ProjectRecords,
    path: &std::path::Path,
    path_provided: bool,
) {
    let walked = projects.iter().flat_map(|project| &project.scan_errors);
    let Some(failure) =
        unwalked_path_failure(root.schema(), root.frame(), path, path_provided, walked)
    else {
        return;
    };
    let reporting: Vec<usize> = (0..projects.len())
        .filter(|&index| projects[index].reports_findings())
        .collect();
    let holder = reporting
        .iter()
        .copied()
        .filter(|&index| failure.path.starts_with(&projects[index].config.root))
        .max_by_key(|&index| projects[index].config.root.components().count())
        .or(reporting.first().copied());
    if let Some(index) = holder {
        projects[index].scan_errors.insert(0, failure);
    }
}
