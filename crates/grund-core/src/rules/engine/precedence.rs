//! Existing citation directions normalized for engine-owned semantic
//! precedence (§FS-rules.6, §AR-rules.1).

use crate::config::{CitationDisjunction, CitationLevel, CitationRules, render_citation_target};
use crate::model::Diagnostic;
use crate::rules::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};

/// One `[citations]` entry as an engine rule, with the findings the direction
/// itself reported. A duplicate rule's finding is absorbed only where one of
/// these names the same code at the same unit or site (§FS-rules.6), because the
/// direction counts a token the rule does not. It is a pair rather than a type
/// so that the engine/checker crossing stays `Diagnostic` (§AR-rules.5).
pub(crate) type Direction = (ParsedRule, Vec<Diagnostic>);

/// The project's `[citations]` directions as engine rules; it reads the rules
/// record alone, not the configuration façade (§AR-config.5).
pub(crate) fn citation_precedence(citations: &CitationRules) -> Vec<ParsedRule> {
    let mut rules = Vec::new();
    for (subject, directions) in &citations.per_kind {
        for (level, entries) in directions.lists() {
            for entry in entries {
                rules.extend(direction_rule(subject, level, entry));
            }
        }
    }
    rules
}

/// The engine rule one entry at `level` expresses, or `None` for a level no rule
/// sentence expresses (§FS-rules.6): `warn`, `warn-not` and `may`.
pub(crate) fn direction_rule(
    subject: &str,
    level: CitationLevel,
    entry: &CitationDisjunction,
) -> Option<ParsedRule> {
    let (level, polarity) = match level {
        CitationLevel::Must => (RuleLevel::Required, RulePolarity::Positive),
        CitationLevel::Should => (RuleLevel::Recommended, RulePolarity::Positive),
        CitationLevel::MustNot => (RuleLevel::Required, RulePolarity::Prohibiting),
        CitationLevel::ShouldNot => (RuleLevel::Recommended, RulePolarity::Prohibiting),
        CitationLevel::Warn | CitationLevel::WarnNot | CitationLevel::May => return None,
    };
    let mut values = entry
        .targets
        .iter()
        .map(render_citation_target)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Some(ParsedRule {
        origin: "citation direction".into(),
        anchor: RuleAnchor {
            path: String::new(),
            line: 0,
            column: None,
        },
        subject: RuleSubject::Kind(subject.to_string(), None),
        level,
        polarity,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values,
            mode: TargetMode::Aggregate,
        },
        cardinality: match polarity {
            RulePolarity::Positive => Cardinality::AT_LEAST_ONE,
            RulePolarity::Prohibiting => Cardinality::NONE,
        },
    })
}
