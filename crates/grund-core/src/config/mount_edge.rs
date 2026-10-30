//! §FS-remote-projects.mount.rule: one mount edge, the record expansion hands the
//! resolver for a project it loads as its own outermost root and shows under an
//! alias. A record rather than a rule, like `AbsentOptionalNamespace`: expansion
//! collects every block's edges onto the `Run` of the config the tree is expanded
//! from, because that run is what the loader of §AR-resolver.mount holds when it
//! grafts the tree's mounts.

use std::path::PathBuf;

use super::record::ConfigLocation;

/// One project a run mounts (§FS-remote-projects.mount.rule): read as its own
/// chain root, from its own config, and shown outside under `alias`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MountEdge {
    /// The whole alias path the run spells the mount with: the edge's own alias
    /// under the alias path of the block that declared it (§FS-workspace.6.1.5).
    pub(crate) alias: String,
    /// The edge's own alias, its segment at the declaring block's sibling level
    /// (§FS-workspace.3).
    pub(crate) segment: String,
    /// The path as the declaring config wrote it, which is how every message
    /// names the target (§FS-errors.4).
    pub(crate) written: String,
    /// The canonical root of the target (§FS-imports.declaration.path), or, for a missing
    /// one, the directory its path names, spelled lexically.
    pub(crate) root: PathBuf,
    /// Whether the path names no directory, so there is nothing to mount and every
    /// citation into the alias is unverified (§FS-imports.checks.import-missing).
    pub(crate) missing: bool,
    /// The declaring block's line the edge is located at, spelled from the run's
    /// report base (§FS-errors.4).
    pub(crate) source: Option<ConfigLocation>,
    /// The `project_name` the declaring block expects the target to declare, and
    /// where it wrote it (§FS-imports.declaration.identity).
    pub(crate) project: String,
    pub(crate) project_source: Option<ConfigLocation>,
}

/// One import edge a run reached (§FS-imports.checks): read live from its tree, or
/// missing, by the alias path the run spells it with and the path as its config
/// wrote it. The loader collects one per edge, a transitive one included, onto
/// every project's run, which is what the report announces and what resolution
/// asks of a citation into a missing one (§FS-workspace.4.3).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ImportRead {
    pub(crate) alias: String,
    pub(crate) written: String,
    /// The table line of the config that declares the import (§FS-errors.4).
    pub(crate) source: Option<ConfigLocation>,
    pub(crate) missing: bool,
}

impl ImportRead {
    /// Whether `namespace` is this missing import's alias path or one below it,
    /// all of which it absorbs (§FS-imports.checks.import-missing).
    pub(crate) fn absorbs(&self, namespace: &str) -> bool {
        self.missing
            && (namespace == self.alias
                || namespace
                    .strip_prefix(self.alias.as_str())
                    .is_some_and(|rest| rest.starts_with('/')))
    }
}
