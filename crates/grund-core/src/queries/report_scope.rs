//! §FS-scan.path: the files a narrowed `list`, `show --batch --all`, `refs` or ID
//! completion reports entries or sites from (§FS-scan.path.show,
//! §FS-scan.path.refs, §FS-completions.2.1). The run has resolved by then: `list`
//! and `show` against the whole owning project, `refs` and completion against what
//! their walk read, the path in one project and each project whole in a workspace
//! (§AR-resolver.3.3). So the path only bounds which declarations and sites print,
//! by the predicate `check <path>` filters its report with (§FS-scan.path.check,
//! §AR-system.principle).

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use crate::checker::{ScanScope, path_report_scope};
use crate::model::{Citation, Declaration, Id, StubResolution};
use crate::resolver::WorkspaceContext;
use crate::sources::paths_same_location;

/// The report scope of a run over `path`: `None` for no path, or one at or above
/// the root of the config the run renders from. A run that loads a workspace
/// renders from the workspace root's, so a path below it and outside every member
/// bounds the members' entries there as it bounds one project's, which is what
/// `check <path>` narrows its report by (§FS-scan.path.list,
/// §FS-check.1.3.6.1).
pub(crate) fn reported_scope(
    context: &WorkspaceContext,
    path: &Path,
    path_provided: bool,
) -> Result<Option<ScanScope>> {
    let config = context.render_config();
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
        || stub_in_scope_declares(scope, decls, |target| {
            target.file == record.file && target.line == record.line
        })
}

/// Whether the run reports `site`, a citation in the project that declares
/// `declarations`: it sits at or under the path, or a stub there stands for the
/// declaration enclosing it, whose citations the stub lends wherever its target is
/// (§FS-scan.path.refs, §FS-declarations.stubs.target.5).
pub(crate) fn reports_site(
    scope: Option<&ScanScope>,
    declarations: &BTreeMap<Id, Vec<Declaration>>,
    site: &Citation,
) -> bool {
    let Some(scope) = scope else {
        return true;
    };
    scope.contains(&site.file)
        || site
            .enclosing_declaration
            .as_ref()
            .and_then(|owner| declarations.get(owner))
            .is_some_and(|decls| {
                stub_in_scope_declares(scope, decls, |target| {
                    paths_same_location(&target.file, &site.file)
                })
            })
}

/// Whether a stub of `decls` at or under the path declares a target record that
/// `stood_for` picks (§FS-declarations.stubs.pairing.4).
fn stub_in_scope_declares(
    scope: &ScanScope,
    decls: &[Declaration],
    stood_for: impl Fn(&Declaration) -> bool,
) -> bool {
    decls.iter().any(|decl| {
        let Some(StubResolution::Declares(records)) = &decl.stub_resolution else {
            return false;
        };
        scope.contains(&decl.file) && records.iter().any(&stood_for)
    })
}
