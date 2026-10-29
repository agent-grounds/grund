//! The *Fix* fillers of the codes §FS-config-v2 checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

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
