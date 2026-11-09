//! One line per directory entry (§FS-check.2.4.2, §AR-scanner.1.entry): the step that keeps the
//! first of the walk's reports of one entry and reason, and the name each of the others lost.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;

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
