//! The release windows in which a finding that only a newly counted
//! numbered-section citation (§FS-rules.7.8), or only a target newly matched at
//! the rule's own nested alias (§FS-rules.7.9), produces warns rather than
//! fails. This module, the site mark it reads and the second field of
//! `evaluate` go away at 0.18.0.

use super::super::facts::{NodeKey, RuleFacts, SiteKey};
use super::super::{Cardinality, RuleLevel, RuleTargets};
use super::index::FactIndex;
use crate::model::Diagnostic;
use std::collections::BTreeSet;

/// The pending clause of §FS-distribution.4.2.3's closed vocabulary, after the
/// error's message unchanged, so the release guard reads the promise (§FS-rules.7.8).
const RAMP_CLAUSE: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";

/// The same pending clause for the corrected pinned-target match (§FS-rules.7.9).
const MATCH_CLAUSE: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.18.0";

/// Which windows a finding falls in. Both may hold, and then the finding names
/// both, §FS-rules.7.8's first (§FS-rules.7.9).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Windows {
    pub(super) counted: bool,
    pub(super) matched: bool,
}

impl Windows {
    /// Only a required-level finding owes a ramp (§FS-rules.7.8, §FS-rules.7.9).
    pub(super) fn at(level: RuleLevel, counted: bool, matched: bool) -> Self {
        if level == RuleLevel::Required {
            Self { counted, matched }
        } else {
            Self::default()
        }
    }

    /// A required count that fails with the newly counted or newly matched sites
    /// and holds without them is ramped and names the windows its sites came
    /// from; one that fails without them stays an error (§FS-rules.7.8,
    /// §FS-rules.7.9). Each site is `(newly counted, newly matched)`.
    pub(super) fn of_count(
        level: RuleLevel,
        cardinality: Cardinality,
        sites: impl IntoIterator<Item = (bool, bool)>,
    ) -> Self {
        let (mut without, mut counted, mut matched) = (0, false, false);
        for (c, m) in sites {
            without += usize::from(!c && !m);
            counted |= c;
            matched |= m;
        }
        if level == RuleLevel::Required && cardinality.contains(without) {
            Self { counted, matched }
        } else {
            Self::default()
        }
    }

    fn any(self) -> bool {
        self.counted || self.matched
    }
}

/// The site counts only because §FS-rules.5.1 now reads its numbered section.
pub(super) fn newly_counted(facts: &RuleFacts, site: &SiteKey) -> bool {
    facts.sites.get(site).is_some_and(|meta| meta.newly_counted)
}

/// Whether the matcher before §FS-rules.7.9's correction took `kind` for one of
/// the targets: it split each entry at its first `/`, so a nested alias path
/// pinned itself as alias `group` and kind `api/AR` and matched nothing.
pub(super) fn matched_before(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
    let RuleTargets::Kinds { values, .. } = targets else {
        return false;
    };
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
/// (§FS-rules.7.8, §FS-rules.7.9).
pub(super) fn route<'a>(
    windows: Windows,
    out: &'a mut Vec<Diagnostic>,
    ramp: &'a mut Vec<Diagnostic>,
    mut message: String,
) -> (&'a mut Vec<Diagnostic>, String) {
    if !windows.any() {
        return (out, message);
    }
    if windows.counted {
        message.push_str(RAMP_CLAUSE);
    }
    if windows.matched {
        message.push_str(MATCH_CLAUSE);
    }
    (ramp, message)
}
