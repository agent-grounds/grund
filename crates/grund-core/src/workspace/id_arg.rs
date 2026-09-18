//! The qualified ID **argument** (§AR-system.2.4): splitting the `<alias>/<ID>`
//! a command is given into the alias path it names and the raw ID tail, and the
//! diagnostic an alias path that is not one slug per level earns
//! (§FS-workspace.1, §FS-workspace.8).
//!
//! A question about the `[workspace]` vocabulary and nothing else: it reads the
//! slug grammar of `config/workspace_block.rs` and no project, no scan and no
//! `Findings`, which is why it stayed here when the loaded project set left for
//! `resolver/` (§AR-system.2.10). The ID tail is deliberately left raw, so the
//! caller parses it with the *target* project's grammar rather than its own
//! (§AR-workspace.2).

use anyhow::{Result, anyhow};

use crate::config::{INVALID_ALIAS_PATH_EXPECTED, invalid_alias_path_segment};

/// Split a CLI ID argument that may carry a qualifying `<alias>/` prefix
/// (§FS-workspace.1). An ID never contains `/`, so the **last** separator is
/// the boundary — that is what lets a nested project be addressed by its whole
/// alias path (§FS-workspace.6.1). Every segment is validated against the slug
/// grammar here, before resolution; the ID tail is deliberately left raw so the
/// caller can parse it with the target project's grammar.
pub(crate) fn split_qualified_id_arg(raw: &str) -> Result<(Option<String>, &str)> {
    if let Some((alias, rest)) = raw.rsplit_once('/') {
        if let Some(message) = invalid_alias_path_message(alias) {
            return Err(anyhow!("{message}"));
        }
        return Ok((Some(alias.to_string()), rest));
    }
    Ok((None, raw))
}

/// §FS-workspace.8: the diagnostic for an alias path that is not one slug per
/// level, naming the **segment** that failed. Naming the whole path against a
/// pattern that forbids `/` would read as "a namespace may not contain `/`",
/// which is the opposite of the rule (§FS-workspace.1) — and in a nested tree the
/// path is usually mostly right. A single-segment path is its own segment, so it
/// is named plainly; an empty segment has nothing to quote and says so.
fn invalid_alias_path_message(alias: &str) -> Option<String> {
    let segments: Vec<&str> = alias.split('/').collect();
    let bad = invalid_alias_path_segment(alias)?;
    Some(if alias.is_empty() {
        format!(
            "invalid project alias: the path before the ID is empty ({INVALID_ALIAS_PATH_EXPECTED})"
        )
    } else if bad.is_empty() {
        format!(
            "invalid project alias `{alias}`: a segment is empty ({INVALID_ALIAS_PATH_EXPECTED})"
        )
    } else if segments.len() == 1 {
        format!("invalid project alias `{alias}` ({INVALID_ALIAS_PATH_EXPECTED})")
    } else {
        format!(
            "invalid project alias segment `{bad}` in `{alias}` ({INVALID_ALIAS_PATH_EXPECTED})"
        )
    })
}
