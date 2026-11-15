//! How one recognized section heading becomes a record on the declaration it
//! sits inside (§AR-scanner.2.2), including which of the two enrollment routes
//! — the exact marker or the kind's declared chapter — made it a value root
//! (§AR-scanner.2.2.8, §FS-values.2.4, §FS-values.2.5). In Markdown the heading is
//! held until the file's bodies are known and then recorded on the innermost
//! declaration whose body holds it (§AR-scanner.2.2.5).

use std::collections::BTreeMap;
use std::path::Path;

use super::embedded_value_context::invalid_embedded_marker;
use super::extents::DeclarationBodies;
use super::tree::heading_level_for_line;
use crate::config::{Schema, kind_value_chapter};
use crate::grammar::{
    SourceScanLine, named_section_component, section_anchor_text, strip_block_closer,
};
use crate::model::{
    Catalog, Declaration, EmbeddedValueRoot, FileRecord, Id, InvalidValueSite, SectionInfo,
    ValueRootOrigin,
};

/// One recognized section heading as the file pass read it, before it is recorded on
/// the declaration that owns it (§AR-scanner.2.2): its path, line, depth and title,
/// and the column of its embedded value marker, if it carries one.
pub(super) struct SectionHeading {
    path: String,
    line: usize,
    level: usize,
    title: String,
    marker_column: Option<usize>,
}

/// Read the section heading `caps` matched on `scan_line` at path `sec`.
/// `markdown_depth` reads the heading at its own `#` run, as in Markdown, a
/// docstring, or a bare line of a block a bare line declared in
/// (§FS-declarations.line.block-comment).
#[allow(clippy::too_many_arguments)]
pub(super) fn read_section_heading(
    caps: &regex::Captures<'_>,
    sec: &str,
    scan_line: &str,
    scan: &SourceScanLine<'_>,
    markdown_depth: bool,
    is_md: bool,
    lineno: usize,
    embedded_marker: Option<usize>,
) -> SectionHeading {
    // §FS-rules.5.1.1: a source heading is titled from its own `#` run up to any
    // closing `*/`, so the comment around it never joins the label.
    // Markdown and docstring lines stay as is.
    let heading = match caps.name("hashes") {
        Some(hashes) if !(is_md || scan.in_py_docstring) => {
            strip_block_closer(&scan_line[hashes.start()..])
        }
        _ => scan_line,
    };
    SectionHeading {
        path: sec.to_string(),
        line: lineno,
        level: heading_level_for_line(scan_line, markdown_depth, caps),
        title: section_anchor_text(heading, sec),
        marker_column: embedded_marker.map(|start| scan.column_offset + start + 1),
    }
}

/// Record `heading` on `decl`, and say whether the line's embedded marker was
/// consumed here — an unattached marker is the caller's finding to report
/// (§FS-values.2.4.4) — with the finding a duplicated marked root earns
/// (§FS-values.2.4.3), for the caller to push. A heading no deeper than `decl`
/// is refused and consumes nothing.
fn record_section_heading(
    decl: &mut Declaration,
    heading: SectionHeading,
    schema: &Schema,
    path: &Path,
) -> (bool, Option<InvalidValueSite>) {
    if heading.level <= decl.heading_level {
        return (false, None);
    }
    let value_root =
        section_value_root(schema, &decl.id.kind, &heading.path, heading.marker_column);
    let mut embedded_marker_attached = value_root.as_ref().is_some_and(EmbeddedValueRoot::marked);
    let mut duplicated_marker = None;
    let info = SectionInfo {
        title: heading.title,
        line: heading.line,
        heading_level: heading.level,
        value: None,
        value_root,
        relative: false,
        front_matter: None,
    };
    // §AR-scanner.2.2.3: a path is recorded once, by the first heading that claims it; later
    // claimants go to `duplicate_sections` so §FS-declarations.checks.duplicate-section can name
    // every colliding line.
    match decl.sections.entry(heading.path.clone()) {
        std::collections::btree_map::Entry::Vacant(slot) => {
            slot.insert(info);
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            if let Some(column) = heading.marker_column {
                embedded_marker_attached = true;
                duplicated_marker = Some(invalid_embedded_marker(
                    Some(decl.id.clone()),
                    path,
                    heading.line,
                    column,
                    "embedded value root coordinate is duplicated",
                ));
            }
            decl.duplicate_sections.push((heading.path, info));
        }
    }
    (embedded_marker_attached, duplicated_marker)
}

