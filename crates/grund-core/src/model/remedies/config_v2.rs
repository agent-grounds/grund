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

/// `config-not-toml`'s *Fix*, case `open-line` (§FS-config-v2.checks.config-not-toml).
pub(crate) fn config_not_toml_open_line(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_NOT_TOML, 2, Some("open-line"), slots)
}

/// `config-not-toml`'s *Fix*, case `element` (§FS-config-v2.checks.config-not-toml).
pub(crate) fn config_not_toml_element(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_NOT_TOML, 2, Some("element"), slots)
}

/// `config-not-toml`'s *Fix*, case `scalar-or-key` (§FS-config-v2.checks.config-not-toml).
pub(crate) fn config_not_toml_scalar_or_key(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_NOT_TOML, 2, Some("scalar-or-key"), slots)
}

/// `config-not-toml`'s *Fix*, case `commas` (§FS-config-v2.checks.config-not-toml).
pub(crate) fn config_not_toml_commas(line: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("line", &line)];
    fill(Code::CONFIG_NOT_TOML, 2, Some("commas"), slots)
}
