use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

use super::init::{InitEvent, verb_appended, verb_updated, verb_wrote};
use super::init_plan::SelectedInitAgentEntrypoints;
use crate::grammar::{
    AGENTS_BLOCK_VERSION, AgentsBlock, AgentsBlockLookup, WORKSPACE_MEMBERS, block_sections,
    find_agents_block,
};
use crate::model::format_path;
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, InitCompanionAgentEntrypoint};
use crate::sources::{Sources, path_exists};
use crate::templates::render_agents_md_from_block;

/// What `init` did to an existing `AGENTS.md`'s managed block — `appended ` (no
/// block before), `updated ` (a supported block whose bytes changed: an older
/// block upgraded, or a same-version block re-rendered against a changed
/// template or config), or `unchanged` (a supported block already byte-identical
/// to the current render — `init` rewrites nothing, §FS-init.2.2/§FS-init.2.3.10,
/// and reports it with the `exists ` prefix like any other untouched file).
///
/// What this file holds: rewriting the managed agent-instructions block inside a
/// document that is otherwise the user's (§FS-init.2.3) — splicing the current
/// render over exactly its bytes, and refusing the two cases that would eat
/// content it does not own, which is why it lives apart from the scaffold
/// writing in `init.rs` (§AR-core-module-layout.1). *Locating* the block and
/// reading its version is `grammar/managed_block.rs` now (§AR-system.2.1), so
/// `grund check`'s read of the same block (§FS-check.3.5) is downward rather
/// than into this file: the locator the checker trusts and the one the writer
/// splices with are still the same function.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AgentsUpdateResult {
    Appended,
    Updated,
    Unchanged,
}

impl AgentsUpdateResult {
    /// The report verb for the file (§FS-init.2.2), conditional under `--dry-run`.
    pub(super) fn verb(&self, dry_run: bool) -> &'static str {
        match self {
            Self::Appended => verb_appended(dry_run),
            Self::Updated => verb_updated(dry_run),
            Self::Unchanged => "exists",
        }
    }
}

/// What one root entrypoint's write did with its `### Workspace members` section
/// (§FS-init.2.3.4.15.8): kept the bytes its block held, or wrote the section from
/// what the run can read — a new file, a block appended, or a block that had none.
/// It is what the absent-member note says per entrypoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MembersSection {
    Kept,
    New,
}

impl MembersSection {
    fn of(kept: &Option<String>) -> Self {
        if kept.is_some() {
            Self::Kept
        } else {
            Self::New
        }
    }
}

/// Returns the file event for the canonical entrypoint and what became of its
/// member list, or an already formatted I/O message for the caller to surface.
/// `keep_members` is whether an optional member is absent from this run, so the
/// list the file holds is kept rather than rendered (§FS-init.2.3.4.15.8).
pub(super) fn write_or_update_canonical_agent_entrypoint(
    target: &Path,
    rel: &str,
    name: &str,
    block: &str,
    force: bool,
    dry_run: bool,
    keep_members: bool,
) -> Result<(InitEvent, MembersSection), String> {
    let dest = target.join(rel);
    if !force && path_exists(&dest) {
        match update_agents_block(&dest, block, rel, dry_run, keep_members) {
            Ok((result, members)) => {
                let verb = result.verb(dry_run);
                let path = rel.to_string();
                Ok((InitEvent { verb, path }, members))
            }
            // Forward slashes on every platform, like report paths
            // (§FS-errors.2.2) — Windows must not leak backslashes.
            Err(err) => Err(format!("update {}: {err}", format_path(&dest))),
        }
    } else {
        // §FS-init.3.4: `--force` keeps none of the file but the member list
        // §FS-init.2.3.4.15.8 keeps, read before the overwrite; unreadable, none.
        let kept = (keep_members && path_exists(&dest))
            .then(|| Sources::on_disk().text(&dest).ok())
            .flatten()
            .and_then(|existing| keep_workspace_members(&existing, block));
        let contents = render_agents_md_from_block(name, kept.as_deref().unwrap_or(block));
        if !dry_run
            && let Some(parent) = dest.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            return Err(format!("create {}: {err}", parent.display()));
        }
        if !dry_run && let Err(err) = fs::write(&dest, contents) {
            return Err(format!("write {}: {err}", dest.display()));
        }
        let event = InitEvent {
            verb: verb_wrote(dry_run),
            path: rel.to_string(),
        };
        Ok((event, MembersSection::of(&kept)))
    }
}

/// Append or update the managed block in an existing agent entrypoint on disk
/// (§FS-init.2.3.10). A supported block is re-rendered from the current
/// template/config even when the schema version already matches — but when that
/// re-render is byte-identical to what is on disk the file is left untouched
/// (`Unchanged`, reported as `exists `), so re-running `grund init` on an
/// up-to-date repo writes nothing (§FS-init.2.2). Under `--dry-run`, the
/// computed result is returned without writing.
///
/// With `keep_members`, the block on disk keeps its `### Workspace members`
/// section and the comparison runs on the spliced render, so `exists`, `updated`
/// and the preview all follow from the one splice (§FS-init.2.3.4.15.8). A scoped
/// file carries no such section and passes `false`.
pub(super) fn update_agents_block(
    dest: &Path,
    block: &str,
    label: &str,
    dry_run: bool,
    keep_members: bool,
) -> Result<(AgentsUpdateResult, MembersSection)> {
    let existing = Sources::on_disk().text(dest)?;
    let kept = keep_members
        .then(|| keep_workspace_members(&existing, block))
        .flatten();
    let (updated, result) = update_agents_text(&existing, kept.as_deref().unwrap_or(block), label)?;
    if !dry_run && result != AgentsUpdateResult::Unchanged {
        fs::write(dest, updated)?;
    }
    Ok((result, MembersSection::of(&kept)))
}

