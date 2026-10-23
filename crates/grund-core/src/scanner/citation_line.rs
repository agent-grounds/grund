//! The normalized view shared by the citation and value passes for one physical
//! source line (§AR-scanner.2.3, §FS-values.3.2).

use std::collections::BTreeMap;
use std::path::Path;

use super::value_context::SourceValueLineContext;
use crate::config::{Frame, Schema};
use crate::grammar::{
    DocstringContent, LexicalSettings, LineLinks, QualifiedSource, bare_token_in_never_rewrite_zone,
};
use crate::model::{Id, InlineCitationSite};

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
    /// Where a Markdown line sits in its file's links, which the never-rewrite
    /// zones of §FS-fmt.2.3 are read in.
    pub(super) links: LineLinks<'a>,
    pub(super) column_offset: usize,
    pub(super) lineno: usize,
    pub(super) path: &'a Path,
    pub(super) schema: &'a Schema,
    pub(super) frame: Frame<'a>,
    /// The file's own hosts and grammar (§FS-config-v2.schema.sources).
    pub(super) lexical: LexicalSettings<'a>,
    /// The resolved language for qualified comment sites (§AR-scanner.2.3.2).
    pub(super) source_forms: Option<&'a str>,
    pub(super) is_md: bool,
    /// Whether this line may be a heading: false inside a raw-text HTML block,
    /// whose lines are never headings (§FS-check.1.1.5.1).
    pub(super) may_be_heading: bool,
    /// The ID this line declares where it is a declaration's heading, a near miss's
    /// included. A citation of it here is none (§AR-scanner.2.3).
    pub(super) declares: Option<&'a Id>,
    /// The bytes on this physical source line that the scanner's shared block
    /// walk recognizes as comment content, and the block they belong to.
    /// Markdown and Python docstrings use their already-normalized `scan_line`
    /// instead (§FS-values.3.2).
    pub(super) value_comment: Option<SourceValueLineContext>,
    pub(super) inline_sites: &'a BTreeMap<usize, InlineCitationSite>,
    pub(super) inline_block_lines: &'a BTreeMap<usize, std::sync::Arc<[String]>>,
}

impl CitationLine<'_> {
    /// The literal entry state of `scan_line`: how many of its leading bytes are
    /// inside a string literal an earlier line opened (§FS-languages.multi-line-string).
    fn literal_entry(&self) -> usize {
        self.docstring.literal_entry_from(self.column_offset)
    }

    /// §AR-scanner.2.3.2: whether the marked qualified citation at `scan_line` offset
    /// `marker_start` is suppressed — asked in the state the line begins in, so the
    /// interior of a multi-line literal is a string here too
    /// (§FS-languages.multi-line-string). All qualified passes share the comment
    /// distinction of §AR-scanner.2.3.2.
    pub(super) fn qualified_suppressed(&self, marker_start: usize) -> bool {
        QualifiedSource {
            forms: self.source_forms,
            comment_prefixes: self.lexical.comment_prefixes,
            comment_range: self
                .docstring
                .is_docstring()
                .then_some((0, self.scan_line.len()))
                .or_else(|| self.value_comment.map(|context| context.range)),
        }
        .suppressed(
            self.scan_line,
            self.literal_entry(),
            self.is_md,
            marker_start,
        )
    }

    /// §AR-scanner.2.3.1: whether the bare token at `scan_line` offset `pos` sits in a
    /// never-rewrite zone, read in the state the line begins in.
    pub(super) fn bare_token_in_zone(&self, pos: usize, marker: &str) -> bool {
        bare_token_in_never_rewrite_zone(
            self.scan_line,
            self.literal_entry(),
            self.is_md,
            pos,
            marker,
        )
    }
}
