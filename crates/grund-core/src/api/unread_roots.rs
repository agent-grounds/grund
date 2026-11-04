//! The warning a configured file root earns when its hidden name or the extension list
//! keeps it out of the walk, and a rule folder earns when the walk read no file of it
//! (§FS-scan.checks.unread-scan-root): built from the rows and refusals the walk
//! recorded (§AR-scanner.1.6), so the report asks no name question of its own
//! (§AR-scanner.1.13). `grund check` and the LSP snapshot both build it here, so an
//! editor and a terminal over one tree say the same thing (§FS-lsp.4.1).

use crate::model::{remedies, spelled_extension};
use std::path::Path;

use crate::config::Config;
use crate::model::{Catalog, Code, Diagnostic, RootOrigin, RootRow};

/// One located warning per unread root or rule folder, at the line of the key that named it
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
    // §FS-errors.4: two entries of one array share its line and sort by message.
    warnings.sort_by(|a, b| (a.line, &a.message).cmp(&(b.line, &b.message)));
    warnings
}

/// `<row> is not read — <reason>, so nothing in it is checked. <fix>.`, in the exact
/// bytes of §FS-scan.checks.unread-scan-root.message.
pub(super) fn unread_root_message(row: &RootRow, v2: bool) -> String {
    let ext = spelled_extension(Path::new(&row.written));
    let label = match (&row.origin, v2) {
        (RootOrigin::Include, _) => format!("[scan] include entry `{}`", row.written),
        (RootOrigin::Home { kind, .. } | RootOrigin::RuleFolder { kind, .. }, false) => {
            format!("[[kinds]] {kind} home `{}`", row.written)
        }
        (RootOrigin::Home { kind, .. }, true) => {
            format!("[schema.kinds.{kind}] file `{}`", row.written)
        }
        (RootOrigin::RuleFolder { kind, .. }, true) => {
            format!("[schema.kinds.{kind}] folder `{}`", row.written)
        }
    };
    let version = if v2 { 2 } else { 1 };
    let list = if v2 {
        "[schema.sources] languages"
    } else {
        "[scan] extensions"
    };
    // §FS-scan.checks.unread-scan-root.message: a folder's reason is what the walk refused
    // in it, never its own name, so `rules.d` does not report `d`.
    let (reason, mut fixes) = if let RootOrigin::RuleFolder { extensions, .. } = &row.origin {
        rule_folder_reason_and_fixes(extensions, v2, list)
    } else if row.refused.hidden {
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
    let fixes: Vec<String> = match (row.origin == RootOrigin::Include, ext, v2) {
        (_, _, true) => vec![remedies::unread_scan_root_language(2)],
        (true, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_drop(1),
        ],
        (true, None, false) => vec![
            remedies::unread_scan_root_rename(1),
            remedies::unread_scan_root_drop(1),
        ],
        (false, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_give(1),
        ],
        (false, None, false) => vec![remedies::unread_scan_root_rename(1)],
    };
    (reason, fixes)
}

/// The reason, then the fixes, for a rule folder, from the extensions the walk refused in
/// it: the folder tables of §FS-scan.checks.unread-scan-root.message. Listing one extension
/// is enough to read a file, so several are offered as alternatives.
fn rule_folder_reason_and_fixes(
    extensions: &[String],
    v2: bool,
    list: &str,
) -> (String, Vec<String>) {
    let quoted = |open: &str, close: &str| -> Vec<String> {
        let each = extensions.iter();
        each.map(|ext| format!("{open}{ext}{close}")).collect()
    };
    let reason = match (extensions.len(), v2) {
        (0, _) => {
            format!("its files have no extension, and {list} reads a file only by its extension")
        }
        (_, true) => format!(
            "no language in {list} reads {}",
            series(&quoted("`.", "`"), "or")
        ),
        (1, false) => format!("{} is not in {list}", series(&quoted("`", "`"), "and")),
        (_, false) => format!("{} are not in {list}", series(&quoted("`", "`"), "and")),
    };
    let fixes = match (extensions, v2) {
        (_, true) => vec![remedies::unread_scan_root_folder_language(2)],
        ([], false) => vec![remedies::unread_scan_root_folder_rename(1)],
        ([only], false) => vec![
            remedies::unread_scan_root_add(1, only),
            remedies::unread_scan_root_folder_give(1),
        ],
        (_, false) => vec![
            remedies::unread_scan_root_add_any(1, series(&quoted("\"", "\""), "or")),
            remedies::unread_scan_root_folder_give(1),
        ],
    };
    (reason, fixes)
}

/// `a`, `a <word> b`, `a, b, <word> c`: the extensions a folder's reason and fix name.
fn series(items: &[String], word: &str) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} {word} {second}"),
        [init @ .., last] => format!("{}, {word} {last}", init.join(", ")),
    }
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
