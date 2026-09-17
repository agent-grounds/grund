//! The writers component (§AR-system.2.8): the edits `grund` produces — citation
//! normalization and the cross-reference links of §FS-fmt, the proposed ID of
//! §FS-id, and the external fact snapshot of §FS-fetch. They consume `Findings`
//! and the tree, are the only components that write to it, and each writes only
//! what its spec names (§REQ-no-data-loss).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). Each file keeps the name of the writer it belongs
//! to rather than dropping a prefix, because this component holds several: the
//! `fmt_*` files are the formatter — the rewrite walk, the two suppressed
//! scopes, the link pass and its target, the value-binding protection, the
//! workspace pass, the completeness proof and the one refusal it carries out —
//! `id.rs` is the ID proposal, and the `fetch*` pair is the snapshot. The init
//! scaffold (§FS-init) and the integrations artifacts (§FS-integrations) are the
//! two writers still outside, as the flat `init*` and `integrations` categories,
//! and join this directory next.
//!
//! What the component does **not** hold any more. Three lexical items went down
//! into `grammar/` (§AR-system.2.1), because a component below was reading each
//! of them upward: the anchor derivation of §DF-github-anchor-fidelity, whole,
//! as `grammar/anchors.rs`; the ID renderer with its qualified form, into
//! `grammar/ids.rs`; and the record of one marked citation on a Markdown line,
//! beside them. The `[fmt] exclude` glob grammar of §FS-config.3.10 went down
//! into `config/fmt_block.rs`, where the reader that validates the key reads it
//! without looking up. What came the other way is the §FS-fmt.6.6 auto-enable
//! pair, out of the deprecated `fmt_cmd.rs`: neither of its two callers is the
//! command (§AR-system.2.9).

mod fetch;
mod fetch_write;
mod fmt_complete_findings;
mod fmt_error;
mod fmt_link_targets;
mod fmt_links;
mod fmt_rewrite;
mod fmt_shorthand_links;
mod fmt_suppress;
mod fmt_value_bindings;
mod fmt_workspace;
mod id;

pub use fetch::{FetchFailure, FetchFailureKind, fetch_snapshot};
pub use fmt_error::FmtScanAbort;

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use fmt_link_targets::markdown_link_target;
pub(crate) use fmt_links::flatten_cross_ref_links;
pub(crate) use fmt_rewrite::{FmtRunOpts, auto_cross_refs_for_scope, fmt_tree};
pub(crate) use fmt_suppress::{FmtDirectives, FmtExcluded};
pub(crate) use fmt_workspace::fmt_workspace_projects;
pub(crate) use id::{format_id, slugify_title};

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// per-line rewrite with its options, which the shorthand-rewrite cases drive
// directly, and the wrapper pass without §FS-fmt.2.4's prebuilt indexes.
#[cfg(test)]
pub(crate) use fmt_links::wrap_markdown_links;
#[cfg(test)]
pub(crate) use fmt_rewrite::{FmtLineOpts, fmt_line};
