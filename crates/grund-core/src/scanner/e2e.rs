use crate::sources::{canonical_path, path_exists, path_holds_file, path_is_dir, path_is_file};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use super::e2e_case::{CaseFailure, read_e2e_case};
use super::e2e_listing::walk_one_level;
use super::scan_error::{
    FailureRoots, FileKind, ScanFailure, failure_place, publish_file_failure, publish_unlisted,
    publish_unlisted_fixture, read_failure_reason, scan_error_reason,
};
use super::tree::ScanError;
use super::walk::scan_roots;

use crate::config::{Frame, Schema, root_scope_roots};
use crate::grammar::{Grammar, literal_after_kind_placeholder, parse_id_arg};
use crate::model::{Catalog, Declaration, DeclarationSource, Id};
use crate::model::{configured_home_path_key, normalize_path_lexically, sort_path_key};

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
/// The cases are the ones the walk's filter met and the configuration does not keep out, so
/// `exclude` and the ignore files keep a case out as they keep out any directory below a root
/// (§FS-declarations.e2e-case.excluded). `walked_cases` is what the scan's walk met. Where it
/// listed the cases root, no directory is listed again (§FS-check.2.4.2). A cases root no walk
/// lists, such as a link out of the project or a folder outside every root, is walked here one
/// level deep by the same walker (§AR-scanner.6.unlistable-root). What a case holds is read
/// whole, as its data.
///
/// `reported` is the scan's failures so far, the walk's among them. A failure to list that root,
/// at open or partway through, is published there, once (§AR-scanner.6.unlistable-root), and so
/// are a case's fixture directory and a case's file (§AR-scanner.1.relisted). Each is at the path
/// that failed, spelled under the configured folder, and the first hides none after it
/// (§AR-scanner.6.6).
///
/// `resolution_wide` is whether a path below the root reads the project's ordinary roots beside it
/// (§FS-check.1.3.6.1): the run's own answer for `check`, and `check`'s answer for `fmt`, which
/// reports the case failures `check` reports on the same scope (§FS-fmt.7.2).
#[allow(clippy::too_many_arguments)]
pub(super) fn scan_e2e_cases(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    resolution_wide: bool,
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
        && !(resolution_wide && cases_root_in_ordinary_scope(schema, frame, &cases_root));
    // §FS-check.2.4.2: the path the run was handed, whose spelling names a file met twice.
    let resolved = explicit_scope
        .then(|| scan_roots(schema, frame, scope, explicit_scope).ok())
        .flatten();
    let run_path = resolved
        .as_ref()
        .filter(|(_, whole)| !whole)
        .and_then(|(roots, _)| roots.first().cloned());

    // §FS-cover.1.2: the spelled path below a case, whose failures alone the pass holds.
    let mut held = None;
    if narrowed_to_path {
        // A path that resolved to the config root holds the whole folder.
        let whole = resolved.as_ref().is_some_and(|(_, whole)| *whole);
        let narrowed = if whole {
            Some((cases_root.clone(), None))
        } else {
            run_path
                .as_deref()
                .and_then(|path| narrowed_root(&spelled_root, &cases_root, path))
                .or_else(|| resolved_root(scope.unwrap_or(Path::new(".")), &cases_root))
        };
        let Some((root, below_case)) = narrowed else {
            return;
        };
        scan_root = root;
        held = below_case;
    } else if !frame.run.scope.full && !cases_root_in_ordinary_scope(schema, frame, &cases_root) {
        // §FS-check.1.3: `--full` cancels `include`, so the e2e cases are in the
        // walk whether or not `include` happens to name their folder.
        return;
    }

    // §FS-check.2.4.3: resolved only for a pass with a failure to judge. Unless the path alone
    // brought this pass here, it reads the cases either way, so the path was not what read them.
    let failure_roots = || {
        let at = FailureRoots::resolve(schema, frame, scope, explicit_scope);
        if narrowed_to_path { at } else { at.unpassed() }
    };
    let failure_at = |spelled: &Path| ScanFailure::Unread {
        place: failure_place(spelled, &failure_roots()),
        version: frame.version,
    };
    // §FS-declarations.e2e-case.excluded: the cases are the ones the walk's filter met and the
    // configuration does not keep out, from the walk's own listing where it listed the root.
    let mut case_dirs = Vec::new();
    // §FS-declarations.e2e-case.id: an `expected.exit` that is a dangling link still names a case.
    if path_holds_file(scan_root.join("expected.exit")) {
        case_dirs.push(scan_root);
    } else if findings.walked_dirs.contains(&home) {
        // §FS-check.2.4.2: no second listing; a scope below the cases root names no case.
        if scan_root == cases_root {
            case_dirs.extend(cases_met(walked_cases, &home, &cases_root));
        }
    } else {
        // §AR-scanner.6.unlistable-root: attempted whatever the walk reported here; failing at
        // open or partway through is the one listing's one failure.
        let below = scan_root.strip_prefix(&cases_root).unwrap_or(Path::new(""));
        let listed = walk_one_level(schema, frame, &home.join(below), &scan_root);
        if let Some(err) = &listed.failure {
            let spelled = spelled_below(&spelled_root, &cases_root, &scan_root);
            let failure = failure_at(&spelled);
            publish_unlisted(&scan_root, spelled, err, None, failure, reported);
        }
        // §FS-check.2.4.2: a walk may have reported the same file already, under another spelling.
        for report in listed.ignore_failures {
            publish_file_failure(
                report.placed(&failure_roots()),
                frame,
                &mut findings.failure_names,
                reported,
            );
        }
        // §AR-scanner.6.6: the cases listed before a failure are still read; none, where the
        // root could not be opened, so nothing is left to read.
        case_dirs.extend(cases_met(&listed.cases, &home, &cases_root));
        let kept = listed
            .kept
            .iter()
            .filter_map(|dir| dir.strip_prefix(&home).ok());
        case_dirs.extend(
            kept.map(|below| cases_root.join(below))
                .filter(|dir| path_holds_file(dir.join("expected.exit"))),
        );
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
        // §FS-declarations.e2e-case.failure: the failure is this case's alone; the walk goes on.
        let case = match read_e2e_case(schema, frame, &dir) {
            Ok(case) => case,
            Err(case_failures) => {
                // §AR-scanner.6.6: each failure at its own path, the case's first hiding nothing.
                for failure in case_failures {
                    let spelled = spelled_below(&spelled_root, &cases_root, failure.path());
                    // §FS-cover.1.2: a path holding none of its failing files holds no failure.
                    if held.as_ref().is_some_and(|held| !spelled.starts_with(held)) {
                        continue;
                    }
                    match failure {
                        CaseFailure::Unread(path, err) => {
                            let failure = failure_at(&spelled);
                            let reason = read_failure_reason(&path, &err);
                            let report = ScanError::new(spelled, reason, failure);
                            publish_file_failure(
                                report,
                                frame,
                                &mut findings.failure_names,
                                reported,
                            );
                        }
                        CaseFailure::Unlisted(_, err) => {
                            let failure = failure_at(&spelled);
                            publish_unlisted_fixture(
                                spelled,
                                &err,
                                &findings.failure_names,
                                failure,
                                reported,
                            );
                        }
                        CaseFailure::Unparsed(_, err) => {
                            let failure = ScanFailure::Unparsed(FileKind::Exit);
                            reported.push(ScanError::new(
                                spelled,
                                scan_error_reason(&err),
                                failure,
                            ));
                        }
                    }
                }
                continue;
            }
        };
        // §FS-cover.1.2: a path below the case reads it for its failures; it declares nothing.
        if held.is_some() {
            continue;
        }
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

/// The case directories among `met`, the ones a walk's filter recorded, that sit directly in the
/// cases root, under its canonical path: those the walk met under the configured folder's spelling
/// or another that resolves to it, each once (§FS-check.2.4.2). One the walk met elsewhere, a link
/// that resolves to a case, is not one of them (§FS-declarations.e2e-case.walk).
fn cases_met<'a>(
    met: &'a [PathBuf],
    home: &'a Path,
    cases_root: &'a Path,
) -> impl Iterator<Item = PathBuf> + 'a {
    met.iter().filter_map(move |met| {
        let (parent, name) = (met.parent()?, met.file_name()?);
        (parent == home || canonical_path(parent).is_ok_and(|dir| dir == cases_root))
            .then(|| cases_root.join(name))
    })
}

