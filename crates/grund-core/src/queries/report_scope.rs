//! §FS-scan.path: the files a narrowed `list` or `show --batch --all` reports
//! entries from (§FS-scan.path.show). The run already resolved against the whole
//! owning project (`load_reported_workspace_context`), so the path only bounds
//! which declarations print, by the predicate `check <path>` filters its report
//! with (§FS-scan.path.check, §AR-system.principle).

use anyhow::Result;
use std::path::Path;

use crate::checker::{ScanScope, path_report_scope};
use crate::model::{Declaration, StubResolution};
use crate::resolver::WorkspaceContext;

/// The report scope of a run over `path`: `None` for no path, one at or above
/// its config root, or a run that loaded a workspace, whose catalog
/// §FS-workspace.8.3 states (§FS-scan.path.list).
pub(crate) fn reported_scope(
    context: &WorkspaceContext,
    path: &Path,
    path_provided: bool,
) -> Result<Option<ScanScope>> {
    if context.workspace_loaded {
        return Ok(None);
    }
    let Some(project) = context.current_project() else {
        return Ok(None);
    };
    let config = &project.config;
    path_report_scope(config.schema(), config.frame(), path, path_provided)
}

/// Whether the run reports `record`, one of the records `decls` pair to: it is
/// declared at or under the path, or a stub there stands in for it, since a
/// stubbed declaration is named at its record (§FS-scan.path.list,
/// §FS-declarations.stubs.pairing.4).
pub(crate) fn reports(
    scope: Option<&ScanScope>,
    decls: &[Declaration],
    record: &Declaration,
) -> bool {
    let Some(scope) = scope else {
        return true;
    };
    scope.contains(&record.file)
        || decls.iter().any(|decl| {
            let Some(StubResolution::Declares(records)) = &decl.stub_resolution else {
                return false;
            };
            scope.contains(&decl.file)
                && records
                    .iter()
                    .any(|target| target.file == record.file && target.line == record.line)
        })
}
