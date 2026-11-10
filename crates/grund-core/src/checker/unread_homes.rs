//! What this run could not read, kept for the one question a citation asks of it: may the run
//! call an ID absent from a kind's home (§FS-check.2.4)? Names only, and only the failures that
//! are this run's own `error:` lines, so that withholding a finding can never move the exit.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::reference_scope::{ScanScope, every_failure_name};
use crate::config::{Frame, Row};
use crate::model::{FailureNames, normalize_path_lexically};
use crate::resolver::{WorkspaceCheckTarget, WorkspaceProject};
use crate::scanner::ScanError;

/// §FS-check.2.4: every name of each failure this run reports as its own, by project root. A
/// mounted project reports no `error:` line here (§FS-imports.findings), so it has no entry.
#[derive(Default)]
pub(crate) struct UnreadNames {
    by_root: BTreeMap<PathBuf, Vec<PathBuf>>,
}

/// The record of a run with nothing to withhold, for a caller with no walk behind it.
static NOTHING_UNREAD: UnreadNames = UnreadNames {
    by_root: BTreeMap::new(),
};

impl UnreadNames {
    pub(crate) fn nothing() -> &'static Self {
        &NOTHING_UNREAD
    }

    /// §FS-check.2.4: the workspace arm's and the editor's record, one entry per reporting project.
    pub(crate) fn of_projects<'a>(
        projects: impl Iterator<Item = &'a WorkspaceProject>,
        report_scope: Option<&ScanScope>,
    ) -> Self {
        let mut unread = Self::default();
        for project in projects {
            unread.record(
                project.config.frame(),
                &project.scan_errors,
                &project.findings.failure_names,
                report_scope,
            );
        }
        unread
    }

    /// §FS-check.2.4, §FS-check.1.3.6.3: records `frame`'s failures that are inside the report
    /// scope, which [`ScanScope::holds_failure`] decides and is what earns the `error:` line. It
    /// reads them before `partition_scan_failures` renames them, because a failure's other names
    /// are found under the name the walk kept. The reason is only the key those names sit under.
    pub(crate) fn record(
        &mut self,
        frame: Frame<'_>,
        failures: &[ScanError],
        names: &FailureNames,
        report_scope: Option<&ScanScope>,
    ) {
        let own = failures
            .iter()
            .filter(|failure| {
                report_scope.is_none_or(|s| s.holds_failure(failure, names).is_some())
            })
            .flat_map(|failure| every_failure_name(failure, names, &frame.run.spellings));
        let recorded = self.by_root.entry(frame.root().to_path_buf()).or_default();
        recorded.extend(own);
    }

    /// §FS-check.2.4: whether a missing ID of `row`'s kind earns no finding in the project at
    /// `root`: the kind fetches and a recorded name is at or under its one home. Per component, so
    /// `values2` is not under `values`, and never upward: a home is a walk root of its own
    /// (§FS-config.3.5.8), so a failure above it does not leave it unread.
    pub(super) fn withholds(&self, root: &Path, row: &Row) -> bool {
        let (Some(home), Some(names)) = (row.fetch_home(), self.by_root.get(root)) else {
            return false;
        };
        let home = normalize_path_lexically(&root.join(home));
        names
            .iter()
            .any(|name| normalize_path_lexically(name).starts_with(&home))
    }

    /// §FS-check.2.4: whether a citation of `kind` that `target`'s catalog declares nowhere earns no
    /// finding, asked alike of a full ID and of a shorthand matching nothing (§FS-check.3.13.3).
    pub(super) fn withholds_absent(&self, target: &WorkspaceCheckTarget<'_>, kind: &str) -> bool {
        target
            .schema
            .fetching_row(kind)
            .is_some_and(|row| self.withholds(&target.run.root, row))
    }
}
