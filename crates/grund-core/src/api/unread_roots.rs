//! The warning a configured root earns when no file in it is read: the recorded file-name
//! refusal or folder evidence (§FS-scan.checks.unread-scan-root), explained by the
//! configuration's one reading (§FS-scan.checks.unread-scan-root). `grund check`
//! and the LSP snapshot both build it here (§FS-lsp.4.1).

use crate::model::remedies;
use std::path::Path;

use crate::config::{Config, Form, builtin_text_for_suffix, can_hold_markdown, file_extension};
use crate::model::{Catalog, Code, Diagnostic, RootOrigin, RootRow, unread_reason};

/// One located warning per unread root, at the line of the key that named it
/// (§FS-scan.checks.unread-scan-root). A warning, so the exit code stays
/// and `success` is withheld (§FS-scan.checks.unread-scan-root); it is a
/// finding about the configured scope, so the caller asks it before the empty-scan
/// caution, which it stands in place of (§FS-scan.checks.unread-scan-root).
pub(super) fn unread_root_warnings(config: &Config, findings: &Catalog) -> Vec<Diagnostic> {
    let v2 = config.project().version == 2;
    let readers = Readers::of(config);
    let mut warnings: Vec<Diagnostic> = findings
        .unread_roots
        .iter()
        .map(|row| Diagnostic {
            code: Code::UNREAD_SCAN_ROOT,
            path: Some(row.config.clone()),
            line: Some(row.line),
            column: None,
            message: {
                let path = config.root.join(refused_file(row));
                let reading = config.reading(&path);
                let kind = match &row.origin {
                    RootOrigin::Home { kind, .. } | RootOrigin::FolderHome { kind, .. } => {
                        config.schema().rows.iter().find(|r| r.name == *kind)
                    }
                    RootOrigin::Include => None,
                };
                let value = kind
                    .is_some_and(|row| config.schema().value_form_repairs.0.contains(&row.name));
                let rule = kind
                    .and_then(|row| row.kind.as_ref())
                    .is_some_and(|kind| kind.form == Form::Rule);
                root_message(row, v2, reading.data_language(), value, rule, readers)
            },
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect();
    // §FS-errors.4: two entries of one `include` share its line and sort by message.
    warnings.sort_by(|a, b| (a.line, &a.message).cmp(&(b.line, &b.message)));
    warnings
}

/// `<row> is not read — <reason>, so nothing in it is checked. <fix>.`, in the exact
/// bytes of §FS-scan.checks.unread-scan-root.
#[cfg(test)]
pub(super) fn unread_root_message(row: &RootRow, v2: bool) -> String {
    root_message(
        row,
        v2,
        None,
        false,
        false,
        Readers {
            document: true,
            text: true,
        },
    )
}

/// Repairs ask the actual admitted readers (§FS-scan.checks.unread-scan-root). No
/// repair drops an `md` entry: one that hides Markdown is refused at load
/// (§FS-config-v2.checks.config-md-not-markdown), so a `.md` file is read as Markdown or not
/// at all, never as data.
#[derive(Clone, Copy)]
struct Readers {
    document: bool,
    text: bool,
}

impl Readers {
    fn of(config: &Config) -> Self {
        let Some(model) = config.schema().sources.model.as_ref() else {
            return Self {
                document: true,
                text: true,
            };
        };
        Self {
            document: can_hold_markdown(model),
            text: !model.text_suffixes().is_empty(),
        }
    }
}

/// §FS-scan.checks.unread-scan-root: a folder's reason names the file the walk refused.
fn refused_file(row: &RootRow) -> &Path {
    match &row.origin {
        RootOrigin::FolderHome { first_refused, .. } => first_refused,
        _ => Path::new(&row.written),
    }
}

/// The one message builder, with the reading and row facts supplied by the configuration
/// (§FS-scan.checks.unread-scan-root).
fn root_message(
    row: &RootRow,
    v2: bool,
    data: Option<&str>,
    value: bool,
    rule: bool,
    readers: Readers,
) -> String {
    // The part of the file name after its last `.`, as the name spells it.
    let ext = file_extension(refused_file(row));
    let label = match (&row.origin, v2) {
        (RootOrigin::Include, _) => format!("[scan] include entry `{}`", row.written),
        (RootOrigin::Home { kind, .. }, false) => {
            format!("[[kinds]] {kind} home `{}`", row.written)
        }
        (RootOrigin::Home { kind, .. }, true) => {
            format!("[schema.kinds.{kind}] file `{}`", row.written)
        }
        (RootOrigin::FolderHome { kind, .. }, false) => {
            format!("[[kinds]] {kind} home `{}`", row.written)
        }
        (RootOrigin::FolderHome { kind, .. }, true) => {
            format!("[schema.kinds.{kind}] folder `{}`", row.written)
        }
    };
    let version = if v2 { 2 } else { 1 };
    let (reason, mut fixes) = if row.refused.hidden {
        hidden_reason_and_fixes(row, ext, version, data, rule, readers)
    } else if matches!(row.origin, RootOrigin::FolderHome { .. }) {
        folder_reason_and_fixes(row, ext, version, data, value, rule, readers)
    } else {
        extension_reason_and_fixes(row, ext, version, data, value, rule, readers)
    };
    // §FS-config.3.4.7: a non-citable row may name the file only as a place.
    if matches!(
        row.origin,
        RootOrigin::Home { citable: false, .. } | RootOrigin::FolderHome { citable: false, .. }
    ) {
        fixes.push(remedies::unread_scan_root_place(version));
    }
    format!(
        "{label} is not read — {reason}, so nothing in it is checked. {}.",
        alternatives(&fixes)
    )
}

/// The reason, then the fixes, for a root only the extension list refuses: the
/// first two tables of §FS-scan.checks.unread-scan-root.
fn extension_reason_and_fixes(
    row: &RootRow,
    ext: Option<&str>,
    version: u32,
    data: Option<&str>,
    value: bool,
    rule: bool,
    readers: Readers,
) -> (String, Vec<String>) {
    let reason = unread_reason(row.refused, ext, version, data);
    if data.is_some() {
        return (reason, data_fixes(version, value, false, readers));
    }
    // Listing the extension is offered only where there is one to list. Each fix is a
    // case of the row's Fix, on the version's side (§FS-errors.5.5.1).
    let fixes: Vec<String> = match (&row.origin, ext, version == 2) {
        (_, _, true) => vec![text_repair(2, false, rule, readers)],
        (RootOrigin::Include, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_drop(1),
        ],
        (RootOrigin::Include, None, false) => vec![
            remedies::unread_scan_root_rename(1),
            remedies::unread_scan_root_drop(1),
        ],
        (_, Some(ext), false) => vec![
            remedies::unread_scan_root_add(1, ext),
            remedies::unread_scan_root_give(1),
        ],
        (_, None, false) => vec![remedies::unread_scan_root_rename(1)],
    };
    (reason, fixes)
}

/// The reason, then the fixes, for a root whose name begins with `.`: the hidden
/// tables of §FS-scan.checks.unread-scan-root. The reason names the extension
/// too where the list refuses it as well, and the one fix is a rename that satisfies
/// both rules (§DF-unread-hidden-root.2.1).
fn hidden_reason_and_fixes(
    row: &RootRow,
    ext: Option<&str>,
    version: u32,
    data: Option<&str>,
    rule: bool,
    readers: Readers,
) -> (String, Vec<String>) {
    let reason = unread_reason(row.refused, ext, version, data);
    let rename = match (row.refused.extension, version == 2) {
        (true, true)
            if !(if data.is_some() || rule {
                readers.document
            } else {
                readers.text
            }) =>
        {
            markdown_repair(2, false, true)
        }
        (true, true) if data.is_some() => remedies::unread_scan_root_hidden_document(2),
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

/// The data-row repairs preserve authored forms and fetches, and admit a Markdown reader
/// before suggesting a rename to one (§FS-scan.checks.unread-scan-root).
fn data_fixes(version: u32, value: bool, folder: bool, readers: Readers) -> Vec<String> {
    if !readers.document {
        let mut repair = markdown_repair(version, folder, false);
        if value {
            repair.replace_range(..1, &repair[..1].to_ascii_lowercase());
            repair = remedies::unread_scan_root_value_repair(version, repair);
        }
        return vec![repair];
    }
    if value {
        return vec![if folder {
            remedies::unread_scan_root_folder_value(version)
        } else {
            remedies::unread_scan_root_value(version)
        }];
    }
    vec![if folder {
        remedies::unread_scan_root_folder_document(version)
    } else {
        remedies::unread_scan_root_document(version)
    }]
}

/// The first refused file's reason with the folder-specific repairs
/// (§FS-scan.checks.unread-scan-root).
fn folder_reason_and_fixes(
    row: &RootRow,
    ext: Option<&str>,
    version: u32,
    data: Option<&str>,
    value: bool,
    rule: bool,
    readers: Readers,
) -> (String, Vec<String>) {
    let reason = unread_reason(row.refused, ext, version, data);
    let fixes = if data.is_some() {
        data_fixes(version, value, true, readers)
    } else if version == 1 {
        vec![match ext {
            Some(ext) => remedies::unread_scan_root_add(1, ext),
            None => remedies::unread_scan_root_folder_rename(1),
        }]
    } else {
        let mut fixes = Vec::new();
        if let Some(language) = ext.and_then(|ext| builtin_text_for_suffix(ext, rule)) {
            fixes.push(if rule {
                remedies::unread_scan_root_folder_add_document(2, language)
            } else {
                remedies::unread_scan_root_folder_add(2, language)
            });
        } else {
            fixes.push(text_repair(2, true, rule, readers));
        }
        fixes
    };
    (reason, fixes)
}

/// File and folder repairs require the same reader class for the authored form
/// (§FS-scan.checks.unread-scan-root).
fn text_repair(version: u32, folder: bool, rule: bool, readers: Readers) -> String {
    if !(if rule { readers.document } else { readers.text }) {
        return markdown_repair(version, folder, false);
    }
    match (folder, rule) {
        (true, true) => remedies::unread_scan_root_folder_document(version),
        (true, false) => remedies::unread_scan_root_folder_language(version),
        (false, true) => remedies::unread_scan_root_document(version),
        (false, false) => remedies::unread_scan_root_language(version),
    }
}

/// Admit a usable document reader before the rename, preserving existing languages
/// (§FS-scan.checks.unread-scan-root).
fn markdown_repair(version: u32, folder: bool, hidden: bool) -> String {
    let rename = if hidden {
        "rename the file to a name that is not hidden and an extension it reads"
    } else if folder {
        "give the files in it an extension it reads"
    } else {
        "give the file an extension it reads"
    };
    remedies::unread_scan_root_document_admit(version, rename)
}
