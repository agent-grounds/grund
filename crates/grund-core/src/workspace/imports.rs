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
use crate::config::{Config, Import, ImportRead, MountEdge, config_file_in};
use crate::model::{
    Armed, Code, Diagnostic, Switch, format_path, normalize_path_lexically, relative_from_base,
};
use crate::sources::{Sources, canonical_path};

/// The mount edges `block`'s imports declare, under `prefix`, the alias path of the
/// block's own level (§FS-imports.declaration.alias, §FS-workspace.6.1.5).
///
/// Each alias joins `siblings`, which already holds the block's root and members:
/// a collision is refused at the import's table line, naming the first claimant by
/// its role and the import by its path as written (§FS-imports.failures). An import
/// whose path names no directory is a missing edge, which the loader announces and
/// mounts nothing for (§FS-imports.checks.import-missing).
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
        let found = import_root(block, import, path)?;
        let missing = found.is_none();
        let root = found.unwrap_or_else(|| normalize_path_lexically(&block.root.join(path)));
        siblings.insert(import.alias.clone(), root.clone());
        edges.push(MountEdge {
            alias: qualify_alias(prefix, &import.alias),
            segment: import.alias.clone(),
            written: path.clone(),
            root,
            missing,
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

/// §FS-imports.checks: the run's word on every import edge it reached, each at the
/// table line of the config that declares it and by the alias path the run spells
/// it with: an error for a missing one, and a warning for each one read live. The
/// errors come first.
pub(crate) fn import_diagnostics(config: &Config) -> (Vec<Diagnostic>, Vec<Diagnostic>) {
    let (missing, live): (Vec<_>, Vec<_>) = config
        .run()
        .workspace
        .imports_read
        .iter()
        .partition(|import| import.missing);
    let diagnostic = |import: &ImportRead, code: Code, message: String| Diagnostic {
        code,
        path: import.source.as_ref().map(|source| source.path.clone()),
        line: import.source.as_ref().map(|source| source.line),
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    };
    let missing = missing.into_iter().map(|import| {
        let message = format!(
            "{} — citations into namespace `{}` were not checked",
            missing_import_message(import),
            import.alias
        );
        diagnostic(import, Code::IMPORT_MISSING, message)
    });
    let live = live.into_iter().map(|import| {
        let (alias, written) = (&import.alias, &import.written);
        let message = format!("import `{alias}` is read live from `{written}`, unpinned");
        diagnostic(import, Code::IMPORT_LIVE, message)
    });
    (missing.collect(), live.collect())
}

/// §FS-check.1.4.1: both import checks are on wherever a config the run reads
/// declares an import, a dependency's included, whether or not any is missing.
pub(crate) fn arm_imports(config: &Config, armed: &mut Armed) {
    if !config.run().workspace.imports_read.is_empty() {
        armed.arm(Switch::Import);
    }
}

/// What a missing import is called wherever it is named: the error's opening and
/// the whole refusal of a query into it (§FS-imports.checks.import-missing).
pub(crate) fn missing_import_message(import: &ImportRead) -> String {
    let (alias, written) = (&import.alias, &import.written);
    format!("import `{alias}` is missing: `{written}` is not a directory")
}
