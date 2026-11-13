//! The E2E case pass's own listing of a directory no walk of the scan listed
//! (§AR-scanner.6.unlistable-root): one level deep, by the scan's own walker, so the rules that
//! keep a case out where the walk lists it keep it out here too
//! (§FS-declarations.e2e-case.excluded).

use crate::sources::{is_directory, observe_input};
use std::path::{Path, PathBuf};
use std::sync::PoisonError;

use super::tree::ScanError;
use super::walk::{Met, MetCases, WalkDirFilter, scannable_walk_builder};
use super::walk_errors::{LinkRecovery, walk_error_path, walk_error_reports};
use crate::config::{Frame, Schema, canonical_config_root};

/// What a walk one level below `dir` met, where no walk of the scan listed it
/// (§AR-scanner.6.unlistable-root).
pub(super) struct OneLevel {
    /// The case directories its filter recorded, as it spelled them.
    pub(super) cases: Vec<PathBuf>,
    /// The directories it kept, as it spelled them. They hold cases only where `dir` lies below
    /// the cases root, as the path a narrowed run was handed may.
    pub(super) kept: Vec<PathBuf>,
    /// Its failure to list `dir`, at open or partway through, which ends the listing.
    pub(super) failure: Option<std::io::Error>,
    /// What the ignore files in `dir` failed with, the rules this listing is filtered by: where
    /// no walk lists `dir`, no other report names them (§FS-check.2.4.2).
    pub(super) ignore_failures: Vec<ScanError>,
}

/// `dir`, spelled under the configured folder, listed by the scan's own walker one level deep, so
/// that `exclude` and the ignore files keep a case out of a cases root no walk listed by the same
/// filter that keeps one out where the walk lists it (§FS-declarations.e2e-case.excluded). It is
/// one walk of the root, never one probe per entry (§GOAL-fast-feedback). Of the walker's errors,
/// its failure to list `dir` is the pass's, as it was when the pass listed `dir` itself, and so is
/// a failure to parse the ignore files in `dir`, whose rules filter this listing. What else it
/// meets is not read here, and stays the walk's to report where it reads it.
pub(super) fn walk_one_level(
    schema: &Schema,
    frame: Frame<'_>,
    dir: &Path,
    canonical: &Path,
) -> OneLevel {
    let mut listed = OneLevel {
        cases: Vec::new(),
        kept: Vec::new(),
        failure: None,
        ignore_failures: Vec::new(),
    };
    let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let met = MetCases::default();
    let physical_root = canonical_config_root(frame.root());
    // §FS-check.6.1.1: the root is covered before its listing, as every walk root is.
    if !observe_input(dir, true) {
        listed.failure = Some(std::io::Error::other("watch input coverage failed"));
        return listed;
    }
    let mut builder = match scannable_walk_builder(
        schema,
        frame,
        dir,
        canonical,
        &physical_root,
        &link_roots,
        &looping_links,
        Met::cases(&met),
    ) {
        Ok(builder) => builder,
        Err(err) => {
            listed.failure = Some(std::io::Error::other(err.to_string()));
            return listed;
        }
    };
    let mut ignore_failure = None;
    for entry in builder.max_depth(Some(1)).build() {
        match entry {
            // §AR-scanner.1.9: a directory's unparsable ignore file rides on its own entry.
            Ok(entry) if entry.depth() == 0 => ignore_failure = entry.error().cloned(),
            Ok(entry) if entry.depth() == 1 && entry.file_type().is_some_and(is_directory) => {
                listed.kept.push(entry.into_path());
            }
            Err(err) if walk_error_path(&err) == Some(dir) => {
                listed.failure = err.into_io_error();
                if listed.failure.is_some() {
                    break;
                }
            }
            Ok(_) | Err(_) => {}
        }
    }
    if let Some(err) = ignore_failure {
        let filter = WalkDirFilter::new(
            schema,
            frame,
            dir,
            canonical,
            &physical_root,
            &link_roots,
            &looping_links,
        );
        let mut recovery = LinkRecovery::default();
        listed.ignore_failures =
            walk_error_reports(err, &filter, &mut recovery, None, schema, frame, dir)
                .into_iter()
                .map(|(report, _)| report)
                .collect();
    }
    listed.cases = std::mem::take(&mut *met.lock().unwrap_or_else(PoisonError::into_inner));
    listed
}
