//! §AR-config.1.5: what one invocation decided and what workspace expansion
//! learned about this checkout. No `Run` field is read from a file, so nothing
//! here is a key and nothing here is lowered; a command-line override of a
//! file's value is a `Run` fact the façade applies over the `Project`'s.
//!
//! What a command decides arrives as an `Invocation`, handed to `open()` in the
//! workspace component, which builds every `Run` from it once
//! (§AR-config.1.5); nothing assigns a scope field of a built `Run`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::call_scope::{PathBase, report_path_base};
use super::kinds::KindTable;
use super::record::AbsentOptionalNamespace;
use super::run_warnings::RunWarning;
use crate::sources::Buffers;

/// §AR-config.1.5: one invocation's facts about one project.
#[derive(Clone)]
pub struct Run {
    /// The config root (§FS-config.1).
    pub root: PathBuf,
    /// The base reports use when `[output] relative_paths = false`
    /// (§FS-config.3.6.1).
    pub cli_base: PathBuf,
    /// The config file this run read, as a report path (§FS-config.1).
    pub config_file: Option<PathBuf>,
    /// The file the read one outranks (§FS-config.1.1).
    pub redundant_config_file: Option<PathBuf>,
    /// Whether discovery found no config at or above where it started, so the
    /// run is held to the built-in defaults (§FS-check.2.2.2.5). Not the same as
    /// `config_file` being `None`: a workspace member with no file of its own
    /// is loaded under its workspace's config, which this run did read.
    pub(crate) found_no_config: bool,
    /// The directory the load spelled every relative location it recorded from:
    /// `config_file`, `redundant_config_file` and a `grund.toml` row's
    /// (§FS-errors.4), kept so a reader joins one back where it was spelled.
    pub(crate) location_base: PathBuf,
    pub scope: RunScope,
    pub workspace: RunWorkspace,
    /// The run's warning channel (§FS-distribution.3.1).
    pub(crate) warnings: Vec<RunWarning>,
    /// The run's `--path-base`, when the caller passed one: it outranks
    /// `relative_paths` for every project the run loads (§FS-cli.3.4). Taken
    /// from the call scope when the run is made, so members carry it too.
    pub(crate) path_base: Option<PathBase>,
    /// The editor's open documents, keyed by physical path, which every reader
    /// of this run reads as the files they name (§AR-system.2.13).
    pub(crate) buffers: Buffers,
}

/// §AR-config.1.5: the scope this invocation asked for.
#[derive(Clone)]
pub struct RunScope {
    /// `grund check --full` (§FS-check.1.3).
    pub full: bool,
    /// An explicit path below the config root (§FS-check.1.3.6.1).
    pub resolution_wide: bool,
    /// Whether the scan classifies citing sides (§AR-scanner.2.4).
    pub classify_citation_sources: bool,
    /// The `cover --lines` ranges (§FS-cover.6.1).
    pub owner_lines: Vec<(usize, usize)>,
    /// `grund check --require-grounding`, which turns the `[reference]` default
    /// on for this run and never off (§FS-check.3.6).
    pub require_grounding: bool,
}

/// §AR-config.1.5: what expansion learned about the workspace around this
/// project (§AR-workspace.6, §FS-workspace.6.1.5, §FS-check.4.9).
#[derive(Clone, Default)]
pub struct RunWorkspace {
    pub boundary_roots: Vec<PathBuf>,
    pub project_roots: Vec<PathBuf>,
    pub scope_path: String,
    pub absent_optional: Vec<AbsentOptionalNamespace>,
    /// Whether any `[workspace]` block expansion walked lists an optional member,
    /// present or absent, which arms the absence check (§FS-check.1.4.1).
    pub(crate) lists_optional: bool,
    /// The run's kind table, built once where the workspace is expanded
    /// (§AR-config.1.5); `None` in a run that expanded none, whose
    /// table is its one project's.
    pub(crate) kinds: Option<Arc<KindTable>>,
}

impl Default for RunScope {
    /// No path, no flag: the configured scope, the citing side classified,
    /// which every read-only command turns off (§AR-scanner.2.4, §AR-benchmarks).
    fn default() -> Self {
        Self {
            full: false,
            resolution_wide: false,
            classify_citation_sources: true,
            owner_lines: Vec::new(),
            require_grounding: false,
        }
    }
}

/// §AR-config.1.5: which projects a command loads and what its path bounds for
/// resolution (§FS-scan.path, §FS-scan.projects). Each command keeps its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Selection {
    /// The config discovery finds for the path, with no member rewrite and no
    /// boundary: what `config show` reads and the embedding preflight climbs.
    Discovery,
    /// The project the path belongs to, resolved within its tree: `refs`,
    /// `cover`, `fmt` and the editor's lookups.
    Project,
    /// `check`, `show`, `list` and `id`: below the root the path bounds the
    /// report, never resolution (§FS-scan.path, §FS-check.1.3.6.1).
    Check,
    /// The editor snapshot: `Check`'s two scopes in a project with no
    /// `[workspace]` (§AR-lsp.5.1.1), and an explicit zero-config folder is the
    /// project's root (§FS-lsp.2.2.1).
    Editor,
}

