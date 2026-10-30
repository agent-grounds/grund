//! The imports of a `[workspace]` block (§FS-imports.declaration): each
//! `[workspace.imports.<alias>]` table of a block the expansion reaches, named at
//! the block's sibling level and handed to the resolver as the mount edge it loads
//! (§AR-workspace.imports, §AR-resolver.mount). Expansion reaches the blocks, so it is
//! where an import's alias meets the root's and the members' (§FS-workspace.3);
//! the target's own config is the loader's to read.

use anyhow::Result;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::expand::qualify_alias;
use super::scope::{config_location_error, project_label};
use crate::config::{Config, Import, MountEdge, config_file_in};
use crate::model::{format_path, relative_from_base};
use crate::sources::{Sources, canonical_path};

/// The mount edges `block`'s imports declare, under `prefix`, the alias path of the
/// block's own level (§FS-imports.declaration.alias, §FS-workspace.6.1.5).
///
/// Each alias joins `siblings`, which already holds the block's root and members:
/// a collision is refused at the import's table line, naming the first claimant by
/// its role and the import by its path as written (§FS-imports.failures). An import
/// whose path names no directory has no root to mount, so it is no edge.
///
/// Every refusal is located in `block`'s config, at the line the failure table
/// names; a target's own config is read by the loader (§AR-resolver.mount).
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
        let Some(root) = import_root(block, import, path)? else {
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

/// The canonical root `import` names, or `None` where nothing is there to mount
/// (§FS-imports.declaration.path, §FS-imports.declaration.root). Each test is made on
/// the canonical path, so a symlink is followed as far as the directory whose
/// identity is then checked, and two spellings of one root are one target.
fn import_root(block: &Config, import: &Import, path: &str) -> Result<Option<PathBuf>> {
    let alias = &import.alias;
    let at_table = |message: String| config_location_error(block, import.source.as_ref(), message);
    let at_path =
        |message: String| config_location_error(block, import.path_source.as_ref(), message);
    let root = match canonical_path(block.root.join(path)) {
        Ok(root) => root,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(at_table(format!(
                "import `{alias}`: cannot read `{path}`: {err}"
            )));
        }
    };
    let own = canonical_path(&block.root).unwrap_or_else(|_| block.root.clone());
    if root == own {
        let message = format!("import `{alias}` resolves to the config root that declares it");
        return Err(at_path(message));
    }
    if own.starts_with(&root) {
        let above = relative_from_base(&own, &root);
        let message = format!(
            "import `{alias}` resolves to `{}`, which contains the config root that declares it",
            format_path(&above)
        );
        return Err(at_path(message));
    }
    if root.starts_with(&own) {
        let message = format!(
            "import `{alias}` resolves inside the config root that declares it; \
             list it in [workspace] members instead"
        );
        return Err(at_path(message));
    }
    // §FS-check.6.1.1: the target's directory is one of the run's inputs.
    if let Err(err) = Sources::on_disk().read_dir(&root) {
        return Err(at_table(format!(
            "import `{alias}`: cannot read `{path}`: {err}"
        )));
    }
    if config_file_in(&root).is_none() {
        let message =
            format!("import `{alias}`: `{path}` holds no grund.toml or .agents/grund.toml");
        return Err(at_table(message));
    }
    Ok(Some(root))
}
