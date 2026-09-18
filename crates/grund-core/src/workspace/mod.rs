//! The workspace component (§AR-system.2.4): the multi-project scope — member
//! expansion, claims, scope narrowing — and the one resolver that maps a
//! citation to its target project (§FS-workspace). It consumes configs and knows
//! no rule and no rendering; its own invariants are §AR-workspace.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `workspace*`
//! category files, one per question §AR-system.2.4 names — which project a scope
//! belongs to, what one `members` list expands to, what the second list adds,
//! which projects the whole tree holds, what a query command therefore holds,
//! and the two findings about a block nobody lists or nobody reads. The
//! `AbsentOptionalNamespace` record went the other way, down into `config/`
//! beside the `Config` field that carries it, and `join_alternatives` came down
//! out of `checker/references.rs` when §AR-system.2.6 became a module, this
//! being the lowest component that spells a candidate list.

mod context;
mod expand;
mod id_candidates;
mod members;
mod optional_members;
mod scope;
mod unlisted;

pub use id_candidates::names_member_id_candidate;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use context::{
    WorkspaceCitationTarget, WorkspaceContext, WorkspaceProject, load_narrowable_workspace_context,
    load_resolved_workspace_context, load_workspace_context, load_workspace_context_with_overlays,
    load_workspace_projects, split_qualified_id_arg,
};
pub(crate) use expand::{
    enclosing_workspace_of, expand_workspace_tree, expand_workspace_tree_with_report_base,
};
pub(crate) use id_candidates::{join_alternatives, with_member_id_candidates};
pub(crate) use members::{
    AncestorWorkspaces, UnreadBlockProbe, WorkspaceMember, absorbed_scan_roots,
    absorbed_scan_warning, undecidable_ancestor_claim_warning, unread_block_scope_root,
    unread_block_warning,
};
pub(crate) use optional_members::{
    absent_only_workspace_caution, absent_optional_member_warnings, namespace_is_unverified,
};
pub(crate) use scope::{
    apply_workspace_boundary, config_location_message, populate_workspace_boundary,
    resolve_workspace_config, scope_is_config_root,
};
pub(crate) use unlisted::unlisted_workspace_block_warnings;

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// absorbed-scan ramp release, the boundary-root form of an expanded member list,
// and the members-only text read the ancestor-claim cases drive directly.
#[cfg(test)]
pub(crate) use members::{
    ABSORBED_SCAN_ERROR_RELEASE, ancestor_member_entries, expand_workspace_members,
};
