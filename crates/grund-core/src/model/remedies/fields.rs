//! The *Fix* fillers of the codes §FS-fields checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `front-matter-heading`'s *Fix* (§FS-fields.checks.front-matter-heading).
pub(crate) fn front_matter_heading() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::FRONT_MATTER_HEADING, 2, None, slots)
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

/// `misplaced-field`'s *Fix* (§FS-fields.checks.misplaced-field).
pub(crate) fn misplaced_field() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISPLACED_FIELD, 2, None, slots)
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

/// `mistitled-field`'s *Fix* (§FS-fields.checks.mistitled-field).
pub(crate) fn mistitled_field() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::MISTITLED_FIELD, 2, None, slots)
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
