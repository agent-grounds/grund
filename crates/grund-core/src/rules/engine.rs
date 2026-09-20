//! Relational chapter-rule evaluation (§FS-rules.5–7, §AR-rules.4–5).

use super::facts::{Completeness, NodeKey, RuleFacts, SiteKey};
use super::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};
use crate::model::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct RuleDiagnostic {
    pub(crate) diagnostic: Diagnostic,
    pub(crate) recommended: bool,
}

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
pub(crate) fn evaluate(rules: &[ParsedRule], facts: &RuleFacts) -> Vec<RuleDiagnostic> {
    let mut groups: BTreeMap<SemanticRule, BTreeSet<String>> = BTreeMap::new();
    for rule in rules {
        groups
            .entry(SemanticRule {
                subject: rule.subject.clone(),
                level: rule.level,
                polarity: rule.polarity,
                relation: rule.relation,
                targets: rule.targets.clone(),
                cardinality: rule.cardinality,
            })
            .or_default()
            .insert(rule.origin.clone());
    }
    let mut out = Vec::new();
    for (rule, origins) in groups {
        let authority = origins.into_iter().collect::<Vec<_>>().join(", ");
        evaluate_one(&rule, &authority, facts, &mut out);
    }
    out
}

/// Post-scan literal resolution belongs with selector semantics, not sentence
/// recognition (§FS-rules.4, §AR-rules.4).
pub(crate) fn subject_resolves(rule: &ParsedRule, facts: &RuleFacts) -> bool {
    match rule.subject {
        RuleSubject::ExactDeclaration(_) | RuleSubject::ExactChapter { .. } => {
            select_subjects(&rule.subject, facts).len() == 1
        }
        _ => true,
    }
}

fn evaluate_one(
    rule: &SemanticRule,
    authority: &str,
    facts: &RuleFacts,
    out: &mut Vec<RuleDiagnostic>,
) {
    let subjects = select_subjects(&rule.subject, facts);
    // §FS-rules.4: every positive cardinality conclusion is closed-world.
    if facts.header.completeness == Completeness::Incomplete
        && rule.polarity == RulePolarity::Positive
    {
        return;
    }
    match rule.relation {
        RuleRelation::HaveChapter => {
            let RuleTargets::Chapter(name) = &rule.targets else {
                return;
            };
            for subject in subjects {
                let count = facts
                    .chapter
                    .iter()
                    .filter(|(chapter, path, display)| {
                        (path.rsplit('.').next() == Some(name)
                            || display.eq_ignore_ascii_case(name))
                            && facts
                                .contains
                                .iter()
                                .any(|(parent, child)| parent == &subject && child == chapter)
                    })
                    .count();
                if !rule.cardinality.contains(count) {
                    push_node(
                        out,
                        rule,
                        facts,
                        &subject,
                        "chapter-cardinality",
                        format!(
                            "{} has {count} {name} chapters; {authority} requires {}",
                            label(facts, &subject),
                            rule.cardinality.wording()
                        ),
                    );
                }
            }
        }
        RuleRelation::Cite if rule.polarity == RulePolarity::Prohibiting => {
            let targets = target_nodes(&rule.targets, facts);
            for subject in subjects {
                for (site, _, target) in &facts.cites {
                    if targets.contains(target) && site_is_in(site, &subject, facts) {
                        let code = if rule.level == RuleLevel::Required {
                            "forbidden-citation"
                        } else {
                            "discouraged-citation"
                        };
                        push_site(
                            out,
                            rule,
                            facts,
                            site,
                            code,
                            format!(
                                "{} {} not cite {} ({authority})",
                                declaration_kind(facts, &subject)
                                    .unwrap_or_else(|| label(facts, &subject)),
                                if rule.level == RuleLevel::Required {
                                    "must"
                                } else {
                                    "should"
                                },
                                target_wording(&rule.targets)
                            ),
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
                        rule,
                        facts,
                        &subject,
                        "uncited-unit",
                        format!(
                            "{} is cited by {} {count} times; {authority} requires {}",
                            label(facts, &subject),
                            target_wording(&rule.targets),
                            rule.cardinality.wording()
                        ),
                    );
                }
            }
        }
    }
}

fn evaluate_cites(
    rule: &SemanticRule,
    authority: &str,
    facts: &RuleFacts,
    subjects: &[NodeKey],
    out: &mut Vec<RuleDiagnostic>,
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
                    .filter(|(site, _, cited)| cited == target && site_is_in(site, subject, facts))
                    .count();
                if !rule.cardinality.contains(count) {
                    push_node(
                        out,
                        rule,
                        facts,
                        subject,
                        "citation-cardinality",
                        format!(
                            "{} cites {} {count} times; {authority} requires {}",
                            label(facts, subject),
                            label(facts, target),
                            rule.cardinality.times_wording()
                        ),
                    );
                }
            }
        } else {
            let count = facts
                .cites
                .iter()
                .filter(|(site, _, target)| {
                    targets.contains(target) && site_is_in(site, subject, facts)
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
                push_node(out, rule, facts, subject, code, message);
            }
        }
    }
}

