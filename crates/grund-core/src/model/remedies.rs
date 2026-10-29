//! The *Fix* fillers, rendered from the §FS-errors.5.5 catalog by
//! `scripts/render_check_codes.py`: edit the catalog, then run the script.
//!
//! A message's remedy is built only by its code's filler below, which fills the
//! row's template from the embedded catalog, so no remedy is worded in the
//! checker and a misspelt slot or case does not compile (§AR-config.9.1).

use std::fmt::Display;

use super::catalog::fill;
use super::codes::Code;

/// `agents-init`'s *Fix*, case `outdated` (§FS-check.3.5).
pub(crate) fn agents_init_outdated(version: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("version", &version)];
    fill(Code::AGENTS_INIT, 2, Some("outdated"), slots)
}

/// `agents-init`'s *Fix*, case `missing` (§FS-check.3.5).
pub(crate) fn agents_init_missing() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::AGENTS_INIT, 2, Some("missing"), slots)
}

/// `agents-init`'s *Fix*, case `refresh` (§FS-check.3.5).
pub(crate) fn agents_init_refresh() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::AGENTS_INIT, 2, Some("refresh"), slots)
}

/// `agents-init`'s *Fix*, case `refresh-in` (§FS-check.3.5).
pub(crate) fn agents_init_refresh_in(dir: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("dir", &dir)];
    fill(Code::AGENTS_INIT, 2, Some("refresh-in"), slots)
}

/// `agents-init`'s *Fix*, case `elsewhere` (§FS-check.3.5).
pub(crate) fn agents_init_elsewhere(dir: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("dir", &dir)];
    fill(Code::AGENTS_INIT, 2, Some("elsewhere"), slots)
}

/// `broken-stub`'s *Fix*, case `one-home` (§FS-declarations.checks.broken-stub).
pub(crate) fn broken_stub_one_home(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::BROKEN_STUB, version, Some("one-home"), slots)
}

/// `broken-stub`'s *Fix*, case `excluded-pattern` (§FS-declarations.checks.broken-stub).
pub(crate) fn broken_stub_excluded_pattern(version: u32, pattern: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("pattern", &pattern)];
    fill(Code::BROKEN_STUB, version, Some("excluded-pattern"), slots)
}

/// `broken-stub`'s *Fix*, case `ignored` (§FS-declarations.checks.broken-stub).
pub(crate) fn broken_stub_ignored(version: u32, file: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("file", &file)];
    fill(Code::BROKEN_STUB, version, Some("ignored"), slots)
}

/// `broken-stub`'s *Fix*, case `excluded-entry` (§FS-declarations.checks.broken-stub).
pub(crate) fn broken_stub_excluded_entry(version: u32, entry: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("entry", &entry)];
    fill(Code::BROKEN_STUB, version, Some("excluded-entry"), slots)
}

/// `chapter-cardinality`'s *Fix* (§FS-check.3.26).
pub(crate) fn chapter_cardinality() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CHAPTER_CARDINALITY, 2, None, slots)
}

/// `citation-cardinality`'s *Fix* (§FS-check.3.27).
pub(crate) fn citation_cardinality() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CITATION_CARDINALITY, 2, None, slots)
}

/// `config-multiline-string`'s *Fix* (§FS-config-v2.checks.config-multiline-string).
pub(crate) fn config_multiline_string(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_MULTILINE_STRING, 2, None, slots)
}

/// `config-not-toml`'s *Fix* (§FS-config-v2.checks.config-not-toml).
pub(crate) fn config_not_toml(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_NOT_TOML, 2, None, slots)
}

/// `config-version-1`'s *Fix* (§FS-check.checks.config-version-1).
pub(crate) fn config_version_1(files: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("files", &files)];
    fill(Code::CONFIG_VERSION_1, 2, None, slots)
}

