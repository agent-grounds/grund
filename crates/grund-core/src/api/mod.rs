//! The api component (§AR-system.2.9): the embedding surface — data-returning
//! functions and the public option and output records every frontend is built
//! on (§AR-bindings.2, §FS-distribution.3). It consumes everything below it and
//! writes to no stream, exits no process and knows no frontend.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are one per surface — `check`,
//! `show`, `complete_ids`, `id`, `config`, `cover`, `fmt`, `refs`, `list` and the
//! editor snapshot — beside the private adapters that fill them: the report
//! conversion in `report.rs`, the `refs` walk in `refs_query.rs`, the run in
//! `run.rs` with the warnings it folds in, and the snapshot's spans in
//! `lsp_ranges.rs`. That is the boundary §AR-core-module-layout.2 names and the
//! size register was holding a ceiling for: the contract is what a caller reads,
//! the adapters are the machinery behind it, and no file holds both.
//!
//! Two things came *up* into this component with the move, both out of the
//! deprecated `check` adapter, because the published `check` was reading them
//! and nothing may read `compat/`: the check run itself — config, scan, check,
//! cautions, config findings — and the scope cautions of §FS-check.2.2 and
//! §FS-check.4.5 that ride with it. Only `command_check` was ever the renderer.
//!
//! Five groups went *down*, each to the lowest component that reads it
//! (§AR-system.4). The published report records `Report`, `Finding` and
//! `FindingSite` are `model/report.rs`'s, beside the internal ones they are
//! rendered from. The show options and the typed show refusal are
//! `queries/show_query.rs`'s and the editor snapshot's records
//! `queries/editor_snapshot.rs`'s, both produced or read by the queries that
//! answer with them. The scan-error record is `scanner/scan_error.rs`'s, the walk
//! being what raises one. And the snapshot path canonicalization of §AR-lsp.5 is
//! `model/paths.rs`'s, because the queries and the writers are siblings and both
//! rebase a path against it. Every one stays `pub` and `lib.rs` re-exports it, so
//! the embedding surface is the same 132 names it was.

mod check;
mod complete_ids;
mod config;
mod config_findings;
mod cover;
mod fmt;
mod id;
mod list;
mod lsp_ranges;
mod lsp_snapshot;
mod refs;
mod refs_query;
mod report;
mod run;
mod scope_cautions;
mod show;

pub use check::{CheckOpts, CheckOutput, check, check_with_opts, scan};
pub use complete_ids::{CompleteIdsOpts, complete_ids, complete_ids_with_run_warnings};
pub use config::{
    ReferenceStyle, config_run_warnings, config_warnings, effective_config, reference_style,
    validate_config,
};
pub use cover::{
    CoverCitation, CoverEntry, CoverOpts, CoverOutput, CoverTextCitation, CoverTextEntry,
    CoverTextOutput, cover, cover_text,
};
pub use fmt::{FmtChange, FmtOpts, FmtOutput, format_references};
pub use id::{IdOpts, IdProposal, IdProposalOutcome, propose_id, propose_id_with_run_warnings};
pub use list::{
    ListEntry, ListOpts, ListOutput, ListSummary, ListValueRoot, list, list_with_run_warnings,
};
pub use lsp_snapshot::lsp_snapshot;
pub use refs::{
    REFS_QUERY_FAILURE_WARNING, RefHit, RefsOpts, RefsOutcome, RefsOutput, RefsQueryFailure,
    RefsQueryFailureKind, refs, refs_outcome, refs_query_failure_is_exit_one,
};
pub use report::render_finding_sites_json;
pub use show::{show, show_with_overlays, show_with_scope};

// What the deprecated `compat/check.rs` renderer reads, by module path because
// nothing here may import it (§AR-system.2.9): the run the published `check` and
// the deprecated one share, so neither can report what the other does not.
pub(crate) use run::run_check;

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// run record, which the fixtures drive directly.
#[cfg(test)]
pub(crate) use run::CheckRun;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1).
#[cfg(test)]
mod tests_check_full_scope;
#[cfg(test)]
mod tests_cover_workspace;
#[cfg(test)]
mod tests_embedding;
#[cfg(test)]
mod tests_external_facts;
#[cfg(all(test, unix))]
mod tests_fmt_errors;
#[cfg(all(test, unix))]
mod tests_fmt_workspace;
#[cfg(test)]
mod tests_refs_query_failures;
#[cfg(test)]
mod tests_shorthand_docstring;
#[cfg(test)]
mod tests_shorthand_surfaces;
