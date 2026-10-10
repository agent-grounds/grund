//! The workspace a config *declares*, read for its aliases alone (§FS-rules.4.1,
//! §FS-workspace.5.1): what `init` judges a rule's object namespaces against
//! without loading a workspace of its own. The checker builds the vocabulary from
//! the schemas returned here and names no façade (§AR-config.5).

use super::expand::expand_workspace_tree;
use crate::config::{Config, Schema};

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
pub(crate) fn declared_member_schemas(config: &Config) -> Vec<(String, Schema)> {
    if !config.workspace_declared {
        return Vec::new();
    }
    let mut root_config = config.clone();
    let entries = expand_workspace_tree(&mut root_config).unwrap_or_else(|_| {
        let mut alone = config.without_members();
        expand_workspace_tree(&mut alone).unwrap_or_default()
    });
    entries
        .into_iter()
        .map(|entry| (entry.alias, entry.config.schema().clone()))
        .collect()
}
