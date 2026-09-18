//! The published `fmt` contract (§AR-system.2.9): the normalizer run over one
//! scope, returning the rewritten locations, the paths it could not read and the
//! writes it refused, with no CLI report printed and no exit code mapped
//! (§FS-fmt.3, §AR-bindings.2).

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::config::display_path;
use crate::model::Finding;
use crate::resolver::{WorkspaceProject, load_workspace_context};
use crate::scanner::ApiScanError;

use super::report::context_run_warnings;
use crate::writers::{FmtRunOpts, auto_cross_refs_for_scope, fmt_tree, fmt_workspace_projects};

#[derive(Clone)]
pub struct FmtOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub write: bool,
    pub add_marker: bool,
    pub cross_refs: bool,
}

impl Default for FmtOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            write: false,
            add_marker: false,
            cross_refs: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FmtChange {
    pub path: String,
    pub line: usize,
    /// The rewrite class, and — for a line that expanded a number-only shorthand
    /// — the text it wrote (§FS-fmt.3). A `String` rather than one of four fixed
    /// labels because that detail is per-site.
    pub label: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FmtOutput {
    pub changes: Vec<FmtChange>,
    /// The paths the walk could not read (§FS-fmt.3) — the CLI prints these and
    /// exits `2`. Non-empty means the rewrite ran over less than the whole tree.
    pub scan_errors: Vec<ApiScanError>,
    /// The files read but not rewritten, because a link reaches them from outside
    /// the config root (§FS-fmt.2.3.2). The CLI names each one on stderr; the
    /// exit code is untouched, because the refusal is the intended behavior.
    pub refused_writes: Vec<String>,
    /// The run's warning channel (§FS-distribution.3.1): the four `[workspace]`
    /// cautions of §FS-check.4.7, §FS-check.4.8, §FS-check.4.10 and
    /// §FS-workspace.6.1, each anchored at the `grund.toml` line its own message
    /// names. A frontend renders each as one CLI-level `warning:` on stderr
    /// (§FS-check.2.1.1); an editor publishes it on that line (§FS-lsp.1.1).
    pub warnings: Vec<Finding>,
}

/// Programmatic `fmt`: run the normalizer and return the changed locations
/// without printing the CLI report or mapping the exit code (§AR-bindings.2).
pub fn format_references(opts: FmtOpts) -> Result<FmtOutput> {
    let context = load_workspace_context(&opts.path, opts.path_provided)?;
    let config = context.render_config().clone();
    let explicit_cross_refs = opts.cross_refs;
    let workspace_for_wrap = if context.workspace_loaded {
        Some(&context)
    } else {
        None
    };
    let walk_all_projects = context.workspace_loaded
        && (!opts.path_provided
            || fs::canonicalize(&opts.path)
                .map(|canonical| canonical == config.root)
                .unwrap_or(false));
    let (changes, scan_errors, refused_writes) = if walk_all_projects {
        // §FS-fmt.3: complete every project's strict readiness pass before a
        // workspace write can touch the first one. The dry pass also discovers
        // shorthand-triggered strictness and aggregates root/member failures.
        let preflight = fmt_workspace_projects(
            &context,
            &config,
            opts.add_marker,
            explicit_cross_refs,
            false,
        )?;
        let walked = if opts.write {
            fmt_workspace_projects(
                &context,
                &config,
                opts.add_marker,
                explicit_cross_refs,
                true,
            )?
        } else {
            preflight
        };
        (walked.changes, walked.scan_errors, walked.refused_writes)
    } else {
        // §FS-fmt.3: same guard as above — a bare `--write` and `--write .` name
        // the same scope and must refuse alike, not one quietly resolving
        // cross-refs/shorthands against a set the other just reported incomplete.
        let reusable_findings = (!opts.path_provided)
            .then(|| context.current_project())
            .flatten()
            .and_then(WorkspaceProject::complete_findings);
        let auto_cross_refs =
            auto_cross_refs_for_scope(&config, Some(&opts.path), opts.path_provided)?;
        let run_opts = FmtRunOpts {
            add_marker: opts.add_marker,
            cross_refs: explicit_cross_refs || auto_cross_refs,
            write: opts.write,
            render: &config,
            workspace: workspace_for_wrap,
            precomputed_findings: reusable_findings,
            // §FS-fmt.6.1: check previews the same index carve-out write applies.
            index_cross_refs: true,
        };
        let walked = fmt_tree(&config, Some(&opts.path), opts.path_provided, &run_opts)?;
        (walked.changes, walked.scan_errors, walked.refused_writes)
    };

    Ok(FmtOutput {
        changes: changes
            .into_iter()
            .map(|(path, line, label)| FmtChange {
                path: display_path(&config, &path),
                line,
                label,
            })
            .collect(),
        scan_errors,
        refused_writes,
        warnings: context_run_warnings(&context),
    })
}
