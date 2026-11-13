//! What each code says of a file reached by two names, once, with no default: a code added to the
//! catalog fails the test beside this file until someone says which side it is on (§AR-scanner.1.8).
//! Only a file reached by two names ever asks, so a linkless tree pays nothing for the lookup.

use crate::model::{Code, Diagnostic};

/// What a finding of a code says of a file reached by two names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Side {
    /// What the file holds: one line at the name listed first, the others in a closing clause.
    Content,
    /// Where the file sits: judged at each spelling, one finding each.
    Place,
    /// Resolved from the spelling it was judged at, and it names that one.
    Judged,
    /// About the run or its configuration, and no scanned file has two names to give it.
    Run,
}

impl Side {
    /// `None` is what the test beside this file refuses: every code of the catalog has a side.
    pub(super) fn of(code: Code) -> Option<Self> {
        SIDES
            .iter()
            .find(|(_, codes)| codes.contains(&code))
            .map(|(side, _)| *side)
    }

    /// The side of one finding. A citation-direction code has two forms. Its file-unit form is
    /// asked of the row that owns the spelling, so it is Place. Its declaration-subject form holds
    /// at any spelling, because the obligation follows the declaration's own kind, so it is Content,
    /// and it is the form that carries a site ([`super::spellings_report::PlaceNames::subject_sites`]).
    pub(super) fn of_finding(finding: &Diagnostic) -> Option<Self> {
        if DIRECTION.contains(&finding.code) && !finding.sites.is_empty() {
            return Some(Self::Content);
        }
        Self::of(finding.code)
    }
}

/// The citation-direction codes, which alone speak of either a file or a declaration.
const DIRECTION: &[Code] = &[
    Code::DISCOURAGED_CITATION,
    Code::FORBIDDEN_CITATION,
    Code::MISSING_CITATION,
    Code::SUGGESTED_CITATION,
];

const SIDES: &[(Side, &[Code])] = &[
    (Side::Content, CONTENT),
    (Side::Place, PLACE),
    (Side::Judged, JUDGED),
    (Side::Run, RUN),
];

/// Read off the text at the line it names. `unreached-declaration` and `uncited-unit` stand at a
/// declaration and count what reaches it, `section-address-changed` at the heading holding the
/// address, and `missing-snapshot` at a citation with nothing to resolve, the twin of `dangling`:
/// no row of the configuration owns any of those answers, so a second name changes none of them.
const CONTENT: &[Code] = &[
    Code::CHAPTER_CARDINALITY,
    Code::CITATION_CARDINALITY,
    Code::CONFLICT_MARKER,
    Code::DANGLING,
    Code::DECLARATION_NEAR_MISS,
    Code::DUPLICATE,
    Code::DUPLICATE_SECTION,
    Code::ESCAPED_CITATION_RESOLVES,
    Code::FRONT_MATTER_HEADING,
    Code::GLOB_CITATION,
    Code::INLINE_CITATION_STYLE,
    Code::INVALID_FIELD_VALUE,
    Code::INVALID_VALUE_BINDING,
    Code::INVALID_VALUE_DECLARATION,
    Code::LOCAL_SECTION_CITATION,
    Code::MISPLACED_FIELD,
    Code::MISSING_FIELD,
    Code::MISSING_SECTION,
    Code::MISSING_SNAPSHOT,
    Code::MISTITLED_FIELD,
    Code::OFF_FORMAT_CITATION,
    Code::ORPHAN_SECTION,
    Code::OVERSIZED_LEAD,
    Code::RELATIVE_SECTION_PARENT,
    Code::SECTION_ADDRESS_CHANGED,
    Code::SECTION_HEADING_LEVEL,
    Code::SECTION_OUTSIDE_DECLARATION,
    Code::SHORTHAND_CITATION,
    Code::SHORTHAND_NUMERIC_RUN,
    Code::UNCITED_UNIT,
    Code::UNDECLARED_CHAPTER,
    Code::UNKNOWN_PROJECT,
    Code::UNMARKED_HEADING,
    Code::UNREACHED_DECLARATION,
    Code::UNREADABLE_FRONT_MATTER,
    Code::UNRECORDED_SECTION_CITATION,
    Code::UNUSED,
    Code::VALUE_MISMATCH,
];

/// Asked of the row that owns a spelling's path. `image-index-entry` never fires now and stands
/// with the index rules it was one of.
const PLACE: &[Code] = &[
    Code::DISCOURAGED_CITATION,
    Code::FORBIDDEN_CITATION,
    Code::IMAGE_INDEX_ENTRY,
    Code::MISPLACED_DECLARATION,
    Code::MISSING_CITATION,
    Code::MISSING_INDEX_ENTRY,
    Code::RULE_FOLDER_FILE_NOT_MARKDOWN,
    Code::SUGGESTED_CITATION,
    Code::UNGROUNDED,
    Code::UNLINKED_INDEX_ENTRY,
];

/// The spelling itself is the finding: a stub's written name, the read that failed, the names a
/// refusal or a move already lists, and the `outside [scan] include` twins, whose message is about
/// the name outside and which a plain run never reaches (§FS-check.1.3.2).
const JUDGED: &[Code] = &[
    Code::BROKEN_STUB,
    Code::EXCLUDED_STUB_TARGET,
    Code::IO,
    Code::UNPARSABLE_FILE,
    Code::OUT_OF_SCOPE_DANGLING,
    Code::OUT_OF_SCOPE_LOCAL_SECTION_CITATION,
    Code::OUT_OF_SCOPE_MISSING_SECTION,
    Code::OUT_OF_SCOPE_OFF_FORMAT_CITATION,
    Code::OUT_OF_SCOPE_SHORTHAND_CITATION,
    Code::OUT_OF_SCOPE_UNKNOWN_PROJECT,
    Code::READING_CONFLICT,
    Code::READING_MOVED,
    Code::STUB_SHAPE,
];

/// The configuration, its imports, and the run's own shape. `yaml-value-source-deferred` never
/// fires now and named no second spelling when it did.
const RUN: &[Code] = &[
    Code::AGENTS_INIT,
    Code::CONFIG_FETCH_NAMES_NOTHING,
    Code::CONFIG_INDEX_NOT_MARKDOWN,
    Code::CONFIG_MD_NOT_MARKDOWN,
    Code::CONFIG_MULTILINE_STRING,
    Code::CONFIG_NOT_TOML,
    Code::CONFIG_RULE_FILE_NOT_MARKDOWN,
    Code::CONFIG_RULE_FOLDER_NOT_MARKDOWN,
    Code::CONFIG_VALUE_HOME_ABSENT,
    Code::CONFIG_VERSION_1,
    Code::DEPRECATED_CONFIG_LOCATION,
    Code::EMPTY_CITATION_OBLIGATION,
    Code::EMPTY_SCAN,
    Code::FULL_SCOPE_IGNORED,
    Code::IMPORT_LIVE,
    Code::IMPORT_MISSING,
    Code::INVALID_RULE,
    Code::INVALID_SECTION_LOCK,
    Code::NOTHING_RECOGNIZED,
    Code::OPTIONAL_MEMBER_ABSENT,
    Code::REDUNDANT_CONFIG,
    Code::SELECTED_CHECK_OFF,
    Code::UNLISTED_WORKSPACE_BLOCK,
    Code::UNREAD_SCAN_ROOT,
    Code::YAML_VALUE_SOURCE_DEFERRED,
];

#[cfg(test)]
#[path = "tests_finding_sides.rs"]
mod tests;
