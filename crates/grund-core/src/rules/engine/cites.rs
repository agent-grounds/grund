//! The `cites` relation's count over one subject (§FS-rules.5.1–5.3): every site
//! a target matches counts, one pinned at the rule's own nested alias included
//! (§FS-config.3.9.3). Mixed sites are ordinary (§FS-rules.7.12).

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{Cardinality, RuleLevel, RuleTargets, TargetMode};
use super::authority::Authority;
use super::index::FactIndex;
use super::selectors::{citation_matches_targets, label, target_nodes, target_wording};
use super::{SemanticRule, push_node};
use crate::model::remedies;
use crate::model::{Code, Diagnostic};
use std::collections::BTreeMap;

/// Count the unit's physical rows once (§FS-rules.5.1, §FS-rules.5.3), then
/// enumerate the declared target universe for per-target coverage (§FS-rules.5.2).
pub(super) fn evaluate_cites(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
    subjects: &[NodeKey],
    out: &mut Vec<Diagnostic>,
) {
    let RuleTargets::Kinds { mode, .. } = &rule.targets else {
        return;
    };
    let targets = target_nodes(&rule.targets, facts, index);
    for subject in subjects {
        if *mode == TargetMode::PerTarget {
            // Each owner's sites, mixed ones as ordinary ones (§FS-rules.7.12).
            let mut counts: BTreeMap<_, usize> = BTreeMap::new();
            for (_, _, cited) in index.citations_in(subject) {
                if let Some(owner) = index.owner(cited) {
                    *counts.entry(owner).or_default() += 1;
                }
            }
            for target in &targets {
                let count = counts.get(target).copied().unwrap_or_default();
                if !rule.cardinality.contains(count) {
                    let message = format!(
                        "{} cites {} {count} times; {authority} requires {}; {}",
                        label(facts, subject),
                        label(facts, target),
                        rule.cardinality.times_wording(),
                        remedies::citation_cardinality()
                    );
                    push_node(
                        out,
                        facts,
                        subject,
                        Code::CITATION_CARDINALITY,
                        message,
                        authority,
                    );
                }
            }
        } else {
            let count = index
                .citations_in(subject)
                .iter()
                .filter(|(_, _, target)| citation_matches_targets(target, &targets, index))
                .count();
            if !rule.cardinality.contains(count) {
                let ordinary = rule.cardinality == Cardinality::AT_LEAST_ONE && count == 0;
                let code = if ordinary {
                    if rule.level == RuleLevel::Required {
                        Code::MISSING_CITATION
                    } else {
                        Code::SUGGESTED_CITATION
                    }
                } else {
                    Code::CITATION_CARDINALITY
                };
                let message = if ordinary {
                    format!(
                        "{} {} cite {} ({authority}); {}",
                        label(facts, subject),
                        if rule.level == RuleLevel::Required {
                            "must"
                        } else {
                            "should"
                        },
                        target_wording(&rule.targets),
                        if rule.level == RuleLevel::Required {
                            remedies::missing_citation()
                        } else {
                            remedies::suggested_citation()
                        }
                    )
                } else {
                    format!(
                        "{} cites {} {count} times; {authority} requires {}; {}",
                        label(facts, subject),
                        target_wording(&rule.targets),
                        rule.cardinality.wording(),
                        remedies::citation_cardinality()
                    )
                };
                push_node(out, facts, subject, code, message, authority);
            }
        }
    }
}