/// The section headings of a Markdown file, held until every declaration and
/// heading of the file is known, each with the misplaced-marker finding it earns
/// if no declaration takes its marker (§FS-values.2.4.4).
#[derive(Default)]
pub(super) struct HeldSections(Vec<(SectionHeading, Option<InvalidValueSite>)>);

impl HeldSections {
    /// Take a section heading the file pass read, and say whether its embedded marker
    /// was consumed. A Markdown heading is held, because a Markdown body may resume
    /// after a nested one and its owner is known only once every body of the file is
    /// (§AR-scanner.2.2.5); a source heading is recorded on
    /// `current`, the declaration whose comment block it sits in.
    pub(super) fn offer(
        &mut self,
        heading: SectionHeading,
        is_md: bool,
        current: Option<&mut Declaration>,
        schema: &Schema,
        path: &Path,
        findings: &mut Catalog,
    ) -> bool {
        if is_md {
            self.0.push((heading, None));
            return false;
        }
        let Some(decl) = current else {
            return false;
        };
        let (attached, duplicated) = record_section_heading(decl, heading, schema, path);
        findings.invalid_value_declarations.extend(duplicated);
        attached
    }

    /// Hand a misplaced value marker on `line` to the section heading held there;
    /// any other marker is returned.
    pub(super) fn hold_marker(
        &mut self,
        line: usize,
        site: InvalidValueSite,
    ) -> Option<InvalidValueSite> {
        match self.0.last_mut() {
            Some((heading, pending)) if heading.line == line => {
                *pending = Some(site);
                None
            }
            _ => Some(site),
        }
    }

    /// Record each held heading, in line order, on the innermost declaration whose
    /// body holds it (§AR-scanner.2.2, §AR-scanner.2.2.5): the
    /// one owner lookup the unmarked-heading pass asks (§AR-scanner.2.4.3), so a
    /// heading after a nested declaration's body is the enclosing declaration's, not
    /// the nested one's. A heading no body holds is offered to the declaration above
    /// it, as the line scan offered it, for the body-span post-pass to reject
    /// (§AR-scanner.2.2.6). Runs before the parent pass, which reads these sections
    /// as relative headings' parents (§AR-scanner.2.2.5).
    pub(super) fn settle(
        self,
        findings: &mut Catalog,
        record: &FileRecord,
        schema: &Schema,
        path: &Path,
    ) {
        if self.0.is_empty() {
            return;
        }
        let bodies = DeclarationBodies::of(record, findings);
        let mut sites: BTreeMap<usize, (Id, usize)> = BTreeMap::new();
        for (id, decls) in &findings.declarations {
            for (index, decl) in decls.iter().enumerate() {
                sites.insert(decl.line, (id.clone(), index));
            }
        }
        let mut markers = Vec::new();
        for (heading, misplaced) in self.0 {
            let owner = bodies
                .owner(heading.line, heading.level)
                .map(|(start, _)| start)
                .or_else(|| {
                    sites
                        .range(..heading.line)
                        .next_back()
                        .map(|(line, _)| *line)
                });
            let Some((id, index)) = owner.and_then(|line| sites.get(&line)) else {
                markers.extend(misplaced);
                continue;
            };
            let decl = &mut findings
                .declarations
                .get_mut(id)
                .expect("site of this file")[*index];
            let (attached, duplicated) = record_section_heading(decl, heading, schema, path);
            markers.extend(duplicated);
            if !attached {
                markers.extend(misplaced.map(|site| InvalidValueSite {
                    id: Some(decl.id.clone()),
                    ..site
                }));
            }
        }
        findings.invalid_value_declarations.extend(markers);
    }
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
