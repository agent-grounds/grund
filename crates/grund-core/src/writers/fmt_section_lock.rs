//! The lock pass of `fmt` (§FS-section-lock.write): what each section lock file
//! must become for the tree — add, refresh, drop, and rebuild a file grund cannot
//! read — from the one judgement `check` reports from, so the write that clears a
//! finding is the finding's own answer. Run only over a whole project, and against
//! the tree as the run leaves it rather than as it read it
//! (§FS-section-lock.write.same-run); the tree pass writes what it returns, as it
//! writes every file `fmt` rewrites.

use anyhow::Result;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::fmt_complete_findings::CompleteScan;
use crate::config::Config;
use crate::model::{Catalog, render_lock};
use crate::resolver::{EntryVerdict, WorkspaceCheckTarget, WorkspaceContext, judge_section_locks};

/// One lock file the pass changes: the text it is brought to, empty for a file
/// to delete, whether it is there now, and its rows as `(line, row)`.
pub(super) struct LockChange {
    pub(super) path: PathBuf,
    pub(super) text: String,
    pub(super) exists: bool,
    pub(super) rows: Vec<(usize, String)>,
}

/// The tree a project's lock pass judges (§FS-section-lock.write.same-run): the
/// project's findings as the run leaves it, and the config that reads it so.
pub(super) struct JudgedTree<'a> {
    config: Cow<'a, Config>,
    findings: JudgedFindings<'a>,
}

enum JudgedFindings<'a> {
    Read(&'a Catalog),
    Scanned(CompleteScan),
}

impl<'a> JudgedTree<'a> {
    /// The tree the lock pass judges `config`'s project by, once its walk has
    /// worked out `left`, the new text of every file it rewrites
    /// (§FS-section-lock.write.same-run). Where it rewrites one, the project is
    /// scanned again under a run whose buffers hold those texts in front of its
    /// own (§AR-system.2.13), so a citation the run writes is entered in that run
    /// and a section it rewrites is fingerprinted as rewritten. Where it rewrites
    /// none, that is the tree it read: `read`, the findings the walk already
    /// holds, or a scan taken now where it holds none, so a run with nothing to
    /// rewrite scans no more than it did (§GOAL-fast-feedback). Either scan
    /// refuses as the cross-reference pass does (§FS-fmt.3.2), and is taken before
    /// anything is written (§FS-fmt.7.4.3).
    pub(super) fn of(
        config: &'a Config,
        render: &Config,
        read: Option<&'a Catalog>,
        left: Vec<(PathBuf, String)>,
    ) -> Result<Self> {
        if left.is_empty() {
            let findings = match read {
                Some(read) => JudgedFindings::Read(read),
                None => JudgedFindings::Scanned(CompleteScan::of_tree_or_abort(config, render)?),
            };
            return Ok(Self {
                config: Cow::Borrowed(config),
                findings,
            });
        }
        let config = config.with_buffers(config.run().buffers.in_front(left));
        let scan = CompleteScan::of_tree_or_abort(&config, render)?;
        Ok(Self {
            config: Cow::Owned(config),
            findings: JudgedFindings::Scanned(scan),
        })
    }

    fn findings(&self) -> &Catalog {
        match &self.findings {
            JudgedFindings::Read(findings) => findings,
            JudgedFindings::Scanned(scan) => scan.findings(),
        }
    }
}

/// Every project of `workspace` a citation into it is judged against, under its
/// alias (§FS-section-lock.file.members): as this run leaves it where `judged`
/// holds the tree at the project's index, and as it was loaded where the run
/// judges none of its locks (§FS-section-lock.write.same-run).
pub(super) fn lock_targets<'a>(
    workspace: Option<&'a WorkspaceContext>,
    judged: &'a [Option<JudgedTree<'_>>],
) -> BTreeMap<String, WorkspaceCheckTarget<'a>> {
    let Some(context) = workspace else {
        return BTreeMap::new();
    };
    let loaded = context.projects.iter().enumerate();
    loaded
        .map(|(index, project)| {
            let target = match judged.get(index).and_then(Option::as_ref) {
                Some(tree) => {
                    let config = &tree.config;
                    WorkspaceCheckTarget::of(tree.findings(), config.schema(), config.frame())
                }
                None => {
                    let (schema, frame) = (project.config.schema(), project.config.frame());
                    WorkspaceCheckTarget::of(&project.findings, schema, frame)
                }
            };
            (project.alias.clone(), target)
        })
        .collect()
}

/// The lock files `judged`'s project changes: each with one row per entry
/// `--write` adds, refreshes or drops and per line it rebuilds, ordered by line,
/// then text (§FS-section-lock.write.report). A citation into another project is
/// judged against `targets`, by its alias (§FS-section-lock.file.members).
pub(super) fn section_lock_changes(
    judged: &JudgedTree<'_>,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> Vec<LockChange> {
    let (config, findings) = (&judged.config, judged.findings());
    let mut changes = Vec::new();
    for judgement in judge_section_locks(findings, config.schema(), config.frame(), targets) {
        let coordinate = |section: &str| judgement.coordinate(section);
        let written = judgement.written_entries();
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
                EntryVerdict::Drop | EntryVerdict::Gone { .. } => "drop",
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
    changes
}
