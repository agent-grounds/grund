use crate::sources::{Sources, canonical_path, path_exists, path_is_dir, path_is_file};
use anyhow::Context;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::read_predicate::CaseEntryFilter;
use super::scan_error::{
    publish_case_file_failure, publish_unlisted, publish_unlisted_fixture, scan_error_reason,
};
use super::tree::ScanError;
use super::walk::scan_roots;
use super::walk_order::sort_paths;

use crate::config::{Frame, Schema, root_scope_roots};
use crate::grammar::{Grammar, literal_after_kind_placeholder, parse_id_arg, split_alias_path};
use crate::model::{Catalog, Declaration, DeclarationSource, E2eCase, E2eSpecRef, Id};
use crate::model::{configured_home_path_key, format_path, sort_path_key};

/// §FS-config.3.5.8, §FS-check.1.3.6.1: whether the project's *ordinary* walk reads
/// the E2E case root at all. A path-scoped run's resolution scope is that ordinary
/// scope union the path, so a cases root the ordinary run skips is read only when the
/// path itself names it.
///
/// The question is asked of `root_scope_roots` rather than of `[scan] include`,
/// because `include` is not the whole of what the ordinary walk starts from: every
/// walked kind home is a root too (§FS-config.3.5.8), and a citable `E2E` kind's
/// folder is one of them. Restating the predicate over `include` alone read the
/// cases root as unwalked wherever `include` did not happen to name it, and so made
/// a `§E2E-…` citation dangle under a path scope that `grund check .` resolves —
/// this ticket's own defect inside the line that decides the widening.
fn cases_root_in_ordinary_scope(schema: &Schema, frame: Frame<'_>, cases_root: &Path) -> bool {
    root_scope_roots(schema, frame.root(), false)
        .iter()
        .any(|root| {
            // The caller canonicalizes the cases root (§AR-scanner.6.2), so the compare
            // has to meet it there; a root that does not exist yet keeps its own spelling.
            let root = canonical_path(root).unwrap_or_else(|_| root.clone());
            cases_root.starts_with(&root) || root.starts_with(cases_root)
        })
}

/// The folder a citable `E2E` kind keeps its cases in (§AR-scanner.6): its first
/// folder, on purpose, since the cases are one tree. Not its first place: a row
/// lists its files before its folders (§FS-config-v2.schema.places), so a row
/// with both would otherwise have no cases root at all.
pub(super) fn e2e_cases_folder(schema: &Schema) -> Option<&str> {
    schema
        .rows
        .iter()
        .find(|row| row.name == "E2E" && row.kind.is_some())
        .and_then(|row| row.folders().next())
}

