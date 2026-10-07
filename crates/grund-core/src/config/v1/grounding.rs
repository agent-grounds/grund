//! The v1 spelling of the grounding pair (§FS-config.3.4.8): the two
//! `[[kinds]]` keys read into the row being parsed, and the heading-level range
//! both they and `[reference] grounding_level` are held to as they are read.
//! What a combination of them means is judged once the project is lowered, in
//! `config/validate.rs` (§AR-config.4); the per-row resolution every reader
//! goes through stays in `config/grounding.rs`.

use anyhow::Result;
use std::path::Path;

use super::kind_table::ParsedKind;
use super::parse::{bail_config, parse_bool, parse_usize};
use crate::config::record::GROUNDING_LEVELS;

/// Where a `[[kinds]]` row wrote each grounding key. The *values* live on the
/// row's `KindConfig` while it is parsed; the lines are held aside and lowered
/// with them, so a rejection anchors at the offending key rather than at the
/// block header (§FS-config.4.3).
#[derive(Default)]
pub(super) struct ParsedGrounding {
    pub(super) require_line: Option<usize>,
    pub(super) level_line: Option<usize>,
}

/// Read one `[[kinds]]` grounding key into the row being parsed
/// (§FS-config.3.4.8.3). Both keys are booleans-and-integers with no defaulting of
/// their own: an absent key stays `None` and inherits `[reference]` later.
pub(super) fn parse_kind_grounding_key(
    path: &Path,
    line_no: usize,
    key: &str,
    value: &str,
    current_kind: &mut Option<ParsedKind>,
) -> Result<()> {
    let Some(slot) = current_kind.as_mut() else {
        bail_config(path, line_no, format!("`{key}` outside of [[kinds]] block"))?;
        unreachable!();
    };
    if key == "require_grounding" {
        slot.config.require_grounding = Some(parse_bool(path, line_no, value)?);
        slot.grounding.require_line = Some(line_no);
    } else {
        let level = parse_usize(path, line_no, value)?;
        check_grounding_level(path, line_no, level)?;
        slot.config.grounding_level = Some(level);
        slot.grounding.level_line = Some(line_no);
    }
    Ok(())
}

/// §FS-config.3.4.8.2: a level outside `1..=6` names no heading Markdown can have,
/// wherever it is written.
pub(super) fn check_grounding_level(path: &Path, line_no: usize, level: usize) -> Result<()> {
    if !GROUNDING_LEVELS.contains(&level) {
        bail_config(
            path,
            line_no,
            format!(
                "`grounding_level` must be a Markdown heading level {}..{} (`{level}` is not)",
                GROUNDING_LEVELS.start(),
                GROUNDING_LEVELS.end()
            ),
        )?;
    }
    Ok(())
}
