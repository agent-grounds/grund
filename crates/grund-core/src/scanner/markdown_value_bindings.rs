//! Re-read bindings on a formatter's current Markdown line with the scanner's
//! existing binding pass, rather than a second binding grammar (§FS-values.8).

use std::collections::BTreeMap;
use std::path::Path;

use super::citation_line::CitationLine;
use super::values::scan_value_bindings;
use crate::config::{Frame, Schema};
use crate::grammar::{
    DocstringContent, LexicalSettings, LineLinks, MarkdownLineCitation, StringReading,
};
use crate::model::{Catalog, Citation};
use crate::workspace::WorkspaceCitationTarget;

/// Classify the link pass's citation occurrences after trigger/marker/shorthand
/// normalization (§FS-values.8). Coordinates belong to these current bytes;
/// split attempts remain the whole-file scan's responsibility.
/// The writer supplies the concern records and lexical reading (§AR-config.5).
pub(crate) fn markdown_value_binding_candidates(
    text: &str,
    links: LineLinks<'_>,
    path: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    lexical: LexicalSettings<'_>,
    citations: &[MarkdownLineCitation],
    workspace_targets: &[WorkspaceCitationTarget],
) -> Catalog {
    let mut candidates = Catalog::default();
    if !text.contains('`') {
        return candidates;
    }
    candidates.citations = citations
        .iter()
        .map(|citation| Citation {
            namespace: citation.namespace.clone(),
            id: citation.id.clone(),
            section: citation.section.clone(),
            file: path.to_path_buf(),
            line: 1,
            column: citation.marker_start + 1,
            has_marker: true,
            shorthand: false,
            local_section: false,
            shorthand_rewritable: true,
            numeric_run: false,
            text: text[citation.marker_start..citation.token_end].to_string(),
            inline_site: None,
            source_kind: String::new(),
            enclosing_declaration: None,
            enclosing_section: None,
        })
        .collect();
    let empty = BTreeMap::new();
    let line = CitationLine {
        scan_line: text,
        raw_line: text,
        docstring: DocstringContent::default(),
        links,
        column_offset: 0,
        lineno: 1,
        path,
        schema,
        frame,
        lexical,
        source_forms: StringReading::NONE,
        is_md: true,
        may_be_heading: true,
        declares: None,
        value_comment: None,
        inline_sites: &empty,
        inline_block_lines: &BTreeMap::new(),
    };
    scan_value_bindings(&line, workspace_targets, 0, &mut candidates);
    candidates
}
