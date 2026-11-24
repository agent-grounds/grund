//! Member boundaries shared by config inspection and walking commands
//! (§FS-config.4.2, §FS-config-explain.2, §FS-workspace.2). Membership is read
//! without loading configs, and matched without expanding sibling directories.

use std::path::{Component, Path, PathBuf};

use super::{
    config_file_in, joined_value, nearest_config, selects_v2, strip_comment, unloaded_string_list,
};
use crate::model::normalize_path_lexically;
use crate::sources::{Sources, canonical_path, lookup_refused, path_lookup};

/// The root and file a reader inspects (§FS-config.4.2, §FS-config-explain.2):
/// the nearest config, unless its membership selects a member with no file.
/// Read only membership here; unknown settings still let explain answer the
/// selected file's version, and sibling configs are never opened.
pub(super) fn inspected_config(start: &Path) -> Option<(PathBuf, Option<PathBuf>)> {
    let (dir, file) = nearest_config(start)?;
    let root = canonical_member_path(&dir);
    if let Ok(entries) = ancestor_member_entries(&file)
        && let Some(member) = member_root_for_scope(&root, &dir, start, &entries)
    {
        let file = config_file_in(&member);
        return Some((member, file));
    }
    Some((dir, Some(file)))
}

/// The member holding `start`, first along the path discovery climbed and then
/// resolved (§FS-config.1.5, §FS-workspace.2, §FS-config.4.2). Both loaded
/// workspace selection and lightweight inspection use this one matcher.
pub(crate) fn member_root_for_scope(
    root: &Path,
    discovered_dir: &Path,
    start: &Path,
    entries: &[String],
) -> Option<PathBuf> {
    let scope = match canonical_path(start) {
        Ok(scope) => scope,
        // §FS-scan.scope.homes.lookup: a refused path belongs to no member.
        Err(_) if path_lookup(start).is_err_and(|error| lookup_refused(&error)) => return None,
        Err(_) => start.to_path_buf(),
    };
    if scope == root {
        return None;
    }
    let lexical = start
        .strip_prefix(discovered_dir)
        .ok()
        .and_then(|relative| {
            entries
                .iter()
                .filter_map(|entry| lexical_member_candidate(entry, relative))
                .max_by_key(|member| member.components().count())
                .map(|member| canonical_member_path(&root.join(member)))
        });
    lexical.or_else(|| {
        entries
            .iter()
            .filter_map(|entry| configured_member_root_candidate(root, entry, &scope))
            .max_by_key(|member| member.components().count())
    })
}

/// The member directory `entry` names below the workspace config, as written
/// (§FS-config.1.5): one child of a `parent/*` entry, or the entry itself.
fn lexical_member_candidate(entry: &str, relative: &Path) -> Option<PathBuf> {
    if let Some(parent) = entry.strip_suffix("/*") {
        let parent = normalize_path_lexically(Path::new(parent));
        let Component::Normal(child) = relative.strip_prefix(&parent).ok()?.components().next()?
        else {
            return None;
        };
        return Some(parent.join(child));
    }
    let member = normalize_path_lexically(Path::new(entry));
    relative.starts_with(&member).then_some(member)
}

/// The same member match after resolving directory links (§FS-workspace.2).
fn configured_member_root_candidate(root: &Path, member: &str, scope: &Path) -> Option<PathBuf> {
    if let Some(parent) = member.strip_suffix("/*") {
        let parent = canonical_member_path(&root.join(parent));
        let relative = scope.strip_prefix(&parent).ok()?;
        let Component::Normal(child) = relative.components().next()? else {
            return None;
        };
        return Some(canonical_member_path(&parent.join(child)));
    }
    let root = canonical_member_path(&root.join(member));
    (scope == root || scope.starts_with(&root)).then_some(root)
}

