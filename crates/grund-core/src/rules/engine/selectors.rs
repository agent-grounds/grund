//! Subject selectors and the fact joins that answer them (§AR-rules.2,
//! §FS-rules.5.1): which units a subject denotes, which declarations a target
//! set names, using shared snapshot indexes (§AR-rules.3.1, §FS-rules.5.3).
//! Nothing here decides a finding — the families in `engine.rs` do — so one
//! join is read the same way by every one of them.
//! Numeric literal matches retain identity independently of report padding
//! (§FS-config.3.2.8).

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{RuleSubject, RuleTargets};
use super::index::FactIndex;
use crate::config::{NamespaceMatch, parse_citation_target_entry};
use std::collections::BTreeSet;

/// Indexed selectors retain their original relation order (§AR-rules.2, §FS-rules.5.3).
pub(super) fn select_subjects(
    subject: &RuleSubject,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> Vec<NodeKey> {
    match subject {
        // §FS-rules.5.2: a whose clause keeps the declarations whose field holds its value.
        RuleSubject::Kind(kind, whose) => index
            .declarations_of_kind(kind)
            .iter()
            .filter(|node| index.selected(node, whose.as_ref(), facts))
            .map(|node| (*node).clone())
            .collect(),
        RuleSubject::ExactDeclaration(wanted) => index
            .labels
            .range(wanted.prefix()..)
            .take_while(|(label, _)| label.starts_with(wanted.prefix()))
            .filter(|(label, _)| wanted.matches(label))
            .flat_map(|(_, nodes)| nodes)
            .map(|node| (*node).clone())
            .collect(),
        // §AR-rules.2, §FS-rules.5.2: `chapter_handle` joins the whole path, and
        // `owns` keeps a chapter at any depth under a declaration of the kind.
        // The path bucket is already in chapter-fact order.
        RuleSubject::ChapterOfKind { kind, name, whose } => index
            .paths
            .get(name.as_str())
            .into_iter()
            .flatten()
            .filter(|node| index.declaration_kind(node) == Some(kind.as_str()))
            .filter(|node| {
                index
                    .owner(node)
                    .is_some_and(|owner| index.selected(owner, whose.as_ref(), facts))
            })
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
                        .is_some_and(|meta| declaration.matches(&meta.label))
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
/// A declaration kind matches a target entry split at its last `/` exactly as a
/// citation is (§FS-config.3.9.3), so a target pinned at the rule's own member by
/// its whole alias path, nested or not, matches what the bare kind matches
/// (§AR-rules.2).
pub(super) fn target_kind_matches(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
    let RuleTargets::Kinds { values, .. } = targets else {
        return false;
    };
    values.iter().any(|target| {
        let Ok(entry) = parse_citation_target_entry(target) else {
            return false;
        };
        let wanted = entry.kind.as_str();
        match &entry.namespace {
            NamespaceMatch::Local => kind == wanted,
            NamespaceMatch::Any => {
                kind == wanted
                    || kind
                        .strip_suffix(wanted)
                        .is_some_and(|alias| alias.ends_with('/'))
            }
            NamespaceMatch::Alias(alias) => {
                kind.strip_prefix(alias.as_str())
                    .and_then(|rest| rest.strip_prefix('/'))
                    == Some(wanted)
                    || (alias == &facts.header.project && kind == wanted)
            }
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
