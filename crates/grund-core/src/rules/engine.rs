//! Relational chapter-rule evaluation (§FS-rules.5–7, §AR-rules.4–5).

mod authority;
mod cites;
mod index;
mod precedence;
mod resolution;
mod selectors;
mod unreached;

use super::facts::{Completeness, NodeKey, RuleFacts, SiteKey};
use super::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
};
use crate::model::remedies;
use crate::model::{Code, Diagnostic, citation_direction_repair};
use std::collections::{BTreeMap, BTreeSet};

use authority::Authority;
pub(crate) use authority::one_rules_authority;
use cites::evaluate_cites;
use index::FactIndex;
pub(crate) use precedence::{Direction, citation_precedence, direction_rule};
pub(crate) use resolution::unresolved_subject_diagnostic;
use resolution::unresolved_subject_diagnostic_indexed;
use selectors::{
    citation_matches_targets, label, select_subjects, target_kind_matches, target_nodes,
    target_wording,
};
use unreached::report_unreached;

#[cfg(test)]
mod tests_mixed_ramp;

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
    precedence: &[Direction],
    facts: &RuleFacts,
) -> Vec<Diagnostic> {
    evaluate_level(rules, precedence, facts, RuleLevel::Required, true)
}

/// Recommendations use the same evaluator and return the same existing
/// `Diagnostic` boundary; only the report channel chosen by the caller differs
/// (§AR-rules.1, §AR-rules.5).
pub(crate) fn evaluate_suggestions(
    rules: &[ParsedRule],
    precedence: &[Direction],
    facts: &RuleFacts,
) -> Vec<Diagnostic> {
    evaluate_level(rules, precedence, facts, RuleLevel::Recommended, false)
}

