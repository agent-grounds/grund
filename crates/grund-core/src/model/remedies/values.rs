//! The *Fix* fillers of the codes §FS-values checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `yaml-value-source-deferred`'s *Fix*, case `wrapped` (§FS-values.2.2.4).
pub(crate) fn yaml_value_source_deferred_wrapped(fix: impl Display, file: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("fix", &fix), ("file", &file)];
    fill(Code::YAML_VALUE_SOURCE_DEFERRED, 2, Some("wrapped"), slots)
}

/// `yaml-value-source-deferred`'s *Fix*, case `move` (§FS-values.2.2.4).
pub(crate) fn yaml_value_source_deferred_move(file: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("file", &file)];
    fill(Code::YAML_VALUE_SOURCE_DEFERRED, 2, Some("move"), slots)
}
