//! Subject selectors and the fact joins that answer them (§FS-rules.2,
//! §FS-rules.5.1): which units a subject denotes, which declarations a target
//! set names, and the containment walks that relate a site or a chapter back to
//! the declaration that owns it. Nothing here decides a finding — the families
//! in `engine.rs` do — so one join is read the same way by every one of them.

use super::super::facts::{NodeKey, RuleFacts, SiteKey};
use super::super::{RuleSubject, RuleTargets};
use std::collections::BTreeSet;

pub(super) fn select_subjects(subject: &RuleSubject, facts: &RuleFacts) -> Vec<NodeKey> {
    match subject {
        RuleSubject::Kind(kind) => facts
            .decl
            .iter()
            .filter(|(_, k)| k == kind)
            .map(|(n, _)| n.clone())
            .collect(),
        RuleSubject::ExactDeclaration(wanted) => facts
            .nodes
            .iter()
            .filter(|(_, m)| &m.label == wanted)
            .map(|(n, _)| n.clone())
            .collect(),
        RuleSubject::ChapterOfKind { kind, name } => facts
            .chapter
            .iter()
            .filter(|(chapter, path, _)| {
                path.rsplit('.').next() == Some(name)
                    && facts.contains.iter().any(|(parent, child)| {
                        child == chapter
                            && facts
                                .decl
                                .iter()
                                .any(|(node, k)| node == parent && k == kind)
                    })
            })
            .map(|(n, _, _)| n.clone())
            .collect(),
        RuleSubject::ExactChapter {
            declaration, path, ..
        } => facts
            .chapter
            .iter()
            .filter(|(node, section, _)| {
                section == path
                    && owning_declaration(facts, node).is_some_and(|owner| {
                        facts
                            .nodes
                            .get(&owner)
                            .is_some_and(|meta| &meta.label == declaration)
                    })
            })
            .map(|(n, _, _)| n.clone())
            .collect(),
    }
}

pub(super) fn target_nodes(targets: &RuleTargets, facts: &RuleFacts) -> BTreeSet<NodeKey> {
    facts
        .decl
        .iter()
        .filter(|(_, kind)| target_kind_matches(targets, kind, facts))
        .map(|(n, _)| n.clone())
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
pub(super) fn site_is_in(site: &SiteKey, node: &NodeKey, facts: &RuleFacts) -> bool {
    facts.site_in.iter().any(|(s, n)| s == site && n == node)
}
pub(super) fn owning_declaration(facts: &RuleFacts, node: &NodeKey) -> Option<NodeKey> {
    if facts.decl.iter().any(|(candidate, _)| candidate == node) {
        return Some(node.clone());
    }
    let mut current = node;
    while let Some((parent, _)) = facts.contains.iter().find(|(_, child)| child == current) {
        if facts.decl.iter().any(|(candidate, _)| candidate == parent) {
            return Some(parent.clone());
        }
        current = parent;
    }
    None
}
pub(super) fn citation_matches_targets(
    cited: &NodeKey,
    targets: &BTreeSet<NodeKey>,
    facts: &RuleFacts,
) -> bool {
    targets.contains(cited)
        || owning_declaration(facts, cited).is_some_and(|owner| targets.contains(&owner))
}
pub(super) fn declaration_kind(facts: &RuleFacts, node: &NodeKey) -> Option<String> {
    if let Some(kind) = facts
        .decl
        .iter()
        .find(|(n, _)| n == node)
        .map(|(_, k)| k.clone())
    {
        return Some(kind);
    }
    let mut current = node;
    while let Some((parent, _)) = facts.contains.iter().find(|(_, child)| child == current) {
        if let Some(kind) = facts
            .decl
            .iter()
            .find(|(candidate, _)| candidate == parent)
            .map(|(_, kind)| kind.clone())
        {
            return Some(kind);
        }
        current = parent;
    }
    None
}
pub(super) fn label(facts: &RuleFacts, node: &NodeKey) -> String {
    facts
        .nodes
        .get(node)
        .map(|m| m.label.clone())
        .unwrap_or_else(|| node.0.clone())
}