/// Discover `e2e/cases/<name>/` directories and register each as an `E2E-<name>`
/// declaration whose body is the case manifest (§AR-scanner.6, §FS-show.2.4) — so
/// `grund check` sees `§E2E-…` citations resolve and `grund refs` finds e2e tests.
///
/// The cases are the ones the one walk met: `walked_cases` is its listing, and where it listed the
/// cases root no directory is listed again, so `exclude` and the ignore files keep a case out as
/// they keep out any path below a root (§FS-declarations.e2e-case.excluded, §FS-check.2.4.2).
/// Only a cases root no walk lists, a link out of the project or a folder outside every root, is
/// listed here (§AR-scanner.6.unlistable-root).
///
/// `reported` is the scan's failures so far, the walk's among them. A failure to list that root,
/// at open or partway through, is published there, once (§AR-scanner.6.unlistable-root), and so
/// are a case's fixture directory and a case's file (§AR-scanner.1.relisted). Each is at the path
/// that failed, spelled under the configured folder, and the first hides none after it
/// (§AR-scanner.6.6).
pub(super) fn scan_e2e_cases(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    walked_cases: &[PathBuf],
    reported: &mut Vec<ScanError>,
    findings: &mut Catalog,
) {
    let Some(folder) = e2e_cases_folder(schema) else {
        return;
    };
    let home = frame.root().join(folder);
    if !path_exists(&home) || !path_is_dir(&home) {
        return;
    }
    let cases_root = canonical_path(&home).unwrap_or_else(|_| home.clone());
    // §FS-check.2.4.2: a failure is published at the folder as the tree spells it, the one lexical
    // path a run can name, whatever `..` a version 1 folder keeps.
    let spelled_root = frame.root().join(configured_home_path_key(folder));
    let mut scan_root = cases_root.clone();

    // §FS-check.1.3.6.1: a path-scoped run reads the cases the ordinary run reads —
    // the whole root, the report filter narrowing afterwards. Only a cases root the
    // ordinary scope misses is read through the path alone, which is the branch below.
    let narrowed_to_path = explicit_scope
        && !(frame.run.scope.resolution_wide
            && cases_root_in_ordinary_scope(schema, frame, &cases_root));
    if narrowed_to_path {
        let scope = scope.unwrap_or(Path::new("."));
        if path_is_file(&scope) {
            return;
        }
        let scope = canonical_path(scope).unwrap_or_else(|_| scope.to_path_buf());
        if scope.starts_with(&cases_root) {
            scan_root = scope;
        } else if !cases_root.starts_with(&scope) {
            return;
        }
    } else if !frame.run.scope.full && !cases_root_in_ordinary_scope(schema, frame, &cases_root) {
        // §FS-check.1.3: `--full` cancels `include`, so the e2e cases are in the
        // walk whether or not `include` happens to name their folder.
        return;
    }

    // §FS-declarations.e2e-case.excluded: a cases root the walk listed is read from its listing,
    // under the spelling its filter tells a case by, and what lies inside a case faces the rules
    // the walk would have asked of it.
    let walk_listed = findings.walked_dirs.contains(&home);
    let entries = walk_listed.then(|| CaseEntryFilter::new(schema, frame));
    let kept_out = |path: &Path, is_dir: bool| {
        entries.as_ref().is_some_and(|entries| {
            entries.keeps_out(&spelled_below(&spelled_root, &cases_root, path), is_dir)
        })
    };
    // §FS-check.2.4.2: the path the run was handed, whose spelling names a file met twice.
    let run_path = explicit_scope
        .then(|| scan_roots(schema, frame, scope, explicit_scope).ok())
        .flatten()
        .filter(|(_, whole)| !whole)
        .and_then(|(roots, _)| roots.into_iter().next());

    let mut case_dirs = Vec::new();
    if path_is_file(&scan_root.join("expected.exit")) {
        case_dirs.push(scan_root);
    } else if walk_listed {
        // §FS-check.2.4.2: the walk's listing; a scope below the cases root names no case.
        if scan_root == cases_root {
            for met in walked_cases {
                let (Some(parent), Some(name)) = (met.parent(), met.file_name()) else {
                    continue;
                };
                if parent == home || canonical_path(parent).is_ok_and(|dir| dir == cases_root) {
                    case_dirs.push(cases_root.join(name));
                }
            }
        }
    } else {
        // §FS-check.6.1.1: cover this effective input before its shared read.
        // §AR-scanner.6.unlistable-root: attempted whatever the walk reported here; failing at
        // open or partway through is the one listing's one failure.
        let listing = Sources::on_disk().list_dir(&scan_root);
        if let Some(err) = &listing.failure {
            let spelled = spelled_below(&spelled_root, &cases_root, &scan_root);
            publish_unlisted(&scan_root, spelled, err, None, reported);
        }
        // §AR-scanner.6.6: the cases listed before a failure are still read; none, where the
        // root could not be opened, so nothing is left to read.
        for path in listing.entries {
            if path_is_dir(&path) && path_is_file(&path.join("expected.exit")) {
                case_dirs.push(path);
            }
        }
    }
    case_dirs.sort_by_key(|path| sort_path_key(path));
    // §FS-check.2.4.2: two walk roots that list the cases root meet each case once each.
    case_dirs.dedup();

    for dir in case_dirs {
        let Some(name) = dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id) = e2e_id_from_case_dir_name(&schema.ids.format, frame.grammar(), name) else {
            continue;
        };
        // §FS-declarations.e2e-case.excluded: without its `expected.exit` a directory is no case.
        if kept_out(&dir.join("expected.exit"), false) {
            continue;
        }
        // §FS-declarations.e2e-case.failure: the failure is this case's alone; the walk goes on.
        let case = match read_e2e_case(schema, frame, &dir, &kept_out) {
            Ok(case) => case,
            Err(case_failures) => {
                // §AR-scanner.6.6: each failure at its own path, the case's first hiding nothing.
                for failure in case_failures {
                    let spelled = spelled_below(&spelled_root, &cases_root, failure.path());
                    match failure {
                        CaseFailure::Unread(_, err) => {
                            publish_case_file_failure(spelled, &err, run_path.as_deref(), reported);
                        }
                        CaseFailure::Unlisted(_, err) => {
                            publish_unlisted_fixture(spelled, &err, reported);
                        }
                        CaseFailure::Unparsed(_, err) => {
                            reported.push((spelled, scan_error_reason(&err)));
                        }
                    }
                }
                continue;
            }
        };
        findings
            .declarations
            .entry(id.clone())
            .or_default()
            .push(Declaration {
                authored_id: e2e_source_id(&schema.ids.format, name),
                id,
                file: dir.clone(),
                line: 1,
                heading_level: 1,
                sections: BTreeMap::new(),
                duplicate_sections: Vec::new(),
                is_stub: false,
                defined_in: None,
                stub_resolution: None,
                e2e_case: Some(case),
                title: None, // §FS-declarations.e2e-case.id: a case has no title.
                // §AR-scanner.2.4.1: an E2E case spans its manifest line only; its
                // obligations evaluate over the case's scanned files, not a body.
                body_start: 1,
                body_end: 1,
                body_has_content: false,
                source: DeclarationSource::Text,
                value_valid: None,
                front_matter: None,
            });
    }
}

