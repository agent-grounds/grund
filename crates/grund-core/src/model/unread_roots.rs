//! Configured roots the scan did not read (§FS-scan.checks.unread-scan-root): a file's
//! name refusals, or a folder's first eligible refused file when neither pass read anything
//! in it (§FS-scan.checks.unread-scan-root.folder). The record names the configured row,
//! and the shared reason names its reader (§FS-scan.checks.unread-scan-root.message).

use std::path::PathBuf;

/// Why the configuration reads no file by this extension, or by a name with none: the
/// one wording of §FS-scan.checks.unread-scan-root.message, cell by cell, for the warning
/// and the broken-stub line of §FS-declarations.stubs.target.3 alike.
pub(crate) fn unread_name_reason(
    extension: Option<&str>,
    version: u32,
    data_language: Option<&str>,
) -> String {
    let list = if version == 2 {
        "[schema.sources] languages"
    } else {
        "[scan] extensions"
    };
    if let (Some(ext), 2, Some(language)) = (extension, version, data_language) {
        return format!(
            "the scan reads `.{ext}` as `{language}`, a data language only a value kind reads"
        );
    }
    match (extension, version == 2) {
        (Some(ext), false) => format!("extension {ext} is not in {list}"),
        (Some(ext), true) => format!("no language in {list} reads `.{ext}`"),
        (None, _) => format!("it has no extension, and {list} reads a file only by its extension"),
    }
}

/// Why the scan reads no file at a name the name rules refuse: the hidden reason, naming the
/// extension too where the list refuses it as well, or the extension reason alone
/// (§FS-scan.checks.unread-scan-root.message, §DF-unread-hidden-root.2.1). `check`'s unread
/// warning and `fetch`'s refusal of an unread home both give it (§FS-fetch.4.2.1), so a
/// correction to it moves both.
pub(crate) fn unread_reason(
    refused: Refused,
    extension: Option<&str>,
    version: u32,
    data_language: Option<&str>,
) -> String {
    if !refused.hidden {
        return unread_name_reason(extension, version, data_language);
    }
    let rule = match (refused.extension, extension) {
        (false, _) => "grund reads no hidden file".to_string(),
        (true, Some(_)) => unread_name_reason(extension, version, data_language),
        (true, None) => "it has no extension".to_string(),
    };
    format!("its name begins with `.`, and {rule}")
}

/// The configuration row that made a file or folder a scan root, and where it was written:
/// the `grund.toml` and the line of the key that named the file
/// (§FS-scan.checks.unread-scan-root.location).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootRow {
    pub(crate) origin: RootOrigin,
    /// The file as the configuration wrote it (§FS-errors.4).
    pub(crate) written: String,
    pub(crate) config: PathBuf,
    pub(crate) line: usize,
    /// The name rules that refused the file, which the root gate fills in once it
    /// has refused it; a row read from the configuration alone has refused nothing.
    pub(crate) refused: Refused,
}

/// The name rules that keep a file root out of the walk: one root refused by both
/// is one record naming both (§DF-unread-hidden-root.2.1).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Refused {
    /// Its name begins with `.`: the hidden reason of
    /// §FS-scan.checks.unread-scan-root.message.
    pub(crate) hidden: bool,
    /// Its extension is not read for declarations, including a data language's reading:
    /// the extension/language reason of §FS-scan.checks.unread-scan-root.message.
    pub(crate) extension: bool,
}

/// Which key made a file a scan root (§FS-scan.checks.unread-scan-root.message).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RootOrigin {
    /// An `[scan] include` entry.
    Include,
    /// A row's `file` home, or a version-2 `files` entry: the row's name, and
    /// whether it declares IDs, which decides whether `scan = false` is offered.
    Home { kind: String, citable: bool },
    /// A configured folder home, carrying the first eligible refused file from the walk
    /// (§FS-scan.checks.unread-scan-root.folder).
    FolderHome {
        kind: String,
        citable: bool,
        home: PathBuf,
        first_refused: PathBuf,
    },
}

impl RootRow {
    /// Whether a non-hidden spelling sits in this folder home. Eligibility and a successful
    /// read of its identity are joined by the scanner (§FS-scan.checks.unread-scan-root.folder).
    pub(crate) fn folder_contains(&self, path: &std::path::Path) -> bool {
        !super::is_hidden(path)
            && matches!(&self.origin, RootOrigin::FolderHome { home, .. } if path.starts_with(home))
    }
}

#[cfg(test)]
#[path = "tests_unread_reasons.rs"]
mod tests;
