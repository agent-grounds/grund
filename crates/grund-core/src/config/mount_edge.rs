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
    /// The canonical root of the target (§FS-scan.identity).
    pub(crate) root: PathBuf,
    /// The declaring block's line the edge is located at, spelled from the run's
    /// report base (§FS-errors.4).
    pub(crate) source: Option<ConfigLocation>,
    /// The `project_name` the declaring block expects the target to declare, and
    /// where it wrote it (§FS-imports.declaration.identity).
    pub(crate) project: String,
    pub(crate) project_source: Option<ConfigLocation>,
}
