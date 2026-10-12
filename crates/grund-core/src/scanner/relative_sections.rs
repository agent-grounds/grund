//! The parent pass (§FS-declarations.section.relative.parent, §AR-scanner.2.2):
//! each relative heading the per-file pass met takes its canonical path from
//! the nearest shallower heading of its declaration body, once every heading of
//! the file is recorded. It runs before the extents are settled, so a resolved
//! relative heading enters the section map, the record and every reader of the
//! two exactly as a complete-path heading does, and nothing parses it again.

use std::collections::BTreeMap;

use super::context::section_path_is_numeric;
use super::extents::{DeclarationBodies, body_end};
use crate::grammar::{named_mode_section_path, reduce_heading_text, relative_heading_form};
use crate::model::{
    Catalog, Declaration, FileRecord, RelativeSectionParent, SectionInfo, UnmarkedHeadingCandidate,
};

/// Resolve every relative candidate of one Markdown file — a heading whose text
/// reads as the relative form (§FS-declarations.section.relative.form) — against its parent
/// (§FS-declarations.section.relative.parent). An accepted one is recorded on its
/// declaration at its canonical path and on the record's heading; one with no
/// usable parent one level up is a [`RelativeSectionParent`]
/// (§FS-declarations.checks.relative-section-parent); and one that is outside
/// every body, or whose title syntax its parent's path refuses, is handed back
/// as the unmarked candidate it is without this form
/// (§FS-declarations.section.relative.where, §FS-declarations.checks.unmarked-heading.1).
///
/// Candidates are taken shallowest first, so every parent is settled — accepted,
/// failed or a claimant of a duplicated path — before a child asks for it.
pub(super) fn resolve_relative_sections(
    findings: &mut Catalog,
    record: &mut FileRecord,
    candidates: Vec<UnmarkedHeadingCandidate>,
    named_sections: bool,
    unmarked: &mut Vec<UnmarkedHeadingCandidate>,
) {
    let bodies = DeclarationBodies::of(record, findings);
    let mut by_owner: BTreeMap<usize, Vec<UnmarkedHeadingCandidate>> = BTreeMap::new();
    for candidate in candidates {
        match bodies.owner(candidate.line, candidate.heading_level) {
            Some((start, _)) => by_owner.entry(start).or_default().push(candidate),
            None => unmarked.push(candidate),
        }
    }
    let mut accepted: BTreeMap<usize, String> = BTreeMap::new();
    for (decl_line, mut owned) in by_owner {
        let Some(decl) = findings
            .declarations
            .values_mut()
            .flatten()
            .find(|decl| decl.line == decl_line)
        else {
            continue;
        };
        let end = body_end(record, decl, None);
        owned.sort_by_key(|candidate| (candidate.heading_level, candidate.line));
        for heading in owned {
            let level = heading.heading_level;
            let parent = parent_path(record, decl, end, heading.line, level);
            let Some(parent) = parent else {
                findings
                    .relative_section_parents
                    .push(RelativeSectionParent {
                        file: heading.file.clone(),
                        line: heading.line,
                        column: heading.column,
                        heading: heading.heading.clone(),
                        owner: decl.id.clone(),
                        written: written(&heading),
                        needed_level: level - 1,
                        under: nearest_usable(record, decl, end, heading.line, level - 1),
                    });
                continue;
            };
            let Some(form) = relative_heading_form(&heading.title) else {
                unmarked.push(heading);
                continue;
            };
            let canonical = if parent.is_empty() {
                form.component.to_string()
            } else {
                format!("{parent}.{}", form.component)
            };
            let numeric = section_path_is_numeric(&canonical);
            // §FS-declarations.section.relative.form: refused wherever the complete
            // form would be — a name with the gate off, `number.name`, the wrong punctuation.
            let shaped = numeric
                || (named_sections
                    && named_mode_section_path(&canonical, |part| {
                        part.bytes().all(|byte| byte.is_ascii_digit())
                    }));
            if !shaped || !form.fits(numeric) {
                unmarked.push(heading);
                continue;
            }
            let info = SectionInfo {
                // §FS-declarations.section.relative.shown: anchored as written and rendered.
                title: reduce_heading_text(&heading.title),
                line: heading.line,
                heading_level: level,
                value: None,
                value_root: None,
                relative: true,
            };
            claim(decl, canonical.clone(), info);
            accepted.insert(heading.line, canonical);
        }
    }
    for heading in &mut record.headings {
        if let Some(path) = accepted.remove(&heading.line) {
            heading.path = Some(path);
            heading.relative = true;
        }
    }
}