/// `conflict-marker`'s *Fix* (§FS-declarations.checks.conflict-marker).
pub(crate) fn conflict_marker() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFLICT_MARKER, 2, None, slots)
}

/// `dangling`'s *Fix*, case `near` (§FS-check.3.1).
pub(crate) fn dangling_near(near: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("near", &near)];
    fill(Code::DANGLING, 2, Some("near"), slots)
}

/// `dangling`'s *Fix*, case `near-escape` (§FS-check.3.1).
pub(crate) fn dangling_near_escape(near: impl Display, escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("near", &near), ("escape", &escape)];
    fill(Code::DANGLING, 2, Some("near-escape"), slots)
}

/// `dangling`'s *Fix*, case `escape` (§FS-check.3.1).
pub(crate) fn dangling_escape(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::DANGLING, 2, Some("escape"), slots)
}

/// `dangling`'s *Fix*, case `fetch` (§FS-check.3.1).
pub(crate) fn dangling_fetch(id: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("id", &id)];
    fill(Code::DANGLING, 2, Some("fetch"), slots)
}

/// `declaration-near-miss`'s *Fix* (§FS-declarations.checks.declaration-near-miss).
pub(crate) fn declaration_near_miss() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DECLARATION_NEAR_MISS, 2, None, slots)
}

/// `deprecated-config-location`'s *Fix* (§FS-check.4.11).
pub(crate) fn deprecated_config_location(home: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("home", &home)];
    fill(Code::DEPRECATED_CONFIG_LOCATION, 2, None, slots)
}

/// `discouraged-citation`'s *Fix* (§FS-check.2.3).
pub(crate) fn discouraged_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DISCOURAGED_CITATION, 2, None, slots)
}

/// `duplicate`'s *Fix*, case `home` (§FS-declarations.checks.duplicate).
pub(crate) fn duplicate_home(site: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("site", &site)];
    fill(Code::DUPLICATE, 2, Some("home"), slots)
}

/// `duplicate`'s *Fix*, case `there` (§FS-declarations.checks.duplicate).
pub(crate) fn duplicate_there() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DUPLICATE, 2, Some("there"), slots)
}

/// `duplicate-section`'s *Fix*, case `headings` (§FS-declarations.checks.duplicate-section).
pub(crate) fn duplicate_section_headings() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DUPLICATE_SECTION, 2, Some("headings"), slots)
}

/// `duplicate-section`'s *Fix*, case `keys` (§FS-declarations.checks.duplicate-section).
pub(crate) fn duplicate_section_keys() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DUPLICATE_SECTION, 2, Some("keys"), slots)
}

/// `empty-citation-obligation`'s *Fix*, case `citable` (§FS-check.2.2.1).
pub(crate) fn empty_citation_obligation_citable(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::EMPTY_CITATION_OBLIGATION, version, Some("citable"), slots)
}

/// `empty-citation-obligation`'s *Fix*, case `grounding` (§FS-check.2.2.1).
pub(crate) fn empty_citation_obligation_grounding(version: u32, kind: impl Display, place: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("kind", &kind), ("place", &place)];
    fill(Code::EMPTY_CITATION_OBLIGATION, version, Some("grounding"), slots)
}

/// `empty-scan`'s *Fix*, case `hidden` (§FS-check.2.2).
pub(crate) fn empty_scan_hidden(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::EMPTY_SCAN, version, Some("hidden"), slots)
}

/// `empty-scan`'s *Fix*, case `no-kinds` (§FS-check.2.2).
pub(crate) fn empty_scan_no_kinds(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::EMPTY_SCAN, version, Some("no-kinds"), slots)
}

/// `empty-scan`'s *Fix*, case `root` (§FS-check.2.2).
pub(crate) fn empty_scan_root(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::EMPTY_SCAN, version, Some("root"), slots)
}

/// `escaped-citation-resolves`'s *Fix* (§FS-check.2.3.1).
pub(crate) fn escaped_citation_resolves(citation: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("citation", &citation)];
    fill(Code::ESCAPED_CITATION_RESOLVES, 2, None, slots)
}

