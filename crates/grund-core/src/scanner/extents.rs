//! The extent rules over the file record (§AR-scanner.2.8.2): where a
//! declaration's body ends, where a section ends, and where a lead ends. Each is
//! one function, run on every scan once the file's declarations are in, and its
//! answer is written on the recorded heading every reader asks.

use std::collections::BTreeMap;

use crate::model::{Catalog, Declaration, FileRecord, HeadingKind};

/// Where `decl`'s body ends (§FS-declarations.body, §AR-scanner.2.4.1): in
/// Markdown, before the next ATX heading of its level or above, declaring or not;
/// in a source file, at the end of the comment block its line opens, capped before
/// the next declaration in that block (`next_decl`, the first declaration line
/// after it). A stub's body is its line.
pub(super) fn body_end(record: &FileRecord, decl: &Declaration, next_decl: Option<usize>) -> usize {
    if decl.is_stub {
        return decl.line;
    }
    if record.markdown {
        return record
            .headings_between(decl.line + 1, usize::MAX)
            .iter()
            .find(|heading| heading.atx && heading.level <= decl.heading_level)
            .map_or(record.total_lines, |heading| heading.line - 1)
            .max(decl.line);
    }
    let block_end = record
        .block_at(decl.line)
        .map_or(decl.line, |block| block.end);
    next_decl
        .filter(|line| *line <= block_end)
        .map_or(block_end, |line| line - 1)
        .max(decl.line)
}

/// The Markdown declaration bodies of one file, each as its heading line, the
/// line it ends on (`body_end`) and its heading level: the one answer to which
/// declaration owns a heading (§FS-declarations.body,
/// §FS-declarations.checks.unmarked-heading.1), asked by the parent pass before
/// the extents are written and by the unmarked-heading pass after.
pub(super) struct DeclarationBodies(Vec<(usize, usize, usize)>);

impl DeclarationBodies {
    pub(super) fn of(record: &FileRecord, findings: &Catalog) -> Self {
        let mut bodies = findings
            .declarations
            .values()
            .flatten()
            .map(|decl| (decl.line, body_end(record, decl, None), decl.heading_level))
            .collect::<Vec<_>>();
        bodies.sort_unstable();
        Self(bodies)
    }

    /// The heading line and body end of the innermost declaration whose body
    /// holds a heading at `level` on `line`: below its heading, within its body,
    /// and deeper than it. `None` for a heading outside every body.
    pub(super) fn owner(&self, line: usize, level: usize) -> Option<(usize, usize)> {
        self.0
            .iter()
            .filter(|(start, end, decl_level)| *start < line && line <= *end && level > *decl_level)
            .max_by_key(|(start, _, _)| *start)
            .map(|(start, end, _)| (*start, *end))
    }
}

/// Whether any line of `decl`'s body after its heading holds more than whitespace
/// (§AR-scanner.2.4.1), read from the heading's offset rather than from the top
/// of the file.
pub(super) fn body_has_content(record: &FileRecord, decl_line: usize, end: usize) -> bool {
    let Some(heading) = record.heading_at(decl_line) else {
        return false;
    };
    record.text()[heading.span.end..]
        .split('\n')
        .skip(1)
        .take(end.saturating_sub(decl_line))
        .any(|line| !line.trim().is_empty())
}

/// Write every declaration's and accepted section's end and lead end onto its
/// recorded heading (§AR-scanner.2.8.1). Declarations come first: a section is
/// capped at its declaration's body, and only a heading inside a body is an
/// accepted section of it.
pub(super) fn assign_extents(record: &mut FileRecord, findings: &Catalog) {
    let mut decls: Vec<&Declaration> = findings.declarations.values().flatten().collect();
    decls.sort_by_key(|decl| decl.line);
    let bodies = declaration_bodies(record, &decls);
    super::record::settle_headings(record, &decls, &bodies);
    for index in 0..record.headings.len() {
        let heading = &record.headings[index];
        let (end, lead_end) = match heading.kind {
            HeadingKind::Declaration => {
                let Some(&(end, _)) = bodies.get(&heading.line) else {
                    continue;
                };
                (end, lead_end(record, heading.line, end, true))
            }
            HeadingKind::Section => {
                let cap = owning_body(&bodies, heading.line).unwrap_or(heading.line);
                let end = section_end(record, index, cap);
                (end, lead_end(record, heading.line, end, false))
            }
            _ => continue,
        };
        let heading = &mut record.headings[index];
        heading.end = end;
        heading.lead_end = lead_end;
    }
}

/// Each of `decls`' body end and heading level by its line, `decls` in line order:
/// the bodies the extents are capped at and the doc-comment headings are read in.
pub(super) fn declaration_bodies(
    record: &FileRecord,
    decls: &[&Declaration],
) -> BTreeMap<usize, (usize, usize)> {
    let mut bodies = BTreeMap::new();
    for (index, decl) in decls.iter().enumerate() {
        let next = decls.get(index + 1).map(|next| next.line);
        bodies.insert(
            decl.line,
            (body_end(record, decl, next), decl.heading_level),
        );
    }
    bodies
}

/// The end of the body that holds `line`: the nearest declaration above it whose
/// body reaches it.
fn owning_body(bodies: &BTreeMap<usize, (usize, usize)>, line: usize) -> Option<usize> {
    bodies
        .range(..line)
        .rev()
        .find(|(_, (end, _))| *end >= line)
        .map(|(_, (end, _))| *end)
}

/// Where the section headed at `record.headings[index]` ends (§AR-scanner.2.4.4,
/// D17b of §DF-one-answer-rulings-parse): before the next heading of its level or
/// above, numbered or plain, declaring or not — in Markdown, the next ATX heading
/// (§FS-declarations.heading) — and never past its declaration's body, `cap`.
pub(super) fn section_end(record: &FileRecord, index: usize, cap: usize) -> usize {
    let section = &record.headings[index];
    record.headings[index + 1..]
        .iter()
        .take_while(|heading| heading.line <= cap)
        .find(|heading| (heading.atx || !record.markdown) && heading.level <= section.level)
        .map_or(cap, |heading| heading.line - 1)
}

/// Where the lead of the declaration or section headed at `line` ends
/// (§FS-declarations.lead): before its first child citable section, or, for a
/// declaration, a deeper declaration inside its body; at its end where neither
/// comes. A plain heading is prose and cuts nothing.
pub(super) fn lead_end(record: &FileRecord, line: usize, end: usize, declaration: bool) -> usize {
    record
        .headings_between(line + 1, end)
        .iter()
        .find(|heading| {
            heading.kind == HeadingKind::Section
                || (declaration && heading.kind == HeadingKind::Declaration)
        })
        .map_or(end, |heading| heading.line - 1)
}
