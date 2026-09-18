//! The shorthand rule asked against a **declaration catalog** rather than a
//! line (§FS-fmt.2.4, §DF-number-only-citation-shorthand): what one `grund fmt`
//! walk indexes to expand a shorthand, the index key a declared ID answers to,
//! and the one declared ID a shorthand token expands to. The last two were
//! parked in the scanner's off-grammar compatibility file while §AR-system.2.5
//! was a file-name category; both are pure functions of the compiled grammar and
//! a token, so they came down here when it became a module (§AR-system.4).

use std::collections::BTreeMap;

use super::ids::parse_id_arg;
use super::shorthand::{ParsedId, ShorthandIndex, parse_id_arg_with_shorthand, shorthand_names};
use crate::model::{Findings, Id};
// §AR-system.4: two upward reads — `Config` is config's own record and
// `WorkspaceContext` the resolver's, both above this one.
use crate::config::Config;
use crate::resolver::WorkspaceContext;

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

/// The shorthand index a declared ID answers to (§DF-number-only-citation-shorthand,
/// §FS-config.3.2): its own number, or — for a persisted off-grammar spelling that
/// is itself a valid shorthand token — the number that spelling parses to. `None`
/// for a declaration no shorthand can name.
pub(super) fn shorthand_index_number(config: &Config, declared: &Id) -> Option<Option<u32>> {
    match declared.legacy_spelling() {
        Some(spelling) => parse_id_arg_with_shorthand(spelling, &config.grammar)
            .ok()
            .filter(|parsed| parsed.shorthand && parsed.section.is_none())
            .map(|parsed| parsed.id.num),
        None if declared.slug.is_some() => Some(declared.num),
        None => None,
    }
}

pub(super) fn unique_shorthand_expansion_target<'a>(
    config: &Config,
    token: &str,
    parsed: &ParsedId,
    declared_ids: &[&'a str],
) -> Option<&'a str> {
    let exact = parsed.section.as_ref().map_or(token, |section| {
        token
            .strip_suffix(&format!("{}{}", config.section_separator, section))
            .unwrap_or(token)
    });
    let mut matches = declared_ids.iter().copied().filter(|declared| {
        *declared == exact
            || parse_id_arg(declared, &config.grammar)
                .is_ok_and(|(id, section)| section.is_none() && shorthand_names(&id, &parsed.id))
    });
    let unique = matches.next()?;
    (matches.next().is_none() && unique != exact).then_some(unique)
}
