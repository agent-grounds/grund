//! A rule's share of the citation-direction obligation (§FS-check.2.2.1.4):
//! what each parsed rule arms beside the rule checks (§FS-check.1.4.1), and the
//! `applies to nothing` line a `must` or `should` rule earns when its subject
//! kind gives it nothing to judge. The table's pass asks the same question in
//! `citations.rs`, and both answer it with the one function there.

use std::collections::{BTreeMap, BTreeSet};

use super::citations::{Obligor, can_apply_to_nothing, empty_citation_obligation_warning};
use super::obligation_units::obligation_units;
use crate::config::{Frame, Owners, Rules, Schema};
use crate::model::{Armed, Catalog, Diagnostic, Switch};
use crate::rules::sentence::{ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject};

/// §FS-check.1.4.1: what each rule turns on beside the rule checks — the obligation
/// a `must` or `should` rule holds its subject to, where that subject can have no
/// unit, and the direction a citation rule reports at its level. A rule the run
/// could not read may have been any of them, and its `invalid-rule` row says why
/// it judged nothing, so it arms them all: every code these switches turn on is a
/// rule-produced code, which the selection carries that row with (§FS-rules.7.6).
pub(super) fn arm_rule_directions(
    parsed_rules: &[ParsedRule],
    schema: &Schema,
    unread: bool,
    armed: &mut Armed,
) {
    if unread {
        for switch in [
            Switch::Obligation,
            Switch::MustDirection,
            Switch::MustNotDirection,
            Switch::ShouldDirection,
            Switch::ShouldNotDirection,
        ] {
            armed.arm(switch);
        }
    }
    for rule in parsed_rules {
        let cites = rule.relation != RuleRelation::HaveChapter;
        match (rule.polarity, rule.level) {
            (RulePolarity::Positive, level) => {
                if obligation_subject(rule)
                    .is_some_and(|kind| can_apply_to_nothing(schema, kind, true).is_some())
                {
                    armed.arm(Switch::Obligation);
                }
                if cites {
                    armed.arm(match level {
                        RuleLevel::Required => Switch::MustDirection,
                        RuleLevel::Recommended => Switch::ShouldDirection,
                    });
                }
            }
            (RulePolarity::Prohibiting, _) if !cites => {}
            (RulePolarity::Prohibiting, RuleLevel::Required) => {
                armed.arm(Switch::MustNotDirection);
            }
            (RulePolarity::Prohibiting, RuleLevel::Recommended) => {
                armed.arm(Switch::ShouldNotDirection);
            }
        }
    }
}

/// §FS-check.2.2.1.4: one `empty-citation-obligation` line per `must` or `should`
/// rule whose subject kind gives it nothing to judge. A prohibition has no unit
/// and never warns, as a prohibition-only table does not.
pub(super) fn empty_rule_obligations(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    parsed_rules: &[ParsedRule],
) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    parsed_rules
        .iter()
        .filter_map(|rule| {
            let kind = obligation_subject(rule)?;
            if !seen.insert((rule.origin.as_str(), kind)) {
                return None;
            }
            empty_rule_obligation_warning(rules, schema, frame, findings, kind, &rule.origin)
        })
        .collect()
}

/// The kind a `must` or `should` rule holds to an obligation in this project, which
/// the line above and the switch that arms it ask alike: none for a prohibition or
/// an exact subject, and none for a kind of another project, which is that
/// project's to ask.
fn obligation_subject(rule: &ParsedRule) -> Option<&str> {
    if rule.polarity != RulePolarity::Positive {
        return None;
    }
    match &rule.subject {
        RuleSubject::Kind(kind, _) | RuleSubject::ChapterOfKind { kind, .. } => {
            (!kind.contains('/')).then_some(kind.as_str())
        }
        RuleSubject::ExactDeclaration(_) | RuleSubject::ExactChapter { .. } => None,
    }
}

/// §FS-check.2.2.1.4: a `must` or `should` rule about `citing_kind` earns the
/// table's warning when the ordinary unit derivation finds no unit — condition 4,
/// asked of the same derivation the table's pass runs — and conditions 2 and 3
/// hold, which the one function below decides for both.
fn empty_rule_obligation_warning(
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &Catalog,
    citing_kind: &str,
    origin: &str,
) -> Option<Diagnostic> {
    // A citable kind's units are its declarations; the citation maps only fill them.
    let owners = Owners::new(schema, frame);
    let no_citations = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    let units = obligation_units(
        citing_kind,
        rules,
        &owners,
        findings,
        &no_citations.0,
        &no_citations.1,
        &no_citations.2,
    );
    if !units.is_empty() {
        return None;
    }
    empty_citation_obligation_warning(
        rules,
        schema,
        frame,
        findings,
        citing_kind,
        Obligor::Rule(origin),
    )
}