/// §AR-config.1.5: what one invocation decided, the one argument every `Run` is
/// built from: the path and what the command selects with it, its flags,
/// `--path-base` and the editor's buffers.
#[derive(Clone)]
pub(crate) struct Invocation {
    pub(crate) path: PathBuf,
    pub(crate) path_provided: bool,
    pub(crate) selection: Selection,
    /// Every scope fact but `resolution_wide`, which `selection` derives once
    /// the root is known.
    pub(crate) scope: RunScope,
    pub(crate) path_base: Option<PathBase>,
    pub(crate) buffers: Buffers,
}

impl Invocation {
    /// A command's invocation over `path`, carrying the `--path-base` its caller
    /// scoped (§FS-cli.3.4): read here, once, where the binding's call becomes
    /// an argument, and nowhere a `Run` is built.
    pub(crate) fn new(path: &Path, path_provided: bool, selection: Selection) -> Self {
        Self {
            path: path.to_path_buf(),
            path_provided,
            selection,
            scope: RunScope::default(),
            path_base: report_path_base(),
            buffers: Buffers::default(),
        }
    }

    /// `grund check --full` (§FS-check.1.3).
    pub(crate) fn full(mut self, full: bool) -> Self {
        self.scope.full = full;
        self
    }

    /// Whether the scan classifies citing sides (§AR-scanner.2.4).
    pub(crate) fn classify(mut self, classify: bool) -> Self {
        self.scope.classify_citation_sources = classify;
        self
    }

    /// The `cover --lines` ranges (§FS-cover.6.1).
    pub(crate) fn owner_lines(mut self, lines: Vec<(usize, usize)>) -> Self {
        self.scope.owner_lines = lines;
        self
    }

    /// `--require-grounding`: on for this run, never off (§FS-check.3.6).
    pub(crate) fn require_grounding(mut self, require: bool) -> Self {
        self.scope.require_grounding = require;
        self
    }

    /// The editor's open documents (§AR-config.1.5).
    pub(crate) fn buffers(mut self, buffers: Buffers) -> Self {
        self.buffers = buffers;
        self
    }

    /// Whether the path is the project's root, however it was spelled: the
    /// question `--full` and a workspace-wide run both ask (§FS-check.1.3.6,
    /// §FS-workspace.5).
    pub(crate) fn names_root(&self, root: &Path) -> bool {
        scope_is_config_root(root, &self.path, self.path_provided)
    }

    /// Whether the walk reads the project's ordinary roots beside the path
    /// (§FS-check.1.3.6.1, §AR-lsp.5.1.1), for a project at `root` that does
    /// or does not declare `[workspace]`.
    pub(super) fn resolution_wide(&self, root: &Path, workspace_declared: bool) -> bool {
        match self.selection {
            Selection::Check => !self.names_root(root),
            Selection::Editor => !workspace_declared && !self.names_root(root),
            Selection::Discovery | Selection::Project => false,
        }
    }

    /// Whether the run's root is the explicit folder an editor named, where no
    /// config governs it (§FS-lsp.2.2.1).
    pub(super) fn anchors_zero_config(&self) -> bool {
        self.selection == Selection::Editor && self.path_provided && self.path.is_dir()
    }
}

/// Whether the requested scope *is* the config root — the scope `[scan] include`
/// governs, and therefore the only one `grund check --full` can widen
/// (§FS-check.1.3.6). It is also what decides a workspace-wide run
/// (§FS-workspace.5): both questions are "did the caller ask for the whole
/// project, however they spelled it?".
pub(crate) fn scope_is_config_root(root: &Path, path: &Path, path_provided: bool) -> bool {
    !path_provided
        || std::fs::canonicalize(path)
            .map(|path| path == root)
            .unwrap_or(false)
}

impl Run {
    /// §AR-config.1.5: the run one invocation makes of the project at `root`,
    /// its reports spelled from `cli_base`. Built by the loaders `open()` calls
    /// and nowhere else; `resolution_wide` is settled once the project is read.
    pub(super) fn of(root: PathBuf, cli_base: PathBuf, invocation: &Invocation) -> Self {
        Self {
            cli_base,
            location_base: root.clone(),
            root,
            config_file: None,
            redundant_config_file: None,
            found_no_config: false,
            scope: RunScope {
                resolution_wide: false,
                ..invocation.scope.clone()
            },
            workspace: RunWorkspace::default(),
            warnings: Vec::new(),
            path_base: invocation.path_base,
            buffers: invocation.buffers.clone(),
        }
    }

    /// The invocation a workspace member of this run is loaded under: the
    /// run's classification, `--full`, `--require-grounding`, `--path-base`
    /// and buffers, but neither its path's resolution scope nor its
    /// `cover --lines` ranges, which are the root's (§AR-scanner.2.4.2,
    /// §FS-check.1.3.8, §FS-check.3.5).
    pub(crate) fn members(&self) -> Invocation {
        Invocation {
            path: self.root.clone(),
            path_provided: false,
            selection: Selection::Project,
            scope: RunScope {
                full: self.scope.full,
                classify_citation_sources: self.scope.classify_citation_sources,
                require_grounding: self.scope.require_grounding,
                ..RunScope::default()
            },
            path_base: self.path_base,
            buffers: self.buffers.clone(),
        }
    }

    /// A run rooted at `root` that has decided nothing yet: the façade a test
    /// states without a config file (§AR-config.5), under the caller's
    /// `--path-base` (§FS-cli.3.4).
    #[cfg(test)]
    pub(crate) fn at(root: PathBuf) -> Self {
        Self::of(
            root.clone(),
            root.clone(),
            &Invocation::new(&root, false, Selection::Project),
        )
    }
}