/// What a narrowed pass reads, decided on the path as the tree spells it against the configured
/// folder, and on nothing else, so neither the case nor the folder being a link changes the answer
/// (§FS-cover.1.2): a path at or above the folder reads the whole folder; a path naming a case or
/// lying below one reads that case, with the spelled path beside it where it lies below the case,
/// whose files alone it holds; a directory below the folder that is no case is read from there, as
/// it was. `None` for any other path, which `resolved_root` then answers.
fn narrowed_root(
    spelled_root: &Path,
    cases_root: &Path,
    run_path: &Path,
) -> Option<(PathBuf, Option<PathBuf>)> {
    let folder = normalize_path_lexically(spelled_root);
    if folder.starts_with(run_path) {
        return Some((cases_root.to_path_buf(), None));
    }
    let below = run_path.strip_prefix(&folder).ok()?;
    let mut parts = below.components();
    let name = match parts.next()? {
        Component::Normal(name) => name,
        _ => return None,
    };
    let case = cases_root.join(name);
    // §AR-scanner.6.1: a case is a directory holding `expected.exit`, as the walk reads one.
    if path_is_file(&case.join("expected.exit")) {
        let held = parts.next().is_some().then(|| spelled_root.join(below));
        return Some((case, held));
    }
    // Below the folder but in no case: a file holds no case, and a directory reads what it held.
    (!path_is_file(run_path)).then(|| (cases_root.join(below), None))
}

/// §FS-cover.1.2: a directory path the spelling decides nothing for reads the cases where it
/// resolves at, above or below the canonical folder, as it did: the target of a cases folder that
/// is a link. The target of a case that is a link resolves outside the folder and reads nothing.
fn resolved_root(scope: &Path, cases_root: &Path) -> Option<(PathBuf, Option<PathBuf>)> {
    if path_is_file(scope) {
        return None;
    }
    let scope = canonical_path(scope).unwrap_or_else(|_| scope.to_path_buf());
    if scope.starts_with(cases_root) {
        Some((scope, None))
    } else {
        cases_root
            .starts_with(&scope)
            .then(|| (cases_root.to_path_buf(), None))
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
    // §FS-declarations.e2e-case.id: a dangling `expected.exit` names a case, which fails at it.
    if path.parent() != Some(cases_root) || !path_holds_file(path.join("expected.exit")) {
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
