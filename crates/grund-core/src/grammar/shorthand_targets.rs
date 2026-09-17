use std::collections::BTreeMap;

use super::shorthand::ShorthandIndex;
use crate::model::Findings;
// §AR-system.4: `Config` is config's own record and `WorkspaceContext` the
// workspace component's, both above this one and reachable through the crate
// root until they move.
use crate::{Config, WorkspaceContext};

/// Everything one `grund fmt` walk needs to expand a shorthand: this project's
/// declaration index, plus one per workspace alias for the qualified form
/// (§FS-fmt.2.4, §FS-workspace.8.5).
///
/// Built once per walk rather than per line. `fmt` visits every marker of every
/// scanned file, so resolving each against a linear scan of the declaration set
/// is quadratic on exactly the tree this rewrite exists to clean up
/// (§GOAL-fast-feedback).
pub(crate) struct ShorthandTargets<'a> {
    /// `None` until the walk has a declaration set — §FS-fmt.2.4 defers that scan
    /// until a shorthand is actually met, so a repo without one never pays for it.
    pub(crate) local: Option<ShorthandIndex<'a>>,
    pub(crate) by_alias: BTreeMap<&'a str, ShorthandAliasTarget<'a>>,
}

/// One aliased project's half of `ShorthandTargets`: its declarations, and the
/// config the canonical ID renders under (a workspace may mix `[id] format`s).
pub(crate) struct ShorthandAliasTarget<'a> {
    pub(crate) config: &'a Config,
    pub(crate) index: ShorthandIndex<'a>,
}

impl<'a> ShorthandTargets<'a> {
    pub(crate) fn new(
        config: &Config,
        findings: Option<&'a Findings>,
        workspace: Option<&'a WorkspaceContext>,
    ) -> Self {
        Self {
            local: findings.map(|found| ShorthandIndex::build(config, found.declarations.keys())),
            by_alias: workspace
                .map(|workspace| {
                    workspace
                        .projects
                        .iter()
                        .map(|project| {
                            (
                                project.alias.as_str(),
                                ShorthandAliasTarget {
                                    config: &project.config,
                                    index: ShorthandIndex::build(
                                        &project.config,
                                        project.findings.declarations.keys(),
                                    ),
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}
