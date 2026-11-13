//! Clause 3 of admission (§FS-fetch.admission.3): `check`'s node-local rules report nothing new
//! in the home. Findings are compared by code and message, with every line of the target a
//! finding records placed by `Readback::place`, so a finding the home already held is never new
//! because the output's length moved it. A line is taken only from what the finding records
//! structurally — its own line, its sites, and for a duplicate the declarations it was made
//! from — and found in the message as the exact words the checker builds from it, so a number in
//! authored text the message quotes is never read as a line.

use std::collections::BTreeMap;

use super::fetch::{FetchFailure, fetch_operational};
use super::fetch_readback::Readback;
use crate::checker::{block_lines, conform, invalid_value_declarations};
use crate::config::{Config, Frame};
use crate::model::{Catalog, Code, Diagnostic, scanned_path_key};

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
/// the target it records placed by `Readback::place`.
fn key(
    readback: &Readback<'_>,
    after: bool,
    frame: Frame<'_>,
    diagnostic: &Diagnostic,
) -> (Code, String, String, String) {
    let path = diagnostic.path.as_deref().unwrap_or(readback.path);
    let line = diagnostic.line.map_or_else(String::new, |line| {
        if readback.same(path) {
            readback.place(after, line)
        } else {
            line.to_string()
        }
    });
    let mut lines: Vec<usize> = diagnostic
        .sites
        .iter()
        .filter(|site| readback.same(&site.path))
        .map(|site| site.line)
        .collect();
    let message = if diagnostic.code == Code::CONFLICT_MARKER {
        // §FS-declarations.checks.conflict-marker: the block's sites are its first and last line.
        let words = |line: &dyn Fn(usize) -> String| match lines.as_slice() {
            [one] => Some(block_lines(line(*one), None)),
            [first, last] => Some(block_lines(line(*first), Some(line(*last)))),
            _ => None,
        };
        match (
            words(&|at| at.to_string()),
            words(&|at| readback.place(after, at)),
        ) {
            (Some(written), Some(placed)) => {
                diagnostic
                    .message
                    .replacen(&format!(", {written}"), &format!(", {placed}"), 1)
            }
            _ => diagnostic.message.clone(),
        }
    } else {
        if diagnostic.code == Code::DUPLICATE {
            lines.extend(declared_with(readback, after, path, diagnostic));
        }
        lines.sort_unstable();
        lines.dedup();
        let shown = frame.display_path(readback.path);
        lines
            .iter()
            .fold(diagnostic.message.clone(), |message, at| {
                replace_site(
                    &message,
                    &format!("{shown}:{at}"),
                    &format!("{shown}:{}", readback.place(after, *at)),
                )
            })
    };
    (
        diagnostic.code,
        scanned_path_key(path).display().to_string(),
        line,
        message,
    )
}

/// The lines in the target of every declaration of the ID a duplicate finding is anchored at,
/// stubs included: the records a duplicate's message names, each as `<file>:<line>`
/// (§FS-declarations.checks.duplicate).
fn declared_with(
    readback: &Readback<'_>,
    after: bool,
    path: &std::path::Path,
    diagnostic: &Diagnostic,
) -> Vec<usize> {
    let catalog = if after {
        readback.after
    } else {
        &readback.before.catalog
    };
    let anchored = |decl: &&crate::model::Declaration| {
        Some(decl.line) == diagnostic.line && scanned_path_key(&decl.file) == scanned_path_key(path)
    };
    catalog
        .declarations
        .values()
        .find(|decls| decls.iter().any(|decl| anchored(&decl)))
        .into_iter()
        .flatten()
        .filter(|decl| readback.same(&decl.file))
        .map(|decl| decl.line)
        .collect()
}

/// `message` with each occurrence of `written`, a site as the checker writes it, put as
/// `placed`. An occurrence counts only where it stands alone: at the start or after a space or
/// an opening parenthesis, and not followed by a digit, so `a.md:9` is never found in `b/a.md:9`
/// or in `a.md:93`.
fn replace_site(message: &str, written: &str, placed: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(at) = rest.find(written) {
        let end = at + written.len();
        let before = if at > 0 {
            rest[..at].chars().last()
        } else {
            out.chars().last()
        };
        let opens = before.is_none_or(|ch| ch == ' ' || ch == '(');
        let closes = !rest[end..].starts_with(|ch: char| ch.is_ascii_digit());
        out.push_str(&rest[..at]);
        out.push_str(if opens && closes { placed } else { written });
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}
