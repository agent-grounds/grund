//! Post-scan resolution of a literal subject (§FS-rules.4): whether the one
//! declaration or chapter a rule spells by name is in the snapshot, and the
//! `invalid-rule` finding the rule earns at its own site when it is not
//! (§FS-rules.7.1). Nothing here evaluates a family — resolution is a question
//! about the sentence rather than about the tree it judges (§AR-rules.4).

use super::super::facts::RuleFacts;
use super::super::{ParsedRule, RuleSubject};
use super::authority::one_rules_authority;
use super::selectors::select_subjects;
use crate::model::Diagnostic;

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
        authority: one_rules_authority(&rule.origin),
    })
}
