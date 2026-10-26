//! One `fmt` walk's plan (§FS-fmt.7.4.3): the new text of every file it
//! rewrites, worked out in both modes with nothing written, then the writes once
//! every refusal is behind the run. Between the two, the lock pass judges the tree
//! the plan leaves (§FS-section-lock.write.same-run); a workspace run plans every
//! project before it judges any project's locks, because a citation crosses into
//! a project whose walk comes after its own.

use anyhow::Result;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::fmt_complete_findings::CompleteScan;
use super::fmt_row::{FmtLockFile, FmtRow};
use super::fmt_section_lock::{JudgedTree, LockChange};
use super::fmt_tree::FmtTreeOutcome;
use crate::config::Config;
use crate::model::{Catalog, SECTION_LOCK_DIR};
use crate::scanner::{ApiScanError, section_locks_present};

/// What one walk worked out and has not written (§FS-fmt.7.4.3).
pub(super) struct TreePlan<'a> {
    /// The rewritten lines, the unreadable paths and the refused writes, as
    /// the run reports them; the lock rows join them at the write.
    pub(super) outcome: FmtTreeOutcome,
    /// Every file the walk rewrites, in walk order, in either mode: a dry run
    /// judges the locks of the same tree `--write` leaves (§FS-fmt.7.3).
    pub(super) planned: Vec<PlannedFile>,
    /// Whether a walked file, as the walk leaves it, may give the scan a
    /// heading whose address is locked once cited (`may_give_locked_heading`).
    pub(super) relative_seen: bool,
    /// Whether the walk covered the whole project (§FS-section-lock.write.rules).
    pub(super) root_scope: bool,
    /// The findings the walk read, if it read any.
    pub(super) read: WalkRead<'a>,
}

/// The findings a walk read: none, the caller's, or a scan it took itself.
pub(super) enum WalkRead<'a> {
    Unread,
    Given(&'a Catalog),
    Scanned(CompleteScan),
}

/// One file's rewrite as the walk planned it: its new text, and the range of
/// the run's changes it accounts for, so a write that fails can report exactly
/// the files written before it.
pub(super) struct PlannedFile {
    pub(super) path: PathBuf,
    pub(super) text: String,
    pub(super) changes: Range<usize>,
}

impl TreePlan<'_> {
    /// The tree this walk's lock pass judges (§FS-section-lock.write.same-run), or
    /// `None` where there is no lock pass: a walk narrower than the whole project,
    /// and one that read no findings where no walked file may give a locked
    /// heading and no lock file exists (§FS-section-lock.write.rules).
    pub(super) fn judged<'p>(
        &'p self,
        config: &'p Config,
        render: &Config,
    ) -> Result<Option<JudgedTree<'p>>> {
        let read = match &self.read {
            WalkRead::Unread => None,
            WalkRead::Given(findings) => Some(*findings),
            WalkRead::Scanned(scan) => Some(scan.findings()),
        };
        if !self.root_scope
            || (read.is_none() && !self.relative_seen && !section_locks_present(config.frame()))
        {
            return Ok(None);
        }
        let left = self.planned.iter();
        let left = left.map(|file| (file.path.clone(), file.text.clone()));
        JudgedTree::of(config, render, read, left.collect()).map(Some)
    }

    /// Carry the plan out (§FS-fmt.7.4.3): the walk and the lock pass have both
    /// returned, so no refusal is left to meet. Under `write` the files go to
    /// disk in walk order and `locks` after them; either way the lock rows join
    /// the run's changes after the files'.
    pub(super) fn write(
        self,
        config: &Config,
        render: &Config,
        locks: Vec<LockChange>,
        write: bool,
    ) -> Result<FmtTreeOutcome> {
        let FmtTreeOutcome {
            mut changes,
            scan_errors,
            refused_writes,
        } = self.outcome;
        // A dry run writes nothing and reports the same rows (§FS-fmt.7.3).
        let to_write: &[PlannedFile] = if write { &self.planned } else { &[] };
        for file in to_write {
            let written = |error| {
                partial_io_error(
                    render,
                    &file.path,
                    "write",
                    error,
                    &changes[..file.changes.start],
                    &scan_errors,
                    &refused_writes,
                )
            };
            // §FS-distribution.3.2.2.1: a failed write still reports the files written before it.
            fs::write(&file.path, &file.text).map_err(written)?;
            #[cfg(feature = "test-binding-writes")]
            if let Err(error) = super::binding_write_fault::after_write() {
                return Err(partial_io_error(
                    render,
                    &file.path,
                    "write",
                    error,
                    &changes[..file.changes.end],
                    &scan_errors,
                    &refused_writes,
                ));
            }
        }
        for lock in locks {
            if write && let Err(error) = write_lock(config, &lock) {
                return Err(partial_io_error(
                    render,
                    &lock.path,
                    "write",
                    error,
                    &changes,
                    &scan_errors,
                    &refused_writes,
                ));
            }
            // §FS-section-lock.write.report: the same test `write_lock` acts on.
            let lock_file = FmtLockFile::of(lock.exists, &lock.text);
            changes.extend(FmtRow::lock_rows(&lock.path, lock.rows, lock_file));
        }
        Ok(FmtTreeOutcome {
            changes,
            scan_errors,
            refused_writes,
        })
    }
}

/// Bring a lock file to `lock.text` (§FS-section-lock.file.shard): one with no
/// entry is deleted, and with it each directory under `.grund/sections/` it
/// leaves empty, that one included.
fn write_lock(config: &Config, lock: &LockChange) -> std::io::Result<()> {
    if !lock.text.is_empty() {
        if let Some(dir) = lock.path.parent() {
            fs::create_dir_all(dir)?;
        }
        return fs::write(&lock.path, &lock.text);
    }
    if lock.exists {
        fs::remove_file(&lock.path)?;
    }
    let sections = config.root.join(SECTION_LOCK_DIR);
    let under = |dir: &&Path| dir.starts_with(&sections);
    for dir in lock.path.ancestors().skip(1).take_while(under) {
        let sources = config.frame().sources();
        if !sources
            .read_dir(dir)
            .is_ok_and(|mut entries| entries.next().is_none())
        {
            break;
        }
        fs::remove_dir(dir)?;
    }
    Ok(())
}

pub(super) fn partial_io_error(
    render: &Config,
    path: &Path,
    verb: &str,
    error: std::io::Error,
    changes: &[FmtRow],
    scan_errors: &[ApiScanError],
    refused_writes: &[String],
) -> anyhow::Error {
    let mut diagnostic = crate::model::OperationDiagnostic::filesystem(
        path,
        &error,
        format!("{verb} {}", path.display()),
    );
    diagnostic.partial_output = serde_json::json!({"changes":changes.iter().map(|row|
            row.partial_json(render)).collect::<Vec<_>>(),
            "scan_errors":scan_errors.iter().map(|e|serde_json::json!({"path":e.path,"message":e.message})).collect::<Vec<_>>(),
            "refused_writes":refused_writes});
    anyhow::Error::new(error).context(diagnostic)
}
