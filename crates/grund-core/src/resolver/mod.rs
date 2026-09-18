//! The resolver component (§AR-system.2.10): the project map workspace expanded
//! out of configs, *loaded* — every project in scope scanned, its off-grammar
//! citations reconciled across the whole set — plus the four answers that are
//! functions of what a run loaded and of nothing else: which project a citation
//! resolves against, a declaration's body by the spans the scan recorded, the
//! link target that body's ID would be wrapped with, and whether a block that
//! opted out leaves a tree no scan reads (§FS-workspace.8, §AR-resolver).
//!
//! It runs scans, which is what puts it **above** the scanner while the config
//! half of the workspace stays below it: member expansion, the claims, scope
//! narrowing and the boundary roots read configs and nothing else, so they are
//! still `workspace/` (§AR-resolver.placement). The test is one question — does
//! the answer need loaded findings, or only configs?
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). Seven groups came from five components, each
//! because a component below was reading it upward or a sibling was answering
//! for it:
//!
//! - `context.rs` and `id_candidates.rs` out of `workspace/`, the loaded project
//!   set and the §FS-workspace.8.1.1 candidate clause that reads it. They are
//!   what had workspace reading the scanner for `ScanError`, the tree scan and
//!   the ID-argument resolver (§AR-system.4).
//! - `legacy_promotion.rs`, the qualified half of §FS-config.3.2's catalog
//!   reconciliation, out of `scanner/legacy.rs`: it takes the whole loaded set,
//!   so leaving it there would have had the scanner name this component's
//!   record. The per-candidate promotion it calls stays the scanner's.
//! - `citation_target.rs` out of `checker/support.rs` and
//!   `checker/references.rs`: the one function §AR-resolver.1 is about, and the
//!   pair of borrows it answers with, neither of which is a rule.
//! - `body.rs` with `point_body.rs` and `e2e_body.rs` out of `queries/`, the
//!   slicing of a declaration's body by recorded span, which the checker's
//!   lead-budget rule read upward out of a sibling's answer (§FS-check.4.13).
//! - `link_targets.rs` out of `writers/fmt_link_targets.rs`, the link a
//!   declaration's ID resolves to, which the checker's index-entry rule read the
//!   same way (§FS-check.3.18).
//! - `unread_block.rs` out of `workspace/members.rs`, the one half of
//!   §FS-check.4.10 that has to run the walker to answer.
//! - `shorthand.rs` out of `grammar/shorthand.rs`, the half of the number-only
//!   shorthand that needs the whole run's catalog: whose grammar parses a
//!   qualified token, whose format renders the canonical ID, whose policy lets a
//!   persisted spelling stand (§AR-resolver.4). Recognizing the shape stays
//!   lexical; only resolving it against every loaded project's declarations had
//!   the grammar naming records three components above it (§AR-system.4).
//!
//! What went the other way, down rather than up, when this component was carved
//! out: three walk-level facts workspace was reading from the scanner at config
//! time — `root_scope_roots` and `canonical_config_root` into
//! `config/scope_roots.rs`, because each reads `[scan] include`, the
//! `[[kinds]]` table and `config.root` and nothing of a tree, and `is_hidden`
//! into `model/paths.rs`, a predicate over a `Path` alone. `is_stub_for_inline_decl`
//! followed them into `model/records.rs` beside `resolve_stub_target`, so the
//! link target could ask which declaration is a stub without reading the checker
//! above it. `WorkspaceCitationTarget` and the qualified ID-argument split
//! stayed in `workspace/`: both are answers about configs and entry text, with
//! no scan in them.

mod body;
mod citation_target;
mod context;
mod e2e_body;
mod id_candidates;
mod legacy_promotion;
mod link_targets;
mod point_body;
mod shorthand;
mod unread_block;

pub use id_candidates::names_member_id_candidate;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use body::{PointBodyCache, extract_declaration_body};
pub(crate) use citation_target::{WorkspaceCheckTarget, citation_resolves, target_for_citation};
pub(crate) use context::{
    WorkspaceContext, WorkspaceProject, load_narrowable_workspace_context,
    load_resolved_workspace_context, load_workspace_context, load_workspace_context_with_overlays,
    load_workspace_projects,
};
pub(crate) use e2e_body::show_e2e_case;
pub(crate) use id_candidates::{join_alternatives, with_member_id_candidates};
pub(crate) use link_targets::{markdown_link_target, markdown_link_target_with_root};
pub(crate) use point_body::point_body_pair;
pub(crate) use shorthand::{
    ShorthandTargets, expand_shorthand_citations_with_origins, shorthand_token_expansion,
};
pub(crate) use unread_block::unread_block_scope_root;

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// single-line form of the shorthand rewrite, without the trigger origins the
// formatter threads through it.
#[cfg(test)]
pub(crate) use shorthand::expand_shorthand_citations;
