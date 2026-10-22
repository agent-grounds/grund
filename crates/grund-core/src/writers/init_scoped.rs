//! `init`'s scoped files (§FS-init.2.3.5.14): each destination's `AGENTS.md`
//! created, appended or updated through the entrypoint splice, the root's
//! companion symlinks mirrored beside it, and every orphan the list on disk
//! names removed. Planned and refused before the first write of the run
//! (§FS-init.1.2.6), written after the root entrypoints.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use super::guidance::Guidance;
use super::init::{verb_updated, verb_wrote};
use super::init_block::{update_agents_block, update_agents_text};
use super::init_output::InitEvent;
use super::scoped_files::{
    OrphanFile, OwnedCompanion, ScopedRecord, orphan_files, owned_companions,
};
use crate::grammar::{AgentsBlockLookup, find_agents_block};
use crate::scanner::CANONICAL_AGENT_ENTRYPOINT;
use crate::sources::{Sources, path_exists, path_is_entry, path_is_symlink};

/// What one run does to scoped files, in report order (§FS-init.2.3.5.14.4).
pub(super) struct ScopedPlan {
    files: Vec<ScopedFile>,
    /// Each orphan's directory, its files `AGENTS.md` first.
    orphans: Vec<Vec<OrphanEntry>>,
}

/// A destination's `AGENTS.md`, its block, and its companions.
struct ScopedFile {
    path: PathBuf,
    rel: String,
    block: String,
    companions: Vec<Companion>,
}

/// A companion `init` writes beside a scoped `AGENTS.md` (§FS-init.2.3.5.14.3):
/// a symlink where the name is free, the block spliced into a standalone file.
enum Companion {
    Link { path: PathBuf, rel: String },
    Splice { path: PathBuf, rel: String },
}

/// A file of an orphan's directory that `init` touches, and what it is
/// (§FS-init.2.3.5.14.3).
struct OrphanEntry {
    path: PathBuf,
    rel: String,
    what: OrphanFile,
}

/// Plan the scoped writes under `target`. `root` is the loaded config's root,
/// which the chapter-rule links of a pending entrypoint are resolved under
/// (§FS-init.2.3.5.10). The list on disk is read here, before the root block
/// is rewritten (§FS-init.2.3.5.13.1).
pub(super) fn plan_scoped(guidance: &Guidance<'_>, root: &Path, target: &Path) -> ScopedPlan {
    let record = ScopedRecord::read(target, Sources::on_disk());
    let live: Vec<&str> = guidance.layout.scoped_dirs().collect();
    let files = live
        .iter()
        .map(|dir| {
            let path = target.join(dir).join(CANONICAL_AGENT_ENTRYPOINT);
            let companions = owned_companions(target, dir)
                .into_iter()
                .map(|(name, owned)| {
                    let path = target.join(dir).join(name);
                    let rel = format!("{dir}/{name}");
                    match owned {
                        OwnedCompanion::Free => Companion::Link { path, rel },
                        OwnedCompanion::Standalone => Companion::Splice { path, rel },
                    }
                })
                .collect();
            ScopedFile {
                block: guidance.scoped_block(dir, &root.join(dir).join(CANONICAL_AGENT_ENTRYPOINT)),
                rel: format!("{dir}/{CANONICAL_AGENT_ENTRYPOINT}"),
                path,
                companions,
            }
        })
        .collect();
    let orphans = record
        .orphans(&live)
        .into_iter()
        .map(|dir| {
            orphan_files(target, dir)
                .into_iter()
                .map(|(name, what)| OrphanEntry {
                    path: target.join(dir).join(name),
                    rel: format!("{dir}/{name}"),
                    what,
                })
                .collect()
        })
        .collect();
    ScopedPlan { files, orphans }
}

/// §FS-init.2.3.5.14.1: a newer or malformed block in any file the plan
/// splices, or a scoped `AGENTS.md` that is a symlink, refuses the run before
/// anything is written; so does an orphan whose delimiters cannot be cut.
pub(super) fn refuse_scoped(plan: &ScopedPlan) -> Option<(PathBuf, anyhow::Error)> {
    let spliced = plan.files.iter().flat_map(|file| {
        let companions = file
            .companions
            .iter()
            .filter_map(|companion| match companion {
                Companion::Splice { path, rel } => Some((path, rel)),
                Companion::Link { .. } => None,
            });
        std::iter::once((&file.path, &file.rel)).chain(companions)
    });
    for (path, rel) in spliced {
        if path_is_symlink(path) {
            return Some((
                path.clone(),
                anyhow!("{rel} is a symlink; grund init does not write through it"),
            ));
        }
        if path_exists(path) {
            let refusal = match Sources::on_disk().text(path) {
                Ok(text) => update_agents_text(&text, "", rel).err(),
                Err(err) => Some(err.into()),
            };
            if let Some(err) = refusal {
                return Some((path.clone(), err));
            }
        }
    }
    let cut = plan
        .orphans
        .iter()
        .flatten()
        .filter(|entry| entry.what != OrphanFile::Linked);
    for OrphanEntry { path, .. } in cut {
        if let Ok(text) = Sources::on_disk().text(path)
            && let AgentsBlockLookup::Malformed { message, .. } = find_agents_block(&text)
        {
            return Some((
                path.clone(),
                anyhow!("malformed grund managed block: {message}"),
            ));
        }
    }
    None
}

