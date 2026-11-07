//! What a walk does with the names it met (§AR-scanner.1.8): the identity pass no longer discards
//! them. It publishes them on the run's `Spellings`, and asks the one rule what they come to
//! (§FS-scan.checks.reading-conflict, §FS-scan.checks.reading-moved).

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::walk_reporting::walk_scannable_files_with_sources;
use crate::config::{Frame, NamesVerdict, Schema, WalkBounds};
use crate::model::{ReadingConflict, ReadingMove};

/// What the names of one walk came to.
#[derive(Default)]
pub(super) struct NameVerdicts {
    pub(super) conflicts: Vec<ReadingConflict>,
    pub(super) moves: Vec<ReadingMove>,
}

/// Each physical file with a second name and the names that reached it, in listing order: the walk
/// order, past the `handed` files a run naming a path walks first, so a run naming either spelling
/// lists the names as the whole run does (§FS-scan.identity.findings.content).
fn name_groups(
    files: &[PathBuf],
    handed: usize,
    resolved: &HashMap<PathBuf, PathBuf>,
) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut groups: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    let mut at: HashMap<&Path, usize> = resolved
        .values()
        .map(|target| (target.as_path(), usize::MAX))
        .collect();
    for file in files[handed..].iter().chain(&files[..handed]) {
        let key = resolved.get(file).map_or(file.as_path(), PathBuf::as_path);
        let Some(slot) = at.get_mut(key) else {
            continue;
        };
        if *slot == usize::MAX {
            *slot = groups.len();
            groups.push((key.to_path_buf(), Vec::new()));
        }
        let names = &mut groups[*slot].1;
        if !names.contains(file) {
            names.push(file.clone());
        }
    }
    groups
}

/// Publish what this walk met and judge it. A tree with no link has nothing in `resolved` and pays
/// nothing here (§FS-scan.identity.reading.every-caller); a narrow run that meets a link to a file
/// whose own name claims nothing does one project walk first, to learn the file's other names.
pub(super) fn settle_names(
    schema: &Schema,
    frame: Frame<'_>,
    files: &[PathBuf],
    handed: usize,
    resolved: &HashMap<PathBuf, PathBuf>,
    whole_project: bool,
) -> Result<NameVerdicts> {
    let spellings = &frame.run.spellings;
    let groups = name_groups(files, handed, resolved);
    if !whole_project
        && groups
            .iter()
            .any(|(physical, _)| !frame.own_name_decides(schema, physical))
    {
        walk_scannable_files_with_sources(schema, frame, None, false, &mut |_, _| {})?;
    }
    // §FS-scan.identity.reading.plain: a handed path can sit under a plain root and be ignored
    // there, so what a plain walk met is asked too: past the handed files, or that one walk's.
    let plain_walk: Option<HashSet<PathBuf>> = if !whole_project {
        Some(
            groups
                .iter()
                .flat_map(|(physical, _)| spellings.names_at(physical))
                .collect(),
        )
    } else if handed > 0 && !groups.is_empty() {
        Some(files[handed..].iter().cloned().collect())
    } else {
        None
    };
    spellings.publish_walk(&groups, whole_project);
    let mut verdicts = NameVerdicts::default();
    if groups.is_empty() {
        return Ok(verdicts);
    }
    let bounds = WalkBounds::plain(schema, frame.root());
    let reaches = |name: &Path| {
        bounds.contains(name) && plain_walk.as_ref().is_none_or(|walk| walk.contains(name))
    };
    for (physical, _) in &groups {
        let names = frame.names_with_a_say(physical, reaches);
        match frame.judge_names(schema, &names) {
            NamesVerdict::Agree => {}
            NamesVerdict::Conflict { readings } => {
                spellings.refuse(physical);
                verdicts.conflicts.push(ReadingConflict { names, readings });
            }
            NamesVerdict::Moved { was, now } => {
                verdicts.moves.push(ReadingMove {
                    file: names[0].clone(),
                    was,
                    now,
                });
            }
        }
    }
    Ok(verdicts)
}
