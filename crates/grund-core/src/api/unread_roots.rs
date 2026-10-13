//! The warning a configured file root earns when its hidden name or the extension list
//! keeps it out of the walk (§FS-scan.checks.unread-scan-root): built from the rows and
//! refusals the root gate recorded (§AR-scanner.1.6), so the report asks no name
//! question of its own (§AR-scanner.1.13). `grund check` and the LSP snapshot both build it here, so an
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
pub(super) fn unread_root_message(row: &RootRow, v2: bool) -> String {
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
    let (reason, mut fixes) = if row.refused.hidden {
        hidden_reason_and_fixes(row, ext, v2, list)
    } else {
        extension_reason_and_fixes(row, ext, v2, list)
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

/// The reason, then the fixes, for a root only the extension list refuses: the
/// first two tables of §FS-scan.checks.unread-scan-root.message.
fn extension_reason_and_fixes(
    row: &RootRow,
    ext: Option<&str>,
    v2: bool,
    list: &str,
) -> (String, Vec<String>) {
    let reason = match (ext, v2) {
        (Some(ext), false) => format!("`{ext}` is not in {list}"),
        (Some(ext), true) => format!("no language in {list} reads `.{ext}`"),
        (None, _) => format!("it has no extension, and {list} reads a file only by its extension"),
    };
    // Listing the extension is offered only where there is one to list.
    let fixes: Vec<String> = match (&row.origin, ext, v2) {
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
    (reason, fixes)
}

/// The reason, then the fixes, for a root whose name begins with `.`: the hidden
/// tables of §FS-scan.checks.unread-scan-root.message. The reason names the extension
/// too where the list refuses it as well, and the one fix is a rename that satisfies
/// both rules (§DF-unread-hidden-root.2.1).
fn hidden_reason_and_fixes(
    row: &RootRow,
    ext: Option<&str>,
    v2: bool,
    list: &str,
) -> (String, Vec<String>) {
    let rule = match (row.refused.extension, ext, v2) {
        (false, _, _) => "grund reads no hidden file".to_string(),
        (true, Some(ext), false) => format!("`{ext}` is not in {list}"),
        (true, Some(ext), true) => format!("no language in {list} reads `.{ext}`"),
        (true, None, _) => "it has no extension".to_string(),
    };
    let reason = format!("its name begins with `.`, and {rule}");
    let rename = match (row.refused.extension, v2) {
        (false, _) => "Rename the file to a name that is not hidden",
        (true, false) => {
            "Rename the file to a name that is not hidden and an extension the list holds"
        }
        (true, true) => {
            "Rename the file to a name that is not hidden and an extension a listed language reads"
        }
    };
    let mut fixes = vec![rename.to_string()];
    if row.origin == RootOrigin::Include {
        fixes.push("drop the entry".to_string());
    }
    (reason, fixes)
}

/// `a`, `a, or b`, `a, b, or c`: the last alternative alone takes the `or`.
fn alternatives(fixes: &[String]) -> String {
    match fixes {
        [] => String::new(),
        [only] => only.clone(),
        [init @ .., last] => format!("{}, or {last}", init.join(", ")),
    }
}
