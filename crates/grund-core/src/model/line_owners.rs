//! What a `cover --lines` request found in one file (§FS-cover.6.3): for each
//! requested range, the runs of lines one declaration owns and, inside each, the
//! runs under one innermost section. The scanner fills these inside the file's
//! pass, while the heading stack is held (§AR-scanner.2.4.4); the cover API
//! renders them.

use std::path::PathBuf;

use super::Id;

/// One scanned file's answers, in the order the ranges were requested.
#[derive(Clone, Debug)]
pub(crate) struct FileLineOwnership {
    pub(crate) file: PathBuf,
    /// The file's line count, so a range ending past it is refused by the caller
    /// with the number the scan saw (§FS-cover.6.4).
    pub(crate) total_lines: usize,
    pub(crate) ranges: Vec<RangeOwnership>,
}

/// One requested range and its owner runs, in line order (§FS-cover.6.3).
#[derive(Clone, Debug)]
pub(crate) struct RangeOwnership {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) owners: Vec<OwnerRun>,
}

/// A maximal run of consecutive lines owned by one declaration (§FS-cover.6.3).
#[derive(Clone, Debug)]
pub(crate) struct OwnerRun {
    pub(crate) declaration: Id,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) sections: Vec<SectionRun>,
}

/// A maximal run of an owner run's lines under one innermost section, `None` for
/// the lead or lines no accepted section contains (§FS-cover.6.3).
#[derive(Clone, Debug)]
pub(crate) struct SectionRun {
    pub(crate) section: Option<String>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
