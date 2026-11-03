//! §AR-config.8.4: the one answer to whether a version-1 file is a block with
//! `include_root = false`, and whether that block migrates compactly
//! (§FS-config-migrate.8.1). Refusal collection, candidate construction, the key
//! map and notices, and the records gate all read the [`Block`] this returns;
//! none of them asks the question again.

use std::path::Path;

use super::keys::Keys;
use super::records_gate::reduce;
use crate::config::project::Project;
use crate::config::v1::{self, LineKind};

/// Why a key, or version 1's defaults, has no reader on the block
/// (§FS-config-migrate.3.2).
pub(super) const UNREAD: &str = "include_root = false, so no project scans this block's own files";

/// What a version-1 file is to migration (§FS-config-migrate.8.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Block {
    /// A project, or a block with `include_root = true`: everything else in
    /// §FS-config-migrate applies unchanged.
    Project,
    /// A block with `include_root = false` that keeps the conservative path,
    /// where only named sections are never refused (§FS-config-migrate.8.1.1).
    OptedOut,
    /// A block with `include_root = false` that migrates compactly
    /// (§FS-config-migrate.8.1.2).
    Compact,
}

impl Block {
    /// §FS-config-migrate.8.1.2: `v1` read from the file whose keys are `keys`.
    /// `scope_on_disk` answers item 1 with the unread-block probe's own scope
    /// and member boundary (§FS-check.4.10.2), and is asked last, only of a
    /// block whose authored keys already pass item 2.
    pub(super) fn of(
        v1: &Project,
        keys: &Keys,
        root: &Path,
        scope_on_disk: &dyn Fn() -> bool,
    ) -> Self {
        let workspace = &v1.workspace;
        if !workspace.declared || workspace.include_root {
            return Self::Project;
        }
        if authors_nothing_read(v1, keys, root) && !scope_on_disk() {
            Self::Compact
        } else {
            Self::OptedOut
        }
    }

    /// §FS-config-migrate.8.1.1: no project reads the block's named sections.
    pub(super) fn opted_out(self) -> bool {
        self != Self::Project
    }

    pub(super) fn compact(self) -> bool {
        self == Self::Compact
    }
}

/// §FS-config-migrate.8.1.2 item 2's first bullet: a key with force on any
/// block, which the compact candidate keeps (§FS-config-migrate.8.1.3).
pub(super) fn kept(section: &str, key: &str) -> bool {
    match section {
        "" => matches!(
            key,
            "grund_config_version" | "project_name" | "project_description"
        ),
        section => section == "workspace" || section.starts_with("workspace."),
    }
}

/// §FS-config-migrate.8.1.2 item 2: `[scan] include` counts as omitted when the
/// file leaves it out, writes `[]`, or writes exactly version 1's default list.
fn include_omitted(v1: &Project, keys: &Keys) -> bool {
    if keys.line("scan", "include").is_none() {
        return true;
    }
    let include = v1.schema.sources.include.as_deref().unwrap_or_default();
    include.is_empty() || Some(include) == v1::default_project().schema.sources.include.as_deref()
}

/// §FS-config-migrate.8.1.2 item 2: every authored key is kept, named sections,
/// an omitted include, `[output]`, or at version 1's default. The last is read
/// as the records gate reads a project (§FS-config-migrate.5.1): the file means
/// what version 1's defaults mean once the other keys are put back.
fn authors_nothing_read(v1: &Project, keys: &Keys, root: &Path) -> bool {
    let citations = |name: &str| name == "citations" || name.starts_with("citations.");
    for record in keys.lines {
        match &record.kind {
            LineKind::Header { name, array, .. }
                if (*array && name == "kinds") || citations(name) =>
            {
                return false;
            }
            LineKind::Key { section, key, .. } => {
                // A `[[kinds]]` entry, `kinds = []` and a `[citations]` table have no default.
                if section == "kinds"
                    || citations(section)
                    || (section.is_empty() && key == "kinds")
                {
                    return false;
                }
                if section == "scan" && key == "include" && !include_omitted(v1, keys) {
                    return false;
                }
            }
            _ => {}
        }
    }
    let mut defaults = v1::default_project();
    defaults.name.clone_from(&v1.name);
    defaults.workspace.clone_from(&v1.workspace);
    defaults
        .presentation
        .description
        .clone_from(&v1.presentation.description);
    defaults.schema.ids.named_sections = v1.schema.ids.named_sections;
    defaults
        .schema
        .sources
        .include
        .clone_from(&v1.schema.sources.include);
    reduce(v1, root, Block::Project) == reduce(&defaults, root, Block::Project)
}
