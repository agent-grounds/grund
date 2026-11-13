//! The *Fix* fillers of the codes §FS-config-v2 checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `config-index-not-markdown`'s *Fix* (§FS-config-v2.checks.config-index-not-markdown).
pub(crate) fn config_index_not_markdown() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_INDEX_NOT_MARKDOWN, 2, None, slots)
}

/// `config-md-not-markdown`'s *Fix* (§FS-config-v2.checks.config-md-not-markdown).
pub(crate) fn config_md_not_markdown() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_MD_NOT_MARKDOWN, 2, None, slots)
}

/// `config-rule-file-not-markdown`'s *Fix* (§FS-config-v2.checks.config-rule-file-not-markdown).
pub(crate) fn config_rule_file_not_markdown() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_RULE_FILE_NOT_MARKDOWN, 2, None, slots)
}

/// `config-rule-folder-not-markdown`'s *Fix* (§FS-config-v2.checks.config-rule-folder-not-markdown).
pub(crate) fn config_rule_folder_not_markdown() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_RULE_FOLDER_NOT_MARKDOWN, 2, None, slots)
}

/// `config-value-home-absent`'s *Fix*, case `file` (§FS-config-v2.checks.config-value-home-absent).
pub(crate) fn config_value_home_absent_file() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_VALUE_HOME_ABSENT, 2, Some("file"), slots)
}

/// `config-value-home-absent`'s *Fix*, case `folder` (§FS-config-v2.checks.config-value-home-absent).
pub(crate) fn config_value_home_absent_folder() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_VALUE_HOME_ABSENT, 2, Some("folder"), slots)
}

/// `config-value-home-invalid`'s *Fix*, case `shape` (§FS-config-v2.checks.config-value-home-invalid).
pub(crate) fn config_value_home_invalid_shape(shape: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("shape", &shape)];
    fill(Code::CONFIG_VALUE_HOME_INVALID, 2, Some("shape"), slots)
}

/// `config-value-home-invalid`'s *Fix*, case `outside` (§FS-config-v2.checks.config-value-home-invalid).
pub(crate) fn config_value_home_invalid_outside() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_VALUE_HOME_INVALID, 2, Some("outside"), slots)
}

/// `config-value-home-invalid`'s *Fix*, case `lookup` (§FS-config-v2.checks.config-value-home-invalid).
pub(crate) fn config_value_home_invalid_lookup() -> String {
    let slots: &[(&str, &dyn Display)] = &[];
    fill(Code::CONFIG_VALUE_HOME_INVALID, 2, Some("lookup"), slots)
}

/// `rule-folder-file-not-markdown`'s *Fix* (§FS-config-v2.checks.rule-folder-file-not-markdown).
pub(crate) fn rule_folder_file_not_markdown(folder: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("folder", &folder)];
    fill(Code::RULE_FOLDER_FILE_NOT_MARKDOWN, 2, None, slots)
}
