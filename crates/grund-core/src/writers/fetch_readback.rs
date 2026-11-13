//! The home as it would read after (§FS-fetch.admission.2, §FS-fetch.admission.5): the one
//! scan's reading with the candidate read in place of the target (§FS-fetch.6.1), held to the
//! tree as the scan read it before the integration ran. A line below the replaced range is
//! compared by its distance from the end of the file, so a declaration the home already held is
//! never new because the output's length moved it; clause 3 places its lines the same way, in
//! `fetch_findings.rs`.

use std::path::Path;

use super::fetch::{FetchFailure, fetch_operational};
use super::fetch_splice::{Owned, first_differing_line, replace};
use super::fetch_target::{Before, Request};
use crate::config::Config;
use crate::grammar::render_catalog_id;
use crate::model::{Catalog, Declaration, FileRecord, scanned_path_key};

/// Both readings of the home and where the output sits in each.
pub(super) struct Readback<'a> {
    pub(super) request: &'a Request<'a>,
    pub(super) before: &'a Before,
    pub(super) after: &'a Catalog,
    /// The selected project with the candidate in front of the target.
    pub(super) view: &'a Config,
    /// The target as the scan names it.
    pub(super) path: &'a Path,
    /// The range the output replaces in the file before, `None` where there was no file.
    pub(super) old: Option<Owned>,
    /// The output's lines in the candidate.
    pub(super) new: Owned,
    pub(super) candidate: &'a str,
    /// A folder home's file, which the output is whole.
    pub(super) whole: bool,
}

impl Readback<'_> {
    pub(super) fn same(&self, path: &Path) -> bool {
        scanned_path_key(path) == scanned_path_key(self.path)
    }

    pub(super) fn after_record(&self) -> Option<&FileRecord> {
        self.after
            .files
            .iter()
            .find(|(file, _)| self.same(file))
            .map(|(_, record)| record.as_ref())
    }

    /// Where line `line` of the target stands once the range is set aside: before it as itself,
    /// in it by its place in the range, after it by its distance from the end.
    pub(super) fn place(&self, after: bool, line: usize) -> String {
        let (range, text) = if after {
            (Some(self.new), self.candidate)
        } else {
            (
                self.old,
                self.before.record(self.path).map_or("", FileRecord::text),
            )
        };
        match range {
            Some(range) if line >= range.first && line <= range.last => {
                format!("@{}", line - range.first)
            }
            // A line past the end is no line of the file, and stays as it is written.
            Some(range) if line > range.last => lines_in(text)
                .checked_sub(line)
                .map_or_else(|| line.to_string(), |from_end| format!("-{from_end}")),
            _ => line.to_string(),
        }
    }

    pub(super) fn local(&self) -> &str {
        self.request.local
    }
}

/// One declaration of the target, as clause 2 compares it, the replaced range set aside.
#[derive(PartialEq)]
struct Reading {
    id: crate::model::Id,
    title: Option<String>,
    level: usize,
    sections: Vec<String>,
    body: Vec<String>,
}

