//! The run's warning channel (§AR-system.2.3): the `[workspace]` cautions a run
//! settles while it resolves its configuration, carried on the `Config` the run
//! was launched with until a frontend renders them (§FS-check.4.7,
//! §FS-check.4.8, §FS-check.4.10, §FS-workspace.6.1, §FS-distribution.3.1).
//!
//! It is here because the `Config` is what every one of these facts is settled
//! from and what every walking command already holds — the same place the
//! `unread_opted_out_blocks` counter sat, which is what a warning channel
//! answers by construction (§DA-engine-renders-nothing).
//!
//! One of the four cannot be answered where it is asked. §FS-check.4.10 is a
//! question about a *walk* — would this block have read something, had it been a
//! project? — and the walker sits above the workspace component
//! (§AR-resolver.placement). So that one travels as the block it was asked of and
//! is settled by `resolver/unread_block.rs`, in the position the ordered list
//! gives it, which is what keeps the order the two kinds print in
//! (§FS-check.4.10).

use std::path::PathBuf;

use super::record::Config;
use crate::model::Diagnostic;

/// One warning the run has earned, settled or still to be answered.
#[derive(Clone)]
pub(crate) enum RunWarning {
    /// A warning the run already has in full: §FS-check.4.7's absorbed scan and
    /// §FS-workspace.6.1's undecidable ancestor claim.
    Settled(Diagnostic),
    /// §FS-check.4.10: a block that opted out of being a project, held with the
    /// run's other project roots until a walker can be asked whether the block's
    /// own tree holds a file a scan would have read.
    UnreadBlock {
        /// The block's own config, its member boundary already populated, so the
        /// probe prunes exactly as the block's own scan would have
        /// (§FS-workspace.6).
        config: Box<Config>,
        /// Where the *rest* of this run's projects are — the other half of the
        /// counterfactual (§FS-check.4.10). Empty for the block the run is
        /// rooted at, whose own member boundary is already the whole prune.
        project_roots: Vec<PathBuf>,
    },
}

impl RunWarning {
    /// §FS-check.4.10: the block to ask, or `None` when this one is not the
    /// finding's subject at all — cheap enough to run at every boundary
    /// population, because it reads two config fields and touches no disk.
    ///
    /// **A block with no member in scope is not this finding.** `include_root =
    /// false` with no members is already a config error at that block's own line
    /// (§FS-workspace.6.1), and a configuration the run refuses is not one it
    /// also cautions about — the caution's two remedies are not the repair that
    /// block needs.
    ///
    /// The held config carries none of the run's warnings itself: it is the
    /// subject of one, not a second copy of the channel.
    pub(crate) fn unread_block(config: &Config, project_roots: Vec<PathBuf>) -> Option<Self> {
        if config.workspace_include_root || config.workspace_boundary_roots.is_empty() {
            return None;
        }
        let mut held = config.clone();
        held.run_warnings = Vec::new();
        Some(Self::UnreadBlock {
            config: Box::new(held),
            project_roots,
        })
    }
}
