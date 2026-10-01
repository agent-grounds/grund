//! Relational chapter-rule evaluation (§FS-rules.5–7, §AR-rules.4–5).

mod authority;
mod precedence;
mod resolution;
mod selectors;
mod unreached;

use super::facts::{Completeness, NodeKey, RuleFacts, SiteKey};
use super::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};
use crate::model::{CITATION_DIRECTION_REPAIR, Diagnostic};
use std::collections::{BTreeMap, BTreeSet};

use authority::Authority;
pub(crate) use authority::one_rules_authority;
pub(crate) use precedence::citation_precedence;
pub(crate) use resolution::unresolved_subject_diagnostic;
use selectors::{
    citation_matches_targets, declaration_kind, label, owning_declaration, select_subjects,
    site_is_in, target_kind_matches, target_nodes, target_wording,
};
use unreached::report_unreached;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct SemanticRule {
    subject: RuleSubject,
    level: RuleLevel,
    polarity: RulePolarity,
    relation: RuleRelation,
    targets: RuleTargets,
    cardinality: Cardinality,
}

/// Deduplicate normalized constraints and evaluate them over one snapshot. The
/// engine never reparses prose or asks a producer for more facts (§AR-rules.4).
pub(crate) fn evaluate(
    rules: &[ParsedRule],
    precedence: &[ParsedRule],
    facts: &RuleFacts,
) -> Vec<Diagnostic> {
    evaluate_level(rules, precedence, facts, RuleLevel::Required, true)
}

/// Recommendations use the same evaluator and return the same existing
/// `Diagnostic` boundary; only the report channel chosen by the caller differs
/// (§AR-rules.1, §AR-rules.5).
pub(crate) fn evaluate_suggestions(
    rules: &[ParsedRule],
    precedence: &[ParsedRule],
    facts: &RuleFacts,
) -> Vec<Diagnostic> {
    evaluate_level(rules, precedence, facts, RuleLevel::Recommended, false)
}

fn evaluate_level(
    rules: &[ParsedRule],
    precedence: &[ParsedRule],
    facts: &RuleFacts,
    level: RuleLevel,
    include_invalid: bool,
) -> Vec<Diagnostic> {
    let mut groups: BTreeMap<SemanticRule, BTreeSet<String>> = BTreeMap::new();
    let precedence = precedence
        .iter()
        .map(SemanticRule::from)
        .collect::<BTreeSet<_>>();
    let mut out = Vec::new();
    for rule in rules {
        if let Some(diagnostic) = unresolved_subject_diagnostic(rule, facts) {
            if include_invalid {
                out.push(diagnostic);
            }
            continue;
        }
        if rule.level != level {
            continue;
        }
        groups
            .entry(SemanticRule::from(rule))
            .or_default()
            .insert(rule.origin.clone());
    }
    for (rule, origins) in groups {
        if precedence.contains(&rule) {
            continue;
        }
        evaluate_one(&rule, &Authority::new(origins), facts, &mut out);
    }
    out
}

impl From<&ParsedRule> for SemanticRule {
    fn from(rule: &ParsedRule) -> Self {
        Self {
            subject: rule.subject.clone(),
            level: rule.level,
            polarity: rule.polarity,
            relation: rule.relation,
            targets: rule.targets.clone(),
            cardinality: rule.cardinality,
        }
    }
}