/// `forbidden-citation`'s *Fix*, case `re-point` (§FS-check.3.12).
pub(crate) fn forbidden_citation_re_point() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::FORBIDDEN_CITATION, 2, Some("re-point"), slots)
}

/// `forbidden-citation`'s *Fix*, case `index-rule` (§FS-check.3.12).
pub(crate) fn forbidden_citation_index_rule(rules: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("rules", &rules)];
    fill(Code::FORBIDDEN_CITATION, 2, Some("index-rule"), slots)
}

/// `forbidden-citation`'s *Fix*, case `index-may` (§FS-check.3.12).
pub(crate) fn forbidden_citation_index_may(kind: impl Display, table: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("kind", &kind), ("table", &table)];
    fill(Code::FORBIDDEN_CITATION, 2, Some("index-may"), slots)
}

/// `forbidden-citation`'s *Fix*, case `index-prohibition` (§FS-check.3.12).
pub(crate) fn forbidden_citation_index_prohibition(level: impl Display, table: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("level", &level), ("table", &table)];
    fill(Code::FORBIDDEN_CITATION, 2, Some("index-prohibition"), slots)
}

/// `front-matter-heading`'s *Fix* (§FS-fields.checks.front-matter-heading).
pub(crate) fn front_matter_heading() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::FRONT_MATTER_HEADING, 2, None, slots)
}

/// `full-scope-ignored`'s *Fix* (§FS-check.1.3.7).
pub(crate) fn full_scope_ignored() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::FULL_SCOPE_IGNORED, 2, None, slots)
}

/// `glob-citation`'s *Fix* (§FS-check.checks.glob-citation).
pub(crate) fn glob_citation(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::GLOB_CITATION, 2, None, slots)
}

/// `inline-citation-style`'s *Fix*, case `prose` (§FS-check.3.10).
pub(crate) fn inline_citation_style_prose() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INLINE_CITATION_STYLE, 2, Some("prose"), slots)
}

/// `inline-citation-style`'s *Fix*, case `lines` (§FS-check.3.10).
pub(crate) fn inline_citation_style_lines() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INLINE_CITATION_STYLE, 2, Some("lines"), slots)
}

/// `inline-citation-style`'s *Fix*, case `columns` (§FS-check.3.10).
pub(crate) fn inline_citation_style_columns() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INLINE_CITATION_STYLE, 2, Some("columns"), slots)
}

/// `inline-citation-style`'s *Fix*, case `layout` (§FS-check.3.10).
pub(crate) fn inline_citation_style_layout() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INLINE_CITATION_STYLE, 2, Some("layout"), slots)
}

/// `invalid-field-value`'s *Fix*, case `heading` (§FS-fields.checks.invalid-field-value).
pub(crate) fn invalid_field_value_heading() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_FIELD_VALUE, 2, Some("heading"), slots)
}

/// `invalid-field-value`'s *Fix*, case `value` (§FS-fields.checks.invalid-field-value).
pub(crate) fn invalid_field_value_value() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_FIELD_VALUE, 2, Some("value"), slots)
}

/// `invalid-field-value`'s *Fix*, case `one-line` (§FS-fields.checks.invalid-field-value).
pub(crate) fn invalid_field_value_one_line() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_FIELD_VALUE, 2, Some("one-line"), slots)
}

/// `invalid-rule`'s *Fix*, case `form` (§FS-check.3.25).
pub(crate) fn invalid_rule_form(form: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("form", &form)];
    fill(Code::INVALID_RULE, 2, Some("form"), slots)
}

/// `invalid-rule`'s *Fix*, case `forms` (§FS-check.3.25).
pub(crate) fn invalid_rule_forms(forms: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("forms", &forms)];
    fill(Code::INVALID_RULE, 2, Some("forms"), slots)
}

