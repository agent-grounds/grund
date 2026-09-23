//! How one recognized section heading becomes a record on the declaration it
//! sits inside (§AR-scanner.2.2), including which of the two enrollment routes
//! — the exact marker or the kind's declared chapter — made it a value root
//! (§AR-scanner.2.2.8, §FS-values.2.4, §FS-values.2.5).

use std::path::Path;

use super::embedded_value_context::push_invalid_embedded_marker;
use super::tree::heading_level_for_line;
use crate::config::{Config, kind_value_chapter};
use crate::grammar::{SourceScanLine, section_anchor_text};
use crate::model::{
    Declaration, EmbeddedValueRoot, Findings, SectionInfo, ValueRootOrigin, named_section_component,
};

/// Record `sec` on `decl`, and say whether the line's embedded marker was
/// consumed here — an unattached marker is the caller's finding to report
/// (§FS-values.2.4.4).
pub(super) fn record_section_heading(
    decl: &mut Declaration,
    caps: &regex::Captures<'_>,
    sec: &str,
    scan_line: &str,
    scan: &SourceScanLine<'_>,
    is_md: bool,
    lineno: usize,
    embedded_marker: Option<usize>,
    config: &Config,
    path: &Path,
    findings: &mut Findings,
) -> bool {
    let heading_level = heading_level_for_line(scan_line, is_md || scan.in_py_docstring, caps);
    if heading_level <= decl.heading_level {
        return false;
    }
    let section_path = sec.to_string();
    let numeric = sec
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    let mut embedded_marker_attached = embedded_marker.is_some() && numeric;
    // §FS-values.2.5 / §AR-scanner.2.2.8: the second enrollment route. A named
    // direct child of the declaration's direct chapter is a root by schema,
    // with no marker to carry; the marker route stays numeric-only.
    let chapter_root = section_is_chapter_value_root(config, &decl.id.kind, sec);
    let info = SectionInfo {
        title: section_anchor_text(scan_line, sec),
        line: lineno,
        heading_level,
        value: None,
        value_root: embedded_marker
            .filter(|_| numeric)
            .map(|marker_start| ValueRootOrigin::Marker {
                column: scan.column_offset + marker_start + 1,
            })
            .or(chapter_root.then_some(ValueRootOrigin::Chapter))
            .map(|origin| EmbeddedValueRoot {
                valid: true,
                origin,
            }),
    };
    // §AR-scanner.2.2.3: a path is recorded once, by the first heading that
    // claims it; later claimants go to `duplicate_sections` so §FS-check.3.16
    // can name every colliding line.
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

/// Whether this section path is a value root by the kind's declared chapter
/// (§FS-values.2.5): exactly `<chapter>.<name>`, where `<chapter>` is the
/// configured handle — matched on the handle, never the displayed title — and
/// `<name>` is one named component. Two components is what makes the chapter
/// *direct* and the root its *direct child*, so a same-named chapter nested
/// deeper (`subsystems.pump.values`) gains nothing.
pub(super) fn section_is_chapter_value_root(config: &Config, kind: &str, section: &str) -> bool {
    let Some(chapter) = kind_value_chapter(config, kind) else {
        return false;
    };
    let mut parts = section.split('.');
    parts.next() == Some(chapter)
        && parts.next().is_some_and(named_section_component)
        && parts.next().is_none()
}
