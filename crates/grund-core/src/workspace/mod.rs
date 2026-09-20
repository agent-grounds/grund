//! The workspace component (§AR-system.2.4): the multi-project scope a run
//! operates on, read out of configs — member expansion, the claims that spell an
//! alias path, scope narrowing and the boundary roots a scan stops at
//! (§FS-workspace). It consumes configs and knows no rule and no rendering; its
//! own invariants are §AR-workspace.
//!
//! This is the **config-time** half. Loading those projects — scanning each one,
//! reconciling citations across the set, and answering which project a citation
//! resolves against — runs scans, so it sits above the scanner as the resolver
//! of §AR-system.2.10 and reads this component downward (§AR-resolver.placement).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `workspace*`
//! category files, one per question §AR-system.2.4 names — which project a scope
//! belongs to, what one `members` list expands to, what the second list adds,
//! which projects the whole tree holds, what a qualified ID argument names, and
//! the two findings about a block nobody lists or nobody reads. `findings.rs`
//! joined them with the three cautions an expansion earns: the sentences are
//! this component's, and every one is returned as a `Diagnostic` rather than
//! printed (§DA-engine-renders-nothing). The `AbsentOptionalNamespace` record
//! went the other way, down into `config/` beside the `Config` field that
//! carries it.
//!
//! Three files left when §AR-system.2.10 became a component, each because it
//! needs loaded findings rather than configs: `context.rs` with the loaded
//! project set and its loaders, `id_candidates.rs` with the candidate clause
//! that searches them — `join_alternatives` went with it, the checker still
//! reading it downward — and the walk half of §FS-check.4.10.2's answer, which
//! the boundary passes below only pose, on the run's warning channel. `WorkspaceCitationTarget` stayed,
//! beside the expansion whose two facts it carries, and the qualified
//! ID-argument split stayed as `id_arg.rs`: both read config text and no scan.

mod expand;
mod findings;
mod id_arg;
mod members;
mod optional_members;
mod scope;
mod unlisted;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use expand::{
    WorkspaceCitationTarget, enclosing_workspace_of, expand_workspace_tree,
    expand_workspace_tree_with_report_base,
};
pub(crate) use findings::{
    block_relative_root, uncovered_block_scope_roots, unread_block_diagnostic,
};
pub(crate) use id_arg::split_qualified_id_arg;
pub(crate) use members::AncestorWorkspaces;
pub(crate) use optional_members::{
    absent_only_workspace_caution, absent_optional_member_warnings, namespace_is_unverified,
};
pub(crate) use scope::{
    apply_workspace_boundary, populate_workspace_boundary, resolve_workspace_config,
    scope_is_config_root,
};
pub(crate) use unlisted::{
    unlisted_workspace_block_run_warnings, unlisted_workspace_block_warnings,
};

// What another component's tests read (§AR-core-module-layout.1.3): the
// boundary-root form of an expanded member list, which the scanner's
// parallel-scan cases compare against. The other two went beside their cases.
#[cfg(test)]
pub(crate) use members::expand_workspace_members;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_absorbed_scan;
#[cfg(test)]
mod tests_claim_answers;
#[cfg(test)]
mod tests_claims;
#[cfg(test)]
mod tests_nested;
#[cfg(test)]
mod tests_optional_members;
#[cfg(test)]
mod tests_run_warning_anchors;
#[cfg(test)]
mod tests_scope;
#[cfg(test)]
mod tests_unlisted_block;
