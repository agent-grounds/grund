//! The model component (§AR-system.2.2): the data every other component passes
//! along — `Findings`, `Declaration`, `Citation` and `Report`, plus the value
//! records (§FS-values.2). It consumes nothing and is types plus tiny helpers,
//! so it knows nothing of the tree, the rules or a frontend. The `Config` record
//! sat here while config was a file-name category and is config's own since
//! §AR-system.2.3 became a module; the grounding-structure records `Findings`
//! carries came the other way, out of the scanner, when §AR-system.2.5 became
//! one (§AR-scanner.2.7).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `model*` and
//! `values` category files, one per record family §AR-system.2.2 names, plus
//! `paths.rs`: the path keys a file is compared by, which came down out of
//! `checker/homes.rs` when §AR-system.2.6 became a module because the scanner
//! read five of them upward (§AR-system.4).
//!
//! Three things came the same way when §AR-system.2.9 became a module, each
//! because a component below the deprecated path was reading it upward. The
//! report's path spellings and the JSON escape joined `paths.rs` and a new
//! `text.rs`, out of the `output` category: `format_path`, `sort_path_key`,
//! `relative_from_base`, `json_escape` and `format_list`, every one a function
//! of a string or a `Path` with no stream and no `Config`. The published report
//! records — `Report`, `Finding` and `FindingSite` — joined `report.rs` beside
//! the internal ones they are rendered from, because the editor snapshot and the
//! show refusal of §FS-lsp and §FS-errors.5 carry them and both are the queries'.
//! And the snapshot canonicalization of §AR-lsp.5 joined `paths.rs` out of
//! `scanner/tree.rs`, the queries and the writers being siblings that both
//! rebase a path against it.

mod e2e;
mod headings;
mod paths;
mod records;
mod report;
mod text;
mod values;

pub use e2e::{E2eCase, E2eSpecRef};
pub use headings::{NearMissHeading, SectionHeadingOutsideDeclaration, UnmarkedHeading};
pub use paths::canonical_snapshot_path;
pub use records::{
    Citation, Declaration, DocCommentBlock, FileHeading, FileStructure, Findings, Id,
    InlineCitationSite, SectionInfo, ShowOutput, ShowSection,
};
pub use report::{Finding, FindingSite, Report};
pub use values::{
    DeclarationSource, EmbeddedValueRoot, InvalidValueSite, ValueBinding, ValueComponent,
    ValueComponentKind,
};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use headings::UnmarkedHeadingCandidate;
pub(crate) use paths::{
    canonicalize_existing_prefix, configured_home_path_key, format_path, normalize_path_lexically,
    paths_same_location, physical_path_key, relative_from_base, scanned_decl_relative_path,
    scanned_path_key, sort_path_key,
};
pub(crate) use records::{
    LegacyCitationCandidate, ShowRenderMode, TextOverlays, resolve_stub_target,
};
pub(crate) use report::{CheckReport, Diagnostic, Site};
pub(crate) use text::{format_list, json_escape};
pub(crate) use values::{
    JSON_NUMBER_RE, authored_component, component_text_is_valid, value_components_equal,
};
