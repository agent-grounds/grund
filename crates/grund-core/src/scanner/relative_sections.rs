//! The parent pass (§FS-declarations.section.relative.parent, §AR-scanner.2.2):
//! each relative heading the per-file pass met takes its canonical path from
//! the nearest shallower heading of its declaration body, once every heading of
//! the file is recorded. It runs before the extents are settled, so a resolved
//! relative heading enters the section map, the record and every reader of the
//! two exactly as a complete-path heading does, and nothing parses it again.

use std::collections::BTreeMap;

use super::context::section_path_is_numeric;
use super::embedded_value_context::invalid_embedded_marker;
use super::extents::{DeclarationBodies, body_end};
use super::held_mixed_headings::seed_held_mixed_headings;
use super::section_record::section_value_root;
use crate::config::Schema;
use crate::grammar::{named_mode_section_path, reduce_heading_text, relative_heading_form};
use crate::model::{
    Catalog, Declaration, EmbeddedValueRoot, FileRecord, HeadingKind, InvalidValueSite,
    RecordedHeading, RelativeSectionParent, SectionInfo, UnmarkedHeadingCandidate,
};

/// A relative candidate, with the finding its value marker earns if no
/// accepted heading takes the marker (§FS-values.2.4.4).
pub(super) type RelativeCandidate = (UnmarkedHeadingCandidate, Option<InvalidValueSite>);

/// The Markdown headings the per-file pass holds until the file's headings are
/// known: the unmarked candidates, and the relative ones the parent pass settles
/// (§FS-declarations.section.relative.form, §FS-declarations.checks.unmarked-heading).
#[derive(Default)]
pub(super) struct HeldHeadings {
    pub(super) unmarked: Vec<UnmarkedHeadingCandidate>,
    relative: Vec<RelativeCandidate>,
}

impl HeldHeadings {
    /// Hold `candidate` as relative when its text reads as the relative form.
    pub(super) fn push(&mut self, candidate: UnmarkedHeadingCandidate) {
        if relative_heading_form(&candidate.title).is_some() {
            self.relative.push((candidate, None));
        } else {
            self.unmarked.push(candidate);
        }
    }

    /// Hand a misplaced value marker on `line` to the relative heading there, for
    /// the parent pass to attach (§FS-values.2.4); any other marker is returned.
    pub(super) fn hold_marker(
        &mut self,
        line: usize,
        site: InvalidValueSite,
    ) -> Option<InvalidValueSite> {
        match self.relative.last_mut() {
            Some((candidate, pending)) if candidate.line == line => {
                *pending = Some(site);
                None
            }
            _ => Some(site),
        }
    }

