//! The *Fix* fillers of the codes §FS-config-v2 checks, rendered from the
//! §FS-errors.5.5 catalog by `scripts/render_check_codes.py`: edit the catalog,
//! then run the script (§AR-config.9.1).

use std::fmt::Display;

use super::super::catalog::fill;
use super::super::codes::Code;

/// `rule-folder-file-not-markdown`'s *Fix* (§FS-config-v2.checks.rule-folder-file-not-markdown).
pub(crate) fn rule_folder_file_not_markdown(folder: impl Display) -> String {
    let slots: &[(&str, &dyn Display)] = &[("folder", &folder)];
    fill(Code::RULE_FOLDER_FILE_NOT_MARKDOWN, 2, None, slots)
}