/// `invalid-rule`'s *Fix*, case `form-enabled` (§FS-check.3.25).
pub(crate) fn invalid_rule_form_enabled(form: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("form", &form)];
    fill(Code::INVALID_RULE, 2, Some("form-enabled"), slots)
}

/// `invalid-rule`'s *Fix*, case `forms-enabled` (§FS-check.3.25).
pub(crate) fn invalid_rule_forms_enabled(forms: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("forms", &forms)];
    fill(Code::INVALID_RULE, 2, Some("forms-enabled"), slots)
}

/// `invalid-rule`'s *Fix*, case `place` (§FS-check.3.25).
pub(crate) fn invalid_rule_place(table: impl Display, target: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("table", &table), ("target", &target)];
    fill(Code::INVALID_RULE, 2, Some("place"), slots)
}

/// `invalid-rule`'s *Fix*, case `place-grounding` (§FS-check.3.25).
pub(crate) fn invalid_rule_place_grounding(table: impl Display, target: impl Display, setting: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("table", &table), ("target", &target), ("setting", &setting)];
    fill(Code::INVALID_RULE, 2, Some("place-grounding"), slots)
}

/// `invalid-rule`'s *Fix*, case `rename` (§FS-check.3.25).
pub(crate) fn invalid_rule_rename() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_RULE, 2, Some("rename"), slots)
}

/// `invalid-rule`'s *Fix*, case `workspace` (§FS-check.3.25).
pub(crate) fn invalid_rule_workspace() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_RULE, 2, Some("workspace"), slots)
}

/// `invalid-section-lock`'s *Fix* (§FS-section-lock.checks.invalid-section-lock).
pub(crate) fn invalid_section_lock() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_SECTION_LOCK, 2, None, slots)
}

/// `invalid-value-binding`'s *Fix* (§FS-check.3.21).
pub(crate) fn invalid_value_binding() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_VALUE_BINDING, 2, None, slots)
}

/// `invalid-value-declaration`'s *Fix* (§FS-check.3.20).
pub(crate) fn invalid_value_declaration() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_VALUE_DECLARATION, 2, None, slots)
}

/// `io`'s *Fix* (§FS-check.2.4).
pub(crate) fn io() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::IO, 2, None, slots)
}

/// `local-section-citation`'s *Fix*, case `write` (§FS-check.3.24).
pub(crate) fn local_section_citation_write(citation: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("citation", &citation)];
    fill(Code::LOCAL_SECTION_CITATION, 2, Some("write"), slots)
}

/// `local-section-citation`'s *Fix*, case `escape` (§FS-check.3.24).
pub(crate) fn local_section_citation_escape(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::LOCAL_SECTION_CITATION, 2, Some("escape"), slots)
}

/// `local-section-citation`'s *Fix*, case `fmt` (§FS-check.3.24).
pub(crate) fn local_section_citation_fmt() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::LOCAL_SECTION_CITATION, 2, Some("fmt"), slots)
}

/// `misplaced-declaration`'s *Fix*, case `unlisted` (§FS-declarations.checks.misplaced-declaration).
pub(crate) fn misplaced_declaration_unlisted(version: u32, entry: impl Display, enclosing: impl Display, component: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("entry", &entry), ("enclosing", &enclosing), ("component", &component)];
    fill(Code::MISPLACED_DECLARATION, version, Some("unlisted"), slots)
}

/// `misplaced-declaration`'s *Fix*, case `roots` (§FS-declarations.checks.misplaced-declaration).
pub(crate) fn misplaced_declaration_roots(version: u32, entry: impl Display, enclosing: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("entry", &entry), ("enclosing", &enclosing)];
    fill(Code::MISPLACED_DECLARATION, version, Some("roots"), slots)
}

/// `misplaced-field`'s *Fix* (§FS-fields.checks.misplaced-field).
pub(crate) fn misplaced_field() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISPLACED_FIELD, 2, None, slots)
}