/// §FS-fetch.admission.2: every other declaration in the home reads as it did before. One the
/// scan read inside the replaced range would be deleted (§FS-fetch.3.1); any other that appears,
/// disappears or changes is named where it stood before.
pub(super) fn keep_others(readback: &Readback<'_>) -> Result<(), FetchFailure> {
    let local = readback.local();
    let before = &readback.before.catalog;
    let grammar = &readback.request.config.grammar;
    let shown = readback.request.shown(readback.path);
    let written = |catalog: &Catalog, decl: &Declaration| {
        decl.authored_id
            .clone()
            .unwrap_or_else(|| render_catalog_id(grammar, &catalog.declarations, &decl.id))
    };
    if let Some(old) = readback.old {
        let inside = |line: usize| line > old.first && line <= old.last;
        let mut deleted: Vec<(usize, String)> = declarations_in(before, readback.path)
            .filter(|decl| inside(decl.line))
            .map(|decl| (decl.line, written(before, decl)))
            .chain(
                before
                    .near_miss_headings
                    .iter()
                    .filter(|near| readback.same(&near.file) && inside(near.line))
                    .map(|near| (near.line, near.text.clone())),
            )
            .collect();
        deleted.sort();
        if let Some((line, other)) = deleted.into_iter().next() {
            return Err(fetch_operational(format!(
                "cannot fetch {local}: {shown}:{line} declares {other} inside {local}, and \
                 replacing {local} would delete it; move {other} out of {local}, and fetch again"
            )));
        }
    }
    let was = readings(
        before,
        readback.path,
        readback.before.record(readback.path),
        readback.old,
    );
    let is = readings(
        readback.after,
        readback.path,
        readback.after_record(),
        Some(readback.new),
    );
    let changed = (0..was.len().max(is.len())).find(|at| {
        was.get(*at).map(|(_, reading)| reading) != is.get(*at).map(|(_, reading)| reading)
    });
    let Some(at) = changed else {
        return Ok(());
    };
    let (line, other) = match (was.get(at), is.get(at)) {
        (Some((decl, _)), _) => (decl.line, written(before, decl)),
        (None, Some((decl, _))) => {
            let before_text = readback
                .before
                .record(readback.path)
                .map_or("", FileRecord::text);
            let line = match decl.line.checked_sub(readback.new.last) {
                Some(below) if below > 0 => lines_in(readback.candidate)
                    .checked_sub(decl.line)
                    .and_then(|from_end| lines_in(before_text).checked_sub(from_end))
                    .unwrap_or(decl.line),
                _ => decl.line,
            };
            (line, written(readback.after, decl))
        }
        (None, None) => return Ok(()),
    };
    Err(fetch_operational(format!(
        "cannot fetch {local}: writing the output would change how {shown}:{line} reads \
         {other}; change the output or {other} so the two stand apart, and fetch again"
    )))
}

/// The target's declarations outside `range`, each as clause 2 compares it, in line order.
fn readings<'a>(
    catalog: &'a Catalog,
    path: &'a Path,
    record: Option<&FileRecord>,
    range: Option<Owned>,
) -> Vec<(&'a Declaration, Reading)> {
    let outside = |line: usize| range.is_none_or(|range| line < range.first || line > range.last);
    let mut found: Vec<(&Declaration, Reading)> = declarations_in(catalog, path)
        .filter(|decl| outside(decl.line))
        .map(|decl| {
            let body = (decl.body_start..=decl.body_end)
                .filter(|line| outside(*line))
                .filter_map(|line| record.and_then(|record| record.line(line)))
                .filter(|text| !text.trim().is_empty())
                .map(|text| text.trim_end().to_string())
                .collect();
            let reading = Reading {
                id: decl.id.clone(),
                title: decl.title.clone(),
                level: decl.heading_level,
                sections: decl
                    .sections
                    .iter()
                    .filter(|(_, info)| outside(info.line))
                    .map(|(path, _)| path.clone())
                    .collect(),
                body,
            };
            (decl, reading)
        })
        .collect();
    found.sort_by_key(|(decl, _)| decl.line);
    found
}

/// §FS-fetch.admission.5: the output spliced again at the range the candidate's own reading
/// records reproduces the candidate byte for byte. A folder home's file is the output whole.
pub(super) fn settled(readback: &Readback<'_>, output: &str) -> Result<(), FetchFailure> {
    if readback.whole {
        return Ok(());
    }
    let Some(record) = readback.after_record() else {
        return Ok(());
    };
    let Some(decl) = declarations_in(readback.after, readback.path)
        .find(|decl| decl.id == *readback.request.id && decl.line == readback.new.first)
    else {
        return Ok(());
    };
    let again = replace(
        readback.candidate,
        &Owned::of(record, decl.line, decl.body_end),
        output,
    );
    if again == readback.candidate {
        return Ok(());
    }
    Err(fetch_operational(format!(
        "cannot fetch {}: a second fetch of this output would change {} at line {} again, so \
         the first is refused",
        readback.local(),
        readback.request.shown(readback.path),
        first_differing_line(readback.candidate, &again)
    )))
}

/// The target's declarations the scan read, stubs left out, the requested one included.
fn declarations_in<'a>(
    catalog: &'a Catalog,
    path: &'a Path,
) -> impl Iterator<Item = &'a Declaration> {
    catalog
        .declarations
        .values()
        .flatten()
        .filter(move |decl| !decl.is_stub && scanned_path_key(&decl.file) == scanned_path_key(path))
}

/// How many lines `text` holds, counted the same way on both sides of a splice.
fn lines_in(text: &str) -> usize {
    text.matches('\n').count() + 1
}