/// Write the plan, pushing one report line per file touched in report order
/// (§FS-init.2.3.5.14.4). Returns whether anything changed, or the path that
/// failed and why.
pub(super) fn write_scoped(
    plan: &ScopedPlan,
    dry_run: bool,
    events: &mut Vec<InitEvent>,
) -> Result<bool, (PathBuf, anyhow::Error)> {
    let mut changed = false;
    let mut push = |verb: &'static str, path: String| {
        changed |= verb != "exists";
        events.push(InitEvent { verb, path });
    };
    for file in &plan.files {
        if !path_is_entry(&file.path) {
            // §FS-init.2.3.5.12.3: a created scoped file is the block alone.
            if !dry_run {
                fs::write(&file.path, &file.block)
                    .map_err(|err| (file.path.clone(), err.into()))?;
            }
            push(verb_wrote(dry_run), file.rel.clone());
        } else {
            let verb = splice(&file.path, &file.block, &file.rel, dry_run)?;
            push(verb, file.rel.clone());
        }
        for companion in &file.companions {
            match companion {
                Companion::Link { path, rel } => {
                    if !dry_run {
                        link_to_agents(path).map_err(|err| (path.clone(), err.into()))?;
                    }
                    let verb = if dry_run { "would-link" } else { "linked" };
                    push(verb, format!("{rel} -> {CANONICAL_AGENT_ENTRYPOINT}"));
                }
                Companion::Splice { path, rel } => {
                    let verb = splice(path, &file.block, rel, dry_run)?;
                    push(verb, rel.clone());
                }
            }
        }
    }
    for orphan in &plan.orphans {
        // §FS-init.2.3.5.14.3: each file loses its block on its own account,
        // and a link goes only with the `AGENTS.md` this run deleted.
        let mut agents_removed = false;
        for OrphanEntry { path, rel, what } in orphan {
            match what {
                OrphanFile::Agents | OrphanFile::Standalone => {
                    let removed = cut_block(path, dry_run)?;
                    agents_removed |= *what == OrphanFile::Agents && removed == Some(true);
                    if let Some(removed) = removed {
                        push(cut_verb(removed, dry_run), rel.clone());
                    }
                }
                OrphanFile::Linked if agents_removed => {
                    if !dry_run {
                        fs::remove_file(path).map_err(|err| (path.clone(), err.into()))?;
                    }
                    push(cut_verb(true, dry_run), rel.clone());
                }
                OrphanFile::Linked => {}
            }
        }
    }
    Ok(changed)
}

/// Splice `block` into the existing file at `path` (§FS-init.2.3.10).
fn splice(
    path: &Path,
    block: &str,
    rel: &str,
    dry_run: bool,
) -> Result<&'static str, (PathBuf, anyhow::Error)> {
    // A scoped file carries no member list to keep (§FS-init.2.3.4.15.8).
    update_agents_block(path, block, rel, dry_run, false)
        .map(|(result, _)| result.verb(dry_run))
        .map_err(|err| (path.to_path_buf(), err))
}

/// The report verb for an orphan's file: removed when it was deleted, updated
/// when its block was cut from it (§FS-init.2.3.5.14.4).
fn cut_verb(removed: bool, dry_run: bool) -> &'static str {
    match (removed, dry_run) {
        (true, true) => "would-remove",
        (true, false) => "removed",
        (false, _) => verb_updated(dry_run),
    }
}

/// Remove the managed block from the file at `path` (§FS-init.2.3.5.14.2): the
/// file is deleted when the block was the whole of it, byte for byte, and
/// otherwise the block is cut through its end delimiter's line ending. `None`
/// when the file is gone or carries no block, which is left alone unreported;
/// `Some(true)` when it was deleted.
pub(super) fn cut_block(
    path: &Path,
    dry_run: bool,
) -> Result<Option<bool>, (PathBuf, anyhow::Error)> {
    let Ok(text) = Sources::on_disk().text(path) else {
        return Ok(None);
    };
    let AgentsBlockLookup::Found(block) = find_agents_block(&text) else {
        return Ok(None);
    };
    let whole = block.start == 0 && block.end == text.len();
    if !dry_run {
        let written = if whole {
            fs::remove_file(path)
        } else {
            fs::write(
                path,
                format!("{}{}", &text[..block.start], &text[block.end..]),
            )
        };
        written.map_err(|err| (path.to_path_buf(), err.into()))?;
    }
    Ok(Some(whole))
}

/// Create `path` as a symlink to the `AGENTS.md` beside it.
#[cfg(unix)]
fn link_to_agents(path: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(CANONICAL_AGENT_ENTRYPOINT, path)
}

/// Create `path` as a symlink to the `AGENTS.md` beside it.
#[cfg(windows)]
fn link_to_agents(path: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(CANONICAL_AGENT_ENTRYPOINT, path)
}