fn evaluate_one(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    out: &mut Vec<Diagnostic>,
) {
    let subjects = select_subjects(&rule.subject, facts);
    // §FS-rules.4: every positive cardinality conclusion is closed-world, and
    // the absence of §FS-rules.5.2's second premise is one of them.
    if facts.header.completeness == Completeness::Incomplete
        && rule.polarity == RulePolarity::Positive
    {
        return;
    }
    // §FS-rules.7: the absence takes the channel its level already has, like
    // every other finding the level produces.
    report_unreached(rule, authority, facts, &subjects, out);
    match rule.relation {
        RuleRelation::HaveChapter => {
            let RuleTargets::Chapter(name) = &rule.targets else {
                return;
            };
            for subject in subjects {
                let count = facts
                    .chapter
                    .iter()
                    .filter(|(chapter, _, display)| {
                        display.eq_ignore_ascii_case(name)
                            && facts
                                .contains
                                .iter()
                                .any(|(parent, child)| parent == &subject && child == chapter)
                    })
                    .count();
                if !rule.cardinality.contains(count) {
                    push_node(
                        out,
                        facts,
                        &subject,
                        "chapter-cardinality",
                        format!(
                            "{} has {count} {name} chapters; {authority} requires {}",
                            label(facts, &subject),
                            rule.cardinality.wording()
                        ),
                        authority,
                    );
                }
            }
        }
        RuleRelation::Cite if rule.polarity == RulePolarity::Prohibiting => {
            let targets = target_nodes(&rule.targets, facts);
            for subject in subjects {
                for (site, _, target) in &facts.cites {
                    if citation_matches_targets(target, &targets, facts)
                        && site_is_in(site, &subject, facts)
                    {
                        // §FS-rules.7.5: the hard prohibition reuses
                        // §FS-check.3.12's wording, repair included, with
                        // `(<RULE-ID>)` for the tail; the recommendation not.
                        let (code, repair) = if rule.level == RuleLevel::Required {
                            ("forbidden-citation", CITATION_DIRECTION_REPAIR)
                        } else {
                            ("discouraged-citation", "")
                        };
                        push_site(
                            out,
                            facts,
                            site,
                            code,
                            format!(
                                "{} {} not cite {} ({authority}){repair}",
                                declaration_kind(facts, &subject)
                                    .unwrap_or_else(|| label(facts, &subject)),
                                if rule.level == RuleLevel::Required {
                                    "must"
                                } else {
                                    "should"
                                },
                                target_wording(&rule.targets)
                            ),
                            authority,
                        );
                    }
                }
            }
        }
        RuleRelation::Cite => evaluate_cites(rule, authority, facts, &subjects, out),
        RuleRelation::BeCitedBy => {
            for subject in subjects {
                let count = facts
                    .cites
                    .iter()
                    .filter(|(_, from, target)| {
                        target == &subject
                            && declaration_kind(facts, from).is_some_and(|kind| {
                                target_kind_matches(&rule.targets, &kind, facts)
                            })
                    })
                    .count();
                if !rule.cardinality.contains(count) {
                    push_node(
                        out,
                        facts,
                        &subject,
                        "uncited-unit",
                        format!(
                            "{} is cited by {} {count} times; {authority} requires {}",
                            label(facts, &subject),
                            target_wording(&rule.targets),
                            rule.cardinality.wording()
                        ),
                        authority,
                    );
                }
            }
        }
    }
}

fn evaluate_cites(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    subjects: &[NodeKey],
    out: &mut Vec<Diagnostic>,
) {
    let RuleTargets::Kinds { mode, .. } = &rule.targets else {
        return;
    };
    let targets = target_nodes(&rule.targets, facts);
    for subject in subjects {
        if *mode == TargetMode::PerTarget {
            for target in &targets {
                let count = facts
                    .cites
                    .iter()
                    .filter(|(site, _, cited)| {
                        owning_declaration(facts, cited).as_ref() == Some(target)
                            && site_is_in(site, subject, facts)
                    })
                    .count();
                if !rule.cardinality.contains(count) {
                    push_node(
                        out,
                        facts,
                        subject,
                        "citation-cardinality",
                        format!(
                            "{} cites {} {count} times; {authority} requires {}",
                            label(facts, subject),
                            label(facts, target),
                            rule.cardinality.times_wording()
                        ),
                        authority,
                    );
                }
            }
        } else {
            let count = facts
                .cites
                .iter()
                .filter(|(site, _, target)| {
                    citation_matches_targets(target, &targets, facts)
                        && site_is_in(site, subject, facts)
                })
                .count();
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
                push_node(out, facts, subject, code, message, authority);
            }
        }
    }
}

fn push_node(
    out: &mut Vec<Diagnostic>,
    facts: &RuleFacts,
    node: &NodeKey,
    code: &'static str,
    message: String,
    authority: &Authority,
) {
    let Some(meta) = facts.nodes.get(node) else {
        return;
    };
    out.push(Diagnostic {
        code,
        path: Some(meta.anchor.path.clone().into()),
        line: Some(meta.anchor.line),
        column: meta.anchor.column,
        message,
        sites: Vec::new(),
        authority: authority.origins.clone(),
    });
}
fn push_site(
    out: &mut Vec<Diagnostic>,
    facts: &RuleFacts,
    site: &SiteKey,
    code: &'static str,
    message: String,
    authority: &Authority,
) {
    let Some(meta) = facts.sites.get(site) else {
        return;
    };
    out.push(Diagnostic {
        code,
        path: Some(meta.anchor.path.clone().into()),
        line: Some(meta.anchor.line),
        column: meta.anchor.column,
        message,
        sites: Vec::new(),
        authority: authority.origins.clone(),
    });
}
