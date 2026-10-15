//! The release window in which a finding that only a target newly matched at
//! the rule's own nested alias (§FS-rules.7.9) produces warns rather than fails.
//! It, the site mark it reads and the second field of `evaluate` go away at
//! 0.19.0. Numbered and mixed section citations count as any other site since
//! §FS-rules.7.8 and §FS-rules.7.12 landed, and a rule declared through a stub
//! and what a stub's target lends since §FS-rules.7.10 and §FS-rules.7.11
//! closed, so none of them marks anything here.

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{Cardinality, RuleLevel, RuleTargets};
use super::index::FactIndex;
use crate::model::Diagnostic;
use std::collections::BTreeSet;

/// The pending clause of §FS-distribution.4.2.3's closed vocabulary, after the
/// error's message unchanged, so the release guard reads the promise: the one for
/// the corrected pinned-target match (§FS-rules.7.9).
const MATCH_CLAUSE: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.19.0";

/// What one counted fact is new by: a target newly matched (§FS-rules.7.9).
/// Mixed sites have no separate mark (§FS-rules.7.12).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Marks {
    pub(super) matched: bool,
}

/// Which window a finding falls in (§FS-rules.7.9).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Windows {
    pub(super) matched: bool,
}

impl Windows {
    /// Only a required-level finding owes a ramp (§FS-rules.7.9).
    pub(super) fn at(level: RuleLevel, marks: Marks) -> Self {
        if level == RuleLevel::Required {
            Self {
                matched: marks.matched,
            }
        } else {
            Self::default()
        }
    }

    /// A required count that fails with the newly matched sites and holds without
    /// them is ramped; one that fails without them stays an error (§FS-rules.7.9).
    /// Numbered and mixed sites are ordinary, so counts they alone break are
    /// errors (§FS-rules.7.8, §FS-rules.7.12).
    pub(super) fn of_count(
        level: RuleLevel,
        cardinality: Cardinality,
        sites: impl IntoIterator<Item = Marks>,
    ) -> Self {
        let ordinary = sites.into_iter().filter(|site| !site.matched).count();
        Self {
            matched: level == RuleLevel::Required && cardinality.contains(ordinary),
        }
    }

    /// A per-target finding for a target only the corrected match takes into the
    /// universe exists only now, whatever its count, so it is always in
    /// §FS-rules.7.9's window.
    pub(super) fn of_new_target(level: RuleLevel) -> Self {
        Self::at(level, Marks { matched: true })
    }
}

/// Whether the matcher before §FS-rules.7.9's correction took `kind` for one of
/// the targets: it split each entry at its first `/`, so a nested alias path
/// pinned itself as alias `group` and kind `api/AR` and matched nothing.
pub(super) fn matched_before(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
    let RuleTargets::Kinds { values, .. } = targets else {
        return false;
    };
    // coordinate-lexer: the old first-`/` matcher, kept for the §FS-rules.7.9 ramp to 0.19.0
    values.iter().any(|target| match target.split_once('/') {
        None => target == kind,
        Some(("*", target_kind)) => {
            kind == target_kind || kind.ends_with(&format!("/{target_kind}"))
        }
        Some((project, target_kind)) => {
            kind == format!("{project}/{target_kind}")
                || (project == facts.header.project && kind == target_kind)
        }
    })
}

/// The declarations the matcher before the correction took for the targets, to
/// tell a newly matched site from one already matched (§FS-rules.7.9).
pub(super) fn nodes_matched_before(
    targets: &RuleTargets,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> BTreeSet<NodeKey> {
    index
        .declarations
        .iter()
        .filter(|(kind, _)| matched_before(targets, kind, facts))
        .flat_map(|(_, nodes)| nodes.iter().map(|node| (*node).clone()))
        .collect()
}

/// Pick the channel for one finding and word its message for it: a ramped
/// finding keeps its code and its message, then names its error release
/// (§FS-rules.7.9).
pub(super) fn route<'a>(
    windows: Windows,
    out: &'a mut Vec<Diagnostic>,
    ramp: &'a mut Vec<Diagnostic>,
    mut message: String,
) -> (&'a mut Vec<Diagnostic>, String) {
    if !windows.matched {
        return (out, message);
    }
    message.push_str(MATCH_CLAUSE);
    (ramp, message)
}
