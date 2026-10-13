//! A configured file root the walk did not read because its name is hidden or its
//! extension is unlisted (§FS-scan.checks.unread-scan-root): recorded by the walk's
//! root gate with the row that made the file a root and every name rule that refused
//! it (§AR-scanner.1.6), so the report names both without asking either rule again
//! (§AR-scanner.1.13). Only `check` reads it (§FS-scan.checks.unread-scan-root.scope).

use std::path::PathBuf;

/// The configuration row that made a file a scan root, and where it was written:
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
    /// Its name begins with `.` (§FS-scan.scope.hidden).
    pub(crate) hidden: bool,
    /// The extension list leaves out its extension, or it has none
    /// (§FS-scan.scope.extensions.root).
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
}
