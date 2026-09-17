//! The model component (§AR-system.2.2): the data every other component passes
//! along — `Findings`, `Declaration`, `Citation`, `Report` and the `Config`
//! record, plus the value records (§FS-values.2). It consumes nothing and is
//! types plus tiny helpers, so it knows nothing of the tree, the rules or a
//! frontend.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `model*` and
//! `values` category files, one per record family §AR-system.2.2 names.

mod e2e;
mod headings;
mod records;
mod report;
mod values;

pub use e2e::{E2eCase, E2eSpecRef};
pub use headings::{NearMissHeading, SectionHeadingOutsideDeclaration, UnmarkedHeading};
pub use records::{
    Citation, CitationDisjunction, CitationLevel, CitationRules, CitationTarget, Config,
    ConfigLocation, Declaration, Findings, Id, InlineCitationSite, KindCitationRules,
    NamespaceMatch, SectionInfo, ShorthandPolicy, ShowOutput, ShowSection,
};
pub use values::{
    DeclarationSource, EmbeddedValueRoot, InvalidValueSite, ValueBinding, ValueComponent,
    ValueComponentKind,
};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use headings::UnmarkedHeadingCandidate;
pub(crate) use records::{
    CODE_SOURCE_KIND, DEFAULT_GROUNDING_LEVEL, GROUNDING_LEVELS, LegacyCitationCandidate,
    ShowRenderMode, TextOverlays, WorkspaceCitationTarget, citing_kind_names,
    declared_homeless_kind, kind_prefixes, non_citable_kind_error, normalize_path_lexically,
    render_qualified_id, resolve_stub_target,
};
pub(crate) use report::{CheckReport, Diagnostic, Site};
pub(crate) use values::{
    JSON_NUMBER_RE, authored_component, component_text_is_valid, kind_uses_values,
    value_components_equal,
};