/// `missing-citation`'s *Fix* (§FS-check.3.11).
pub(crate) fn missing_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_CITATION, 2, None, slots)
}

/// `missing-field`'s *Fix*, case `chapter` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_chapter(field: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("field", &field)];
    fill(Code::MISSING_FIELD, 2, Some("chapter"), slots)
}

/// `missing-field`'s *Fix*, case `block` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_block() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_FIELD, 2, Some("block"), slots)
}

/// `missing-field`'s *Fix*, case `not-first` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_not_first() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_FIELD, 2, Some("not-first"), slots)
}

/// `missing-field`'s *Fix*, case `doc-comment` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_doc_comment() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_FIELD, 2, Some("doc-comment"), slots)
}

/// `missing-field`'s *Fix*, case `byte-order-mark` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_byte_order_mark(delimiter: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("delimiter", &delimiter)];
    fill(Code::MISSING_FIELD, 2, Some("byte-order-mark"), slots)
}

/// `missing-field`'s *Fix*, case `key` (§FS-fields.checks.missing-field).
pub(crate) fn missing_field_key() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_FIELD, 2, Some("key"), slots)
}

/// `missing-index-entry`'s *Fix* (§FS-check.3.18).
pub(crate) fn missing_index_entry(entry: impl Display, index: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("entry", &entry), ("index", &index)];
    fill(Code::MISSING_INDEX_ENTRY, 2, None, slots)
}

/// `missing-section`'s *Fix* (§FS-check.3.2).
pub(crate) fn missing_section(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::MISSING_SECTION, 2, None, slots)
}

/// `missing-snapshot`'s *Fix*, case `near` (§FS-check.4.12).
pub(crate) fn missing_snapshot_near(near: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("near", &near)];
    fill(Code::MISSING_SNAPSHOT, 2, Some("near"), slots)
}

/// `missing-snapshot`'s *Fix*, case `near-escape` (§FS-check.4.12).
pub(crate) fn missing_snapshot_near_escape(near: impl Display, escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("near", &near), ("escape", &escape)];
    fill(Code::MISSING_SNAPSHOT, 2, Some("near-escape"), slots)
}

/// `missing-snapshot`'s *Fix*, case `escape` (§FS-check.4.12).
pub(crate) fn missing_snapshot_escape(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::MISSING_SNAPSHOT, 2, Some("escape"), slots)
}

/// `missing-snapshot`'s *Fix*, case `fetch` (§FS-check.4.12).
pub(crate) fn missing_snapshot_fetch(id: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("id", &id)];
    fill(Code::MISSING_SNAPSHOT, 2, Some("fetch"), slots)
}

/// `mistitled-field`'s *Fix* (§FS-fields.checks.mistitled-field).
pub(crate) fn mistitled_field() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISTITLED_FIELD, 2, None, slots)
}

/// `nothing-recognized`'s *Fix* (§FS-check.4.5).
pub(crate) fn nothing_recognized() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::NOTHING_RECOGNIZED, 2, None, slots)
}

/// `orphan-section`'s *Fix* (§FS-declarations.checks.orphan-section).
pub(crate) fn orphan_section() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::ORPHAN_SECTION, 2, None, slots)
}

/// `oversized-lead`'s *Fix* (§FS-declarations.checks.oversized-lead).
pub(crate) fn oversized_lead(coordinate: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("coordinate", &coordinate)];
    fill(Code::OVERSIZED_LEAD, 2, None, slots)
}

/// `redundant-config`'s *Fix* (§FS-check.4.3).
pub(crate) fn redundant_config() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::REDUNDANT_CONFIG, 2, None, slots)
}

/// `relative-section-parent`'s *Fix* (§FS-declarations.checks.relative-section-parent).
pub(crate) fn relative_section_parent() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::RELATIVE_SECTION_PARENT, 2, None, slots)
}

