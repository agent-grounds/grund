//! The *Fix* fillers of the codes §FS-config-v2 checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `config-fetch-names-nothing`'s *Fix* (§FS-config-v2.checks.config-fetch-names-nothing).
pub(crate) fn config_fetch_names_nothing(setting: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("setting", &setting)];
    fill(Code::CONFIG_FETCH_NAMES_NOTHING, 2, None, slots)
}