/// `failed`, a canonical path at or below the canonical `cases_root`, spelled as the tree
/// spells it: under the configured folder, never at its resolved target, which may lie
/// outside the project (§FS-check.2.4.2, §FS-declarations.e2e-case.failure).
fn spelled_below(spelled_root: &Path, cases_root: &Path, failed: &Path) -> PathBuf {
    match failed.strip_prefix(cases_root) {
        Ok(below) if !below.as_os_str().is_empty() => spelled_root.join(below),
        _ => spelled_root.to_path_buf(),
    }
}

/// Map an `e2e/cases/<name>/` directory name to its `E2E-<name>` `Id` under the
/// repo's `[id] format` (§AR-scanner.6.1, §FS-config.3.4).
pub(super) fn e2e_id_from_case_dir_name(
    id_format: &str,
    grammar: &Grammar,
    name: &str,
) -> Option<Id> {
    let raw = e2e_source_id(id_format, name)?;
    let (id, section) = parse_id_arg(&raw, grammar).ok()?;
    if section.is_none() && id.kind == "E2E" {
        Some(id)
    } else {
        None
    }
}

/// Keep the case-directory spelling beside its numeric identity
/// (§FS-config.3.2.8, §AR-scanner.6.1).
fn e2e_source_id(id_format: &str, name: &str) -> Option<String> {
    let separator = literal_after_kind_placeholder(id_format)?;
    Some(format!("E2E{separator}{name}"))
}

/// Direct `e2e/cases/<name>/` directories are E2E manifest declarations
/// (§AR-scanner.6.1), so the ordinary file walk must not scan their fixture repos.
pub(super) fn is_direct_e2e_case_dir(
    path: &Path,
    cases_root: Option<&Path>,
    id_format: &str,
    grammar: &Grammar,
) -> bool {
    let Some(cases_root) = cases_root else {
        return false;
    };
    if path.parent() != Some(cases_root) || !path_is_file(&path.join("expected.exit")) {
        return false;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| e2e_id_from_case_dir_name(id_format, grammar, name))
        .is_some()
}

/// Inverse of `e2e_id_from_case_dir_name`: strip the `E2E` prefix off a rendered ID
/// to get the `e2e/cases/<name>/` directory `grund id` tells the author to create
/// (§FS-id.2, §AR-scanner.6.1).
pub(crate) fn e2e_case_dir_name(schema: &Schema, rendered: &str) -> String {
    let prefix = format!(
        "E2E{}",
        literal_after_kind_placeholder(&schema.ids.format).unwrap_or("-")
    );
    rendered
        .strip_prefix(&prefix)
        .unwrap_or(rendered)
        .to_string()
}

/// One failure reading a case, with the path that failed (§AR-scanner.6.6).
enum CaseFailure {
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
    fn path(&self) -> &Path {
        match self {
            Self::Unread(path, _) | Self::Unlisted(path, _) | Self::Unparsed(path, _) => path,
        }
    }
}

/// Read one e2e case directory into an `E2eCase` — the words of `command.args`
/// (none when absent) and whether it is present (§AR-scanner.6.2), `expected.exit`, `spec.refs`, and the recursive fixture file list —
/// the data `grund E2E-<name>` renders or checks (§FS-show.2.4, §FS-config.3.9). A failure does
/// not end the read: every file and fixture directory is attempted, and a case with any failure
/// hands them all back and declares nothing (§AR-scanner.6.6). A manifest or fixture directory
/// `kept_out` names is not read, as if it were absent (§FS-declarations.e2e-case.excluded).
fn read_e2e_case(
    schema: &Schema,
    frame: Frame<'_>,
    dir: &Path,
    kept_out: &dyn Fn(&Path, bool) -> bool,
) -> Result<E2eCase, Vec<CaseFailure>> {
    let mut failed = Vec::new();
    let command_args = dir.join("command.args");
    // §AR-scanner.6.2: the record keeps the manifest; the implicit `check` is the text form's.
    let has_command_args = path_is_file(&command_args) && !kept_out(&command_args, false);
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
    collect_relative_fixture_files(dir, dir, kept_out, &mut fixtures, &mut failed);
    fixtures.sort_by_key(|path| sort_path_key(path));
    let spec_refs = read_e2e_spec_refs(schema, frame, dir, kept_out, &mut failed);
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
    kept_out: &dyn Fn(&Path, bool) -> bool,
    failed: &mut Vec<CaseFailure>,
) -> Vec<E2eSpecRef> {
    let path = dir.join("spec.refs");
    if !path_is_file(&path) || kept_out(&path, false) {
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
/// alike on every filesystem (§FS-errors.4.1). A directory `kept_out` names is not listed
/// (§FS-declarations.e2e-case.excluded).
fn collect_relative_fixture_files(
    root: &Path,
    dir: &Path,
    kept_out: &dyn Fn(&Path, bool) -> bool,
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
            if !kept_out(&path, true) {
                collect_relative_fixture_files(root, &path, kept_out, files, failed);
            }
        } else {
            files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
}