/// Project identity resolves links once a member is selected (§FS-config.1.5).
fn canonical_member_path(path: &Path) -> PathBuf {
    canonical_path(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The `[workspace] members` and `optional_members` entries of a config, read
/// from the file text **on its own**: the parser's own line and section rules
/// applied to those two keys, and nothing else — no other key, no shape rule
/// (§FS-workspace.2.3), no grammar rebuild (§FS-workspace.6.1.7).
///
/// §FS-workspace.2.2.9: both lists claim, by the same entry-text rule. An optional
/// entry claims the directory below it exactly as a `members` entry does, so a run
/// started inside a *present* one reads its alias path out of that claim and
/// spells itself the way the workspace root does; an absent entry claims a
/// directory no run can be started inside, so including it costs nothing. Reading
/// only `members` here left a present optional member's subtree named from a
/// directory nothing lists, which is the disagreement §FS-workspace.6.1.5 exists to
/// prevent.
///
/// The ancestor climb needs the claim answerable for a config that does not
/// *load*, because the key that decides the claim is the very one a load error
/// may be about: read from a loaded config, every failure above a repository was
/// silently equal to "claims nothing", and the alias prefix the rule protects
/// collapsed. `Err` is the residue where even this read cannot answer — the file
/// cannot be read, or the value is not a list — and carries the reason the
/// warning that says so has to name.
///
/// A value is read as the config's own version reads it (§FS-workspace.6.1.7.5).
/// A version-2 config's goes through its reader's own `joined_value`: one written
/// across lines, or ending in the trailing comma TOML allows, is the list the
/// config loads, and one still open at the end of the file, or at a line that ends
/// inside a string, is residue, as the load refuses it
/// (§FS-config-v2.reader.5.strings). Read one
/// line at a time, such a list was "not a list", and a workspace written that way
/// silenced every unlisted block below it. Another key's array is joined only to
/// be passed over; one that never closes swallows lines a claim may stand on, and
/// taking it as residue silenced blocks no ancestor names, so the read goes on at
/// the next line instead, as it did before arrays were joined, and so it does after
/// one refused at its open line. A version-1 value
/// stays on its key's line, as the v1 reader reads every value (§FS-config.3), so
/// a v1 list written across lines is not a list here either: joining it made it a
/// claim, and the v1 reader's load error then failed a run below a stray config
/// that is not that run's problem.
pub(crate) fn ancestor_member_entries(config_path: &Path) -> Result<Vec<String>, String> {
    // §FS-check.6.1.1: cover this effective input before its shared read.
    let text = Sources::on_disk()
        .text(config_path)
        .map_err(|err| err.to_string())?;
    // §FS-config.3: only the version-2 reader reads an array across lines.
    let spans_lines = selects_v2(&text);
    let mut in_workspace = false;
    let mut entries = Vec::new();
    let mut optional = Vec::new();
    let mut lines = text.lines().enumerate();
    while let Some((idx, raw_line)) = lines.next() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            // Read exactly as `parse_config_file` reads a section header, so the
            // forms it *rejects* still count as the block that claims: a
            // `[[workspace]]` above a repository has to be able to fail the run
            // it claims, not vanish from it.
            in_workspace = line.trim_matches(['[', ']']) == "workspace";
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // Last assignment wins, exactly as it does in a full parse. Each key names
        // itself in the residue, because the sentence that reports one is what tells
        // the reader which line to open (§FS-check.3.29).
        let (slot, name) = match key.trim() {
            "members" if in_workspace => (&mut entries, "members"),
            "optional_members" if in_workspace => (&mut optional, "optional_members"),
            _ => {
                // Another key's array that closes is passed over whole, so no line in it
                // reads as a header or a key; one that never closes, or has an open line,
                // holds no claim, so the read goes on at the next line (§FS-workspace.6.1.7.5).
                if spans_lines {
                    let mut ahead = lines.clone();
                    if joined_value(value, idx + 1, &mut ahead).value.is_some() {
                        lines = ahead;
                    }
                }
                continue;
            }
        };
        // A version-2 claim still open at the end of the file, or at its open line, is not a
        // list, exactly where the load refuses it (§FS-config-v2.reader.5.strings).
        let value = match spans_lines {
            true => joined_value(value, idx + 1, &mut lines).value,
            false => Some(value.trim().to_string()),
        };
        *slot = value
            .and_then(|value| unloaded_string_list(spans_lines, config_path, idx + 1, &value))
            .ok_or_else(|| format!("`{name}` is not a list of strings"))?;
    }
    entries.extend(optional);
    Ok(entries)
}
