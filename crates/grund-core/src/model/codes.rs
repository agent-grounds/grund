//! The `check` code handles, rendered from the §FS-errors.5.5 catalog by
//! `scripts/render_check_codes.py`: edit the catalog, then run the script.
//!
//! A finding is built only from a handle below, so a code the catalog does not
//! have cannot be reported (§AR-system.2.2). What each code means and what turns
//! it on is the rendered catalog's, read by `catalog.rs` (§AR-config.9.1).

/// One code of the §FS-errors.5.5 catalog. Its field is the model's own, so the
/// codes are the constants below and the run's three cautions that are not
/// check findings, which `report.rs` names beside the record that carries them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Code(pub(super) &'static str);

impl Code {
    /// §FS-check.3.5
    pub const AGENTS_INIT: Code = Code("agents-init");
    /// §FS-declarations.checks.broken-stub
    pub const BROKEN_STUB: Code = Code("broken-stub");
    /// §FS-check.3.26
    pub const CHAPTER_CARDINALITY: Code = Code("chapter-cardinality");
    /// §FS-check.3.27
    pub const CITATION_CARDINALITY: Code = Code("citation-cardinality");
    /// §FS-config-v2.checks.config-multiline-string
    pub const CONFIG_MULTILINE_STRING: Code = Code("config-multiline-string");
    /// §FS-config-v2.checks.config-not-toml
    pub const CONFIG_NOT_TOML: Code = Code("config-not-toml");
    /// §FS-check.checks.config-version-1
    pub const CONFIG_VERSION_1: Code = Code("config-version-1");
    /// §FS-declarations.checks.conflict-marker
    pub const CONFLICT_MARKER: Code = Code("conflict-marker");
    /// §FS-check.3.1
    pub const DANGLING: Code = Code("dangling");
    /// §FS-declarations.checks.declaration-near-miss
    pub const DECLARATION_NEAR_MISS: Code = Code("declaration-near-miss");
    /// §FS-check.4.11
    pub const DEPRECATED_CONFIG_LOCATION: Code = Code("deprecated-config-location");
    /// §FS-check.2.3
    pub const DISCOURAGED_CITATION: Code = Code("discouraged-citation");
    /// §FS-declarations.checks.duplicate
    pub const DUPLICATE: Code = Code("duplicate");
    /// §FS-declarations.checks.duplicate-section
    pub const DUPLICATE_SECTION: Code = Code("duplicate-section");
    /// §FS-check.2.2.1
    pub const EMPTY_CITATION_OBLIGATION: Code = Code("empty-citation-obligation");
    /// §FS-check.2.2
    pub const EMPTY_SCAN: Code = Code("empty-scan");
    /// §FS-check.2.3.1
    pub const ESCAPED_CITATION_RESOLVES: Code = Code("escaped-citation-resolves");
    /// §FS-declarations.checks.excluded-stub-target
    pub const EXCLUDED_STUB_TARGET: Code = Code("excluded-stub-target");
    /// §FS-check.3.12
    pub const FORBIDDEN_CITATION: Code = Code("forbidden-citation");
    /// §FS-fields.checks.front-matter-heading
    pub const FRONT_MATTER_HEADING: Code = Code("front-matter-heading");
    /// §FS-check.1.3.7
    pub const FULL_SCOPE_IGNORED: Code = Code("full-scope-ignored");
    /// §FS-check.checks.glob-citation
    pub const GLOB_CITATION: Code = Code("glob-citation");
    /// §FS-citations.checks.image-index-entry
    pub const IMAGE_INDEX_ENTRY: Code = Code("image-index-entry");
    /// §FS-imports.checks.import-live
    pub const IMPORT_LIVE: Code = Code("import-live");
    /// §FS-imports.checks.import-missing
    pub const IMPORT_MISSING: Code = Code("import-missing");
    /// §FS-check.3.10
    pub const INLINE_CITATION_STYLE: Code = Code("inline-citation-style");
    /// §FS-fields.checks.invalid-field-value
    pub const INVALID_FIELD_VALUE: Code = Code("invalid-field-value");
    /// §FS-check.3.25
    pub const INVALID_RULE: Code = Code("invalid-rule");
    /// §FS-section-lock.checks.invalid-section-lock
    pub const INVALID_SECTION_LOCK: Code = Code("invalid-section-lock");
    /// §FS-check.3.21
    pub const INVALID_VALUE_BINDING: Code = Code("invalid-value-binding");
    /// §FS-check.3.20
    pub const INVALID_VALUE_DECLARATION: Code = Code("invalid-value-declaration");
    /// §FS-check.2.4
    pub const IO: Code = Code("io");
    /// §FS-check.3.24
    pub const LOCAL_SECTION_CITATION: Code = Code("local-section-citation");
    /// §FS-declarations.checks.misplaced-declaration
    pub const MISPLACED_DECLARATION: Code = Code("misplaced-declaration");
    /// §FS-fields.checks.misplaced-field
    pub const MISPLACED_FIELD: Code = Code("misplaced-field");
    /// §FS-check.3.11
    pub const MISSING_CITATION: Code = Code("missing-citation");
    /// §FS-fields.checks.missing-field
    pub const MISSING_FIELD: Code = Code("missing-field");
    /// §FS-check.3.18
    pub const MISSING_INDEX_ENTRY: Code = Code("missing-index-entry");
    /// §FS-check.3.2
    pub const MISSING_SECTION: Code = Code("missing-section");
    /// §FS-check.4.12
    pub const MISSING_SNAPSHOT: Code = Code("missing-snapshot");
    /// §FS-fields.checks.mistitled-field
    pub const MISTITLED_FIELD: Code = Code("mistitled-field");
    /// §FS-check.4.5
    pub const NOTHING_RECOGNIZED: Code = Code("nothing-recognized");
    /// §FS-check.checks.off-format-citation
    pub const OFF_FORMAT_CITATION: Code = Code("off-format-citation");
    /// §FS-check.4.9
    pub const OPTIONAL_MEMBER_ABSENT: Code = Code("optional-member-absent");
    /// §FS-declarations.checks.orphan-section
    pub const ORPHAN_SECTION: Code = Code("orphan-section");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_DANGLING: Code = Code("out-of-scope-dangling");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_LOCAL_SECTION_CITATION: Code =
        Code("out-of-scope-local-section-citation");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_MISSING_SECTION: Code = Code("out-of-scope-missing-section");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_OFF_FORMAT_CITATION: Code = Code("out-of-scope-off-format-citation");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_SHORTHAND_CITATION: Code = Code("out-of-scope-shorthand-citation");
    /// §FS-check.3.14
    pub const OUT_OF_SCOPE_UNKNOWN_PROJECT: Code = Code("out-of-scope-unknown-project");
    /// §FS-declarations.checks.oversized-lead
    pub const OVERSIZED_LEAD: Code = Code("oversized-lead");
    /// §FS-check.4.3
    pub const REDUNDANT_CONFIG: Code = Code("redundant-config");
    /// §FS-declarations.checks.relative-section-parent
    pub const RELATIVE_SECTION_PARENT: Code = Code("relative-section-parent");
    /// §FS-section-lock.checks.section-address-changed
    pub const SECTION_ADDRESS_CHANGED: Code = Code("section-address-changed");
    /// §FS-declarations.checks.section-heading-level
    pub const SECTION_HEADING_LEVEL: Code = Code("section-heading-level");
    /// §FS-declarations.checks.section-outside-declaration
    pub const SECTION_OUTSIDE_DECLARATION: Code = Code("section-outside-declaration");
    /// §FS-check.1.4.1
    pub const SELECTED_CHECK_OFF: Code = Code("selected-check-off");
    /// §FS-check.3.13
    pub const SHORTHAND_CITATION: Code = Code("shorthand-citation");
    /// §FS-check.3.15
    pub const SHORTHAND_NUMERIC_RUN: Code = Code("shorthand-numeric-run");
    /// §FS-declarations.checks.stub-shape
    pub const STUB_SHAPE: Code = Code("stub-shape");
    /// §FS-check.2.3
    pub const SUGGESTED_CITATION: Code = Code("suggested-citation");
    /// §FS-check.3.28
    pub const UNCITED_UNIT: Code = Code("uncited-unit");
    /// §FS-fields.checks.undeclared-chapter
    pub const UNDECLARED_CHAPTER: Code = Code("undeclared-chapter");
    /// §FS-check.3.6
    pub const UNGROUNDED: Code = Code("ungrounded");
    /// §FS-check.3.8
    pub const UNKNOWN_PROJECT: Code = Code("unknown-project");
    /// §FS-check.3.17
    pub const UNLINKED_INDEX_ENTRY: Code = Code("unlinked-index-entry");
    /// §FS-check.3.29
    pub const UNLISTED_WORKSPACE_BLOCK: Code = Code("unlisted-workspace-block");
    /// §FS-declarations.checks.unmarked-heading
    pub const UNMARKED_HEADING: Code = Code("unmarked-heading");
    /// §FS-rules.checks.unreached-declaration
    pub const UNREACHED_DECLARATION: Code = Code("unreached-declaration");
    /// §FS-scan.checks.unread-scan-root
    pub const UNREAD_SCAN_ROOT: Code = Code("unread-scan-root");
    /// §FS-fields.checks.unreadable-front-matter
    pub const UNREADABLE_FRONT_MATTER: Code = Code("unreadable-front-matter");
    /// §FS-section-lock.checks.unrecorded-section-citation
    pub const UNRECORDED_SECTION_CITATION: Code = Code("unrecorded-section-citation");
    /// §FS-check.4.1
    pub const UNUSED: Code = Code("unused");
    /// §FS-check.3.22
    pub const VALUE_MISMATCH: Code = Code("value-mismatch");
    /// §FS-values.2.2.4
    pub const YAML_VALUE_SOURCE_DEFERRED: Code = Code("yaml-value-source-deferred");

