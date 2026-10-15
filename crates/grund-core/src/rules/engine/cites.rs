//! The `cites` relation's count over one subject (§FS-rules.5.1–5.3), with the
//! ramp windows a count only newly matched, lent or mixed sites break
//! (§FS-rules.7.9, §FS-rules.7.11, §FS-rules.7.12).

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{Cardinality, RuleLevel, RuleTargets, TargetMode};
use super::authority::Authority;
use super::index::FactIndex;
use super::ramp::{Marks, Windows, node_lent, nodes_matched_before, route, site_marks};
use super::selectors::{citation_matches_targets, label, target_nodes, target_wording};
use super::{SemanticRule, push_node};
use crate::model::Diagnostic;
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
    ramp: &mut Vec<Diagnostic>,
) {
    let RuleTargets::Kinds { mode, .. } = &rule.targets else {
        return;
    };
    let targets = target_nodes(&rule.targets, facts, index);
    let before = nodes_matched_before(&rule.targets, facts, index);
    for subject in subjects {
        if *mode == TargetMode::PerTarget {
            // Each owner's sites, marked lent and mixed or not (§FS-rules.7.11,
            // §FS-rules.7.12).
            let mut counts: BTreeMap<_, Vec<Marks>> = BTreeMap::new();
            for (site, _, cited) in index.citations_in(subject) {
                if let Some(owner) = index.owner(cited) {
                    counts
                        .entry(owner)
                        .or_default()
                        .push(site_marks(facts, site, false));
                }
            }
            for target in &targets {
                let sites = counts.get(target).map(Vec::as_slice).unwrap_or_default();
                let count = sites.len();
                if !rule.cardinality.contains(count) {
                    let windows = if before.contains(target) {
                        Windows::of_count(rule.level, rule.cardinality, sites.iter().copied())
                    } else {
                        Windows::of_new_target(rule.level, rule.cardinality, sites.iter().copied())
                    }
                    .of_subject(rule.level, node_lent(facts, subject));
                    let (sink, message) = route(
                        windows,
                        out,
                        ramp,
                        format!(
                            "{} cites {} {count} times; {authority} requires {}",
                            label(facts, subject),
                            label(facts, target),
                            rule.cardinality.times_wording()
                        ),
                    );
                    push_node(
                        sink,
                        facts,
                        subject,
                        "citation-cardinality",
                        message,
                        authority,
                    );
                }
            }
        } else {
            let sites = index
                .citations_in(subject)
                .iter()
                .filter(|(_, _, target)| citation_matches_targets(target, &targets, index))
                .map(|(site, _, target)| {
                    site_marks(
                        facts,
                        site,
                        !citation_matches_targets(target, &before, index),
                    )
                })
                .collect::<Vec<_>>();
            let count = sites.len();
            if !rule.cardinality.contains(count) {
                let ordinary = rule.cardinality == Cardinality::AT_LEAST_ONE && count == 0;
                let code = if ordinary {
                    if rule.level == RuleLevel::Required {
                        "missing-citation"
                    } else {
                        "suggested-citation"
                    }
                } else {
                    "citation-cardinality"
                };
                let message = if ordinary {
                    format!(
                        "{} {} cite {} ({authority})",
                        label(facts, subject),
                        if rule.level == RuleLevel::Required {
                            "must"
                        } else {
                            "should"
                        },
                        target_wording(&rule.targets)
                    )
                } else {
                    format!(
                        "{} cites {} {count} times; {authority} requires {}",
                        label(facts, subject),
                        target_wording(&rule.targets),
                        rule.cardinality.wording()
                    )
                };
                // §FS-rules.7.9, §FS-rules.7.11, §FS-rules.7.12: a count the newly
                // matched, lent or mixed sites alone break warns.
                let windows = Windows::of_count(rule.level, rule.cardinality, sites)
                    .of_subject(rule.level, node_lent(facts, subject));
                let (sink, message) = route(windows, out, ramp, message);
                push_node(sink, facts, subject, code, message, authority);
            }
        }
    }
}
