//! A marked candidate read as a pattern rather than a citation
//! (§FS-check.1.1.11): recorded by the scanner where it consumed the token
//! whole, so the checker can report it (§FS-check.checks.glob-citation) without
//! any pass ever having taken an edge to its prefix.

use std::path::PathBuf;

/// One pattern written where a citation belongs: the marker's line and column,
/// and the candidate as written after the marker.
pub(crate) struct GlobCitation {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) token: String,
}
