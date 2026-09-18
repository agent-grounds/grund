use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::workspace::{
    UnreadBlockProbe, WorkspaceMember, absorbed_scan_roots, absorbed_scan_warning,
    config_location_message, undecidable_ancestor_claim_warning, unread_block_scope_root,
    unread_block_warning,
};

pub(crate) fn warn_if_members_absorb_scan(config: &Config, members: &[WorkspaceMember]) {
    let covered = absorbed_scan_roots(config, members);
    if covered.is_empty() {
        return;
    }
    eprintln!(
        "warning: {}",
        config_location_message(
            config.workspace_members_source.as_ref(),
            absorbed_scan_warning(&covered),
        )
    );
}

/// §FS-check.4.10: say so when a block that set `include_root = false` still holds
/// files of its own. It is no project, and the enclosing scan stops at the member
/// boundary (§FS-workspace.6), so those files are read by nobody — a declaration
/// there reaches no catalog and a citation there is never checked
/// (§GOAL-no-dangling-refs). grund#71 reproduced exactly that: `check`,
/// `check --full` and `list` all silent over two dangling citations, all exiting 0.
///
/// The mirror of [`warn_if_members_absorb_scan`] above — *would this block have
/// read something, had it been a project?* — and it takes that finding's shape: a
/// CLI-level `warning:` on stderr, asked where a run first populates a block's
/// boundary, which is what puts it on every command that walks while leaving each
/// block asked once.
///
/// `project_roots` is where the *rest* of this run's projects are, and it is the
/// other half of the counterfactual; `unread_block_scope_root` in
/// `workspace/members.rs` is what reads it.
///
/// It returns how many lines it printed. This is the one of the two that can fire
/// on an otherwise clean run — §FS-workspace.2.1's block always earns the
/// empty-scan caution beside it — so it is the one whose caller has to know that
/// stderr is no longer empty and `success` must not be printed (§FS-check.2.1).
///
/// It names no release, deliberately. A grouping directory holding a README it
/// does not need checked is a correct configuration and no key records that
/// intent, so the finding is never eligible to become an error
/// (§DF-unread-opted-out-block.2.3) — unlike both of its siblings.
pub(crate) fn warn_unread_block(probe: &UnreadBlockProbe, project_roots: &[PathBuf]) -> usize {
    let Some(root) = unread_block_scope_root(probe, project_roots) else {
        return 0;
    };
    let config = &probe.config;
    eprintln!(
        "warning: {}",
        config_location_message(
            config
                .workspace_include_root_source
                .as_ref()
                .or(config.workspace_section_source.as_ref()),
            unread_block_warning(&root),
        )
    );
    1
}

/// §FS-workspace.6.1: the one residue of the members-only read — a config whose
/// `members` text cannot be obtained, so the claim is undecidable in *both*
/// directions. Failing would let one unreadable `grund.toml` above a repository
/// break every run inside it; staying silent is what let a claiming ancestor
/// re-spell the subtree below it. So the run continues and says what it could not
/// answer, in the CLI-level `warning:` shape on stderr (§FS-errors.2.2) —
/// naming the config against the root this run was launched at, like every other
/// diagnostic from an ancestor block (§FS-errors.4).
pub(crate) fn warn_undecidable_ancestor_claim(
    config_path: &Path,
    report_base: &Path,
    reason: &str,
) {
    eprintln!(
        "warning: {}",
        undecidable_ancestor_claim_warning(config_path, report_base, reason)
    );
}
