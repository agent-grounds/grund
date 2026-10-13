//! The warning a configured file root earns when the extension list keeps it out of
//! the walk (§FS-scan.checks.unread-scan-root): built from the rows the root gate
//! recorded (§AR-scanner.1.6), so the report asks no extension question of its own
//! (§AR-scanner.1.13). `grund check` and the LSP snapshot both build it here, so an
//! editor and a terminal over one tree say the same thing (§FS-lsp.4.1).

use std::path::Path;

use crate::config::Config;
use crate::model::{Catalog, Diagnostic, RootOrigin, RootRow};

/// One located warning per unread root, at the line of the key that named it
/// (§FS-scan.checks.unread-scan-root.location). A warning, so the exit code stays
/// and `success` is withheld (§FS-scan.checks.unread-scan-root.channel); it is a
/// finding about the configured scope, so the caller asks it before the empty-scan
/// caution, which it stands in place of (§FS-scan.checks.unread-scan-root.empty-scan).
pub(super) fn unread_root_warnings(config: &Config, findings: &Catalog) -> Vec<Diagnostic> {
    let v2 = config.project().version == 2;
    let mut warnings: Vec<Diagnostic> = findings
        .unread_roots
        .iter()
        .map(|row| Diagnostic {
            code: "unread-scan-root",
            path: Some(row.config.clone()),
            line: Some(row.line),
            column: None,
            message: unread_root_message(row, v2),
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect();
    // §FS-errors.4: two entries of one `include` share its line and sort by message.
    warnings.sort_by(|a, b| (a.line, &a.message).cmp(&(b.line, &b.message)));
    warnings
}

/// `<row> is not read — <reason>, so nothing in it is checked. <fix>.`, in the exact
/// bytes of §FS-scan.checks.unread-scan-root.message.
fn unread_root_message(row: &RootRow, v2: bool) -> String {
    // The part of the file name after its last `.`, as the name spells it.
    let ext = Path::new(&row.written)
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty());
    let label = match (&row.origin, v2) {
        (RootOrigin::Include, _) => format!("[scan] include entry `{}`", row.written),
        (RootOrigin::Home { kind, .. }, false) => {
            format!("[[kinds]] {kind} home `{}`", row.written)
        }
        (RootOrigin::Home { kind, .. }, true) => {
            format!("[schema.kinds.{kind}] file `{}`", row.written)
        }
    };
    let list = if v2 {
        "[schema.sources] languages"
    } else {
        "[scan] extensions"
    };
    let reason = match (ext, v2) {
        (Some(ext), false) => format!("`{ext}` is not in {list}"),
        (Some(ext), true) => format!("no language in {list} reads `.{ext}`"),
        (None, _) => format!("it has no extension, and {list} reads a file only by its extension"),
    };
    // Listing the extension is offered only where there is one to list.
    let mut fixes: Vec<String> = match (&row.origin, ext, v2) {
        (_, _, true) => vec!["Give the file an extension a listed language reads".to_string()],
        (RootOrigin::Include, Some(ext), false) => vec![
            format!("Add \"{ext}\" to [scan] extensions"),
            "drop the entry".to_string(),
        ],
        (RootOrigin::Include, None, false) => vec![
            "Rename the file to an extension the list holds".to_string(),
            "drop the entry".to_string(),
        ],
        (RootOrigin::Home { .. }, Some(ext), false) => vec![
            format!("Add \"{ext}\" to [scan] extensions"),
            "give the file an extension the list holds".to_string(),
        ],
        (RootOrigin::Home { .. }, None, false) => {
            vec!["Rename the file to an extension the list holds".to_string()]
        }
    };
    // §FS-config.3.4.7: a non-citable row may name the file only as a place.
    if matches!(row.origin, RootOrigin::Home { citable: false, .. }) {
        fixes.push("set `scan = false` on the row if it only lists a place".to_string());
    }
    format!(
        "{label} is not read — {reason}, so nothing in it is checked. {}.",
        alternatives(&fixes)
    )
}

/// `a`, `a, or b`, `a, b, or c`: the last alternative alone takes the `or`.
fn alternatives(fixes: &[String]) -> String {
    match fixes {
        [] => String::new(),
        [only] => only.clone(),
        [init @ .., last] => format!("{}, or {last}", init.join(", ")),
    }
}