/// §FS-init.2.3.4.15.8: `block`, the current render, with its `### Workspace
/// members` section replaced by the bytes of the one the managed block in
/// `existing` holds, from that heading to the next, in whatever rendering wrote
/// them. `None` where `existing` has no block, its block has no such section, or
/// the render has none to put it in place of: the section is then rendered.
///
/// The cut is `grammar::block_sections`, the one `check` leaves the section out
/// by (§FS-check.3.5.5), so the bytes kept here are exactly the bytes it does not
/// compare. Nothing is copied between files: each entrypoint keeps its own.
pub(crate) fn keep_workspace_members(existing: &str, block: &str) -> Option<String> {
    let AgentsBlockLookup::Found(on_disk) = find_agents_block(existing) else {
        return None;
    };
    let kept = block_sections(&existing[on_disk.start..on_disk.end])
        .into_iter()
        .find(|section| section.noun() == WORKSPACE_MEMBERS)?;
    let rendered = block_sections(block);
    if !rendered
        .iter()
        .any(|section| section.noun() == WORKSPACE_MEMBERS)
    {
        return None;
    }
    let mut spliced = String::with_capacity(block.len() + kept.bytes.len());
    let mut body = 0;
    for section in rendered {
        body += section.bytes.len();
        spliced.push_str(if section.noun() == WORKSPACE_MEMBERS {
            kept.bytes
        } else {
            section.bytes
        });
    }
    // The sections partition everything before the end delimiter; the rest is it.
    spliced.push_str(&block[body..]);
    Some(spliced)
}

/// The pure string transform behind `update_agents_block`: splice the current
/// managed block into `existing`, preserving everything outside it byte-for-byte
/// — including the block's position and any CRLF endings (§FS-init.2.3.1,
/// §FS-init.2.3.2). Returns `Unchanged` when the splice would reproduce
/// `existing` exactly. A newer-than-supported block is an error.
pub(crate) fn update_agents_text(
    existing: &str,
    block: &str,
    label: &str,
) -> Result<(String, AgentsUpdateResult)> {
    match spliceable_agents_block(existing, label)? {
        Some(existing_block) => {
            // A legacy H2-bounded block is migrated to the delimited form by
            // this same splice (§FS-init.2.3.10): the replacement `block` carries
            // the delimiters, and the legacy span is what gets replaced.
            let mut updated = String::with_capacity(existing.len() + block.len());
            updated.push_str(&existing[..existing_block.start]);
            updated.push_str(block);
            updated.push_str(&existing[existing_block.end..]);
            let result = if updated == existing {
                AgentsUpdateResult::Unchanged
            } else {
                AgentsUpdateResult::Updated
            };
            Ok((updated, result))
        }
        None => {
            let separator = if existing.is_empty() || existing.ends_with("\n\n") {
                ""
            } else if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let mut updated = String::with_capacity(existing.len() + separator.len() + block.len());
            updated.push_str(existing);
            updated.push_str(separator);
            updated.push_str(block);
            Ok((updated, AgentsUpdateResult::Appended))
        }
    }
}

/// The one classification of an existing entrypoint's managed block, which both
/// the write pass and the pre-write pass ask (§FS-init.1.2.6): the block to
/// splice over, `None` when there is none, or the refusal for a malformed
/// (§FS-init.2.3.11) or newer (§FS-init.2.3.10.2) block.
fn spliceable_agents_block(existing: &str, label: &str) -> Result<Option<AgentsBlock>> {
    match find_agents_block(existing) {
        // §FS-init.2.3.11: splicing against broken delimiters risks eating user
        // content, so a malformed block is a hard error and the file is left
        // untouched.
        AgentsBlockLookup::Malformed { message, .. } => {
            Err(anyhow!("malformed grund managed block: {message}"))
        }
        AgentsBlockLookup::Found(existing_block) => {
            if existing_block.version > AGENTS_BLOCK_VERSION {
                return Err(anyhow!(
                    "{label} contains newer grund init block v{}; this binary supports v{}",
                    existing_block.version,
                    AGENTS_BLOCK_VERSION
                ));
            }
            Ok(Some(existing_block))
        }
        AgentsBlockLookup::Absent => Ok(None),
    }
}

/// §FS-init.1.2.6: read every planned entrypoint the write pass would update and
/// classify its block before the first write, so a refused block anywhere in the
/// plan refuses the whole run. Walks the plan in write order — the canonical
/// entrypoint (only when `--force` does not overwrite it), then each existing
/// companion — and returns the first refusal with the path it names and the
/// label the write pass would give it.
pub(super) fn refuse_planned_agent_blocks(
    target: &Path,
    plan: &SelectedInitAgentEntrypoints,
    force: bool,
) -> Option<(PathBuf, anyhow::Error)> {
    let canonical = (plan.canonical && !force)
        .then(|| target.join(CANONICAL_AGENT_ENTRYPOINT))
        .filter(|dest| path_exists(dest))
        .map(|dest| (dest, CANONICAL_AGENT_ENTRYPOINT.to_string()));
    let companions = plan
        .companions
        .iter()
        .filter_map(|entrypoint| match entrypoint {
            InitCompanionAgentEntrypoint::Existing(path) => {
                let rel = format_path(path.strip_prefix(target).unwrap_or(path));
                Some((path.clone(), rel))
            }
            InitCompanionAgentEntrypoint::MissingAlias(_) => None,
        });
    canonical
        .into_iter()
        .chain(companions)
        .find_map(|(path, label)| {
            let existing = match Sources::on_disk().text(&path) {
                Ok(existing) => existing,
                Err(err) => return Some((path, err.into())),
            };
            spliceable_agents_block(&existing, &label)
                .err()
                .map(|err| (path, err))
        })
}
