//! The workspace a config *declares*, read for its configs alone (§FS-rules.4.1,
//! §FS-workspace.5.1): what `init` judges a rule's object namespaces against
//! without loading a workspace of its own, and the kinds a narrowed run reads
//! (§AR-config.1.5). The checker builds the vocabulary from the schemas
//! returned here and names no façade (§AR-config.5).

use anyhow::Result;

use super::expand::{WorkspaceProjectEntry, expand_workspace_projects};
use super::scope::open;
use crate::config::{Config, Invocation, Schema, install_kind_table};

/// Each project the workspace `config` declares, by alias, with its schema —
/// empty where `config` declares no `[workspace]`.
///
/// Never a workspace climbed to from above: judging off a climbed tree would make
/// the same bytes valid or invalid depending on what sits beside the checkout
/// (§FS-workspace.5.1, §DF-unverifiable-rule-scope). Only stage 1 of the load,
/// because the aliases are all that is wanted. Best-effort: an expansion that
/// fails costs only the members it could not reach, and a config that declares
/// `[workspace]` keeps at least its own namespace — the entry an empty `members`
/// yields, recovered by expanding the same tree with the member list emptied
/// (§FS-rules.4.1.1).
pub(crate) fn declared_member_schemas(config: &Config) -> Vec<(String, Schema, u32)> {
    declared_projects(config)
        .0
        .into_iter()
        .map(|entry| {
            let version = entry.config.project().version;
            (entry.alias, entry.config.schema().clone(), version)
        })
        .collect()
}

/// The config a command whose `<path>` narrows the walk runs on: `cover` and
/// `cover --lines` (§FS-scan.path.cover). This is the one
/// place a narrowed scan of a config that declares `[workspace]` is set up, so
/// each of them judges the pinned and `*/` `[citations]` targets its whole run
/// judges and fails the load on the same first one
/// (§AR-config.1.5, §REQ-one-answer.scope.narrowed, §FS-config.3.9.5.2). A
/// scope that is the config root is the whole run, and its expansion builds
/// the table.
pub(crate) fn open_narrowable(invocation: &Invocation) -> Result<Config> {
    let mut config = open(invocation)?;
    if config.workspace_declared && !invocation.names_root(&config.root) {
        install_declared_kind_table(&mut config)?;
    }
    Ok(config)
}

/// §AR-config.1.5: a narrowed run of a config that declares
/// `[workspace]` loads no member's files and still reads the kinds its declared
/// workspace configures, so it judges the pinned and `*/` `[citations]`
/// targets a full run judges, with the same one call — unless the declared
/// read fell back, whose table is no ground for a refusal (§FS-config.3.9.5.2).
/// Nothing else of the declared read reaches the run: a rule object here stays
/// §FS-rules.4.1's.
fn install_declared_kind_table(config: &mut Config) -> Result<()> {
    let (mut projects, whole) = declared_projects(config);
    install_kind_table(
        config,
        projects
            .iter_mut()
            .map(|entry| (entry.alias.as_str(), &mut entry.config)),
        whole,
    )
}

/// Each project the workspace `config` declares, best-effort as
/// [`declared_member_schemas`] says, and whether that is all of them — `false`
/// where the expansion failed and only the config's own namespace was
/// recovered; empty and whole where it declares no `[workspace]`.
fn declared_projects(config: &Config) -> (Vec<WorkspaceProjectEntry>, bool) {
    if !config.workspace_declared {
        return (Vec::new(), true);
    }
    let mut root_config = config.clone();
    match expand_workspace_projects(&mut root_config, &config.root) {
        Ok(entries) => (entries, true),
        Err(_) => {
            let mut alone = config.without_members();
            let entries = expand_workspace_projects(&mut alone, &config.root).unwrap_or_default();
            (entries, false)
        }
    }
}
