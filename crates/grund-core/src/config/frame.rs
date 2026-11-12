//! What a stage below the writers is handed beside the concerns it reads
//! (§AR-checker.1, §DA-config-concern-records.2.3): the `Run` and the
//! `Compiled` grammar of §AR-config.1.5, which are not concerns, and how this
//! run names the project and spells a path.
//!
//! The spelling is presentation's (`[output] relative_paths`, §FS-config.3.6)
//! outranked by the run's `--path-base` (§FS-cli.3.4), so the façade settles it
//! before the frame is handed down: a stage spells a path through `Display`
//! and never reads the key (§AR-config.5).

use crate::sources::Sources;
use std::path::{Path, PathBuf};

use super::compiled::{Compiled, RefusedHome};
use super::project::Schema;
use super::project::file_extension;
use super::run::Run;
use crate::grammar::Grammar;
use crate::model::{Refused, format_path, is_hidden, relative_from_base, unread_reason};

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
    /// The canonical roots of the mounts the run loaded: a file under one lies
    /// outside the root and is still reported relative to the base, with `..`
    /// (§FS-imports.paths, §FS-config.3.6).
    pub(crate) mounted: &'a [PathBuf],
}

impl<'a> Frame<'a> {
    /// What every reader of this run asks for a file's text and existence: the
    /// run's buffers in front of the disk (§AR-system.2.13).
    pub(crate) fn sources(&self) -> Sources<'a> {
        Sources::of(&self.run.buffers)
    }

    /// The config root (§FS-config.1).
    pub fn root(&self) -> &'a Path {
        &self.run.root
    }

    /// The home of `kind` under `root` that `fetch` refuses and why, `None` where it writes. A
    /// `file` home: a whole-value kind's first place, when a value home reads that file as a
    /// value source (§FS-fetch.4, §FS-values.terms); any kind's, when the scan does not read it
    /// (§FS-fetch.4.2.1); and any kind's, when the text scan reads it as source (§FS-fetch.4.2).
    /// A `folder` home: the `<ID>.md` for `unclaimed`, the local ID no file in the home
    /// declares, when the scan does not read it as Markdown (§FS-fetch.4.2.1); the caller's scan
    /// says which ID that is, and `None` asks nothing of a folder. The path and the kind decide,
    /// and no file is read. The walk's own predicate, which reads ignore files, is asked after
    /// this by `scanner::fetch_refused_home`, the one function the refusal and the remedy that
    /// must not name the refused command both ask (§FS-check.3.1.3.1).
    pub(crate) fn fetch_refused_by_name(
        &self,
        schema: &Schema,
        root: &Path,
        kind: &str,
        unclaimed: Option<&str>,
    ) -> Option<(PathBuf, RefusedHome)> {
        let row = schema.rows.iter().find(|row| row.name == kind)?;
        let Some(file) = row.file() else {
            let home = self.fetch_snapshot_path(schema, root, kind, unclaimed?)?;
            let reason = self.unread_reason(schema, &home).or_else(|| {
                let reading = self.reading(schema, &home);
                let language = reading.language.map(|language| language.name.as_str());
                (!reading.markdown).then(|| match language {
                    Some(language) => {
                        format!("the scan reads `.md` as {language}, not as Markdown")
                    }
                    None => "the scan does not read `.md` as Markdown".to_string(),
                })
            })?;
            return Some((home, RefusedHome::Unread(reason)));
        };
        let home = root.join(file);
        let reading = self.reading(schema, &home);
        let value_kind = schema.value_rows().any(|row| row.name == kind);
        let reason = if value_kind && reading.value_format().is_some() {
            RefusedHome::ValueSource
        } else if let Some(reason) = self.unread_reason(schema, &home) {
            RefusedHome::Unread(reason)
        } else if !reading.read_as_source() {
            return None;
        } else if value_kind {
            RefusedHome::SourceFileValue
        } else {
            RefusedHome::SourceFile
        };
        Some((home, reason))
    }

    /// The `<ID>.md` a `folder` home's fetch of `local` creates (§FS-fetch.5), `None` for a kind
    /// with no folder home.
    pub(crate) fn fetch_snapshot_path(
        &self,
        schema: &Schema,
        root: &Path,
        kind: &str,
        local: &str,
    ) -> Option<PathBuf> {
        let row = schema.rows.iter().find(|row| row.name == kind)?;
        Some(root.join(row.folder()?).join(format!("{local}.md")))
    }

    /// Why the walk's name rules keep `path` out of the scan, in the words of `check`'s unread
    /// warning (§FS-scan.checks.unread-scan-root.message), `None` where they read it.
    fn unread_reason(&self, schema: &Schema, path: &Path) -> Option<String> {
        let refused = Refused {
            hidden: is_hidden(path),
            extension: !schema.sources.lists_extension(path),
        };
        (refused.hidden || refused.extension)
            .then(|| unread_reason(refused, file_extension(path), self.version))
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

    /// `path` as this run's report spells it, the report base `.` (§AR-config.1.7).
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

    /// Whether `path` is the report base itself, asked of the path and never of its
    /// spelling (§AR-config.1.7): what a message that leaves the base unnamed asks.
    pub fn is_base(&self, path: &Path) -> bool {
        path == self.base()
    }

    /// The one spelling of a path a report names (§AR-config.1.7): relative to the repo
    /// root by default, or relative to the CLI base directory under
    /// `--path-base=invocation` or `[output] relative_paths = false` (§FS-config.3.6.1 —
    /// an in-root target outside that base uses bounded `..`), and the report base
    /// itself `.`, never the empty string (§FS-cli.3.4).
    pub fn path(&self, path: &Path) -> String {
        let base = self.base();
        // §FS-imports.paths: a mounted file is the second exception to "never above the
        // report base" (§FS-config.3.6).
        if path.starts_with(base)
            || (!self.from_root && path.starts_with(self.root))
            || self.mounted.iter().any(|root| path.starts_with(root))
        {
            report_path(base, path)
        } else {
            format_path(path)
        }
    }
}

/// `path` as a report names it from `base`, with `..` for a target outside it and `base`
/// itself `.`: the one spelling of a reported path (§AR-config.1.7, §FS-cli.3.4), which
/// [`Display::path`] gives a run and a reader that holds only a base asks directly.
pub(crate) fn report_path(base: &Path, path: &Path) -> String {
    let spelled = format_path(&relative_from_base(base, path));
    if spelled.is_empty() {
        ".".to_owned()
    } else {
        spelled
    }
}
