//! A configured file root the walk did not read because its name is hidden or its
//! extension is unlisted (§FS-scan.checks.unread-scan-root): recorded by the walk's
//! root gate with the row that made the file a root and every name rule that refused
//! it (§AR-scanner.1.6), so the report names both without asking either rule again
//! (§AR-scanner.1.13). A rule folder the walk read no file of is recorded the same way,
//! after every root, with the extensions its loop refused
//! (§FS-scan.checks.unread-scan-root.rule-folder). Only `check` reads either
//! (§FS-scan.checks.unread-scan-root.scope).

use std::path::{Path, PathBuf};

/// The configuration row that made a file a scan root, or named a rule folder, and where
/// it was written: the `grund.toml` and the line of the key that named it
/// (§FS-scan.checks.unread-scan-root.location).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootRow {
    pub(crate) origin: RootOrigin,
    /// The file or folder as the configuration wrote it (§FS-errors.4).
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
    /// The extension list leaves out its extension, or it has none: the extension
    /// reason of §FS-scan.checks.unread-scan-root.message.
    pub(crate) extension: bool,
}

/// Which key named what was not read (§FS-scan.checks.unread-scan-root.message).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RootOrigin {
    /// An `[scan] include` entry.
    Include,
    /// A row's `file` home, or a version-2 `files` entry: the row's name, and
    /// whether it declares IDs, which decides whether `scan = false` is offered.
    Home { kind: String, citable: bool },
    /// A rule kind's `folder` home, or an entry of its version-2 `folders`: the row's
    /// name, and each extension the walk refused in the folder, once, as the file names
    /// spell it, in byte order. Empty where every refused file has none
    /// (§FS-scan.checks.unread-scan-root.rule-folder).
    RuleFolder {
        kind: String,
        extensions: Vec<String>,
    },
}

/// The part of a file name after its last `.`, as the name spells it, or nothing
/// (§FS-scan.checks.unread-scan-root.message).
pub(crate) fn spelled_extension(path: &Path) -> Option<&str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty())
}
