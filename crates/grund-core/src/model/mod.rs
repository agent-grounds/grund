//! The model component (§AR-system.2.2): the data every other component passes
//! along — `Catalog`, `Declaration`, `Citation` and `Report`, plus the value
//! records (§FS-values.2). It consumes nothing and is types plus tiny helpers,
//! so it knows nothing of the tree, the rules or a frontend. The `Config` record
//! sat here while config was a file-name category and is config's own since
//! §AR-system.2.3 became a module; the grounding-structure records `Catalog`
//! carries came the other way, out of the scanner, when §AR-system.2.5 became
//! one (§AR-scanner.2.7).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `model*` and
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
//! show refusal of §FS-lsp and §FS-errors.5.2 carry them and both are the queries'.
//! The snapshot canonicalization of §AR-lsp.5 joined `paths.rs` out of
//! `scanner/tree.rs` the same way, and left again, with the physical path keys
//! and the input observer that sat in `check_inputs.rs`, for the sources
//! component when §AR-system.2.13 became one: each asks the disk, and only
//! sources does (§AR-system.5).

mod armed;
// §AR-config.9.1: the rendered catalog's rows, which checker and config read.
pub(crate) mod catalog;
mod codes;
mod conflict_block;
mod e2e;
mod excluded_stub_targets;
mod expected;
mod failure;
mod file_record;
mod headings;
mod id;
mod line_owners;
mod missing_stub_repair;
mod paths;
mod patterns;
mod records;
mod report;
mod section_lock;
mod stub_resolution;
mod text;
mod unread_roots;
mod values;

pub use armed::{Armed, Switch};
pub use catalog::{ClauseBy, CodeClause, CodeEnabled, CodeSeverity};
pub use codes::Code;
pub use e2e::{E2eCase, E2eSpecRef};
pub(crate) use expected::GENERATED_SECTIONS;
pub use expected::{Expected, ExpectedEntrypoint, ExpectedSection, ScopedExpectation};
pub(crate) use failure::OperationContext;
pub use failure::OperationDiagnostic;
pub use headings::{
    NearMissHeading, RelativeHint, RelativeSectionParent, SectionHeadingOutsideDeclaration,
    UnmarkedHeading,
};
pub(crate) use line_owners::{FileLineOwnership, OwnerRun, RangeOwnership, SectionRun};
pub use records::{
    Catalog, Citation, Declaration, DocCommentBlock, FileHeading, FileStructure, Id,
    InlineCitationSite, SectionInfo, ShowOutput, ShowSection,
};
pub use report::{Finding, FindingSite, Report};
pub use values::{
    DeclarationSource, EmbeddedValueRoot, InvalidValueSite, ValueBinding, ValueComponent,
    ValueComponentKind, ValueRootOrigin,
};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use conflict_block::ConflictBlock;
pub(crate) use excluded_stub_targets::ExcludingRule;
pub(crate) use file_record::{
    BlockForm, FileRecord, FileRecords, FileText, HeadingKind, LineZone, RecordedBlock,
    RecordedHeading,
};
pub(crate) use headings::UnmarkedHeadingCandidate;
// §AR-system.2.2: checker and queries share the guidance.
pub(crate) use missing_stub_repair::MissingStubRepair;
pub(crate) use paths::{
    configured_home_path_key, format_path, is_hidden, normalize_path_lexically, relative_from_base,
    scanned_decl_relative_path, scanned_path_key, sort_path_key,
};
pub(crate) use patterns::GlobCitation;
pub(crate) use records::{LegacyCitationCandidate, LocalSectionCitationCandidate, ShowRenderMode};
pub(crate) use report::{CheckReport, Diagnostic, LANDED_CLAUSE, Site};
pub(crate) use section_lock::{
    LockEntry, LockLines, SECTION_LOCK_DIR, SectionLockFile, locked, name_under_number,
    named_section_path, read_lock_lines, render_lock, section_lock_path,
};
pub(crate) use stub_resolution::{
    Paired, PairedDeclarations, StubResolution, paired_declarations, section_resolves,
};
pub(crate) use text::{CITATION_DIRECTION_REPAIR, format_list, json_escape, plural};
pub(crate) use unread_roots::{Refused, RootOrigin, RootRow};
pub(crate) use values::{
    JSON_NUMBER_RE, authored_component, component_text_is_valid, first_unequal_component,
    joined_value_components, positive_numeric_component, root_literal_parts,
    value_binding_section_ends_in_coordinate, value_components_equal,
};
