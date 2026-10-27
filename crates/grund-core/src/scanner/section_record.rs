//! How one recognized section heading becomes a record on the declaration it
//! sits inside (§AR-scanner.2.2), including which of the two enrollment routes
//! — the exact marker or the kind's declared chapter — made it a value root
//! (§AR-scanner.2.2.8, §FS-values.2.4, §FS-values.2.5).

use std::path::Path;

use super::embedded_value_context::push_invalid_embedded_marker;
use super::tree::heading_level_for_line;
use crate::config::{Schema, kind_value_chapter};
use crate::grammar::{
    SourceScanLine, named_section_component, section_anchor_text, strip_block_closer,
};
use crate::model::{Catalog, Declaration, EmbeddedValueRoot, SectionInfo, ValueRootOrigin};

/// Record `sec` on `decl`, and say whether the line's embedded marker was
/// consumed here — an unattached marker is the caller's finding to report
/// (§FS-values.2.4.4). `markdown_depth` reads the heading at its own `#` run, as
/// in Markdown, a docstring, or a bare line of a block a bare line declared in
/// (§FS-declarations.line.block-comment).
pub(super) fn record_section_heading(
    decl: &mut Declaration,
    caps: &regex::Captures<'_>,
    sec: &str,
    scan_line: &str,
    scan: &SourceScanLine<'_>,
    markdown_depth: bool,
    is_md: bool,
    lineno: usize,
    embedded_marker: Option<usize>,
    schema: &Schema,
    path: &Path,
    findings: &mut Catalog,
) -> bool {
    let heading_level = heading_level_for_line(scan_line, markdown_depth, caps);
    if heading_level <= decl.heading_level {
        return false;
    }
    let section_path = sec.to_string();
    let marker_column = embedded_marker.map(|start| scan.column_offset + start + 1);
    let value_root = section_value_root(schema, &decl.id.kind, sec, marker_column);
    let mut embedded_marker_attached = value_root.as_ref().is_some_and(EmbeddedValueRoot::marked);
    // §FS-rules.5.1.1: a source heading is titled from its own `#` run up to any
    // closing `*/`, so the comment around it never joins the label.
    // Markdown and docstring lines stay as is.
    let heading = match caps.name("hashes") {
        Some(hashes) if !(is_md || scan.in_py_docstring) => {
            strip_block_closer(&scan_line[hashes.start()..])
        }
        _ => scan_line,
    };
    let info = SectionInfo {
        title: section_anchor_text(heading, sec),
        line: lineno,
        heading_level,
        value: None,
        value_root,
        relative: false,
        front_matter: None,
    };
    // §AR-scanner.2.2.3: a path is recorded once, by the first heading that claims it; later
    // claimants go to `duplicate_sections` so §FS-declarations.checks.duplicate-section can name
    // every colliding line.
    match decl.sections.entry(section_path.clone()) {
        std::collections::btree_map::Entry::Vacant(slot) => {
            slot.insert(info);
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            if let Some(marker_start) = embedded_marker {
                embedded_marker_attached = true;
                push_invalid_embedded_marker(
                    findings,
                    Some(decl.id.clone()),
                    path,
                    lineno,
                    scan.column_offset,
                    marker_start,
                    "embedded value root coordinate is duplicated",
                );
            }
            decl.duplicate_sections.push((section_path, info));
        }
    }
    embedded_marker_attached
}

/// The value root a section recorded at `sec` in a declaration of `kind` is, if
/// any: marked when its heading carries the marker, at `marker_column`, and its
/// path is numeric (§FS-values.2.4); otherwise a root by the kind's declared
/// chapter (§FS-values.2.5, §AR-scanner.2.2.8), which carries no marker. The
/// marker route stays numeric-only. A relative heading is enrolled here too, at
/// the canonical path its parent gave it (§FS-values.2.4).
pub(super) fn section_value_root(
    schema: &Schema,
    kind: &str,
    sec: &str,
    marker_column: Option<usize>,
) -> Option<EmbeddedValueRoot> {
    let numeric = sec
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    marker_column
        .filter(|_| numeric)
        .map(|column| ValueRootOrigin::Marker { column })
        .or(section_is_chapter_value_root(schema, kind, sec).then_some(ValueRootOrigin::Chapter))
        .map(|origin| EmbeddedValueRoot {
            valid: true,
            origin,
        })
}

/// Whether this section path is a value root by the kind's declared chapter
/// (§FS-values.2.5): exactly `<chapter>.<name>`, where `<chapter>` is the
/// configured handle — matched on the handle, never the displayed title — and
/// `<name>` is one named component. Two components is what makes the chapter
/// *direct* and the root its *direct child*, so a same-named chapter nested
/// deeper (`subsystems.pump.values`) gains nothing.
pub(super) fn section_is_chapter_value_root(schema: &Schema, kind: &str, section: &str) -> bool {
    let Some(chapter) = kind_value_chapter(schema, kind) else {
        return false;
    };
    let mut parts = section.split('.');
    parts.next() == Some(chapter)
        && parts.next().is_some_and(named_section_component)
        && parts.next().is_none()
}
