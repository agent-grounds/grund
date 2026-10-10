//! What a stage below the writers is handed beside the concerns it reads
//! (§AR-checker.1, §DA-config-concern-records.2.3): the `Run` and the
//! `Compiled` grammar of §AR-config.1.5, which are not concerns, and how this
//! run names the project and spells a path.
//!
//! The spelling is presentation's (`[output] relative_paths`, §FS-config.3.6)
//! outranked by the run's `--path-base` (§FS-cli.3.4), so the façade settles it
//! before the frame is handed down: a stage spells a path through `Display`
//! and never reads the key (§AR-config.5).

use std::path::Path;

use super::compiled::Compiled;
use super::run::Run;
use crate::grammar::Grammar;
use crate::model::{format_path, relative_from_base};

/// §AR-checker.1: the frame a stage runs in. `Copy`, so a stage hands it down
/// as freely as it would a reference.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    pub run: &'a Run,
    pub compiled: &'a Compiled,
    /// The project's stable `name`, the scope a standalone rule adaptation
    /// selects (§AR-rules.3); `None` where the project names none.
    pub name: Option<&'a str>,
    /// The alias a workspace run checks this project under, `None` outside one
    /// (§FS-workspace.8.1): a finding spells its own coordinate qualified by it.
    pub alias: Option<&'a str>,
    /// The format version that spelled the project's file (§FS-config.5): a
    /// finding that names a config table spells it the way that file does
    /// (§FS-config-v2.rules.citations).
    pub version: u32,
    /// How this run's report spells a path — the report root's, which in a
    /// workspace is not this project's (§FS-workspace.8.1).
    pub display: Display<'a>,
}

/// How a report spells a path (§FS-config.3.6.1, §FS-cli.3.4): from the config
/// root, or from the CLI base with bounded `..` for an in-root target outside it.
#[derive(Clone, Copy)]
pub struct Display<'a> {
    pub root: &'a Path,
    pub cli_base: &'a Path,
    pub from_root: bool,
}

impl<'a> Frame<'a> {
    /// The config root (§FS-config.1).
    pub fn root(&self) -> &'a Path {
        &self.run.root
    }

    /// The compiled ID grammar (§AR-config.1.5).
    pub fn grammar(&self) -> &'a Grammar {
        &self.compiled.grammar
    }

    /// This frame with the report spelled as `display` spells it — a member
    /// checked inside a workspace run (§FS-workspace.8.1).
    pub fn displayed_by(self, display: Display<'a>) -> Self {
        Self { display, ..self }
    }

    /// This frame for the workspace member checked as `alias`
    /// (§FS-workspace.8.1).
    pub fn checked_as(self, alias: Option<&'a str>) -> Self {
        Self { alias, ..self }
    }

    /// `path` as this run's report spells it (§FS-config.3.6.1).
    pub fn display_path(&self, path: &Path) -> String {
        self.display.path(path)
    }
}

impl Display<'_> {
    /// The directory every reported path is spelled from (§FS-cli.3.4).
    pub fn base(&self) -> &Path {
        if self.from_root {
            self.root
        } else {
            self.cli_base
        }
    }

    /// Render a path the way reports show it: relative to the repo root by
    /// default, or relative to the CLI base directory under
    /// `--path-base=invocation` or `[output] relative_paths = false`
    /// (§FS-config.3.6.1 — an in-root target outside that base uses bounded `..`).
    pub fn path(&self, path: &Path) -> String {
        let base = self.base();
        let relative = path
            .strip_prefix(base)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| {
                if !self.from_root && path.starts_with(self.root) {
                    relative_from_base(base, path)
                } else {
                    path.to_path_buf()
                }
            });
        format_path(&relative)
    }
}
