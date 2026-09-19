//! The `[workspace]` half of the config (§FS-config.3.8): the entry and alias
//! grammar a `members` / `optional_members` list is validated against at config
//! load, and the sentences that name what a bad entry got wrong.
//!
//! Beside `citations.rs`, `kind_table.rs` and `grounding.rs` for the same reason
//! they are their own files: `[workspace]` is one section of `grund.toml` with a
//! grammar of its own — a relative path per entry, one slug per alias level, and
//! one entry per list — and cross-key rules only the pair of lists can settle
//! (§AR-core-module-layout.1). Every rule here is a property of the entry
//! *text*, which is why it is taken here rather than where the directory is
//! looked for: the same config is then rejected in the checkout that has the
//! member and in the checkout that does not (§FS-workspace.2.2).
//!
//! What the block *means* — expansion, claims, aliases, the absent namespaces —
//! is the workspace component's (§AR-system.2.4), and it reads these rules
//! downward so one entry is judged by one grammar wherever it is read.

use anyhow::{Result, anyhow};
use std::path::Path;

use super::record::Config;
use crate::model::format_path;

/// §AR-workspace.5.2, §FS-workspace.2.2: both `[workspace]` member lists
/// shape-checked, on every load and whichever sections the file declared. The
/// one entry point the reader calls, the way it calls `validate_citation_rules`
/// for `[citations]`: which refusals a list earns is this file's, and the reader
/// only has to know that the block has some (§AR-core-module-layout.1).
pub(super) fn validate_workspace_lists(config: &Config) -> Result<()> {
    if let Some(source) = &config.workspace_members_source {
        for member in &config.workspace_members {
            validate_workspace_member(&source.path, source.line, member)?;
        }
    }
    if let Some(source) = &config.workspace_optional_members_source {
        for member in &config.workspace_optional_members {
            validate_optional_workspace_member(
                &source.path,
                source.line,
                member,
                &config.workspace_members,
            )?;
        }
    }
    Ok(())
}

/// §FS-workspace.2, §FS-config.3.8: the shape one `members` entry must have — a
/// relative path inside the block, or that path with a trailing `/*` glob. Every
/// refusal is a property of the entry text, so it is taken here rather than
/// where the directory is looked for.
fn validate_workspace_member(path: &Path, line: usize, member: &str) -> Result<()> {
    let member_path = Path::new(member);
    if member.is_empty()
        || member_path.is_absolute()
        || member_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
                    | std::path::Component::CurDir
                    | std::path::Component::ParentDir
            )
        })
        || member.contains('\\')
        || member.split('/').enumerate().any(|(index, part)| {
            part.is_empty()
                || part == "."
                || part == ".."
                || (index == 0 && looks_like_windows_drive_prefix(part))
        })
        || member.matches('*').count() > 1
        || (member.contains('*') && !member.ends_with("/*"))
    {
        return Err(anyhow!(
            "{}:{line}: invalid [workspace] member `{member}` (expected relative path or trailing /* glob)",
            format_path(path),
        ));
    }
    Ok(())
}

fn looks_like_windows_drive_prefix(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// §AR-workspace.5.3: the alias slug grammar, as one predicate — the same shape
/// `PROJECT_ALIAS_PATTERN` compiles into the citation regexes, asked of a name
/// rather than of a token.
pub(crate) fn is_valid_project_alias(alias: &str) -> bool {
    let mut chars = alias.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// What an alias path is expected to look like, named in every diagnostic that
/// refuses one — the slug grammar, and the fact that a path carries one segment
/// per workspace level (§FS-workspace.1.1, §FS-workspace.6.1).
pub(crate) const INVALID_ALIAS_PATH_EXPECTED: &str =
    "expected [a-z][a-z0-9-]*, one segment per workspace level";

/// Return the first invalid segment of an alias path, preserving empty segments
/// so each caller can render its surface's diagnostic without changing the
/// shared validation rule (§FS-workspace.1.1).
pub(crate) fn invalid_alias_path_segment(alias: &str) -> Option<&str> {
    alias
        .split('/')
        .find(|segment| !is_valid_project_alias(segment))
}

/// The `invalid workspace project alias` sentence, shared by the entry text
/// refused here and the member refused for its `project_name` where the alias is
/// derived, so the two read as one rule (§AR-workspace.5.3).
pub(crate) fn invalid_project_alias_message(alias: &str) -> String {
    format!("invalid workspace project alias `{alias}` (expected [a-z][a-z0-9-]*)")
}

/// §FS-workspace.2.2.2: the alias of an optional member is the entry's last path
/// segment. An absent member has no config to read `project_name` from and no
/// directory to take a basename from, so the entry text is the only name the
/// block that lists it can recover — and taking it in both checkouts is what makes
/// one citation text mean one thing in a full tree and a partial one.
pub(crate) fn optional_member_alias_segment(member: &str) -> &str {
    member.rsplit('/').next().unwrap_or(member)
}

/// §FS-workspace.2.2.7 "one entry belongs to one list": one sentence for the two
/// places that can catch the contradiction — the entry text here and the
/// canonical roots at expansion — because it is one rule, and which of the two saw
/// it is not something the author has to know.
pub(crate) fn both_member_lists_message(entry: &str) -> String {
    format!("`{entry}` is listed in both [workspace] members and optional_members")
}

/// §FS-workspace.2.2 / §FS-workspace.2.2.2: the four refusals `optional_members`
/// adds to the shape rules `members` already carries. All four are properties of
/// the entry text alone, which is why they are taken at config load — before any
/// directory is looked for, so the same config is rejected in the checkout that
/// has the member and in the checkout that does not.
///
/// The both-lists refusal is here for that reason above all. Behind the `is_dir`
/// test it could only fire in the checkout that *has* the member, and the other
/// checkout — where the `members` entry fails first — was told to list the entry
/// in `optional_members`, which is where the author had already put it
/// (§FS-config.4.3). What is wrong is the pair of lists, and the pair reads the
/// same in every checkout. The canonical comparison in the workspace component's
/// `expand_optional_members` stays for the collisions no entry text shows: a
/// `members` glob that expands onto this entry, or two paths that resolve to one
/// directory.
fn validate_optional_workspace_member(
    path: &Path,
    line: usize,
    member: &str,
    plain: &[String],
) -> Result<()> {
    validate_workspace_member(path, line, member)?;
    // §FS-workspace.2.2.7 "one entry belongs to one list": compared as paths, so a
    // trailing slash is the same entry rather than a second one.
    if plain
        .iter()
        .any(|entry| Path::new(entry) == Path::new(member))
    {
        return Err(anyhow!(
            "{}:{line}: {}",
            format_path(path),
            both_member_lists_message(member),
        ));
    }
    // §FS-workspace.2.2.6: an absent parent directory names no namespaces, so a
    // glob here would appear to work and do nothing. The message names the shape
    // that works, because a user who has just been refused needs the form to write.
    if member.contains('*') {
        return Err(anyhow!(
            "{}:{line}: [workspace] optional_members may not use a glob: `{member}` — an absent \
             parent names no namespaces; list one concrete entry per namespace instead",
            format_path(path),
        ));
    }
    let alias = optional_member_alias_segment(member);
    if !is_valid_project_alias(alias) {
        return Err(anyhow!(
            "{}:{line}: {} for workspace member `{member}`",
            format_path(path),
            invalid_project_alias_message(alias),
        ));
    }
    Ok(())
}
