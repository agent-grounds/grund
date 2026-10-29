//! The *Fix* fillers of the codes §FS-scan checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

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
