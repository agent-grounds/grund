//! Existing citation directions normalized for engine-owned semantic
//! precedence (§FS-rules.6, §AR-rules.1).

use crate::config::{Config, render_citation_target};
use crate::rules::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};

pub(crate) fn citation_precedence(config: &Config) -> Vec<ParsedRule> {
    let mut rules = Vec::new();
    for (subject, directions) in &config.citations.per_kind {
        for (level, polarity, entries) in [
            (
                RuleLevel::Required,
                RulePolarity::Positive,
                &directions.must,
            ),
            (
                RuleLevel::Recommended,
                RulePolarity::Positive,
                &directions.should,
            ),
            (
                RuleLevel::Required,
                RulePolarity::Prohibiting,
                &directions.must_not,
            ),
            (
                RuleLevel::Recommended,
                RulePolarity::Prohibiting,
                &directions.should_not,
            ),
        ] {
            for entry in entries {
                let mut values = entry
                    .targets
                    .iter()
                    .map(render_citation_target)
                    .collect::<Vec<_>>();
                values.sort();
                values.dedup();
                rules.push(ParsedRule {
                    origin: "citation direction".into(),
                    anchor: RuleAnchor {
                        path: String::new(),
                        line: 0,
                        column: None,
                    },
                    subject: RuleSubject::Kind(subject.clone()),
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
                });
            }
        }
    }
    rules
}
