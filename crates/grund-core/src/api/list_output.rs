//! Published declaration/chapter catalog records (§FS-list.2, §FS-rules.8).

use crate::model::Finding;
use crate::scanner::ApiScanError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListEntry {
    pub project: Option<String>,
    pub id: String,
    pub section: Option<String>,
    /// Owning project's separator for canonical text coordinates. It is not a
    /// wire field; JSON keeps `id` and `section` separate (§FS-rules.8).
    pub section_separator: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub title: Option<String>,
    pub stub: bool,
    pub defines: Option<String>,
    pub refs: usize,
    pub duplicate: bool,
    pub value_roots: Vec<ListValueRoot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListValueRoot {
    pub id: String,
    pub valid: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSummary {
    pub project: Option<String>,
    pub kind: String,
    pub title: String,
    pub home: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListOutput {
    pub output_format: String,
    pub workspace: bool,
    pub entries: Vec<ListEntry>,
    pub summaries: Vec<ListSummary>,
    pub scan_errors: Vec<ApiScanError>,
    /// Run warnings stay separate from catalog findings (§FS-distribution.3.1).
    pub warnings: Vec<Finding>,
}
