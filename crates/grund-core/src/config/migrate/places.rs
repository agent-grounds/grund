//! What v1's `[scan] include` and homes become as version-2 places
//! (§FS-config-migrate.mapping): each include entry a folder or a file of the
//! `code` row by what it is on disk, an entry at or inside a walked home a
//! duplicate, and the refusals where version 2 would place a file elsewhere —
//! a missing entry (§FS-config-migrate.3.1.3) and nested homes
//! (§FS-config-migrate.3.1.5).

use std::fs;
use std::path::Path;

use super::keys::Keys;
use super::record::{MigrateRefusal, RefusalAnchor};
use crate::config::project::Project;
use crate::config::rows::{Extent, Place};

/// The `code` row's places, and what the include list says beside them.
#[derive(Default)]
pub(super) struct Include {
    pub(super) folders: Vec<String>,
    pub(super) files: Vec<String>,
    /// An entry at or inside a walked home, with the home's row.
    pub(super) duplicates: Vec<(String, String)>,
    pub(super) refusals: Vec<MigrateRefusal>,
}

impl Include {
    /// The places the complement gains, files then folders, each walked.
    pub(super) fn places(&self) -> Vec<Place> {
        let place = |extent| Place {
            extent,
            scanned: true,
            source: None,
        };
        let files = self
            .files
            .iter()
            .map(|path| place(Extent::File(path.clone())));
        let folders = self
            .folders
            .iter()
            .map(|path| place(Extent::Folder(path.clone())));
        files.chain(folders).collect()
    }
}

/// A home's path, with its row and whether the walk reads it.
fn homes(v1: &Project) -> Vec<(&str, &str, bool)> {
    v1.schema
        .places()
        .filter_map(|(row, place)| match &place.extent {
            Extent::Folder(path) | Extent::File(path) => Some((row, path.as_str(), place.scanned)),
            Extent::Complement => None,
        })
        .collect()
}

fn at_or_inside(path: &str, home: &str) -> bool {
    path == home
        || path
            .strip_prefix(home)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// §FS-config-migrate.mapping `[scan] include`: each entry by what it is on
/// disk under `root`. Only an authored list refuses; a default entry that does
/// not exist read nothing in v1 and is dropped.
pub(super) fn include(v1: &Project, keys: &Keys, root: &Path) -> Include {
    let mut out = Include::default();
    let Some(entries) = &v1.schema.sources.include else {
        return out;
    };
    let line = keys.line("scan", "include");
    let anchor = line.map_or(RefusalAnchor::Unset, RefusalAnchor::Line);
    let homes = homes(v1);
    for raw in entries {
        let entry = raw
            .trim_start_matches("./")
            .trim_end_matches('/')
            .to_string();
        let refuse = |what: String| MigrateRefusal {
            anchor,
            what,
            details: Vec::new(),
            repair: format!("remove `{entry}` from include under version 1"),
        };
        if entry.starts_with('/') || entry == ".." || entry.starts_with("../") {
            out.refusals.push(refuse(format!(
                "[scan] include names `{entry}`, which is outside the config root, and a version 2 place is inside it"
            )));
            continue;
        }
        if let Some((row, home, scanned)) =
            homes.iter().find(|(_, home, _)| at_or_inside(&entry, home))
        {
            if *scanned {
                out.duplicates.push((entry.clone(), row.to_string()));
            } else {
                out.refusals.push(refuse(format!(
                    "[scan] include names `{entry}`, inside `{row}` home `{home}`, which is listed and not walked; version 2 walks no place inside it"
                )));
            }
            continue;
        }
        if let Some(member) = v1
            .workspace
            .members
            .iter()
            .chain(&v1.workspace.optional_members)
            .find(|m| at_or_inside(&entry, m.trim_end_matches('/')))
        {
            out.refusals.push(refuse(format!(
                "[scan] include names `{entry}`, inside workspace member `{member}`, whose own config governs it"
            )));
            continue;
        }
        let on_disk = root.join(&entry);
        if on_disk.is_dir() {
            out.folders.push(entry);
        } else if on_disk.is_file() {
            out.files.push(entry);
        } else if line.is_some() {
            out.refusals.push(MigrateRefusal {
                anchor,
                what: format!(
                    "[scan] include names `{entry}`, which does not exist, and version 2 must know whether it is a folder or a file"
                ),
                details: Vec::new(),
                repair: format!(
                    "create {entry}, or remove it from include; grund check under version 1 shows the effect"
                ),
            });
        }
    }
    out
}

/// §FS-config-migrate.3.1.5: a home inside another row's folder home, with
/// every file whose owner would change.
pub(super) fn nested_homes(v1: &Project, keys: &Keys, root: &Path) -> Vec<MigrateRefusal> {
    let homes = homes(v1);
    let mut out = Vec::new();
    for (inner_row, inner, _) in &homes {
        let Some((outer_row, outer, _)) = homes.iter().find(|(row, home, _)| {
            row != inner_row
                && inner
                    .strip_prefix(home)
                    .is_some_and(|rest| rest.starts_with('/'))
        }) else {
            continue;
        };
        let line = keys
            .row_line(inner_row, "folder")
            .or_else(|| keys.row_line(inner_row, "file"));
        let files = scanned_files(v1, root, inner);
        let details = if files.is_empty() {
            Vec::new()
        } else {
            vec![format!("would change owner: {}", files.join(", "))]
        };
        out.push(MigrateRefusal {
            anchor: line.map_or(RefusalAnchor::Unset, RefusalAnchor::Line),
            what: format!(
                "[[kinds]] `{inner_row}` home `{inner}` is inside `{outer_row}` home `{outer}`; version 1 gives the files in both to the complement, version 2 to the deepest place"
            ),
            details,
            repair: "un-nest the two homes under version 1".into(),
        });
    }
    out
}

/// The files under `root/path` the v1 walk reads by their extension, skipping
/// the excluded names, as sorted paths from the root.
pub(super) fn scanned_files(v1: &Project, root: &Path, path: &str) -> Vec<String> {
    let sources = &v1.schema.sources;
    let mut out = Vec::new();
    let start = root.join(path);
    if start.is_file() {
        return vec![path.to_string()];
    }
    let mut stack = vec![(start, path.to_string())];
    while let Some((dir, rel)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if sources.exclude.contains(&name) {
                continue;
            }
            let child = format!("{rel}/{name}");
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push((entry.path(), child));
            } else if Path::new(&name)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| sources.extensions.iter().any(|e| e == ext))
            {
                out.push(child);
            }
        }
    }
    out.sort();
    out
}
