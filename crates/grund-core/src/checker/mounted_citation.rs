//! §FS-imports.findings: what a consumer citation earns from a mount whose own findings
//! are suppressed. A duplicate the mount declares is reported nowhere inside it, so the
//! citation it breaks fails at the consumer's own site rather than resolving to one of
//! the two (§FS-remote-projects.differences.consumer-site).

use crate::model::{CheckReport, Citation, Code, Diagnostic};
use crate::resolver::WorkspaceCheckTarget;

/// Reports the consumer-site form of a duplicate in a mount and says whether it did:
/// not where `target` is no mount or declares the cited ID once.
pub(super) fn report_mounted_duplicate(
    cite: &Citation,
    target: &WorkspaceCheckTarget<'_>,
    report: &mut CheckReport,
) -> bool {
    let Some(duplicate) = mounted_duplicate(cite, target) else {
        return false;
    };
    report.errors.push(duplicate);
    true
}

/// The duplicate's line, which names the citation and the namespace that declares its
/// ID more than once.
fn mounted_duplicate(cite: &Citation, target: &WorkspaceCheckTarget<'_>) -> Option<Diagnostic> {
    target.run.workspace.mounted_root.as_ref()?;
    let count = target.catalog.declarations.get(&cite.id)?.len();
    let namespace = cite.namespace.as_deref().unwrap_or_default();
    (count > 1).then(|| Diagnostic {
        code: Code::DUPLICATE,
        path: Some(cite.file.clone()),
        line: Some(cite.line),
        column: Some(cite.column),
        message: format!(
            "{} names a duplicate declaration: `{namespace}` declares it {count} times",
            cite.text.trim()
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    })
}
