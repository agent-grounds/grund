//! Subject selectors and the fact joins that answer them (§FS-rules.2,
//! §FS-rules.5.1): which units a subject denotes, which declarations a target
//! set names, using shared snapshot indexes (§AR-rules.3.1, §FS-rules.5.3).
//! Nothing here decides a finding — the families in `engine.rs` do — so one
//! join is read the same way by every one of them.

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{RuleSubject, RuleTargets};
use super::index::FactIndex;
use std::collections::BTreeSet;

/// Indexed selectors retain their original relation order (§FS-rules.2, §FS-rules.5.3).
pub(super) fn select_subjects(
    subject: &RuleSubject,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> Vec<NodeKey> {
    match subject {
        RuleSubject::Kind(kind) => index
            .declarations_of_kind(kind)
            .iter()
            .map(|node| (*node).clone())
            .collect(),
        RuleSubject::ExactDeclaration(wanted) => index
            .labels
            .get(wanted.as_str())
            .into_iter()
            .flatten()
            .map(|node| (*node).clone())
            .collect(),
        // §FS-rules.2.1, §FS-rules.5.2: `chapter_handle` joins the whole path, and
        // `owns` keeps a chapter at any depth under a declaration of the kind.
        // The path bucket is already in chapter-fact order.
        RuleSubject::ChapterOfKind { kind, name } => index
            .paths
            .get(name.as_str())
            .into_iter()
            .flatten()
            .filter(|node| index.declaration_kind(node) == Some(kind.as_str()))
            .map(|node| (*node).clone())
            .collect(),
        RuleSubject::ExactChapter {
            declaration, path, ..
        } => index
            .paths
            .get(path.as_str())
            .into_iter()
            .flatten()
            .filter(|node| {
                index.owner(node).is_some_and(|owner| {
                    facts
                        .nodes
                        .get(owner)
                        .is_some_and(|meta| &meta.label == declaration)
                })
            })
            .map(|node| (*node).clone())
            .collect(),
    }
}

/// Target universes come from declarations, including zero-site targets (§FS-rules.5.1).
pub(super) fn target_nodes(
    targets: &RuleTargets,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> BTreeSet<NodeKey> {
    index
        .declarations
        .iter()
        .filter(|(kind, _)| target_kind_matches(targets, kind, facts))
        .flat_map(|(_, nodes)| nodes.iter().map(|node| (*node).clone()))
        .collect()
}
pub(super) fn target_kind_matches(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
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
pub(super) fn target_wording(targets: &RuleTargets) -> String {
    match targets {
        RuleTargets::Kinds { values, .. } => values.join(" or "),
        RuleTargets::Chapter(name) => name.clone(),
    }
}
/// Outbound kinds include nested targets via their cached owner (§FS-rules.5.2).
pub(super) fn citation_matches_targets(
    cited: &NodeKey,
    targets: &BTreeSet<NodeKey>,
    index: &FactIndex<'_>,
) -> bool {
    targets.contains(cited)
        || index
            .owner(cited)
            .is_some_and(|owner| targets.contains(owner))
}
pub(super) fn label(facts: &RuleFacts, node: &NodeKey) -> String {
    facts
        .nodes
        .get(node)
        .map(|m| m.label.clone())
        .unwrap_or_else(|| node.0.clone())
}