fn select_subjects(subject: &RuleSubject, facts: &RuleFacts) -> Vec<NodeKey> {
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
            .filter(|(chapter, path, display)| {
                (path.rsplit('.').next() == Some(name) || display.eq_ignore_ascii_case(name))
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
        RuleSubject::ExactChapter { declaration, path } => {
            let wanted = format!("{declaration}.{path}");
            facts
                .chapter
                .iter()
                .filter(|(n, _, _)| facts.nodes.get(n).is_some_and(|m| m.label == wanted))
                .map(|(n, _, _)| n.clone())
                .collect()
        }
    }
}

fn target_nodes(targets: &RuleTargets, facts: &RuleFacts) -> BTreeSet<NodeKey> {
    facts
        .decl
        .iter()
        .filter(|(_, kind)| target_kind_matches(targets, kind, facts))
        .map(|(n, _)| n.clone())
        .collect()
}
fn target_kind_matches(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
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
fn target_wording(targets: &RuleTargets) -> String {
    match targets {
        RuleTargets::Kinds { values, .. } => values.join(" or "),
        RuleTargets::Chapter(name) => name.clone(),
    }
}
fn site_is_in(site: &SiteKey, node: &NodeKey, facts: &RuleFacts) -> bool {
    facts.site_in.iter().any(|(s, n)| s == site && n == node)
}
fn declaration_kind(facts: &RuleFacts, node: &NodeKey) -> Option<String> {
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
fn label(facts: &RuleFacts, node: &NodeKey) -> String {
    facts
        .nodes
        .get(node)
        .map(|m| m.label.clone())
        .unwrap_or_else(|| node.0.clone())
}

fn push_node(
    out: &mut Vec<RuleDiagnostic>,
    rule: &SemanticRule,
    facts: &RuleFacts,
    node: &NodeKey,
    code: &'static str,
    message: String,
) {
    let Some(meta) = facts.nodes.get(node) else {
        return;
    };
    out.push(RuleDiagnostic {
        recommended: rule.level == RuleLevel::Recommended,
        diagnostic: Diagnostic {
            code,
            path: Some(meta.anchor.path.clone().into()),
            line: Some(meta.anchor.line),
            column: meta.anchor.column,
            message,
            sites: Vec::new(),
        },
    });
}
fn push_site(
    out: &mut Vec<RuleDiagnostic>,
    rule: &SemanticRule,
    facts: &RuleFacts,
    site: &SiteKey,
    code: &'static str,
    message: String,
) {
    let Some(meta) = facts.sites.get(site) else {
        return;
    };
    out.push(RuleDiagnostic {
        recommended: rule.level == RuleLevel::Recommended,
        diagnostic: Diagnostic {
            code,
            path: Some(meta.anchor.path.clone().into()),
            line: Some(meta.anchor.line),
            column: meta.anchor.column,
            message,
            sites: Vec::new(),
        },
    });
}
