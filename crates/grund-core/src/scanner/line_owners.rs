//! Which declaration and section own a line of one file (§AR-scanner.2.4.3,
//! §AR-scanner.2.4.4): the one lookup the citation post-pass and a
//! `cover --lines` request both ask (§FS-cover.6.2). A requested range is
//! resolved inside the file's pass, beside the record whose headings close a
//! section (§AR-scanner.2.8).

use std::path::Path;

use crate::config::FRONT_MATTER;
use crate::model::{
    Catalog, FileLineOwnership, FileRecord, HeadingKind, Id, OwnerRun, RangeOwnership, SectionRun,
};

/// One file's declaration bodies with their accepted sections, and the
/// record's fence-aware headings that close them: the single place a line is given
/// its owner (§AR-scanner.2.4.3, §AR-scanner.2.4.4). The citation post-pass and a
/// `cover --lines` request both ask [`LineOwners::owner_at`], so a line and a
/// citation on it can never be given different owners (§FS-cover.6.2).
pub(super) struct LineOwners {
    /// `(body_start, body_end, id, sections)`, each section as
    /// `(path, heading line, last line)` in line order.
    bodies: Vec<(usize, usize, Id, Vec<(String, usize, usize)>)>,
    /// The last line of the file's front matter, `None` where it has none
    /// (§FS-check.1.1.5.2).
    front_matter: Option<usize>,
    /// Where the block is a front-matter field, its key coordinates as
    /// `(path, first line, last line)`, `None` where it is not
    /// (§FS-fields.front-matter.surfaces).
    front_matter_keys: Option<Vec<(String, usize, usize)>>,
}

impl LineOwners {
    /// Taken after the body spans are assigned, so the lookup is a scan of a
    /// small local list rather than of the file.
    pub(super) fn new(findings: &Catalog, record: &FileRecord) -> Self {
        let bodies = findings
            .declarations
            .values()
            .flatten()
            .map(|decl| {
                // §AR-scanner.2.4.4: a section ends where the record's extent rule ends it.
                let mut sections = decl
                    .sections
                    .iter()
                    .filter_map(|(path, info)| {
                        let heading = record
                            .heading_at(info.line)
                            .filter(|heading| heading.kind == HeadingKind::Section)?;
                        Some((path.clone(), info.line, heading.end))
                    })
                    .collect::<Vec<_>>();
                sections.sort_by_key(|(_, line, _)| *line);
                (decl.body_start, decl.body_end, decl.id.clone(), sections)
            })
            .collect();
        // §FS-fields.front-matter.coordinates: the entries with a slice, never headings.
        let mut field = None;
        for decl in findings.declarations.values().flatten() {
            let slices = decl.sections.iter().filter_map(|(path, info)| {
                let slice = info.front_matter.as_ref()?;
                Some((path.clone(), info.line, slice.end_line))
            });
            let mut slices: Vec<_> = slices.collect();
            if slices.iter().any(|(path, _, _)| path == FRONT_MATTER) {
                slices.retain(|(path, _, _)| path != FRONT_MATTER);
                field = Some(slices);
            }
        }
        Self {
            bodies,
            front_matter: record.front_matter,
            front_matter_keys: field,
        }
    }

    /// The declaration whose body contains `line` — the nearest preceding one when
    /// bodies nest — and the innermost accepted section of it containing `line`,
    /// or `None` for a line no body contains (§AR-scanner.2.4.3,
    /// §AR-scanner.2.4.4, §FS-cover.6.2). A line of the file's front matter
    /// belongs to the file's first declaration, the one whose body starts
    /// earliest, under no section, though no body range covers it
    /// (§FS-check.1.1.5.2.1). Where the block is that declaration's front-matter
    /// field, the line is in the key whose slice holds it, and in `front-matter`
    /// outside every key (§FS-fields.front-matter.surfaces).
    pub(super) fn owner_at(&self, line: usize) -> Option<(&Id, Option<String>)> {
        if self.front_matter.is_some_and(|last| line <= last) {
            let (_, _, id, _) = self.bodies.iter().min_by_key(|(start, _, _, _)| *start)?;
            let section = self.front_matter_keys.as_ref().map(|keys| {
                keys.iter()
                    .find(|(_, start, end)| *start <= line && line <= *end)
                    .map_or_else(|| FRONT_MATTER.to_string(), |(path, _, _)| path.clone())
            });
            return Some((id, section));
        }
        let (_, _, id, sections) = self
            .bodies
            .iter()
            .filter(|(start, end, _, _)| *start <= line && line <= *end)
            // Nearest preceding declaration: the one whose body starts latest.
            .max_by_key(|(start, _, _, _)| *start)?;
        Some((id, enclosing_section(sections, line)))
    }
}

/// Resolve the nearest accepted chapter, closed where the record closes it: at a
/// heading of its level or above, numbered, plain or a duplicate claimant
/// (§AR-scanner.2.4.4).
fn enclosing_section(sections: &[(String, usize, usize)], site_line: usize) -> Option<String> {
    let (path, _, end) = sections
        .iter()
        .filter(|(_, line, _)| *line <= site_line)
        .max_by_key(|(_, line, _)| *line)?;
    (site_line <= *end).then(|| path.clone())
}

/// Record, for each range in `requested`, its owner runs and their section runs
/// (§FS-cover.6.3). A range is clipped to the file's last line; refusing one that
/// ends past it is the caller's, which knows how the range was written
/// (§FS-cover.6.4).
pub(super) fn resolve_requested_lines(
    findings: &mut Catalog,
    path: &Path,
    record: &FileRecord,
    requested: &[(usize, usize)],
) {
    let total_lines = record.total_lines;
    let owners = LineOwners::new(findings, record);
    let ranges = requested
        .iter()
        .map(|&(start, end)| RangeOwnership {
            start,
            end,
            owners: owner_runs(&owners, start, end.min(total_lines)),
        })
        .collect();
    findings.line_ownership.push(FileLineOwnership {
        file: path.to_path_buf(),
        total_lines,
        ranges,
    });
}

/// §FS-cover.6.3: the maximal runs of lines owned by one declaration, each split
/// into maximal runs under one innermost section. Lines nothing owns end a run
/// and appear in none, so one declaration a nested body interrupts appears twice.
fn owner_runs(owners: &LineOwners, start: usize, end: usize) -> Vec<OwnerRun> {
    let mut runs: Vec<OwnerRun> = Vec::new();
    let mut previous_owned = false;
    for line in start..=end {
        let Some((id, section)) = owners.owner_at(line) else {
            previous_owned = false;
            continue;
        };
        let continues = previous_owned && runs.last().is_some_and(|run| run.declaration == *id);
        previous_owned = true;
        if !continues {
            runs.push(OwnerRun {
                declaration: id.clone(),
                start: line,
                end: line,
                sections: vec![SectionRun {
                    section,
                    start: line,
                    end: line,
                }],
            });
            continue;
        }
        let run = runs.last_mut().expect("a continued run exists");
        run.end = line;
        match run.sections.last_mut() {
            Some(last) if last.section == section => last.end = line,
            _ => run.sections.push(SectionRun {
                section,
                start: line,
                end: line,
            }),
        }
    }
    runs
}
