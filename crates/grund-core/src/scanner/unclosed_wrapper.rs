//! The unclosed-wrapper note (§FS-citations.checks.unclosed-wrapper): a marked
//! Markdown citation whose wrap never closes by the one extent of a link
//! destination but would close at a later `)` by the first-`)` reading `fmt`
//! keeps until grund 0.19.0 (§FS-citations.wrap.recognized). The scan notes it as it reads
//! the line, through the one bracket test `fmt` asks, and `check` reports what was
//! noted, as it does for `stub-shape`. An off-grammar citation is noted where the
//! catalog promotes it (`legacy.rs`), from the opener this pass saw.

use super::citation_line::CitationLine;
use super::citations::scanned_citation_rewritable;
use crate::config::{Frame, Schema};
use crate::grammar::{DocstringContent, FmtDirectives, cross_ref_wrap};
use crate::model::{Catalog, Declaration, UnclosedWrapperNote};

/// One Markdown file's pass for the note: `None` for every other file, which
/// holds no wrap. It follows the `grund:fmt` regions of §FS-fmt.2.5.2 the way
/// `fmt` does. A line in an off region is still noted, marked suppressed,
/// because `fmt` links a kind's index entry there (§FS-fmt.2.5.3); `check`
/// keeps only what `fmt` would link.
pub(super) struct UnclosedWrappers<'a> {
    directives: Option<FmtDirectives<'a>>,
    /// `Some(suppressed)` on a line `fmt` reads; `None` on a directive line and
    /// outside Markdown.
    reach: Option<bool>,
}

impl<'a> UnclosedWrappers<'a> {
    pub(super) fn new(schema: &'a Schema, frame: Frame<'a>, is_md: bool) -> Self {
        Self {
            directives: is_md.then(|| FmtDirectives::new(frame.compiled.lexical(schema), true)),
            reach: None,
        }
    }

    /// Take the next unfenced line: a directive line is out of `fmt`'s reach,
    /// and a line of a `grund:fmt off` region is suppressed (§FS-fmt.2.5.2).
    pub(super) fn read(&mut self, line: &str) {
        self.reach = self.directives.as_mut().and_then(|directives| {
            (!directives.consume(line, DocstringContent::default()))
                .then(|| !directives.rewriting())
        });
    }

    /// Whether the marker at `marker_start` opens a wrap `fmt` could read: a `[`
    /// right before it, on a prose line in reach that holds a `](`, not a
    /// declaration heading line (§FS-fmt.2.3), and not in inline code or a
    /// destination, zones `fmt` leaves alone. `Some(suppressed)` when it does.
    pub(super) fn opener(
        &self,
        line: &CitationLine<'_>,
        marker_start: usize,
        current: Option<&Declaration>,
    ) -> Option<bool> {
        let suppressed = self.reach?;
        (line.scan_line.contains("](")
            && marker_start > 0
            && line.scan_line.as_bytes()[marker_start - 1] == b'['
            && !current.is_some_and(|decl| decl.line == line.lineno)
            && scanned_citation_rewritable(line, marker_start))
        .then_some(suppressed)
    }

    /// Note each citation the line recorded from `from` on whose wrap never closes
    /// before a later `)`.
    pub(super) fn note(
        &self,
        line: &CitationLine<'_>,
        from: usize,
        current: Option<&Declaration>,
        findings: &mut Catalog,
    ) {
        if self.reach.is_none() || !line.scan_line.contains("](") {
            return;
        }
        let marker = line.schema.citation.marker.as_str();
        let notes = findings.citations[from..]
            .iter()
            .filter(|citation| citation.has_marker && citation.line == line.lineno)
            .filter_map(|citation| {
                let label = citation.text.strip_prefix(marker)?;
                let marker_start = citation.column.checked_sub(1 + line.column_offset)?;
                let suppressed = self.opener(line, marker_start, current)?;
                let end = marker_start + citation.text.len();
                let wrap = cross_ref_wrap(line.scan_line, marker_start, end)?;
                wrap.unclosed_before_paren().then(|| UnclosedWrapperNote {
                    file: line.path.to_path_buf(),
                    line: line.lineno,
                    column: citation.column,
                    namespace: citation.namespace.clone(),
                    id: citation.id.clone(),
                    label: label.to_string(),
                    suppressed,
                })
            })
            .collect::<Vec<_>>();
        findings.unclosed_wrapper_notes.extend(notes);
    }
}