    /// Run the parent pass over the relative headings, leaving the unmarked ones
    /// in line order.
    pub(super) fn resolve(
        &mut self,
        findings: &mut Catalog,
        record: &mut FileRecord,
        named_sections: bool,
        schema: &Schema,
    ) {
        if self.relative.is_empty() {
            return;
        }
        let relative = std::mem::take(&mut self.relative);
        resolve_relative_sections(
            findings,
            record,
            relative,
            named_sections,
            schema,
            &mut self.unmarked,
        );
        self.unmarked.sort_by_key(|candidate| candidate.line);
    }
}

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
/// failed or a claimant of a duplicated path — before a child asks for it. An
/// accepted heading is a value root as its complete form would be, enrolled at
/// its canonical path (§FS-values.2.4, §FS-values.2.5); a marker no accepted
/// heading takes is reported as the per-file pass would have.
fn resolve_relative_sections(
    findings: &mut Catalog,
    record: &mut FileRecord,
    candidates: Vec<RelativeCandidate>,
    named_sections: bool,
    schema: &Schema,
    unmarked: &mut Vec<UnmarkedHeadingCandidate>,
) {
    let bodies = DeclarationBodies::of(record, findings);
    let mut by_owner: BTreeMap<usize, Vec<RelativeCandidate>> = BTreeMap::new();
    let mut markers = Vec::new();
    for (candidate, marker) in candidates {
        match bodies.owner(candidate.line, candidate.heading_level) {
            Some((start, _)) => by_owner.entry(start).or_default().push((candidate, marker)),
            None => {
                markers.extend(marker);
                unmarked.push(candidate);
            }
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
        owned.sort_by_key(|(candidate, _)| (candidate.heading_level, candidate.line));
        for (heading, mut marker) in owned {
            let level = heading.heading_level;
            let canonical = match relative_answer(
                record,
                decl,
                end,
                heading.line,
                level,
                &heading.title,
                named_sections,
            ) {
                RelativeAnswer::Section(canonical) => canonical,
                RelativeAnswer::NoParent => {
                    markers.extend(marker);
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
                }
                RelativeAnswer::Refused => {
                    markers.extend(marker);
                    unmarked.push(heading);
                    continue;
                }
            };
            let column = marker.as_ref().and_then(|site| site.column);
            let value_root = section_value_root(schema, &decl.id.kind, &canonical, column);
            if value_root
                .as_ref()
                .is_some_and(|root| root.marker_column().is_some())
            {
                marker = None;
            }
            let info = SectionInfo {
                // §FS-declarations.section.relative.shown: anchored as written and rendered.
                title: reduce_heading_text(&heading.title),
                line: heading.line,
                heading_level: level,
                value: None,
                value_root,
                relative: true,
            };
            // §FS-values.2.4.3: a duplicated root's marker is reported where it lies.
            if let Some((line, Some(column))) = claim(decl, canonical.clone(), info) {
                let message = "embedded value root coordinate is duplicated";
                let id = Some(decl.id.clone());
                markers.push(invalid_embedded_marker(
                    id,
                    &heading.file,
                    line,
                    column,
                    message,
                ));
            }
            markers.extend(marker);
            accepted.insert(heading.line, canonical);
        }
    }
    findings.invalid_value_declarations.extend(markers);
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
/// Returns the line of the heading that became the duplicate, with its value
/// marker's column if it was a marked root.
pub(super) fn claim(
    decl: &mut Declaration,
    path: String,
    info: SectionInfo,
) -> Option<(usize, Option<usize>)> {
    let duplicate = match decl.sections.remove(&path) {
        Some(first) if first.line < info.line => {
            decl.sections.insert(path.clone(), first);
            info
        }
        Some(later) => {
            decl.sections.insert(path.clone(), info);
            later
        }
        None => {
            decl.sections.insert(path, info);
            return None;
        }
    };
    let marker = duplicate
        .value_root
        .as_ref()
        .and_then(EmbeddedValueRoot::marker_column);
    let at = (duplicate.line, marker);
    decl.duplicate_sections.push((path, duplicate));
    Some(at)
}

/// What a relative heading is in its declaration's body (§FS-declarations.section.relative):
/// the section at its canonical path, a heading with no usable parent one level up
/// (§FS-declarations.checks.relative-section-parent), or a line the form's rules refuse,
/// which keeps the meaning it has without this form.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RelativeAnswer {
    Section(String),
    NoParent,
    Refused,
}

/// The one answer to what the relative heading titled `title`, at `level` on `line`
/// of `decl`'s body ending on `end`, would be: its parent by
/// §FS-declarations.section.relative.parent, then the title syntax its canonical
/// path takes (§FS-declarations.section.relative.form). The Markdown parent pass
/// and the doc-comment warning of §FS-declarations.checks.parse-moves.2 (shape 4)
/// both ask it, so the two readings cannot drift.
pub(crate) fn relative_answer(
    record: &FileRecord,
    decl: &Declaration,
    end: usize,
    line: usize,
    level: usize,
    title: &str,
    named_sections: bool,
) -> RelativeAnswer {
    let Some(parent) = parent_path(record, decl, end, line, level) else {
        return RelativeAnswer::NoParent;
    };
    let Some(form) = relative_heading_form(title) else {
        return RelativeAnswer::Refused;
    };
    let canonical = if parent.is_empty() {
        form.component.to_string()
    } else {
        format!("{parent}.{}", form.component)
    };
    let numeric = section_path_is_numeric(&canonical);
    // §FS-declarations.section.relative.form: refused wherever the complete form
    // would be — a name with the gate off, the wrong punctuation; a name under a
    // number is a section, as its complete form is (§FS-config.3.3.1).
    let shaped = numeric
        || (named_sections
            && named_mode_section_path(&canonical, |part| {
                part.bytes().all(|byte| byte.is_ascii_digit())
            }));
    if !shaped || !form.fits(numeric) {
        return RelativeAnswer::Refused;
    }
    RelativeAnswer::Section(canonical)
}

/// 0.19.0's answer for each relative heading of one source doc-comment declaration
/// body, which 0.18.0 keeps as prose (§FS-declarations.section.relative.where): the
/// parent pass run on a copy of `decl`, shallowest first and claiming as it goes,
/// so a relative parent is settled before its child asks for it. The complete
/// mixed headings 0.18.0 holds back as prose are claimed on the copy first, as the
/// per-file pass records a complete heading before the parent pass runs, so a
/// relative heading below one is named under it
/// (§FS-declarations.checks.parse-moves.1). Only the plain headings the record
/// holds in the body are read, so the file is not.
pub(crate) fn doc_comment_relative_answers<'a>(
    record: &'a FileRecord,
    decl: &Declaration,
    end: usize,
    named_sections: bool,
) -> Vec<(&'a RecordedHeading, RelativeAnswer)> {
    let plain = record
        .headings_between(decl.line + 1, end)
        .iter()
        .filter(|heading| heading.kind == HeadingKind::Plain);
    let mut candidates: Vec<&RecordedHeading> = plain
        .clone()
        .filter(|heading| relative_heading_form(record.heading_text(heading)).is_some())
        .collect();
    if candidates.is_empty() {
        return Vec::new();
    }
    candidates.sort_by_key(|heading| (heading.level, heading.line));
    let mut settled = decl.clone();
    if named_sections {
        seed_held_mixed_headings(record, decl, plain, &mut settled);
    }
    let mut answers = Vec::with_capacity(candidates.len());
    for heading in candidates {
        let title = record.heading_text(heading);
        let answer = relative_answer(
            record,
            &settled,
            end,
            heading.line,
            heading.level,
            title,
            named_sections,
        );
        if let RelativeAnswer::Section(canonical) = &answer {
            let info = SectionInfo {
                title: reduce_heading_text(title),
                line: heading.line,
                heading_level: heading.level,
                value: None,
                value_root: None,
                relative: true,
            };
            claim(&mut settled, canonical.clone(), info);
        }
        answers.push((heading, answer));
    }
    answers.sort_by_key(|(heading, _)| heading.line);
    answers
}

/// Whether `heading` is one a body's section reading sees: an ATX heading in
/// Markdown, and every heading the record holds of a source doc-comment, which it
/// read through the comment's envelope (§FS-languages.comment-wrapper.headings).
fn read_as_heading(record: &FileRecord, heading: &RecordedHeading) -> bool {
    heading.atx || !record.markdown
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
        .find(|heading| read_as_heading(record, heading) && heading.level < level)?;
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
    let orphan = decl.missing_section_prefix(path).is_some();
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
        .filter(|heading| read_as_heading(record, heading) && heading.level < needed)
        .find_map(|heading| usable(decl, end, heading.line))
        .unwrap_or_default()
}

/// A relative heading as its error quotes it: the `#` run, a space, and its `.`
/// and component (§FS-declarations.checks.relative-section-parent).
fn written(heading: &UnmarkedHeadingCandidate) -> String {
    let component = relative_heading_form(&heading.title).map_or("", |form| form.component);
    format!("{} .{component}", "#".repeat(heading.heading_level))
}
