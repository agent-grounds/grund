//! The queries component (§AR-system.2.7): one answer per question, read from
//! the `Findings` a run already produced — a declaration body (§FS-show), the
//! citers of an ID (§FS-refs), the catalog (§FS-list), per-file coverage
//! (§FS-cover), shell completions (§FS-completions), and the editor's hover and
//! on-type answers (§FS-lsp). A query is the data half: it knows no rendering,
//! because the text and JSON shapes belong to the frontends.
//!
//! Three files left when §AR-system.2.10 became a component, all one fact: a
//! declaration's body sliced by the spans a scan recorded is a function of the
//! loaded findings rather than of one command's answer, and the checker's
//! lead-budget rule was reading it upward out of this component
//! (§FS-check.4.13, §AR-system.4). `body.rs` is `resolver/body.rs` now, the
//! point-body pair `resolver/point_body.rs`, and the E2E case manifest that
//! pair asks for `resolver/e2e_body.rs`, out of `show.rs` with the three text
//! helpers only the slicer reads (§AR-resolver.placement).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `show*`, `list*`
//! and editor-answer category files, one per question — the show entry points,
//! the batch adapter over one loaded context, the catalog's shared citation
//! counts, the point-size catalog, and the two editor answers.
//! The editor pair is `editor_hover` and `editor_on_type` rather than `lsp` and
//! `on_type`: the engine names no frontend's protocol (§AR-system.4), and what
//! these two hold is the hover body and the keystroke rule of §FS-lsp, which an
//! LSP server transports but does not own (§AR-lsp).
//!
//! Two files came in when §AR-system.2.9 became a module, both because a record
//! the api's contract declared was produced or read here (§AR-system.4).
//! `show_query.rs` is what a show query is asked and what it refuses with — the
//! options of §FS-distribution.3.1 and the typed refusal of §FS-errors.5.2, which
//! `show.rs` raises and the batch read upward. `editor_snapshot.rs` is the
//! editor's snapshot vocabulary of §FS-lsp.1, which the title hover read upward;
//! the walk that *fills* a snapshot stays in `api/lsp_snapshot.rs`, needing the
//! whole pipeline beneath it. What went the other way is the snapshot path
//! canonicalization, down into `model/paths.rs` where the on-type rule and the
//! formatter's suppression both read it (§AR-lsp.5.1).
//!
//! This is the first component whose files were **split** rather than moved.
//! Only data-returning questions stayed here; the five argv/rendering adapters
//! retired with the engine process frontend (§AR-system.2.9.1). The coverage
//! index is `cover` in `api/cover.rs`, while completion scripts are rendered by
//! the CLI. What crosses back the other way is `measure_point_text`, which went
//! down into `config/point_sizes.rs` beside the `PointSizeUnit` whose meaning it
//! is.

mod batch;
mod citation_counts;
mod editor_hover;
mod editor_on_type;
mod editor_snapshot;
mod show;
mod show_query;
mod sizes;

pub use batch::{BatchShowFailure, BatchShowQuery, BatchShowRecord, show_batch_with_scope};
pub use editor_hover::{
    LspUsage, citation_under_title, lsp_hover_with_kind_title, lsp_title_hover_body,
};
pub use editor_on_type::{DeclaredId, LineEdit, can_replace_trigger_at, on_type_line_edits};
pub use editor_snapshot::{
    LspCitation, LspDeclaration, LspFindingRange, LspSnapshot, LspSnapshotOpts,
    LspSnapshotWithMetadata, LspStub,
};
pub use show_query::{ShowFormat, ShowMode, ShowOpts, ShowQueryError};
pub use sizes::{ListSizeEntry, ListSizeMeasurement, ListSizeOpts, ListSizeOutput, list_sizes};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use citation_counts::ListCitationCounts;
pub(crate) use show::{render_show_output_json, show_declaration_with_overlays};

// What other components' tests read (§AR-core-module-layout.1.3).
#[cfg(test)]
pub(crate) use show::show_declaration;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_lsp_hover;
#[cfg(test)]
mod tests_workspace_message_paths;
