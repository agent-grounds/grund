//! The lock pass of `fmt` (§FS-section-lock.write): what each section lock file
//! must become for the tree — add, refresh, drop, and rebuild a file grund cannot
//! read — from the one judgement `check` reports from, so the write that clears a
//! finding is the finding's own answer. Run only over a whole project; the tree
//! pass writes what it returns, as it writes every file `fmt` rewrites.

use anyhow::Result;
use std::path::PathBuf;

use super::fmt_complete_findings::CompleteScan;
use crate::config::Config;
use crate::model::{Catalog, render_lock};
use crate::resolver::{EntryVerdict, judge_section_locks};
use crate::scanner::section_locks_present;

/// One lock file the pass changes: the text it is brought to, empty for a file
/// to delete, whether it is there now, and its rows as `(line, row)`.
pub(super) struct LockChange {
    pub(super) path: PathBuf,
    pub(super) text: String,
    pub(super) exists: bool,
    pub(super) rows: Vec<(usize, String)>,
}

/// The lock files a run over `config`'s whole project changes, and nothing when
/// the walk's scope was narrower (`root_scope`, §FS-section-lock.write.rules):
/// each with one row per entry `--write` adds, refreshes or drops and per line it
/// rebuilds, ordered by line, then text (§FS-section-lock.write.report). The
/// project's findings are `findings` where the run has them; otherwise it is
/// scanned whole, refusing as the cross-reference pass does (§FS-fmt.3.2), and
/// only where a lock file exists or a walked file may hold a relative heading.
pub(super) fn section_lock_changes(
    config: &Config,
    render: &Config,
    root_scope: bool,
    findings: Option<&Catalog>,
    relative_seen: bool,
) -> Result<Vec<LockChange>> {
    if !root_scope
        || (findings.is_none() && !relative_seen && !section_locks_present(config.frame()))
    {
        return Ok(Vec::new());
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
    let mut changes = Vec::new();
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
        changes.push(LockChange {
            path: judgement.path.clone(),
            text,
            exists: judgement.text.is_some(),
            rows,
        });
    }
    Ok(changes)
}
