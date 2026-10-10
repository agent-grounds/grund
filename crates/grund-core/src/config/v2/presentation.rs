//! The presentation concern's keys (§FS-config-v2.presentation) and the
//! envelope's `[workspace]`. Each executed key means its v1 counterpart; what
//! the format defines and this grund does not execute is refused at its line
//! (§FS-config-v2.rollout).

use anyhow::Result;

use super::tables::not_yet;
use super::walk::Reader;
use crate::config::fmt_block::validate_fmt_exclude;
use crate::config::v1::{parse_bool, parse_string, parse_string_list};

/// `[presentation]`: v1's `project_description`, `trigger` and `conversation`.
pub(super) fn presentation_key(
    r: &mut Reader,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let presentation = |r: &mut Reader| parse_string(r.path, line, value);
    match key {
        "description" => {
            let description = presentation(r)?;
            // §FS-config.3: it feeds single-line member bullets.
            if description.contains(['\n', '\r']) {
                return r.fail(
                    line,
                    "[presentation] description must be a single line".to_string(),
                );
            }
            r.project.presentation.description = Some(description);
        }
        "trigger" => r.project.presentation.trigger = presentation(r)?,
        "conversation" => {
            let opinion = presentation(r)?;
            if opinion != "link" {
                return r.fail(
                    line,
                    format!("unknown [presentation] conversation `{opinion}` (expected link)"),
                );
            }
            r.project.presentation.conversation = Some(opinion);
        }
        // §FS-config-v2.rollout: where rules render is #460's.
        "rules" => return r.fail(line, not_yet("[presentation] rules")),
        _ => return Ok(false),
    }
    Ok(true)
}

/// `[presentation.kinds.<NAME>]`: the kind's `title`.
pub(super) fn kind_key(
    r: &mut Reader,
    kind: &str,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    if key != "title" {
        return Ok(false);
    }
    let title = parse_string(r.path, line, value)?;
    r.project.presentation.kinds.insert(kind.to_string(), title);
    Ok(true)
}

/// `[presentation.fmt]`: `exclude` is v1's `[fmt] exclude`; `anchors = "github"`
/// and `links = "index"` are the formatter's existing effect, and any other
/// value is #459's.
pub(super) fn fmt_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        "anchors" | "links" => {
            let executed = if key == "anchors" { "github" } else { "index" };
            if parse_string(r.path, line, value)? != executed {
                return r.fail(line, not_yet(&format!("[presentation.fmt] {key}")));
            }
        }
        "exclude" => {
            let patterns = parse_string_list(r.path, line, value)?;
            if let Err(message) = validate_fmt_exclude(&patterns) {
                return r.fail(line, message);
            }
            r.project.presentation.fmt.exclude = patterns;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `[workspace]`: the envelope's member lists, read as v1 reads them
/// (§FS-config.3.8); their meanings are judged once for both readers.
pub(super) fn workspace_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    let at = r.at(line);
    let workspace = &mut r.project.workspace;
    match key {
        "members" => {
            workspace.members = parse_string_list(r.path, line, value)?;
            workspace.members_source = at;
        }
        "optional_members" => {
            workspace.optional_members = parse_string_list(r.path, line, value)?;
            workspace.optional_members_source = at;
        }
        "include_root" => {
            workspace.include_root = parse_bool(r.path, line, value)?;
            workspace.include_root_source = at;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