/// `section-address-changed`'s *Fix* (§FS-section-lock.checks.section-address-changed).
pub(crate) fn section_address_changed(new_address: impl Display, cited_address: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("new address", &new_address), ("cited address", &cited_address)];
    fill(Code::SECTION_ADDRESS_CHANGED, 2, None, slots)
}

/// `section-heading-level`'s *Fix* (§FS-declarations.checks.section-heading-level).
pub(crate) fn section_heading_level() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::SECTION_HEADING_LEVEL, 2, None, slots)
}

/// `section-outside-declaration`'s *Fix* (§FS-declarations.checks.section-outside-declaration).
pub(crate) fn section_outside_declaration() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::SECTION_OUTSIDE_DECLARATION, 2, None, slots)
}

/// `selected-check-off`'s *Fix*, case `off` (§FS-check.1.4.1).
pub(crate) fn selected_check_off_off(switches: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("switches", &switches)];
    fill(Code::SELECTED_CHECK_OFF, 2, Some("off"), slots)
}

/// `selected-check-off`'s *Fix*, case `migrate` (§FS-check.1.4.1).
pub(crate) fn selected_check_off_migrate(files: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("files", &files)];
    fill(Code::SELECTED_CHECK_OFF, 2, Some("migrate"), slots)
}

/// `shorthand-citation`'s *Fix*, case `write` (§FS-check.3.13).
pub(crate) fn shorthand_citation_write(citation: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("citation", &citation)];
    fill(Code::SHORTHAND_CITATION, 2, Some("write"), slots)
}

/// `shorthand-citation`'s *Fix*, case `full` (§FS-check.3.13).
pub(crate) fn shorthand_citation_full() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::SHORTHAND_CITATION, 2, Some("full"), slots)
}

/// `shorthand-numeric-run`'s *Fix* (§FS-check.3.15).
pub(crate) fn shorthand_numeric_run(citation: impl Display, escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("citation", &citation), ("escape", &escape)];
    fill(Code::SHORTHAND_NUMERIC_RUN, 2, None, slots)
}

/// `suggested-citation`'s *Fix* (§FS-check.2.3).
pub(crate) fn suggested_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::SUGGESTED_CITATION, 2, None, slots)
}

/// `uncited-unit`'s *Fix* (§FS-check.3.28).
pub(crate) fn uncited_unit() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNCITED_UNIT, 2, None, slots)
}

/// `undeclared-chapter`'s *Fix*, case `chapter` (§FS-fields.checks.undeclared-chapter).
pub(crate) fn undeclared_chapter_chapter() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNDECLARED_CHAPTER, 2, Some("chapter"), slots)
}

/// `undeclared-chapter`'s *Fix*, case `key` (§FS-fields.checks.undeclared-chapter).
pub(crate) fn undeclared_chapter_key() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNDECLARED_CHAPTER, 2, Some("key"), slots)
}

/// `undeclared-chapter`'s *Fix*, case `undeclarable-key` (§FS-fields.checks.undeclared-chapter).
pub(crate) fn undeclared_chapter_undeclarable_key() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNDECLARED_CHAPTER, 2, Some("undeclarable-key"), slots)
}

/// `ungrounded`'s *Fix* (§FS-check.3.6).
pub(crate) fn ungrounded() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNGROUNDED, 2, None, slots)
}

/// `unknown-project`'s *Fix*, case `root` (§FS-check.3.8).
pub(crate) fn unknown_project_root(root: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("root", &root)];
    fill(Code::UNKNOWN_PROJECT, 2, Some("root"), slots)
}

/// `unknown-project`'s *Fix*, case `subtree` (§FS-check.3.8).
pub(crate) fn unknown_project_subtree() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNKNOWN_PROJECT, 2, Some("subtree"), slots)
}

