//! The init options and report vocabulary (§FS-init.1, §FS-init.4.1).
use super::init_guidance::InitNext;
use super::init_plan::InitAgentEntrypointSelection;
use crate::model::Finding;
use std::path::PathBuf;

#[derive(Clone)]
pub struct InitOpts {
    pub target: PathBuf,
    /// Explicit generated project identity. When absent, `init` uses the
    /// target-local configured name before the basename (§FS-init.2.3.8).
    pub name: Option<String>,
    /// `--description` — pending one-line `project_description` for a freshly
    /// written config (§FS-init.1, §DF-workspace-member-descriptions).
    pub description: Option<String>,
    pub docs: bool,
    pub force: bool,
    pub dry_run: bool,
    /// `--check` — the `--dry-run` preview taken as a verdict (§FS-init.1):
    /// writes nothing, reports what `--dry-run` reports, and leaves the caller
    /// to exit `1` when any reported event is a change (§FS-init.4.1). It implies
    /// `dry_run` inside `init` rather than opening a second path through it.
    pub check: bool,
    /// `--no-vcs` — scaffold into a target no version-control marker covers
    /// (§FS-init.1.2.3). Lifts that rule and only that one; it is not `--force`,
    /// which decides whether files `init` owns get overwritten (§FS-init.3).
    pub no_vcs: bool,
    pub agent_selection: InitAgentEntrypointSelection,
}

impl Default for InitOpts {
    fn default() -> Self {
        Self {
            target: PathBuf::from("."),
            name: None,
            description: None,
            docs: false,
            force: false,
            dry_run: false,
            check: false,
            no_vcs: false,
            agent_selection: InitAgentEntrypointSelection::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitEvent {
    pub verb: &'static str,
    pub path: String,
}

impl InitEvent {
    /// Whether this event reports work rather than a path that was already
    /// current. Every verb but `exists` is a change — `wrote`/`appended`/
    /// `updated` and their `would-` forms alike. The one definition of the
    /// predicate: it suppresses the `next:` block (§FS-init.2.2.2) and it decides
    /// the `--check` exit code (§FS-init.4.1), which is why those two agree.
    pub fn is_change(&self) -> bool {
        self.verb != "exists"
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InitOutput {
    pub events: Vec<InitEvent>,
    /// Located validation findings discovered before any write (§FS-rules.4).
    /// They are ordinary report rows and make `init` exit 1, not operational
    /// failures that would exit 2.
    pub errors: Vec<Finding>,
    /// Things the run could not do that the caller would otherwise have to
    /// notice for itself (§FS-init.2.3.4.17.4). Reported, never fatal.
    pub notes: Vec<String>,
    pub next: Option<InitNext>,
    /// The run's warning channel (§FS-distribution.3.1): the `[workspace]`
    /// cautions the walk-up settled — §FS-check.4.10's unread opted-out block
    /// and §FS-workspace.6.1.7.5's undecidable ancestor claim; an absorbed scan
    /// fails the expansion instead, which leaves the section out
    /// (§FS-check.3.30.2). `init` expands the outermost workspace above
    /// its target to teach the alias set, so it resolves a block's member
    /// boundary like every other walking command and owes the reader the same
    /// lines (§FS-check.2.1.1).
    pub warnings: Vec<Finding>,
}

impl InitOutput {
    /// Whether the run reported anything left to do — the verdict `--check`
    /// draws from the report it just printed (§FS-init.4.1). Notes and the
    /// `next:` block are deliberately not consulted: a note is a report, not a
    /// finding.
    pub fn has_pending_changes(&self) -> bool {
        self.events.iter().any(InitEvent::is_change)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitError {
    pub output: InitOutput,
    pub message: String,
}

impl InitError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            output: InitOutput::default(),
            message: message.into(),
        }
    }

    pub(super) fn with_events(events: Vec<InitEvent>, message: impl Into<String>) -> Self {
        Self {
            output: InitOutput {
                events,
                ..InitOutput::default()
            },
            message: message.into(),
        }
    }
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for InitError {}
