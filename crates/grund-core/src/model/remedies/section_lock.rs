//! The *Fix* fillers of the codes §FS-section-lock checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `invalid-section-lock`'s *Fix* (§FS-section-lock.checks.invalid-section-lock).
pub(crate) fn invalid_section_lock() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::INVALID_SECTION_LOCK, 2, None, slots)
}

/// `section-address-changed`'s *Fix* (§FS-section-lock.checks.section-address-changed).
pub(crate) fn section_address_changed(new_address: impl Display, cited_address: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("new address", &new_address), ("cited address", &cited_address)];
    fill(Code::SECTION_ADDRESS_CHANGED, 2, None, slots)
}

/// `unrecorded-section-citation`'s *Fix* (§FS-section-lock.checks.unrecorded-section-citation).
pub(crate) fn unrecorded_section_citation() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNRECORDED_SECTION_CITATION, 2, None, slots)
}
