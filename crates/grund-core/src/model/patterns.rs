//! A marked candidate read as a pattern rather than a citation
//! (§FS-check.1.1.11): recorded by the scanner where it consumed the token
//! whole, so the checker can report it (§FS-check.checks.glob-citation) without
//! any pass ever having taken an edge to its prefix. Beside it, the marked
//! token its kind's format does not spell (§FS-check.1.1.12), recorded the same
//! way for §FS-check.checks.off-format-citation.

use std::path::PathBuf;

/// One pattern written where a citation belongs: the marker's line and column,
/// and the candidate as written after the marker.
pub(crate) struct GlobCitation {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) token: String,
}

/// A marked token that opens like an ID of a configured kind, that its kind's
/// format does not spell and that nothing declares, as `scanner/off_format.rs`
/// reads it: recorded by the promotion step where it would have dropped the
/// token, so the checker can report it (§FS-check.checks.off-format-citation)
/// while no pass ever takes an edge for it.
pub(crate) struct OffFormatCitation {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    /// The token as written after the marker, its alias and section path included.
    pub(crate) token: String,
    /// The kind it opens with, in the project the token names.
    pub(crate) kind: String,
    /// What that kind's IDs follow there: `<key> = "<template>" with <patterns>`.
    pub(crate) format: String,
}
