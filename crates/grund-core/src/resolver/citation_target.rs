//! The one function that maps a citation to the project it resolves against
//! (§AR-system.2.10, §AR-resolver.1): the target's `Findings` and the `Config`
//! its ID is parsed and rendered with, or `None` when the alias is unknown
//! (§FS-workspace.1, §FS-workspace.8).
//!
//! Every consumer of citations goes through here rather than matching on
//! `citation.namespace` itself, which is what keeps `check`, the editor jump and
//! every query command agreeing on what "qualified" means (§AR-resolver.1). A
//! `None` return is never a silent skip: the calling rule turns it into a
//! located diagnostic (§AR-resolver.2).
//!
//! It sat in `checker/support.rs` with the rules that call it, which had the
//! grammar's shorthand resolution reading the target record upward out of the
//! checker (§AR-system.4). The record is a pair of borrows and the function a
//! map lookup — neither is a rule — so both belong to the component that holds
//! the loaded set they are borrowed from.

use std::collections::BTreeMap;

use crate::config::Config;
use crate::model::{Citation, Findings};

/// One project a citation can resolve against, as a rule needs it: the findings
/// the ID is looked up in and the config that spells it, because a workspace may
/// mix `[id] format`s (§FS-workspace.1, §AR-workspace.2).
pub(crate) struct WorkspaceCheckTarget<'a> {
    pub(crate) findings: &'a Findings,
    pub(crate) config: &'a Config,
}

/// §AR-resolver.1: the resolver itself. An unqualified citation resolves against
/// `local`, a qualified one against the alias-selected project, and an unknown
/// alias returns `None` for the caller to locate.
pub(crate) fn target_for_citation<'a>(
    cite: &Citation,
    local: &'a Findings,
    local_config: &'a Config,
    workspace: &'a BTreeMap<String, WorkspaceCheckTarget<'a>>,
) -> Option<WorkspaceCheckTarget<'a>> {
    match cite.namespace.as_deref() {
        Some(namespace) => workspace.get(namespace).map(|target| WorkspaceCheckTarget {
            findings: target.findings,
            config: target.config,
        }),
        None => Some(WorkspaceCheckTarget {
            findings: local,
            config: local_config,
        }),
    }
}

/// Whether a citation's target declares the ID — the same resolution above,
/// asked as a yes/no by the rules that only need that (§FS-check.3.1).
pub(crate) fn citation_resolves(
    cite: &Citation,
    local: &Findings,
    local_config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> bool {
    target_for_citation(cite, local, local_config, workspace)
        .map(|target| target.findings.declarations.contains_key(&cite.id))
        .unwrap_or(false)
}
