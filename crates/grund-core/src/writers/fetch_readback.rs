//! The home as it would read after (§FS-fetch.admission.2, §FS-fetch.admission.3,
//! §FS-fetch.admission.5): the selected project's scan with the candidate in front of the target,
//! held to the tree as the scan read it before the integration ran. A line below the replaced
//! range is compared by its distance from the end of the file, so a finding or a declaration the
//! home already held is never new because the output's length moved it.

use std::collections::BTreeMap;
use std::path::Path;

use super::fetch::{FetchFailure, fetch_operational};
use super::fetch_splice::{Owned, first_differing_line, replace};
use super::fetch_target::{Before, Request};
use crate::checker::{conform, invalid_value_declarations};
use crate::config::{Config, Frame};
use crate::grammar::render_catalog_id;
use crate::model::{Catalog, Code, Declaration, Diagnostic, FileRecord, scanned_path_key};

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
    fn same(&self, path: &Path) -> bool {
        scanned_path_key(path) == scanned_path_key(self.path)
    }

    fn after_record(&self) -> Option<&FileRecord> {
        self.after
            .files
            .iter()
            .find(|(file, _)| self.same(file))
            .map(|(_, record)| record.as_ref())
    }

    /// Where line `line` of the target stands once the range is set aside: before it as itself,
    /// in it by its place in the range, after it by its distance from the end.
    fn place(&self, after: bool, line: usize) -> String {
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
            Some(range) if line > range.last => format!("-{}", lines_in(text) - line),
            _ => line.to_string(),
        }
    }

    fn local(&self) -> &str {
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
                Some(below) if below > 0 => {
                    lines_in(before_text) - (lines_in(readback.candidate) - decl.line)
                }
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

/// §FS-fetch.admission.3: `check`'s node-local rules and the scan's invalid-value records report
/// nothing new in the home, warnings included. The relational rules are not run.
pub(super) fn no_new_findings(readback: &Readback<'_>) -> Result<(), FetchFailure> {
    let config = readback.request.config;
    let held = findings(readback, config, &readback.before.catalog);
    let mut was: BTreeMap<(Code, String, String, String), usize> = BTreeMap::new();
    for diagnostic in &held {
        *was.entry(key(readback, false, config.frame(), diagnostic))
            .or_default() += 1;
    }
    let mut found = findings(readback, readback.view, readback.after);
    // §FS-fetch.admission.3: the first new finding in the order `grund check` prints them.
    found.sort_by(crate::checker::diagnostic_cmp);
    for diagnostic in found {
        let held = was
            .get_mut(&key(readback, true, readback.view.frame(), &diagnostic))
            .filter(|count| **count > 0);
        if let Some(count) = held {
            *count -= 1;
            continue;
        }
        return Err(new_finding(readback, &diagnostic));
    }
    Ok(())
}

/// The refusal for a finding the candidate adds, in the output or elsewhere in the home.
fn new_finding(readback: &Readback<'_>, diagnostic: &Diagnostic) -> FetchFailure {
    let local = readback.local();
    let path = diagnostic.path.as_deref().unwrap_or(readback.path);
    let line = diagnostic.line.unwrap_or_default();
    let new = readback.new;
    if readback.same(path) && line >= new.first && line <= new.last {
        let at = line + 1 - new.first;
        // §FS-fetch.3.2: an unmarked heading keeps fetch's own line.
        if diagnostic.code == Code::UNMARKED_HEADING {
            let heading = readback
                .after_record()
                .and_then(|record| record.line(line))
                .unwrap_or_default();
            return fetch_operational(format!(
                "fetch output line {at}: heading `{}` is not a section of `{local}`; number it, \
                 fence it, or make it a bold label",
                heading.trim_end()
            ));
        }
        return fetch_operational(format!("fetch output line {at}: {}", diagnostic.message));
    }
    let shown = readback.request.shown(path);
    match diagnostic.line {
        Some(line) => fetch_operational(format!(
            "cannot fetch {local}: {shown}:{line}: {}",
            diagnostic.message
        )),
        None => fetch_operational(format!(
            "cannot fetch {local}: {shown}: {}",
            diagnostic.message
        )),
    }
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

/// The errors and warnings `check`'s node-local half reports in the home on one reading.
fn findings(readback: &Readback<'_>, config: &Config, catalog: &Catalog) -> Vec<Diagnostic> {
    let report = conform(config.schema(), catalog, config.frame());
    report
        .errors
        .into_iter()
        .chain(report.warnings)
        .chain(invalid_value_declarations(catalog, config.frame()))
        .filter(|diagnostic| {
            diagnostic
                .path
                .as_deref()
                .is_some_and(|path| readback.request.home.holds(path))
        })
        .collect()
}

/// A finding as clause 3 compares it: code, file, and its line and message with every line of
/// the target placed by `Readback::place`.
fn key(
    readback: &Readback<'_>,
    after: bool,
    frame: Frame<'_>,
    diagnostic: &Diagnostic,
) -> (Code, String, String, String) {
    let path = diagnostic.path.as_deref().unwrap_or(readback.path);
    let in_target = readback.same(path);
    let line = diagnostic.line.map_or_else(String::new, |line| {
        if in_target {
            readback.place(after, line)
        } else {
            line.to_string()
        }
    });
    let shown = format!("{}:", frame.display_path(readback.path));
    // A finding names lines of the target as `path:N` anywhere, and as `line N` or `lines N–M`
    // in a finding on the target itself.
    let mut markers = vec![shown.as_str()];
    if in_target {
        markers.extend(["line ", "lines ", "–"]);
    }
    let mut message = String::new();
    let mut rest = diagnostic.message.as_str();
    while let Some((at, marker)) = markers
        .iter()
        .filter_map(|marker| rest.find(marker).map(|at| (at, *marker)))
        .min()
    {
        let from = at + marker.len();
        let digits = rest[from..].bytes().take_while(u8::is_ascii_digit).count();
        message.push_str(&rest[..from]);
        let number = &rest[from..from + digits];
        match number.parse::<usize>() {
            Ok(line) => message.push_str(&readback.place(after, line)),
            Err(_) => message.push_str(number),
        }
        rest = &rest[from + digits..];
    }
    message.push_str(rest);
    (
        diagnostic.code,
        scanned_path_key(path).display().to_string(),
        line,
        message,
    )
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