/// Share one immutable fact index across this level's semantic groups and
/// literal resolution (§FS-rules.5.3, §AR-rules.3.1).
fn evaluate_level(
    rules: &[ParsedRule],
    precedence: &[Direction],
    facts: &RuleFacts,
    level: RuleLevel,
    include_invalid: bool,
) -> Vec<Diagnostic> {
    let index = FactIndex::new(facts);
    let mut groups: BTreeMap<SemanticRule, BTreeSet<String>> = BTreeMap::new();
    let precedence = reported_by_direction(precedence);
    let mut out = Vec::new();
    for rule in rules {
        if let Some(diagnostic) = unresolved_subject_diagnostic_indexed(rule, facts, &index) {
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
        let authority = Authority::new(origins);
        let Some(reported) = precedence.get(&rule) else {
            evaluate_one(&rule, &authority, facts, &index, &mut out);
            continue;
        };
        // §FS-rules.6: a duplicate of a direction is evaluated all the same, and
        // loses only the findings the direction reported itself, unit by unit.
        let mut own = Vec::new();
        evaluate_one(&rule, &authority, facts, &index, &mut own);
        out.extend(own.into_iter().filter(|finding| {
            !reported
                .iter()
                .any(|direction| same_place(direction, finding))
        }));
    }
    out
}

/// Whether two findings are reported alike, as config-to-rule deduplication
/// compares them: the same code at the same unit or site (§FS-rules.6).
fn same_place(a: &Diagnostic, b: &Diagnostic) -> bool {
    a.code == b.code && a.path == b.path && a.line == b.line && a.column == b.column
}

/// Each direction's semantic constraint and everything entries expressing it
/// reported: entries that normalize alike are one direction (§FS-rules.6).
fn reported_by_direction(precedence: &[Direction]) -> BTreeMap<SemanticRule, Vec<&Diagnostic>> {
    let mut directions: BTreeMap<SemanticRule, Vec<&Diagnostic>> = BTreeMap::new();
    for (rule, reported) in precedence {
        directions
            .entry(SemanticRule::from(rule))
            .or_default()
            .extend(reported);
    }
    directions
}

/// Whether any of `rules` expresses the same constraint as one of `directions`
/// (§FS-rules.6), so that only then are the directions' findings gathered.
pub(crate) fn duplicates_a_direction(rules: &[ParsedRule], directions: &[ParsedRule]) -> bool {
    let directions = directions
        .iter()
        .map(SemanticRule::from)
        .collect::<BTreeSet<_>>();
    rules
        .iter()
        .any(|rule| directions.contains(&SemanticRule::from(rule)))
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

/// Each family reads only the selected units' indexed facts (§FS-rules.5.3),
/// retaining the finite-snapshot clauses (§FS-rules.5.2).
fn evaluate_one(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
    out: &mut Vec<Diagnostic>,
) {
    let subjects = select_subjects(&rule.subject, facts, index);
    // §FS-rules.4: every positive cardinality conclusion is closed-world, and
    // the absence of §FS-rules.5.2's second premise is one of them.
    if facts.header.completeness == Completeness::Incomplete
        && rule.polarity == RulePolarity::Positive
    {
        return;
    }
    // §FS-rules.7: the absence takes the channel its level already has, like
    // every other finding the level produces.
    report_unreached(rule, authority, facts, index, &subjects, out);
    match rule.relation {
        RuleRelation::HaveChapter => {
            let RuleTargets::Chapter(name) = &rule.targets else {
                return;
            };
            for subject in subjects {
                // §FS-rules.7.11: a chapter a stub's target lends counts as a scanned one.
                // §FS-fields.front-matter.surfaces: a front-matter unit, with no display
                // name, is neither counted nor listed.
                let chapters: Vec<(&NodeKey, &str, &str)> = index
                    .chapters_of(&subject)
                    .iter()
                    .filter_map(|row| {
                        let (node, coordinate, display_name) = &facts.chapter[*row];
                        Some((node, coordinate.as_str(), display_name.as_deref()?))
                    })
                    .collect();
                let count = chapters
                    .iter()
                    .filter(|(_, _, display_name)| display_name.eq_ignore_ascii_case(name))
                    .count();
                if !rule.cardinality.contains(count) {
                    // §FS-rules.7.2.1: show accepted direct chapters in source order,
                    // retaining the complete released prefix (§FS-errors.3).
                    let mut observed = chapters;
                    observed.sort_by_key(|(node, _, _)| {
                        facts.nodes.get(*node).map(|meta| meta.anchor.line)
                    });
                    let observed = if observed.is_empty() {
                        "none".to_string()
                    } else {
                        observed
                            .iter()
                            .map(|(_, coordinate, display_name)| {
                                format!("\"{coordinate}: {display_name}\"")
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    let message = format!(
                        "{} has {count} {name} chapters; {authority} requires {}; expected display name \"{name}\" (case-insensitive, not section handle); observed direct chapters: {observed}; {}",
                        label(facts, &subject),
                        rule.cardinality.wording(),
                        remedies::chapter_cardinality()
                    );
                    push_node(
                        out,
                        facts,
                        &subject,
                        Code::CHAPTER_CARDINALITY,
                        message,
                        authority,
                    );
                }
            }
        }
        RuleRelation::Cite if rule.polarity == RulePolarity::Prohibiting => {
            let targets = target_nodes(&rule.targets, facts, index);
            for subject in subjects {
                for (site, _, target) in index.citations_in(&subject) {
                    if citation_matches_targets(target, &targets, index) {
                        // §FS-rules.7.5: hard prohibitions reuse §FS-check.3.12's ordinary
                        // repair with `(<RULE-ID>)` authority; recommendations omit it.
                        // §FS-check.3.12.2 finalizes required index-entry repairs.
                        let (code, repair) = if rule.level == RuleLevel::Required {
                            (Code::FORBIDDEN_CITATION, citation_direction_repair())
                        } else {
                            (
                                Code::DISCOURAGED_CITATION,
                                format!(" — {}", remedies::discouraged_citation()),
                            )
                        };
                        let message = format!(
                            "{} {} not cite {} ({authority}){repair}",
                            index
                                .declaration_kind(&subject)
                                .map(str::to_owned)
                                .unwrap_or_else(|| label(facts, &subject)),
                            if rule.level == RuleLevel::Required {
                                "must"
                            } else {
                                "should"
                            },
                            target_wording(&rule.targets)
                        );
                        push_site(out, facts, site, code, message, authority);
                    }
                }
            }
        }
        RuleRelation::Cite => {
            evaluate_cites(rule, authority, facts, index, &subjects, out);
        }
        RuleRelation::BeCitedBy => {
            for subject in subjects {
                // §FS-rules.3.4: a numbered section of the subject's own body
                // reaches it here, an ordinary site since §FS-rules.7.8 landed, and a
                // citing kind counts wherever a pinned target matches it (§FS-config.3.9.3).
                let count = index
                    .citations_to(&subject)
                    .iter()
                    .filter(|(_, from, _)| {
                        index
                            .declaration_kind(from)
                            .is_some_and(|kind| target_kind_matches(&rule.targets, kind, facts))
                    })
                    .count();
                if !rule.cardinality.contains(count) {
                    let message = format!(
                        "{} is cited by {} {count} times; {authority} requires {}; {}",
                        label(facts, &subject),
                        target_wording(&rule.targets),
                        rule.cardinality.wording(),
                        remedies::uncited_unit()
                    );
                    push_node(out, facts, &subject, Code::UNCITED_UNIT, message, authority);
                }
            }
        }
    }
}

fn push_node(
    out: &mut Vec<Diagnostic>,
    facts: &RuleFacts,
    node: &NodeKey,
    code: Code,
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
    code: Code,
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
