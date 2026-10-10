//! The normalized view shared by the citation and value passes for one physical
//! source line (§AR-scanner.2.3, §FS-values.3.2).

use std::collections::BTreeMap;
use std::path::Path;

use super::value_context::SourceValueLineContext;
use crate::config::{Frame, Schema};
use crate::grammar::DocstringContent;
use crate::model::InlineCitationSite;

pub(super) struct CitationLine<'a> {
    pub(super) scan_line: &'a str,
    /// The untransformed source line. `scan_line` may be a *slice* of it — a
    /// Python docstring's interior with the quotes stripped (§AR-scanner.4.4) — so a
    /// position on this line and a position on that one are not the same number.
    /// Every never-rewrite question is asked at a **raw-line** offset and routed to
    /// the right text by `docstring` below (§FS-fmt.2.3.1.1).
    pub(super) raw_line: &'a str,
    /// Where this line's Python docstring content sits in `raw_line`
    /// (§FS-fmt.2.3.1.1) — the view every never-rewrite question is asked through,
    /// so a docstring line is judged on the text `fmt` reads there too.
    pub(super) docstring: DocstringContent<'a>,
    pub(super) column_offset: usize,
    pub(super) lineno: usize,
    pub(super) path: &'a Path,
    pub(super) schema: &'a Schema,
    pub(super) frame: Frame<'a>,
    pub(super) is_md: bool,
    /// Whether this line may be a heading: false inside a raw-text HTML block,
    /// whose lines are never headings (§FS-check.1.1.5.1).
    pub(super) may_be_heading: bool,
    /// The bytes on this physical source line that the scanner's shared block
    /// walk recognizes as comment content, and the block they belong to.
    /// Markdown and Python docstrings use their already-normalized `scan_line`
    /// instead (§FS-values.3.2).
    pub(super) value_comment: Option<SourceValueLineContext>,
    pub(super) inline_sites: &'a BTreeMap<usize, InlineCitationSite>,
    pub(super) inline_block_lines: &'a BTreeMap<usize, std::sync::Arc<[String]>>,
}