/// `unknown-project`'s *Fix*, case `near` (§FS-check.3.8).
pub(crate) fn unknown_project_near(near: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("near", &near)];
    fill(Code::UNKNOWN_PROJECT, 2, Some("near"), slots)
}

/// `unlinked-index-entry`'s *Fix* (§FS-check.3.17).
pub(crate) fn unlinked_index_entry() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNLINKED_INDEX_ENTRY, 2, None, slots)
}

/// `unlisted-workspace-block`'s *Fix* (§FS-check.3.29).
pub(crate) fn unlisted_workspace_block(entry: impl Display, enclosing: impl Display, scan: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("entry", &entry), ("enclosing", &enclosing), ("scan", &scan)];
    fill(Code::UNLISTED_WORKSPACE_BLOCK, 2, None, slots)
}

/// `unmarked-heading`'s *Fix* (§FS-declarations.checks.unmarked-heading).
pub(crate) fn unmarked_heading(heading: impl Display, address: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("heading", &heading), ("address", &address)];
    fill(Code::UNMARKED_HEADING, 2, None, slots)
}

/// `unreached-declaration`'s *Fix* (§FS-rules.checks.unreached-declaration).
pub(crate) fn unreached_declaration() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREACHED_DECLARATION, 2, None, slots)
}

/// `unread-scan-root`'s *Fix*, case `language` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_language(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("language"), slots)
}

/// `unread-scan-root`'s *Fix*, case `hidden` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_hidden(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("hidden"), slots)
}

/// `unread-scan-root`'s *Fix*, case `hidden-language` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_hidden_language(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("hidden-language"), slots)
}

/// `unread-scan-root`'s *Fix*, case `drop` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_drop(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("drop"), slots)
}

/// `unread-scan-root`'s *Fix*, case `place` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_place(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("place"), slots)
}

/// `unread-scan-root`'s *Fix*, case `add` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_add(version: u32, ext: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("ext", &ext)];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("add"), slots)
}

/// `unread-scan-root`'s *Fix*, case `rename` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_rename(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("rename"), slots)
}

/// `unread-scan-root`'s *Fix*, case `give` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_give(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("give"), slots)
}

/// `unread-scan-root`'s *Fix*, case `hidden-list` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_hidden_list(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("hidden-list"), slots)
}

/// `unreadable-front-matter`'s *Fix*, case `yaml` (§FS-fields.checks.unreadable-front-matter).
pub(crate) fn unreadable_front_matter_yaml() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREADABLE_FRONT_MATTER, 2, Some("yaml"), slots)
}

/// `unreadable-front-matter`'s *Fix*, case `toml` (§FS-fields.checks.unreadable-front-matter).
pub(crate) fn unreadable_front_matter_toml() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREADABLE_FRONT_MATTER, 2, Some("toml"), slots)
}

/// `unrecorded-section-citation`'s *Fix* (§FS-section-lock.checks.unrecorded-section-citation).
pub(crate) fn unrecorded_section_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNRECORDED_SECTION_CITATION, 2, None, slots)
}

/// `unused`'s *Fix* (§FS-check.4.1).
pub(crate) fn unused() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNUSED, 2, None, slots)
}

/// `value-mismatch`'s *Fix* (§FS-check.3.22).
pub(crate) fn value_mismatch() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::VALUE_MISMATCH, 2, None, slots)
}

/// `yaml-value-source-deferred`'s *Fix*, case `wrapped` (§FS-values.2.2.4).
pub(crate) fn yaml_value_source_deferred_wrapped(fix: impl Display, file: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("fix", &fix), ("file", &file)];
    fill(Code::YAML_VALUE_SOURCE_DEFERRED, 2, Some("wrapped"), slots)
}

/// `yaml-value-source-deferred`'s *Fix*, case `move` (§FS-values.2.2.4).
pub(crate) fn yaml_value_source_deferred_move(file: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("file", &file)];
    fill(Code::YAML_VALUE_SOURCE_DEFERRED, 2, Some("move"), slots)
}
