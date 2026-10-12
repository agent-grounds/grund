//! The unclosed-wrapper note (§FS-citations.checks.unclosed-wrapper): a marked
//! Markdown citation whose wrap never closes by the one extent of a link
//! destination but would close at a later `)` by the first-`)` reading `fmt`
//! keeps until grund 0.19.0 (§FS-citations.wrap.recognized). The scan notes it as it reads
//! the line, through the one bracket test `fmt` asks, and `check` reports what was
//! noted, as it does for `stub-shape`.

use super::citation_line::CitationLine;
use super::citations::scanned_citation_rewritable;
use crate::config::{Frame, Schema};
use crate::grammar::{DocstringContent, FmtDirectives, cross_ref_wrap};
use crate::model::{Catalog, Declaration, UnclosedWrapperNote};

/// One Markdown file's pass for the note: `None` for every other file, which
/// holds no wrap. It follows the `grund:fmt` regions of §FS-fmt.2.5.2 the way
/// `fmt` does, because a line out of `fmt`'s reach keeps its bytes in every
/// release, so no verdict moves there.
pub(super) struct UnclosedWrappers<'a> {
    directives: Option<FmtDirectives<'a>>,
    in_reach: bool,
}

impl<'a> UnclosedWrappers<'a> {
    pub(super) fn new(schema: &'a Schema, frame: Frame<'a>, is_md: bool) -> Self {
        Self {
            directives: is_md.then(|| FmtDirectives::new(frame.compiled.lexical(schema), true)),
            in_reach: false,
        }
    }

    /// Take the next unfenced line: whether `fmt` may write on it, a directive
    /// line and a `grund:fmt off` region being out of its reach (§FS-fmt.2.5.2).
    pub(super) fn read(&mut self, line: &str) {
        self.in_reach = self.directives.as_mut().is_some_and(|directives| {
            !directives.consume(line, DocstringContent::default()) && directives.rewriting()
        });
    }

    /// Note each citation the line recorded from `from` on whose wrap never closes
    /// before a later `)`. Asked only of a prose line in reach that holds a `](`,
    /// and not of a declaration heading line (§FS-fmt.2.3); a citation inside
    /// inline code or a destination sits in a zone `fmt` leaves alone.
    pub(super) fn note(
        &self,
        line: &CitationLine<'_>,
        from: usize,
        current: Option<&Declaration>,
        findings: &mut Catalog,
    ) {
        if !self.in_reach
            || !line.scan_line.contains("](")
            || current.is_some_and(|decl| decl.line == line.lineno)
        {
            return;
        }
        let marker = line.schema.citation.marker.as_str();
        let notes = findings.citations[from..]
            .iter()
            .filter(|citation| citation.has_marker && citation.line == line.lineno)
            .filter_map(|citation| {
                let label = citation.text.strip_prefix(marker)?;
                let marker_start = citation.column.checked_sub(1 + line.column_offset)?;
                let end = marker_start + citation.text.len();
                let wrap = cross_ref_wrap(line.scan_line, marker_start, end)?;
                (wrap.unclosed_before_paren() && scanned_citation_rewritable(line, marker_start))
                    .then(|| UnclosedWrapperNote {
                        file: line.path.to_path_buf(),
                        line: line.lineno,
                        namespace: citation.namespace.clone(),
                        id: citation.id.clone(),
                        label: label.to_string(),
                    })
            })
            .collect::<Vec<_>>();
        findings.unclosed_wrapper_notes.extend(notes);
    }
}
