//! The lock pass of `fmt` (§FS-section-lock.write): bring each section lock file
//! to what the tree needs — add, refresh, drop, and rebuild a file grund cannot
//! read — from the one judgement `check` reports from, so the write that clears a
//! finding is the finding's own answer. Run only over a whole project.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::fmt_complete_findings::CompleteScan;
use crate::config::Config;
use crate::model::{Catalog, SECTION_LOCK_DIR, render_lock};
use crate::resolver::{EntryVerdict, judge_section_locks};

/// Whether `text` holds a line that could be a relative heading
/// (§FS-declarations.section.relative.form): what makes a tree with no lock
/// directory worth a scan for the lock pass at all.
pub(super) fn may_hold_relative_heading(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start_matches(' ');
        let text = line.trim_start_matches('#');
        text.len() < line.len()
            && text.starts_with([' ', '\t'])
            && text.trim_start_matches([' ', '\t']).starts_with('.')
    })
}

/// Run the lock pass over `config`'s project when `scope` is its config root
/// (§FS-section-lock.write.rules): append one row per entry `--write` adds,
/// refreshes or drops and per line it rebuilds, ordered by path, line, then text
/// (§FS-section-lock.write.report), and with `write` bring each file there. The
/// project's findings are `findings` where the run has them; otherwise it is
/// scanned whole, refusing as the cross-reference pass does (§FS-fmt.3.2), and
/// only where a lock directory exists or a walked file may hold a relative heading.
pub(super) fn fmt_section_locks(
    config: &Config,
    render: &Config,
    scope: Option<&Path>,
    findings: Option<&Catalog>,
    relative_seen: bool,
    write: bool,
    changes: &mut Vec<(PathBuf, usize, String)>,
) -> Result<()> {
    let root = &config.root;
    let at_root =
        scope.is_none_or(|scope| fs::canonicalize(scope).ok() == fs::canonicalize(root).ok());
    let lock_dir = root.join(SECTION_LOCK_DIR);
    if !at_root || (findings.is_none() && !relative_seen && !lock_dir.is_dir()) {
        return Ok(());
    }
    let scanned;
    let findings = match findings {
        Some(findings) => findings,
        None => {
            scanned = CompleteScan::of_tree_or_abort(config, render)?;
            scanned.findings()
        }
    };
    let separator = &config.schema().ids.section_separator;
    for judgement in judge_section_locks(findings, config.schema(), config.frame()) {
        let coordinate = |section: &str| format!("{}{separator}{section}", judgement.rendered);
        let written = judgement.written_entries(separator);
        let text = render_lock(&written);
        let mut rows = judgement
            .lines
            .invalid
            .iter()
            .map(|line| (*line, "section lock: rebuild".to_string()))
            .collect::<Vec<_>>();
        for (entry, verdict) in &judgement.verdicts {
            let verb = match verdict {
                EntryVerdict::Refresh(_) => "refresh",
                EntryVerdict::Drop => "drop",
                // §FS-section-lock.write.never-a-move: a move stays reported.
                EntryVerdict::Keep | EntryVerdict::Moved { .. } => continue,
            };
            rows.push((
                entry.line,
                format!("section lock: {verb} {}", coordinate(&entry.section)),
            ));
        }
        // An add is reported at the line it occupies after the write.
        for (section, _, _) in &judgement.unrecorded {
            let added = coordinate(section);
            let line = text
                .lines()
                .position(|line| line.split_once(' ').is_some_and(|(at, _)| at == added))
                .map_or(1, |index| index + 1);
            rows.push((line, format!("section lock: add {added}")));
        }
        if rows.is_empty() {
            continue;
        }
        rows.sort();
        if write {
            write_lock(&judgement.path, judgement.text.is_some(), &text)?;
        }
        changes.extend(
            rows.into_iter()
                .map(|(line, row)| (judgement.path.clone(), line, row)),
        );
    }
    Ok(())
}

/// Write `text` as the lock at `path`; an empty one is deleted, and with it a
/// `.grund/sections/` left empty (§FS-section-lock.file.shard).
fn write_lock(path: &Path, exists: bool, text: &str) -> Result<()> {
    if !text.is_empty() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, text)?;
        return Ok(());
    }
    if exists {
        fs::remove_file(path)?;
    }
    if let Some(dir) = path.parent()
        && fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none())
    {
        fs::remove_dir(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::may_hold_relative_heading;

    #[test]
    fn only_a_dotted_heading_asks_for_the_lock_pass() {
        assert!(may_hold_relative_heading("# FS-a: A\n\n### .1: Username\n"));
        assert!(!may_hold_relative_heading(
            "# FS-a: A\n\n## 1. One\n.1 is prose\n"
        ));
        assert!(!may_hold_relative_heading("#.1 not a heading\n"));
    }
}
