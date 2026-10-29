//! The *Fix* fillers of the codes §FS-rules checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `unreached-declaration`'s *Fix* (§FS-rules.checks.unreached-declaration).
pub(crate) fn unreached_declaration() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::UNREACHED_DECLARATION, 2, None, slots)
}
