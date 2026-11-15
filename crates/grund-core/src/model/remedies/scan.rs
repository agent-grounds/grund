//! The *Fix* fillers of the codes §FS-scan checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `reading-conflict`'s *Fix* (§FS-scan.checks.reading-conflict).
pub(crate) fn reading_conflict() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::READING_CONFLICT, 2, None, slots)
}

/// `reading-moved`'s *Fix* (§FS-scan.checks.reading-moved).
pub(crate) fn reading_moved() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::READING_MOVED, 2, None, slots)
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

/// `unread-scan-root`'s *Fix*, case `hidden-document` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_hidden_document(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("hidden-document"), slots)
}

/// `unread-scan-root`'s *Fix*, case `unmap` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_unmap(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("unmap"), slots)
}

/// `unread-scan-root`'s *Fix*, case `value` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_value(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("value"), slots)
}

/// `unread-scan-root`'s *Fix*, case `document` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_document(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("document"), slots)
}

/// `unread-scan-root`'s *Fix*, case `folder-add` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_add(version: u32, language: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("language", &language)];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-add"), slots)
}

/// `unread-scan-root`'s *Fix*, case `folder-add-document` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_add_document(version: u32, language: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("language", &language)];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-add-document"), slots)
}

/// `unread-scan-root`'s *Fix*, case `folder-language` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_language(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-language"), slots)
}

/// `unread-scan-root`'s *Fix*, case `folder-document` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_document(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-document"), slots)
}

/// `unread-scan-root`'s *Fix*, case `folder-value` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_value(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-value"), slots)
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

/// `unread-scan-root`'s *Fix*, case `folder-rename` (§FS-scan.checks.unread-scan-root).
pub(crate) fn unread_scan_root_folder_rename(version: u32) -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREAD_SCAN_ROOT, version, Some("folder-rename"), slots)
}
