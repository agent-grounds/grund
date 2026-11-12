//! The roots a walk starts from (§FS-scan.scope, §AR-scanner.1.6): `[scan] include` or the
//! whole root, and an explicit path beside or in place of them, resolved once for every
//! reader of the scope. Apart from the walk itself, which only starts from what this answers.

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use super::walk_boundaries::is_directory_symlink;
use crate::config::{Frame, Schema, canonical_config_root, root_scope_roots};
use crate::model::normalize_path_lexically;
use crate::sources::{canonical_path, path_lookup};

/// The directories (or single file) the walk starts from, and so which files a run
/// reads (§FS-scan.scope): `[scan] include`
/// resolved against the repo root, otherwise the whole root — and, for a run
/// given a `[path]` argument below that root, the path beside them, because the
/// path narrows the report and not the walk (§FS-config.3.5.7, §AR-scanner.1.6,
/// §FS-check.1.3.6.1).
///
/// Beside the roots, whether the scope is the whole project: no path, or one that
/// resolves to the config root — the one resolution of the scope, which the lock
/// pass of `fmt` reads rather than resolving it again (§FS-section-lock.write).
pub(super) fn scan_roots(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<(Vec<PathBuf>, bool)> {
    resolve_scan_roots(
        schema,
        frame,
        scope,
        explicit_scope,
        frame.run.scope.full,
        frame.run.scope.resolution_wide,
    )
}

/// §FS-check.1.3.1: `full` cancels `[scan] include` for this walk and nothing else
/// — an explicit path argument still narrows the report, and `exclude`, the ignore
/// files, and `extensions` are untouched. `check --full` asks both ways: once with
/// `true` to walk the whole root, and once with `false` to learn which of what it
/// read was inside the configured scope (§FS-check.3.14).
///
/// §FS-check.1.3.6.1: `widen` is the other axis, and the walk is the one caller
/// that passes it. It says the explicit path is this run's *report* scope, so the
/// roots returned are the ordinary ones union the path; `full` is then irrelevant,
/// since a path leaves the flag nothing to cancel (§FS-check.1.3.6). The two scope
/// layers ask with `widen = false`, because each wants the path as the *bound* it
/// is to them rather than as the walk's extra root (§AR-resolver.3.3).
pub(crate) fn scan_roots_for(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    full: bool,
    widen: bool,
) -> Result<Vec<PathBuf>> {
    Ok(resolve_scan_roots(schema, frame, scope, explicit_scope, full, widen)?.0)
}

/// `scan_roots_for`, and whether the scope it resolved is the config root.
fn resolve_scan_roots(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    full: bool,
    widen: bool,
) -> Result<(Vec<PathBuf>, bool)> {
    if explicit_scope {
        let scope = scope.unwrap_or(Path::new("."));
        // §FS-scan.scope.homes.lookup.path: absent only when the lookup says so, as the walk
        // asks of every root; a refused path, link or not (§FS-scan.scope.links.owed.refused),
        // and a looping link (§FS-scan.scope.links.unresolvable.named) are the walk's to report.
        if matches!(path_lookup(scope), Ok(false)) {
            return Err(anyhow!("path does not exist: {}", scope.display()));
        }
        let lexical_scope = if scope.is_absolute() {
            normalize_path_lexically(scope)
        } else {
            std::env::current_dir()
                .map(|cwd| normalize_path_lexically(&cwd.join(scope)))
                .unwrap_or_else(|_| normalize_path_lexically(scope))
        };
        let resolved = canonical_path(scope).unwrap_or_else(|_| lexical_scope.clone());
        // §FS-config.3.5.1, §FS-config.3.5.2.1: keep the lexical spelling for an
        // in-tree link and an external directory-link root; resolve other roots.
        let scope =
            if lexical_scope.starts_with(frame.root()) || is_directory_symlink(&lexical_scope) {
                lexical_scope
            } else {
                walk_root_under_config_root(frame, &resolved)
            };
        if resolved == canonical_config_root(frame.root()) {
            return Ok((root_scope_roots(schema, frame.root(), full), true));
        }
        // §FS-check.1.3.6.1: the ordinary roots union the path, the path first so its
        // spelling wins for the files it names (§FS-check.1.3.2). `full` is no part of
        // it — an explicit path leaves the flag nothing to cancel (§FS-check.1.3.6).
        if widen {
            let mut roots = vec![scope];
            roots.extend(root_scope_roots(schema, frame.root(), false));
            return Ok((roots, false));
        }
        return Ok((vec![scope], false));
    }
    Ok((root_scope_roots(schema, frame.root(), full), true))
}

/// A resolved scope re-expressed under the spelling `config.root` wears
/// (§FS-config.3.5.2.1). Resolving the scope above answers "which directory is
/// this", and on a root that is itself reached through a link — a symlinked
/// `~/work`, macOS resolving `/var` to `/private/var` — it throws away the
/// answer to "what is it called here": the walk would start at the physical
/// path, every finding would be spelled physically, and `relative_paths` could
/// not strip a root those paths no longer begin with (§FS-config.3.6). A root
/// that is already canonical — every root `grund` discovers for itself — takes
/// the first branch and the whole question costs one `stat` per run.
fn walk_root_under_config_root(frame: Frame<'_>, resolved: &Path) -> PathBuf {
    let canonical_root = canonical_config_root(frame.root());
    if canonical_root == frame.root() {
        return resolved.to_path_buf();
    }
    match resolved.strip_prefix(&canonical_root) {
        Ok(rest) => frame.root().join(rest),
        Err(_) => resolved.to_path_buf(),
    }
}
