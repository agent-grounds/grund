//! The §FS-check.4.10 answer (§AR-system.2.10): whether a block that opted out
//! of being a project leaves a tree no scan reaches, which only a walk can say.
//!
//! The *question* is workspace's and is posed at config time — `RunWarning::unread_block`
//! reads two config fields and touches no disk (§AR-workspace.placement). Answering it
//! runs the scanner's walker against the block's own config in the
//! counterfactual where the block had been a project, so the answer sits above
//! the scanner and reads it downward, which is what retires the one walk this
//! finding had workspace reading upward (§AR-system.4,
//! §AR-resolver.placement).
//!
//! The scope roots it probes are the block's own list, shared with the
//! §FS-workspace.2.1 finding that reads it from the other end: one definition of
//! "this block's own scope", so a `[[kinds]]` home or a home the config lists
//! without walking moves both rules together (§FS-config.3.5.8).

use std::path::PathBuf;

use crate::config::{Config, RunWarning, canonical_config_root};
use crate::model::{Diagnostic, format_path};
use crate::scanner::walk_reads_any_file;
use crate::workspace::{block_relative_root, uncovered_block_scope_roots, unread_block_diagnostic};

/// §FS-check.4.10: the first root of this block's own scope that holds a file the
/// block would have read as a project, named under the block root — or `None`,
/// which is every configuration this finding stays silent about.
///
/// **One root, not every root.** §FS-workspace.2.1.1's claim is universal and its
/// list is the evidence for it; this claim is existential, one edit clears every
/// root at once, and probing the rest would buy nothing the answer depends on. The
/// one named is the first in *scope order* — `[scan] include` in config order,
/// then the `[[kinds]]` homes — because that order is fixed, while which file the
/// walk hands back first is not (§FS-errors.4).
///
/// The walk runs against the block's own config with its member boundary set and
/// with the run's project roots, its own among them, so [`walk_reads_any_file`]
/// prunes exactly as the block's own scan would have: at each member on the way
/// down, and at every other project of the run in the directions the member list
/// cannot see — which is what a directory symlink out of the block's scope root
/// takes (§FS-workspace.6.2). Its own root belongs there because the walker prunes
/// what *another* project owns, and the block owns its tree in the counterfactual
/// this finding asks about. Which projects those are is a property of the run: a
/// run rooted at the block does not know the projects above it, and neither would
/// the scan it is standing in for. `scan_full` is off for the same reason
/// [`block_scope_roots`] asks the default scope — this is a property of the
/// configuration rather than of one walk (§FS-check.1.3.10).
pub(crate) fn unread_block_scope_root(
    config: &Config,
    project_roots: &[PathBuf],
) -> Option<String> {
    let mut walk = config.clone();
    walk.workspace_project_roots = project_roots.to_vec();
    walk.workspace_project_roots
        .push(canonical_config_root(config));
    walk.scan_full = false;
    uncovered_block_scope_roots(config)
        .into_iter()
        .find(|root| walk_reads_any_file(&walk, root))
        .map(|root| format_path(block_relative_root(config, &root)))
}

/// §FS-check.4.7.7, §FS-check.4.10.11, §FS-workspace.6.1, §FS-distribution.3.1: the
/// run's warning channel, settled — every caution the workspace pass already had
/// in full, and every opted-out block it could only pose, in the one order the
/// reader sees them in.
///
/// This is the lowest component that can answer all four, because §FS-check.4.10.2's
/// question is about a walk and the walker sits above the workspace pass
/// (§AR-resolver.placement). Nothing here renders: what comes back is a
/// `Diagnostic` per warning, for whichever frontend asked (§AR-bindings.2).
pub(crate) fn settled_run_warnings(config: &Config) -> Vec<Diagnostic> {
    config
        .run_warnings
        .iter()
        .filter_map(|warning| match warning {
            RunWarning::Settled(diagnostic) => Some(diagnostic.clone()),
            RunWarning::UnreadBlock {
                config,
                project_roots,
            } => unread_block_scope_root(config, project_roots)
                .map(|root| unread_block_diagnostic(config, &root)),
        })
        .collect()
}
