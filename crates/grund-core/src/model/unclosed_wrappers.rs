//! A cross-reference wrap whose destination never closes before a later `)` on
//! its line (§FS-citations.checks.unclosed-wrapper): recorded by the scanner as
//! it reads the citation, so the checker reports it without reading the file a
//! second time. Nothing else reads it: until grund 0.19.0 `fmt` leaves the line
//! as written.

use std::path::PathBuf;

use super::Id;

/// One unclosed wrap, at its citation's line: the ID it would link, its alias
/// when qualified, and the citation as written after the marker.
pub(crate) struct UnclosedWrapperNote {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) namespace: Option<String>,
    pub(crate) id: Id,
    pub(crate) label: String,
}
