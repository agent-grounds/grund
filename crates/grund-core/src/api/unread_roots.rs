//! The warning a configured file root earns when its hidden name or the extension list
//! keeps it out of the walk (§FS-scan.checks.unread-scan-root): built from the rows and
//! refusals the root gate recorded (§AR-scanner.1.6), so the report asks no name
//! question of its own (§AR-scanner.1.13). `grund check` and the LSP snapshot both build it here, so an
//! editor and a terminal over one tree say the same thing (§FS-lsp.4.1).

use crate::model::remedies;
use std::path::Path;

use crate::config::Config;
use crate::model::{Catalog, Code, Diagnostic, RootOrigin, RootRow};

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
            code: Code::UNREAD_SCAN_ROOT,
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
    let version = if v2 { 2 } else { 1 };
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
        fixes.push(remedies::unread_scan_root_place(version));
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
    // Listing the extension is offered only where there is one to list. Each fix is a
    // case of the row's Fix, on the version's side (§FS-errors.5.5.1).
    let fixes: Vec<String> = match (&row.origin, ext, v2) {
        (_, _, true) => vec![remedies::unread_scan_root_language(2)],
        (RootOrigin::Include, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_drop(1),
        ],
        (RootOrigin::Include, None, false) => vec![
            remedies::unread_scan_root_rename(1),
            remedies::unread_scan_root_drop(1),
        ],
        (RootOrigin::Home { .. }, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_give(1),
        ],
        (RootOrigin::Home { .. }, None, false) => vec![remedies::unread_scan_root_rename(1)],
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
    let version = if v2 { 2 } else { 1 };
    let rename = match (row.refused.extension, v2) {
        (false, _) => remedies::unread_scan_root_hidden(version),
        (true, false) => remedies::unread_scan_root_hidden_list(1),
        (true, true) => remedies::unread_scan_root_hidden_language(2),
    };
    let mut fixes = vec![rename];
    if row.origin == RootOrigin::Include {
        fixes.push(remedies::unread_scan_root_drop(version));
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
