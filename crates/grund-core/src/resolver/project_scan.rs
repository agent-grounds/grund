//! One project of a run, loaded and scanned, or only loaded where the run asks which
//! project an alias path names (§AR-resolver.3, §FS-imports.writes.answer).

use anyhow::Result;
use std::path::Path;

use super::context::WorkspaceProject;
use crate::config::{ProjectRecords, display_location};
use crate::model::{Catalog, OperationDiagnostic};
use crate::scanner::{
    ScanError, scan_error_message, scan_tree_with_workspace, unwalked_path_failure,
};
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
/// every project whole and starts at no path (§AR-scanner.1.6). It goes first among the failures
/// of the deepest project that holds the path, else of the first project that reports, so a
/// command that stops at a project's first failure stops at the path, and one that lists them all
/// lists it once. Where no project reports, it is handed back, the run's own failure: a run that
/// dropped it would pass over a tree it never read.
#[must_use = "a failure no project holds is the run's own, and dropping it passes an unread tree"]
pub(crate) fn report_unwalked_path(
    projects: &mut [WorkspaceProject],
    root: &ProjectRecords,
    path: &Path,
    path_provided: bool,
) -> Option<ScanError> {
    let walked = projects.iter().flat_map(|project| &project.scan_errors);
    let failure = unwalked_path_failure(root.schema(), root.frame(), path, path_provided, walked)?;
    let reporting: Vec<usize> = (0..projects.len())
        .filter(|&index| projects[index].reports_findings())
        .collect();
    let holder = reporting
        .iter()
        .copied()
        .filter(|&index| failure.path.starts_with(&projects[index].config.root))
        .max_by_key(|&index| projects[index].config.root.components().count())
        .or(reporting.first().copied());
    let Some(index) = holder else {
        return Some(failure);
    };
    projects[index].scan_errors.insert(0, failure);
    None
}

/// The scan failure a `show` query stops at, with every failure of its project (§AR-scanner.1.6):
/// the failure at the run's `<path>`, wherever it sorts among the failures of whichever project
/// holds it, since a walk that met the directory refusing the lookup lists that directory first;
/// else the first failure of the first `fallback` project that met one. `show` falls back to the
/// ID's project and `show --batch` to every project in order, so the two stop at the path whenever
/// the path is what the run could not read.
pub(crate) fn stopping_failure<'c>(
    projects: &'c [WorkspaceProject],
    fallback: impl IntoIterator<Item = &'c WorkspaceProject>,
) -> Option<(&'c ScanError, &'c [ScanError])> {
    let at_path = projects.iter().find_map(|project| {
        let failures = project.scan_errors.as_slice();
        Some((failures.iter().find(|f| f.at_passed_path())?, failures))
    });
    at_path.or_else(|| {
        let failures = fallback
            .into_iter()
            .map(|project| project.scan_errors.as_slice())
            .find(|failures| !failures.is_empty())?;
        Some((failures.first()?, failures))
    })
}

/// The refusal of a query that stops at `first` (§FS-distribution.3.3.2): a located filesystem
/// outcome with every failure of `all` in its details, rendered against the run's config rather
/// than the failed project's, the spelling `check` uses for the same tree, the base `.`
/// (§FS-workspace.8.7.3, §FS-cli.3.4).
pub(crate) fn scan_failure_refusal(
    render: &ProjectRecords,
    first: &ScanError,
    all: &[ScanError],
) -> OperationDiagnostic {
    let path = display_location(render, &first.path);
    let message = format!("{path}: {}", scan_error_message(first));
    let mut refusal = OperationDiagnostic::new("filesystem", first.code().as_str(), message);
    refusal.path = Some(path);
    let failures = all.iter().map(|failure| {
        serde_json::json!({
            "path": display_location(render, &failure.path),
            "message": scan_error_message(failure),
            "code": failure.code().as_str(),
        })
    });
    refusal.details = serde_json::json!({"scan_errors": failures.collect::<Vec<_>>()});
    refusal
}
