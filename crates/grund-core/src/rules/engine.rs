//! Relational chapter-rule evaluation (§FS-rules.5–7, §AR-rules.4–5).

mod precedence;

use super::facts::{Completeness, NodeKey, RuleFacts, SiteKey};
use super::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};
use crate::checker::CITATION_DIRECTION_REPAIR;
use crate::model::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) use precedence::citation_precedence;

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
        let authority = origins.into_iter().collect::<Vec<_>>().join(", ");
        evaluate_one(&rule, &authority, facts, &mut out);
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

/// Exact subjects resolve in the engine because resolution is selector
/// semantics, not checker orchestration (§FS-rules.2, §AR-rules.1).
pub(crate) fn unresolved_subject_diagnostic(
    rule: &ParsedRule,
    facts: &RuleFacts,
) -> Option<Diagnostic> {
    if subject_resolves(rule, facts) {
        return None;
    }
    let literal = match &rule.subject {
        RuleSubject::ExactDeclaration(value) => value.clone(),
        RuleSubject::ExactChapter {
            declaration,
            path,
            separator,
        } => format!("{declaration}{separator}{path}"),
        _ => return None,
    };
    Some(Diagnostic {
        code: "invalid-rule",
        path: (!rule.anchor.path.is_empty()).then(|| rule.anchor.path.clone().into()),
        line: Some(if rule.origin == "--rule" {
            1
        } else {
            rule.anchor.line
        }),
        column: rule.anchor.column,
        message: format!(
            "{} is not a valid rule: literal subject {literal} does not resolve",
            rule.origin
        ),
        sites: Vec::new(),
    })
}

fn evaluate_one(
    rule: &SemanticRule,
    authority: &str,
    facts: &RuleFacts,
    out: &mut Vec<Diagnostic>,
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
                        // §FS-check.3.12's wording, repair suffix included,
                        // with `(<RULE-ID>)` for the authority tail. The
                        // recommendation reuses `discouraged-citation`, which
                        // carries no repair.
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
                push_node(out, facts, subject, code, message);
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
fn owning_declaration(facts: &RuleFacts, node: &NodeKey) -> Option<NodeKey> {
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
fn citation_matches_targets(
    cited: &NodeKey,
    targets: &BTreeSet<NodeKey>,
    facts: &RuleFacts,
) -> bool {
    targets.contains(cited)
        || owning_declaration(facts, cited).is_some_and(|owner| targets.contains(&owner))
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
    out: &mut Vec<Diagnostic>,
    facts: &RuleFacts,
    node: &NodeKey,
    code: &'static str,
    message: String,
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
    });
}
fn push_site(
    out: &mut Vec<Diagnostic>,
    facts: &RuleFacts,
    site: &SiteKey,
    code: &'static str,
    message: String,
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
    });
}
