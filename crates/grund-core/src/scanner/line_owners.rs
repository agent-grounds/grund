//! Which declaration and section own a line of one file (§AR-scanner.2.4.3,
//! §AR-scanner.2.4.4): the one lookup the citation post-pass and a
//! `cover --lines` request both ask (§FS-cover.6.2). A requested range is
//! resolved inside the file's pass, beside the record whose headings close a
//! section (§AR-scanner.2.8).

use std::path::Path;

use crate::model::{
    Catalog, FileLineOwnership, FileRecord, Id, OwnerRun, RangeOwnership, SectionRun,
};

use super::compat::TodayOwnerEnds;

/// One file's declaration bodies with their accepted sections, and the
/// record's fence-aware headings that close them: the single place a line is given
/// its owner (§AR-scanner.2.4.3, §AR-scanner.2.4.4). The citation post-pass and a
/// `cover --lines` request both ask [`LineOwners::owner_at`], so a line and a
/// citation on it can never be given different owners (§FS-cover.6.2).
pub(super) struct LineOwners<'a> {
    /// `(body_start, body_end, id, sections)`, each section as
    /// `(path, heading line, heading level)` in line order.
    bodies: Vec<(usize, usize, Id, Vec<(String, usize, usize)>)>,
    ends: TodayOwnerEnds<'a>,
}

impl<'a> LineOwners<'a> {
    /// Taken after the body spans are assigned, so the lookup is a scan of a
    /// small local list rather than of the file.
    pub(super) fn new(findings: &Catalog, record: &'a FileRecord) -> Self {
        let bodies = findings
            .declarations
            .values()
            .flatten()
            .map(|decl| {
                let mut sections = decl
                    .sections
                    .iter()
                    .map(|(path, info)| (path.clone(), info.line, info.heading_level))
                    .collect::<Vec<_>>();
                sections.sort_by_key(|(_, line, _)| *line);
                (decl.body_start, decl.body_end, decl.id.clone(), sections)
            })
            .collect();
        Self {
            bodies,
            ends: TodayOwnerEnds::new(record),
        }
    }

    /// The declaration whose body contains `line` — the nearest preceding one when
    /// bodies nest — and the innermost accepted section of it containing `line`,
    /// or `None` for a line no body contains (§AR-scanner.2.4.3,
    /// §AR-scanner.2.4.4, §FS-cover.6.2).
    pub(super) fn owner_at(&self, line: usize) -> Option<(&Id, Option<String>)> {
        let (_, _, id, sections) = self
            .bodies
            .iter()
            .filter(|(start, end, _, _)| *start <= line && line <= *end)
            // Nearest preceding declaration: the one whose body starts latest.
            .max_by_key(|(start, _, _, _)| *start)?;
        Some((id, enclosing_section(sections, &self.ends, line)))
    }
}

/// Resolve the nearest accepted chapter, closed where 0.17 closes it
/// (§FS-rules.2, §AR-scanner.2.4.4, `TodayOwnerEnds`).
fn enclosing_section(
    sections: &[(String, usize, usize)],
    ends: &TodayOwnerEnds<'_>,
    site_line: usize,
) -> Option<String> {
    let (path, line, depth) = sections
        .iter()
        .filter(|(_, line, _)| *line <= site_line)
        .max_by_key(|(_, line, _)| *line)?;
    (site_line <= ends.end(sections, *line, *depth)).then(|| path.clone())
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
fn owner_runs(owners: &LineOwners<'_>, start: usize, end: usize) -> Vec<OwnerRun> {
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
