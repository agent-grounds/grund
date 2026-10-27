//! What a fetched kind's missing snapshot is (§FS-config.3.4.10.3): the
//! `missing-snapshot` warning where the kind's resolution is `should`, and the
//! `dangling` error otherwise. The reference-resolution family in
//! `references.rs` asks it at each site, and `judge` asks it once to record
//! whether the warning is on (§FS-check.1.4.1), so the two cannot disagree.

use std::collections::BTreeMap;

use crate::config::{KindResolution, Schema};
use crate::model::{Armed, Switch};
use crate::resolver::WorkspaceCheckTarget;

/// A kind's `resolve` policy, asked by the citation's namespace (§FS-check.4.12).
pub(super) type Resolve<'a> = dyn Fn(Option<&str>, &str) -> Option<KindResolution> + 'a;

/// §FS-config.3.4.10.3: a fetched `kind`'s missing snapshot is the
/// `missing-snapshot` warning where its resolution in the rules of the project
/// `namespace` names is `should`, and the `dangling` error anywhere else. The one
/// reading the emit site in `references.rs` and [`arm_missing_snapshot`] both ask.
pub(super) fn snapshot_warns(resolve: &Resolve<'_>, namespace: Option<&str>, kind: &str) -> bool {
    resolve(namespace, kind) == Some(KindResolution::Should)
}

/// §FS-check.1.4.1: `missing-snapshot` is on where a citation this pass judges can
/// be one — a fetched kind of the project, or of a workspace target, that warns by
/// [`snapshot_warns`]. `fetch` alone resolves `must` (§FS-config.3.4.10.2), so it
/// leaves the warning off.
pub(super) fn arm_missing_snapshot(
    schema: &Schema,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    resolve: &Resolve<'_>,
    armed: &mut Armed,
) {
    let targets = std::iter::once((None, schema)).chain(
        workspace
            .iter()
            .map(|(alias, target)| (Some(alias.as_str()), target.schema)),
    );
    if targets.into_iter().any(|(namespace, schema)| {
        schema
            .rows
            .iter()
            .any(|row| row.fetches() && snapshot_warns(resolve, namespace, &row.name))
    }) {
        armed.arm(Switch::SnapshotWarning);
    }
}