    /// Every selectable code, sorted: the selector vocabulary of §FS-check.1.
    pub const ALL: &'static [Code] = &[
        Code::AGENTS_INIT,
        Code::BROKEN_STUB,
        Code::CHAPTER_CARDINALITY,
        Code::CITATION_CARDINALITY,
        Code::CONFIG_MULTILINE_STRING,
        Code::CONFIG_NOT_TOML,
        Code::CONFIG_VERSION_1,
        Code::CONFLICT_MARKER,
        Code::DANGLING,
        Code::DECLARATION_NEAR_MISS,
        Code::DEPRECATED_CONFIG_LOCATION,
        Code::DISCOURAGED_CITATION,
        Code::DUPLICATE,
        Code::DUPLICATE_SECTION,
        Code::EMPTY_CITATION_OBLIGATION,
        Code::EMPTY_SCAN,
        Code::ESCAPED_CITATION_RESOLVES,
        Code::EXCLUDED_STUB_TARGET,
        Code::FORBIDDEN_CITATION,
        Code::FRONT_MATTER_HEADING,
        Code::FULL_SCOPE_IGNORED,
        Code::GLOB_CITATION,
        Code::IMAGE_INDEX_ENTRY,
        Code::IMPORT_LIVE,
        Code::IMPORT_MISSING,
        Code::INLINE_CITATION_STYLE,
        Code::INVALID_FIELD_VALUE,
        Code::INVALID_RULE,
        Code::INVALID_SECTION_LOCK,
        Code::INVALID_VALUE_BINDING,
        Code::INVALID_VALUE_DECLARATION,
        Code::IO,
        Code::LOCAL_SECTION_CITATION,
        Code::MISPLACED_DECLARATION,
        Code::MISPLACED_FIELD,
        Code::MISSING_CITATION,
        Code::MISSING_FIELD,
        Code::MISSING_INDEX_ENTRY,
        Code::MISSING_SECTION,
        Code::MISSING_SNAPSHOT,
        Code::MISTITLED_FIELD,
        Code::NOTHING_RECOGNIZED,
        Code::OFF_FORMAT_CITATION,
        Code::OPTIONAL_MEMBER_ABSENT,
        Code::ORPHAN_SECTION,
        Code::OUT_OF_SCOPE_DANGLING,
        Code::OUT_OF_SCOPE_LOCAL_SECTION_CITATION,
        Code::OUT_OF_SCOPE_MISSING_SECTION,
        Code::OUT_OF_SCOPE_OFF_FORMAT_CITATION,
        Code::OUT_OF_SCOPE_SHORTHAND_CITATION,
        Code::OUT_OF_SCOPE_UNKNOWN_PROJECT,
        Code::OVERSIZED_LEAD,
        Code::REDUNDANT_CONFIG,
        Code::RELATIVE_SECTION_PARENT,
        Code::SECTION_ADDRESS_CHANGED,
        Code::SECTION_HEADING_LEVEL,
        Code::SECTION_OUTSIDE_DECLARATION,
        Code::SELECTED_CHECK_OFF,
        Code::SHORTHAND_CITATION,
        Code::SHORTHAND_NUMERIC_RUN,
        Code::STUB_SHAPE,
        Code::SUGGESTED_CITATION,
        Code::UNCITED_UNIT,
        Code::UNDECLARED_CHAPTER,
        Code::UNGROUNDED,
        Code::UNKNOWN_PROJECT,
        Code::UNLINKED_INDEX_ENTRY,
        Code::UNLISTED_WORKSPACE_BLOCK,
        Code::UNMARKED_HEADING,
        Code::UNREACHED_DECLARATION,
        Code::UNREAD_SCAN_ROOT,
        Code::UNREADABLE_FRONT_MATTER,
        Code::UNRECORDED_SECTION_CITATION,
        Code::UNUSED,
        Code::VALUE_MISMATCH,
        Code::YAML_VALUE_SOURCE_DEFERRED,
    ];

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// The handle a selector or a query spells, or `None` for a code the catalog
    /// does not have.
    pub fn named(code: &str) -> Option<Code> {
        Code::ALL
            .binary_search_by(|known| known.0.cmp(code))
            .ok()
            .map(|index| Code::ALL[index])
    }

    /// The code a citation outside the default scope reports in place of this one
    /// under `--full`, where it has one (§FS-check.3.14).
    pub fn out_of_scope(self) -> Option<Code> {
        match self.0 {
            "dangling" => Some(Code::OUT_OF_SCOPE_DANGLING),
            "local-section-citation" => Some(Code::OUT_OF_SCOPE_LOCAL_SECTION_CITATION),
            "missing-section" => Some(Code::OUT_OF_SCOPE_MISSING_SECTION),
            "off-format-citation" => Some(Code::OUT_OF_SCOPE_OFF_FORMAT_CITATION),
            "shorthand-citation" => Some(Code::OUT_OF_SCOPE_SHORTHAND_CITATION),
            "unknown-project" => Some(Code::OUT_OF_SCOPE_UNKNOWN_PROJECT),
            _ => None,
        }
    }
}

/// A code reads as the name it is printed by, so a finding can be asked about by
/// name; only a constant above makes one.
impl PartialEq<&str> for Code {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
