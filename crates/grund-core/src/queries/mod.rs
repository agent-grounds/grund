//! The queries component (§AR-system.2.7): one answer per question, read from
//! the `Findings` a run already produced — a declaration body (§FS-show), the
//! citers of an ID (§FS-refs), the catalog (§FS-list), per-file coverage
//! (§FS-cover), shell completions (§FS-completions), and the editor's hover and
//! on-type answers (§FS-lsp). A query is the data half: it knows no rendering,
//! because the text and JSON shapes belong to the frontends.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `show*`, `list*`
//! and editor-answer category files, one per question — the body slicer and the
//! show entry points, the batch adapter over one loaded context, the catalog's
//! shared citation counts, the point-size catalog, and the two editor answers.
//! The editor pair is `editor_hover` and `editor_on_type` rather than `lsp` and
//! `on_type`: the engine names no frontend's protocol (§AR-system.4), and what
//! these two hold is the hover body and the keystroke rule of §FS-lsp, which an
//! LSP server transports but does not own (§AR-lsp).
//!
//! Two files came in when §AR-system.2.9 became a module, both because a record
//! the api's contract declared was produced or read here (§AR-system.4).
//! `show_query.rs` is what a show query is asked and what it refuses with — the
//! options of §FS-distribution.3.1 and the typed refusal of §FS-errors.5, which
//! `show.rs` raises and the batch read upward. `editor_snapshot.rs` is the
//! editor's snapshot vocabulary of §FS-lsp.1, which the title hover read upward;
//! the walk that *fills* a snapshot stays in `api/lsp_snapshot.rs`, needing the
//! whole pipeline beneath it. What went the other way is the snapshot path
//! canonicalization, down into `model/paths.rs` where the on-type rule and the
//! formatter's suppression both read it (§AR-lsp.5).
//!
//! This is the first component whose files were **split** rather than moved.
//! Five of them wrote to a stream, because the deprecated `main_entry()` path
//! renders inside the engine (§AR-system.2.9): the `command_*` adapters that
//! parse argv, print text or JSON and return an `ExitCode` are the deprecated
//! path's, not a query's, so they are `compat/show.rs`, `compat/refs.rs`,
//! `compat/cover.rs`, `compat/list.rs` and `compat/completions.rs`. `refs`,
//! `cover` and `completions` had no data half at all — the citer walk is
//! `command_refs`'s own, the coverage index is `cover` in `api/cover.rs`, and a
//! completion script is bytes printed to stdout — so those three files went to
//! `compat/` whole. What crosses back the other way is `measure_point_text`,
//! which went down into `config/point_sizes.rs` beside the `PointSizeUnit` whose
//! meaning it is.

mod batch;
mod body;
mod citation_counts;
mod editor_hover;
mod editor_on_type;
mod editor_snapshot;
mod show;
mod show_query;
mod sizes;

pub use batch::{BatchShowFailure, BatchShowQuery, BatchShowRecord, show_batch_with_scope};
pub use editor_hover::{LspUsage, citation_under_title, lsp_title_hover_body};
pub use editor_on_type::{DeclaredId, LineEdit, can_replace_trigger_at, on_type_line_edits};
pub use editor_snapshot::{
    LspCitation, LspDeclaration, LspFindingRange, LspSnapshot, LspSnapshotOpts, LspStub,
};
pub use show_query::{ShowFormat, ShowMode, ShowOpts, ShowQueryError};
pub use sizes::{ListSizeEntry, ListSizeMeasurement, ListSizeOpts, ListSizeOutput, list_sizes};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use body::{PointBodyCache, point_body_pair};
pub(crate) use citation_counts::ListCitationCounts;
pub(crate) use show::{render_show_output_json, show_declaration, show_declaration_with_overlays};
