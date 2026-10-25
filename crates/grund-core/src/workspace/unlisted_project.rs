//! Context for placement errors beneath unread project configs
//! (§FS-declarations.checks.misplaced-declaration.unlisted-project). Discovery and
//! the quiet workspace claim test are shared with the standalone workspace rule;
//! neither a candidate nor this explanation establishes a new scan boundary.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::expand::enclosing_workspace_of;
use super::members::{AncestorWorkspaces, canonical_workspace_path};
use crate::config::{Frame, Run, Schema, config_file_in, root_scope_roots, unread_project_name};
use crate::model::{format_path, relative_from_base};

/// One quiet claim cache for both uses of an unlisted config
/// (§FS-declarations.checks.misplaced-declaration.unlisted-project, §FS-check.3.29.2).
pub(super) struct UnlistedProjects<'a> {
    run: &'a Run,
    root: PathBuf,
    ancestors: AncestorWorkspaces,
}

impl<'a> UnlistedProjects<'a> {
    /// §FS-declarations.checks.misplaced-declaration.unlisted-project: reuse ancestor
    /// answers throughout a run rather than loading every candidate as a project.
    pub(super) fn new(run: &'a Run) -> Self {
        Self {
            run,
            root: canonical_workspace_path(&run.root),
            ancestors: AncestorWorkspaces::quiet_for_run_at(&run.root),
        }
    }

    /// Whether `dir` holds a config no enclosing workspace lists: not a project root of
    /// this run, and claimed by no ancestor. Absence holds only where every ancestor's
    /// member text was obtained (§FS-workspace.6.1.7.5): a named claim that cannot
    /// expand, or an ancestor whose `members` or `optional_members` cannot be read,
    /// leaves the claim undecidable, and undecidable is not unlisted. The standalone
    /// unlisted-workspace finding (§FS-check.3.29.2) and the placement context
    /// (§FS-declarations.checks.misplaced-declaration.unlisted-project) ask this one
    /// question.
    pub(super) fn is_unlisted(&mut self, dir: &Path) -> bool {
        let canonical = canonical_workspace_path(dir);
        if canonical == self.root
            || self
                .run
                .workspace
                .project_roots
                .iter()
                .chain(&self.run.workspace.boundary_roots)
                .any(|root| root == &canonical)
        {
            return false;
        }
        matches!(
            enclosing_workspace_of(dir, &self.run.cli_base, &mut self.ancestors),
            Ok(None)
        ) && !self.ancestors.has_undecidable_ancestor(dir)
    }
}

/// §FS-declarations.checks.misplaced-declaration.unlisted-project: config-only
/// context for the directories the checker asks about. This layer neither judges
/// declarations nor changes diagnostics; it answers discovery and ownership.
pub(crate) fn unlisted_project_contexts<'a>(
    schema: &Schema,
    frame: Frame<'_>,
    directories: impl Iterator<Item = &'a Path>,
) -> BTreeMap<PathBuf, String> {
    let mut contexts = BTreeMap::new();
    // Probe only ancestors of actual errors, once each. This also reaches a config
    // above a configured file/directory scan root without starting another walk.
    let dirs: BTreeSet<_> = directories
        .flat_map(|parent| {
            parent
                .ancestors()
                .take_while(|dir| dir.starts_with(frame.root()) && *dir != frame.root())
        })
        .collect();
    if dirs.is_empty() {
        return contexts;
    }
    let mut projects = UnlistedProjects::new(frame.run);
    let roots = root_scope_roots(schema, frame.root(), frame.run.scope.full);
    for dir in dirs {
        let Some(config_path) = config_file_in(dir) else {
            continue;
        };
        if !projects.is_unlisted(dir) {
            continue;
        }
        // An unreadable candidate adds no new refusal or claim about its contents.
        let Ok(text) = frame.sources().text(&config_path) else {
            continue;
        };
        let name = unread_project_name(&text, &config_path)
            .map(|name| format!(" (project {name})"))
            .unwrap_or_default();
        contexts.insert(
            dir.to_path_buf(),
            placement_context(frame, dir, &config_path, &name, &roots),
        );
    }
    contexts
}

/// §FS-declarations.checks.misplaced-declaration.unlisted-project: report paths
/// describe locations, while member entries and exclusion patterns are edits
/// relative to the owning config root. A configured root needs a second edit.
fn placement_context(
    frame: Frame<'_>,
    dir: &Path,
    config_path: &Path,
    name: &str,
    roots: &[PathBuf],
) -> String {
    let entry = format_path(&relative_from_base(frame.root(), dir));
    let enclosing = frame
        .run
        .config_file
        .as_ref()
        .map(|path| frame.run.location_base.join(path))
        .unwrap_or_else(|| frame.root().join("grund.toml"));
    let enclosing = frame.display_path(&enclosing);
    let owner = frame.name.unwrap_or("root");
    let exclusion = if frame.version == 2 {
        format!("exclude \"{entry}/\" with [schema.sources] exclude in {enclosing}")
    } else {
        // §FS-config.3.5.16: v1 matches directory names at every depth, never paths.
        let component = dir.file_name().unwrap_or_default().to_string_lossy();
        format!(
            "exclude \"{component}\" with [scan] exclude in {enclosing} \
             (excludes all directories named \"{component}\" at every depth)"
        )
    };
    let canonical = canonical_workspace_path(dir);
    let root_advice = if roots
        .iter()
        .any(|root| canonical_workspace_path(root).starts_with(&canonical))
    {
        format!(
            "; also remove configured scan roots at or beneath \"{entry}\" in {enclosing} \
             (source includes or kind homes), because exclude does not prune scan roots"
        )
    } else {
        String::new()
    };
    format!(
        " — {nested}{name} is not read: no [workspace] members entry lists {entry}; \
         kind homes from {enclosing} (project {owner}) apply; \
         add \"{entry}\" to [workspace] members in {enclosing}, \
         or {exclusion}{root_advice}",
        nested = frame.display_path(config_path),
    )
}
