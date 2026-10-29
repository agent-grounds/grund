//! The *Fix* fillers of the codes §FS-declarations checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

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

/// `conflict-marker`'s *Fix* (§FS-declarations.checks.conflict-marker).
pub(crate) fn conflict_marker() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFLICT_MARKER, 2, None, slots)
}

/// `declaration-near-miss`'s *Fix* (§FS-declarations.checks.declaration-near-miss).
pub(crate) fn declaration_near_miss() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::DECLARATION_NEAR_MISS, 2, None, slots)
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

/// `relative-section-parent`'s *Fix* (§FS-declarations.checks.relative-section-parent).
pub(crate) fn relative_section_parent() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::RELATIVE_SECTION_PARENT, 2, None, slots)
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

/// `unmarked-heading`'s *Fix* (§FS-declarations.checks.unmarked-heading).
pub(crate) fn unmarked_heading(heading: impl Display, address: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("heading", &heading), ("address", &address)];
    fill(Code::UNMARKED_HEADING, 2, None, slots)
}
