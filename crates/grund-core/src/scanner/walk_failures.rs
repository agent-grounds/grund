//! One line per directory entry (§FS-check.2.4.2, §AR-scanner.1.entry): the step that keeps the
//! first of the walk's reports of one entry and reason, and the name each of the others lost.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;

use super::scan_error::FailureRoots;
use super::tree::ScanError;
use super::walk_errors::link_identity;
use crate::model::{FailureNames, sort_path_key};

/// §FS-errors.4.1: the walk meets its unreadable paths in readdir order, and the text report and
/// the API hand over this one order.
pub(super) fn sort_failures(failures: &mut [ScanError]) {
    failures.sort_by_key(|failure| (sort_path_key(&failure.path), failure.reason.clone()));
}

/// Keeps the first report of each directory entry and reason and answers with the names the
/// rest stood for (§AR-scanner.1.entry). The caller has each root's reports sorted and in root
/// order, so first is the name a file reached twice keeps (§AR-scanner.1.8).
///
/// A path `link_identity` cannot answer for compares as written, which costs a line too many and
/// never hides a failure (§REQ-no-missed-citation.1). Fewer than two reports have nothing to
/// compare, so a clean walk and a one-failure walk resolve nothing (§GOAL-fast-feedback).
pub(super) fn one_report_per_entry(failures: &mut Vec<ScanError>) -> FailureNames {
    let mut names = FailureNames::default();
    if failures.len() < 2 {
        return names;
    }
    let mut first = HashMap::<(PathBuf, String), PathBuf>::new();
    failures.retain(|failure| {
        let (path, reason) = (&failure.path, &failure.reason);
        let entry = link_identity(path).unwrap_or_else(|| path.clone());
        match first.entry((entry, reason.clone())) {
            Entry::Vacant(vacant) => {
                vacant.insert(path.clone());
                true
            }
            Entry::Occupied(occupied) => {
                names.keep(occupied.get(), reason, path);
                false
            }
        }
    });
    names
}

/// Settle each entry once in encounter order, including its typed place, then select its
/// whole-run name from the ordinary roots' listing order (§FS-check.2.4.2, §FS-check.2.4.3).
/// Moving the name must not reclassify a failure met below a passed looping link as a failure
/// to open that passed root. The handed root still contributes every alias for CLI scoping.
pub(super) fn whole_run_failures(
    failures: &mut Vec<ScanError>,
    handed: usize,
    at: &FailureRoots,
) -> FailureNames {
    if handed == 0 {
        let names = one_report_per_entry(failures);
        *failures = std::mem::take(failures)
            .into_iter()
            .map(|failure| failure.placed(at))
            .collect();
        return names;
    }
    let mut ordinary = HashMap::new();
    for (position, failure) in failures[handed..].iter().enumerate() {
        ordinary
            .entry((failure.path.clone(), failure.reason.clone()))
            .or_insert(position);
    }
    let encountered = one_report_per_entry(failures);
    let mut names = FailureNames::default();
    for failure in failures.iter_mut() {
        let aliases: Vec<_> = encountered
            .of(&failure.path, &failure.reason)
            .map(|name| name.to_path_buf())
            .collect();
        let selected = aliases.iter().min_by_key(|name| {
            ordinary
                .get(&((*name).clone(), failure.reason.clone()))
                .copied()
                .unwrap_or(usize::MAX)
        });
        let mut placed = failure.clone().placed(at);
        if let Some(selected) = selected {
            placed.path = selected.clone();
        }
        for alias in aliases {
            names.keep(&placed.path, &placed.reason, &alias);
        }
        *failure = placed;
    }
    names
}
