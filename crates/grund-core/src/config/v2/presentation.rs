//! The presentation concern's keys (§FS-config-v2.presentation) and the
//! envelope's `[workspace]`. Each executed key means its v1 counterpart; what
//! the format defines and this grund does not execute is refused at its line
//! (§FS-config-v2.rollout).

use anyhow::Result;

use super::tables::not_yet;
use super::walk::Reader;
use crate::config::fmt_block::validate_fmt_exclude;
use crate::config::v1::parse_bool;

/// `[presentation]`: v1's `project_description`, `trigger` and `conversation`.
pub(super) fn presentation_key(
    r: &mut Reader,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let presentation = |r: &mut Reader| r.string(line, value);
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
        // §FS-config-v2.presentation: where each row's guidance is written.
        "rules" => {
            r.project.presentation.rules_at_root = match presentation(r)?.as_str() {
                "home" => false,
                "root" => true,
                other => {
                    return r.fail(
                        line,
                        format!("unknown [presentation] rules `{other}` (expected home or root)"),
                    );
                }
            };
        }
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
    let title = r.string(line, value)?;
    r.project.presentation.kinds.insert(kind.to_string(), title);
    Ok(true)
}

/// `[presentation.fmt]`: `exclude` is v1's `[fmt] exclude`; `anchors = "github"`
/// and `links = "index"` are the formatter's existing effect, and any other
/// value is refused with quoted value guidance (§FS-config-v2.rollout.fmt-values).
pub(super) fn fmt_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        "anchors" | "links" => {
            let executed = if key == "anchors" { "github" } else { "index" };
            let supplied = r.string(line, value)?;
            if supplied != executed {
                let refusal = not_yet(&format!("[presentation.fmt] {key}"));
                return r.fail(
                    line,
                    format!(
                        "{refusal}; unsupported value {supplied:?}; \
                         executes only {key} = {executed:?}"
                    ),
                );
            }
        }
        "exclude" => {
            let patterns = r.string_list(line, value)?;
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
    match key {
        "members" => {
            r.project.workspace.members = r.string_list(line, value)?;
            r.project.workspace.members_source = at;
        }
        "optional_members" => {
            r.project.workspace.optional_members = r.string_list(line, value)?;
            r.project.workspace.optional_members_source = at;
        }
        "include_root" => {
            let workspace = &mut r.project.workspace;
            workspace.include_root = parse_bool(r.path, line, value)?;
            workspace.include_root_source = at;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