/// Record `path` on `decl` by the first-claimant rule (§AR-scanner.2.2.3): the
/// heading earliest in the file holds it and every later one is a duplicate,
/// whichever form each was written in (§FS-declarations.checks.duplicate-section.5).
fn claim(decl: &mut Declaration, path: String, info: SectionInfo) {
    match decl.sections.remove(&path) {
        Some(first) if first.line < info.line => {
            decl.sections.insert(path.clone(), first);
            decl.duplicate_sections.push((path, info));
        }
        Some(later) => {
            decl.sections.insert(path.clone(), info);
            decl.duplicate_sections.push((path, later));
        }
        None => {
            decl.sections.insert(path, info);
        }
    }
}

/// The path of the parent a relative heading at `level` on `line` takes, empty
/// for the declaration itself, or `None` when the nearest shallower heading is
/// not one level up or not usable (§FS-declarations.section.relative.parent).
fn parent_path(
    record: &FileRecord,
    decl: &Declaration,
    end: usize,
    line: usize,
    level: usize,
) -> Option<String> {
    let nearest = record
        .headings_between(decl.line, line - 1)
        .iter()
        .rev()
        .find(|heading| heading.atx && heading.level < level)?;
    if nearest.level + 1 != level {
        return None;
    }
    usable(decl, end, nearest.line)
}

/// The path of the heading on `line` when it is the declaration heading (empty)
/// or a usable section (§FS-declarations.section.relative.parent).
fn usable(decl: &Declaration, end: usize, line: usize) -> Option<String> {
    if line == decl.line {
        return Some(String::new());
    }
    let (path, _) = decl.sections.iter().find(|(_, info)| info.line == line)?;
    usable_section(decl, end, path).then(|| path.clone())
}

/// Whether `path` is a usable section of `decl`, whose body ends on `end`:
/// recorded, the only claimant of its path in the body, and not an orphan
/// (§FS-declarations.section.relative.parent). The unmarked-heading clause asks
/// the same of a suggested coordinate's parent (§FS-declarations.checks.unmarked-heading.2.1).
pub(super) fn usable_section(decl: &Declaration, end: usize, path: &str) -> bool {
    let in_body = |at: usize| decl.line < at && at <= end;
    let Some(first) = decl.sections.get(path).filter(|info| in_body(info.line)) else {
        return false;
    };
    let later = decl
        .duplicate_sections
        .iter()
        .any(|(other, info)| other == path && in_body(info.line));
    // §FS-declarations.checks.orphan-section: a name-bearing path needs every prefix.
    let parts = path.split('.').collect::<Vec<_>>();
    let orphan = !section_path_is_numeric(path)
        && (1..parts.len()).any(|cut| !decl.sections.contains_key(&parts[..cut].join(".")));
    first.heading_level > decl.heading_level && !later && !orphan
}

/// Where a failed relative heading needed its parent: the nearest heading before
/// it shallower than `needed` that is the declaration or a usable section
/// (§FS-declarations.checks.relative-section-parent). The declaration heading
/// always qualifies.
fn nearest_usable(
    record: &FileRecord,
    decl: &Declaration,
    end: usize,
    line: usize,
    needed: usize,
) -> String {
    record
        .headings_between(decl.line, line - 1)
        .iter()
        .rev()
        .filter(|heading| heading.atx && heading.level < needed)
        .find_map(|heading| usable(decl, end, heading.line))
        .unwrap_or_default()
}

/// A relative heading as its error quotes it: the `#` run, a space, and its `.`
/// and component (§FS-declarations.checks.relative-section-parent).
fn written(heading: &UnmarkedHeadingCandidate) -> String {
    let component = relative_heading_form(&heading.title).map_or("", |form| form.component);
    format!("{} .{component}", "#".repeat(heading.heading_level))
}
