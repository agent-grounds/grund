//! The *Fix* fillers of the codes §FS-check checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

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

/// `config-version-1`'s *Fix* (§FS-check.checks.config-version-1).
pub(crate) fn config_version_1(files: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("files", &files)];
    fill(Code::CONFIG_VERSION_1, 2, None, slots)
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

/// `dangling`'s *Fix*, case `pattern` (§FS-check.3.1).
pub(crate) fn dangling_pattern() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DANGLING, 2, Some("pattern"), slots)
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

/// `missing-citation`'s *Fix* (§FS-check.3.11).
pub(crate) fn missing_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISSING_CITATION, 2, None, slots)
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

/// `nothing-recognized`'s *Fix* (§FS-check.4.5).
pub(crate) fn nothing_recognized() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::NOTHING_RECOGNIZED, 2, None, slots)
}

/// `off-format-citation`'s *Fix* (§FS-check.checks.off-format-citation).
pub(crate) fn off_format_citation(escape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("escape", &escape)];
    fill(Code::OFF_FORMAT_CITATION, 2, None, slots)
}

/// `redundant-config`'s *Fix* (§FS-check.4.3).
pub(crate) fn redundant_config() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::REDUNDANT_CONFIG, 2, None, slots)
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
