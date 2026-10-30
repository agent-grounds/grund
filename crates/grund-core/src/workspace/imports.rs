//! The imports of a `[workspace]` block (§FS-imports.declaration): each
//! `[workspace.imports.<alias>]` table of a block the expansion reaches, named at
//! the block's sibling level and handed to the resolver as the mount edge it loads
//! (§AR-workspace.imports, §AR-resolver.mount). Expansion reaches the blocks, so it is
//! where an import's alias meets the root's and the members' (§FS-workspace.3);
//! the target's own config is the loader's to read.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::expand::qualify_alias;
use super::scope::{config_location_error, project_label};
use crate::config::{Config, MountEdge};
use crate::sources::canonical_path;

/// The mount edges `block`'s imports declare, under `prefix`, the alias path of the
/// block's own level (§FS-imports.declaration.alias, §FS-workspace.6.1.5).
///
/// Each alias joins `siblings`, which already holds the block's root and members:
/// a collision is refused at the import's table line, naming the first claimant by
/// its role and the import by its path as written (§FS-imports.failures). An import
/// whose path names no directory has no root to mount, so it is no edge.
pub(super) fn collect_imports(
    block: &Config,
    top_config: &Config,
    prefix: &str,
    siblings: &mut BTreeMap<String, PathBuf>,
    edges: &mut Vec<MountEdge>,
) -> Result<()> {
    for import in &block.project().workspace.imports {
        // The reader refuses a table that lacks either key.
        let (Some(path), Some(project)) = (&import.path, &import.project) else {
            continue;
        };
        if let Some(first) = siblings.get(&import.alias) {
            let message = format!(
                "duplicate workspace project alias `{}` ({} and import `{path}`)",
                qualify_alias(prefix, &import.alias),
                project_label(top_config, first),
            );
            return Err(config_location_error(
                block,
                import.source.as_ref(),
                message,
            ));
        }
        let Ok(root) = canonical_path(block.root.join(path)) else {
            continue;
        };
        siblings.insert(import.alias.clone(), root.clone());
        edges.push(MountEdge {
            alias: qualify_alias(prefix, &import.alias),
            segment: import.alias.clone(),
            written: path.clone(),
            root,
            source: import.source.clone(),
            project: project.clone(),
            project_source: import.project_source.clone(),
        });
    }
    Ok(())
}
