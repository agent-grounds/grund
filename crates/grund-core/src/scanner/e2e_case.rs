//! What one E2E case holds, read into the record `grund E2E-<name>` renders and checks
//! (§AR-scanner.6.2, §FS-show.2.4): its manifest, its `spec.refs`, and its fixture list.
//!
//! `e2e.rs` decides which case directories are read, by the walk's own filter
//! (§FS-declarations.e2e-case.excluded); this file reads one of them, whole. Nothing here asks
//! `exclude` or an ignore file of a path inside the case, since what a case holds is the case's
//! data, and a failure here is the case's alone (§FS-declarations.e2e-case.failure).

use anyhow::Context;
use std::path::{Path, PathBuf};

use super::walk_order::sort_paths;
use crate::config::{Frame, Schema};
use crate::grammar::{parse_id_arg, split_alias_path};
use crate::model::{E2eCase, E2eSpecRef, format_path, sort_path_key};
use crate::sources::{Sources, path_holds_file, path_is_dir};

/// One failure reading a case, with the path that failed (§AR-scanner.6.6).
pub(super) enum CaseFailure {
    /// A file of the case that could not be read or decoded, which the walk or another case may
    /// have met under another spelling (§FS-check.2.4.2).
    Unread(PathBuf, std::io::Error),
    /// A fixture directory that could not be listed, which the walk may have failed on too
    /// (§AR-scanner.1.relisted).
    Unlisted(PathBuf, std::io::Error),
    /// An `expected.exit` that does not parse, which only this pass can say, so nothing is
    /// compared (§AR-scanner.6.6).
    Unparsed(PathBuf, anyhow::Error),
}

impl CaseFailure {
    pub(super) fn path(&self) -> &Path {
        match self {
            Self::Unread(path, _) | Self::Unlisted(path, _) | Self::Unparsed(path, _) => path,
        }
    }
}

/// Read one e2e case directory into an `E2eCase` — the words of `command.args`
/// (none when absent) and whether it is present (§AR-scanner.6.2), `expected.exit`, `spec.refs`, and the recursive fixture file list —
/// the data `grund E2E-<name>` renders or checks (§FS-show.2.4, §FS-config.3.9). A failure does
/// not end the read: every file and fixture directory is attempted, and a case with any failure
/// hands them all back and declares nothing (§AR-scanner.6.6). What a case holds is read whole,
/// whatever `exclude` or an ignore file says of a path inside it, since it is the case's data
/// (§FS-declarations.e2e-case.excluded).
pub(super) fn read_e2e_case(
    schema: &Schema,
    frame: Frame<'_>,
    dir: &Path,
) -> Result<E2eCase, Vec<CaseFailure>> {
    let mut failed = Vec::new();
    let command_args = dir.join("command.args");
    // §AR-scanner.6.2: the record keeps the manifest; the implicit `check` is the text form's.
    // §FS-declarations.e2e-case.failure: a dangling link is read and fails; only no name is none.
    let has_command_args = path_holds_file(&command_args);
    let args = if has_command_args {
        read_case_file(frame, &command_args, &mut failed)
            .map(|text| text.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let exit_path = dir.join("expected.exit");
    let expected_exit = read_case_file(frame, &exit_path, &mut failed).and_then(|text| {
        let parsed = text
            .trim()
            .parse::<i32>()
            .with_context(|| format!("parse {}/expected.exit", format_path(dir)));
        parsed
            .map_err(|err| failed.push(CaseFailure::Unparsed(exit_path, err)))
            .ok()
    });
    let mut fixtures = Vec::new();
    collect_relative_fixture_files(dir, dir, &mut fixtures, &mut failed);
    fixtures.sort_by_key(|path| sort_path_key(path));
    let spec_refs = read_e2e_spec_refs(schema, frame, dir, &mut failed);
    match expected_exit {
        Some(expected_exit) if failed.is_empty() => Ok(E2eCase {
            dir: dir.to_path_buf(),
            args,
            has_command_args,
            expected_exit,
            fixtures,
            spec_refs,
        }),
        _ => Err(failed),
    }
}

/// One file of a case as a save would write it (§AR-system.2.13), its failure recorded at that
/// file (§FS-declarations.e2e-case.failure).
fn read_case_file(frame: Frame<'_>, path: &Path, failed: &mut Vec<CaseFailure>) -> Option<String> {
    // §FS-check.6.1.1: cover this effective input before its shared read.
    match frame.sources().text(path) {
        Ok(text) => Some(text.into_owned()),
        Err(err) => {
            failed.push(CaseFailure::Unread(path.to_path_buf(), err));
            None
        }
    }
}

fn read_e2e_spec_refs(
    schema: &Schema,
    frame: Frame<'_>,
    dir: &Path,
    failed: &mut Vec<CaseFailure>,
) -> Vec<E2eSpecRef> {
    let path = dir.join("spec.refs");
    // §FS-declarations.e2e-case.failure: a dangling link is read and fails; only no name is none.
    if !path_holds_file(&path) {
        return Vec::new();
    }
    read_case_file(frame, &path, failed)
        .map(|text| {
            text.lines()
                .filter_map(|line| e2e_spec_ref_from_line(schema, frame, line.trim()))
                .collect()
        })
        .unwrap_or_default()
}

fn e2e_spec_ref_from_line(schema: &Schema, frame: Frame<'_>, line: &str) -> Option<E2eSpecRef> {
    let token = line.split_whitespace().next()?;
    if token.is_empty() {
        return None;
    }
    let token = token.strip_prefix(&schema.citation.marker).unwrap_or(token);
    // §FS-workspace.6.1: a test label, read without the slug check that would refuse it.
    let (namespace, id_text) = match split_alias_path(token) {
        (Some(namespace), id_text) if !namespace.is_empty() => {
            (Some(namespace.to_string()), id_text)
        }
        _ => (None, token),
    };
    if let Ok((id, _section)) = parse_id_arg(id_text, frame.grammar()) {
        return Some(E2eSpecRef {
            namespace,
            kind: id.kind,
        });
    }
    let kind = id_text.split_once('-')?.0;
    if kind.is_empty() {
        return None;
    }
    Some(E2eSpecRef {
        namespace,
        kind: kind.to_string(),
    })
}

/// The fixture files below `dir`. A directory that cannot be listed is recorded at that directory
/// and the rest are still listed (§AR-scanner.6.6), in the report's order, so two failures print
/// alike on every filesystem (§FS-errors.4.1).
fn collect_relative_fixture_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
    failed: &mut Vec<CaseFailure>,
) {
    // §FS-check.6.1.1: cover this effective input before its shared read.
    let mut listing = Sources::on_disk().list_dir(dir);
    if let Some(err) = listing.failure {
        failed.push(CaseFailure::Unlisted(dir.to_path_buf(), err));
    }
    sort_paths(&mut listing.entries);
    for path in listing.entries {
        if path_is_dir(&path) {
            collect_relative_fixture_files(root, &path, files, failed);
        } else {
            files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
}
