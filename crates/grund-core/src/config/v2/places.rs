//! What a v2 place may name, judged once every row and `[workspace]` are read
//! (§FS-config-v2.schema.places.boundaries): one place belongs to one row,
//! lexically and through a link; a folder place does not link out of the root;
//! and no place sits in a workspace member. Each refusal is located at the
//! place's key. The lexical rule, a place outside the root, is the reader's, at
//! the key itself (`schema.rs`).

use std::path::{Path, PathBuf};

use super::schema::{PlaceDraft, RowDraft};
use crate::config::project::Members;
use crate::sources::canonical_path;

/// §FS-config-v2.schema.places.boundaries: every place against every earlier
/// one, the root and the members, at most one refusal per place.
pub(super) fn judge(
    rows: &[RowDraft],
    root: &Path,
    workspace: &Members,
    late: &mut Vec<(usize, String)>,
) {
    let canonical_root = canonical_path(root).unwrap_or_else(|_| root.to_path_buf());
    let members: Vec<&str> = workspace
        .members
        .iter()
        .chain(&workspace.optional_members)
        .map(String::as_str)
        .collect();
    let mut seen: Vec<(&RowDraft, &PlaceDraft, Option<PathBuf>)> = Vec::new();
    for row in rows {
        for place in &row.places {
            let clause = format!("`[schema.kinds.{}] {}`", row.name, place.key());
            let at = |what: String| {
                (
                    place.line,
                    format!("place `{}` in {clause} {what}", place.written),
                )
            };
            let target = canonical_path(root.join(place.path())).ok();
            // The same place, or one target, twice (§FS-config-v2.schema.places.boundaries.3).
            let earlier = seen.iter().find(|(_, other, other_target)| {
                other.path() == place.path()
                    || target.is_some() && target.as_ref() == other_target.as_ref()
            });
            if let Some((other_row, other, _)) = earlier {
                let what = format!(
                    "is already a place of `[schema.kinds.{}]` (line {}); a place belongs to one row",
                    other_row.name, other.line
                );
                late.push(at(what));
            } else if place.key() == "folders"
                && target
                    .as_ref()
                    .is_some_and(|target| !target.starts_with(&canonical_root))
            {
                // Refused, not pruned (§FS-config-v2.schema.places.boundaries.3).
                late.push(at("links out of the project root".to_string()));
            } else if let Some(member) = members
                .iter()
                .find_map(|member| member_holding(member, place.path()))
            {
                // §FS-config-v2.schema.places.boundaries.5, §FS-workspace.6.
                let what =
                    format!("is inside workspace member `{member}`, which the root never reads");
                late.push(at(what));
            }
            seen.push((row, place, target));
        }
    }
}

/// The member `pattern` names that holds `place`, both config-root-relative:
/// the member itself, or, for a `<dir>/*` pattern, the one directory under it
/// the place sits in (§FS-workspace.6).
fn member_holding(pattern: &str, place: &str) -> Option<String> {
    let pattern = segments(pattern);
    let place = segments(place);
    if pattern.is_empty() || place.len() < pattern.len() {
        return None;
    }
    let matched = pattern
        .iter()
        .zip(&place)
        .all(|(pattern, part)| *pattern == "*" || pattern == part);
    matched.then(|| place[..pattern.len()].join("/"))
}

/// The named segments of a config-root-relative path, without `.` or empty ones.
fn segments(path: &str) -> Vec<&str> {
    Path::new(path)
        .components()
        .filter_map(|part| match part {
            std::path::Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect()
}
