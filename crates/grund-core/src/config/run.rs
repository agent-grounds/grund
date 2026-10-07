//! §AR-config.1.5: what one invocation decided and what workspace expansion
//! learned about this checkout. No `Run` field is read from a file, so nothing
//! here is a key and nothing here is lowered; a command-line override of a
//! file's value is a `Run` fact the façade applies over the `Project`'s.

use std::path::PathBuf;

use super::call_scope::{PathBase, report_path_base};
use super::record::AbsentOptionalNamespace;
use super::run_warnings::RunWarning;

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
    pub scope: RunScope,
    pub workspace: RunWorkspace,
    /// The run's warning channel (§FS-distribution.3.1).
    pub(crate) warnings: Vec<RunWarning>,
    /// The run's `--path-base`, when the caller passed one: it outranks
    /// `relative_paths` for every project the run loads (§FS-cli.3.4). Taken
    /// from the call scope when the run is made, so members carry it too.
    pub(crate) path_base: Option<PathBase>,
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
}

impl Run {
    /// A run rooted at `root` that has decided nothing yet: reports relative to
    /// the root, no file read, and the citing side classified, which every
    /// read-only command turns off (§AR-scanner.2.4, §AR-benchmarks).
    pub(crate) fn at(root: PathBuf) -> Self {
        Self {
            cli_base: root.clone(),
            root,
            config_file: None,
            redundant_config_file: None,
            scope: RunScope {
                full: false,
                resolution_wide: false,
                classify_citation_sources: true,
                owner_lines: Vec::new(),
                require_grounding: false,
            },
            workspace: RunWorkspace::default(),
            warnings: Vec::new(),
            path_base: report_path_base(),
        }
    }
}
