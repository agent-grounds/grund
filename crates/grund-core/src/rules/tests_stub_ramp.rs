//! The window of a rule declared through a stub whose target the scan does not
//! read (§FS-rules.7.10): a required finding only such rules author warns and
//! names 0.19.0; one another rule also authors stays an error; an
//! `invalid-rule` about the rule stays an error; and a finding §FS-rules.7.8 or
//! §FS-rules.7.9 already ramped keeps its clauses and gains this one after them.

use super::engine::route_stubbed_rules;
use crate::model::Diagnostic;
use std::collections::BTreeSet;

const STUB: &str = "; a rule declared through a stub now applies, and this warning becomes an error in grund 0.19.0";
const COUNTED: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";
const MATCHED: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.18.0";

fn finding(code: &'static str, message: &str, authority: &[&str]) -> Diagnostic {
    Diagnostic {
        code,
        path: Some("docs/fs/a.md".into()),
        line: Some(3),
        column: None,
        message: message.to_string(),
        sites: Vec::new(),
        authority: authority.iter().map(|origin| origin.to_string()).collect(),
    }
}

fn stubbed() -> BTreeSet<String> {
    BTreeSet::from(["RULE-r".to_string()])
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics.iter().map(|d| d.message.as_str()).collect()
}

#[test]
fn a_finding_only_a_stubbed_rule_authors_warns_until_0_19_0() {
    let mut errors = vec![finding(
        "forbidden-citation",
        "FS must not cite GOAL (RULE-r)",
        &["RULE-r"],
    )];
    let mut warnings = Vec::new();
    route_stubbed_rules(&stubbed(), &mut errors, &mut warnings);
    assert!(errors.is_empty());
    assert_eq!(
        messages(&warnings),
        vec![format!("FS must not cite GOAL (RULE-r){STUB}").as_str()]
    );
    assert_eq!(
        warnings[0].code, "forbidden-citation",
        "the code it will carry as an error"
    );
}

#[test]
fn a_finding_an_inline_rule_also_authors_stays_an_error() {
    let mut errors = vec![finding(
        "forbidden-citation",
        "FS must not cite GOAL (RULE-q, RULE-r)",
        &["RULE-q", "RULE-r"],
    )];
    let mut warnings = Vec::new();
    route_stubbed_rules(&stubbed(), &mut errors, &mut warnings);
    assert_eq!(
        messages(&errors),
        vec!["FS must not cite GOAL (RULE-q, RULE-r)"]
    );
    assert!(warnings.is_empty());
}

#[test]
fn an_invalid_rule_about_a_stubbed_rule_stays_an_error() {
    let mut errors = vec![finding(
        "invalid-rule",
        "RULE-r is not a valid rule: x",
        &["RULE-r"],
    )];
    let mut warnings = Vec::new();
    route_stubbed_rules(&stubbed(), &mut errors, &mut warnings);
    assert_eq!(errors.len(), 1);
    assert!(warnings.is_empty());
}

#[test]
fn a_finding_ramped_by_7_8_and_7_9_carries_every_clause_in_order() {
    let mut errors = Vec::new();
    let mut warnings = vec![
        finding("citation-cardinality", &format!("m{COUNTED}"), &["RULE-r"]),
        finding(
            "citation-cardinality",
            &format!("m{COUNTED}{MATCHED}"),
            &["RULE-r"],
        ),
        finding("citation-cardinality", &format!("m{COUNTED}"), &["RULE-q"]),
    ];
    route_stubbed_rules(&stubbed(), &mut errors, &mut warnings);
    assert!(errors.is_empty());
    assert_eq!(
        messages(&warnings),
        vec![
            format!("m{COUNTED}{STUB}").as_str(),
            format!("m{COUNTED}{MATCHED}{STUB}").as_str(),
            format!("m{COUNTED}").as_str(),
        ]
    );
}

#[test]
fn no_stubbed_rule_moves_nothing() {
    let mut errors = vec![finding("forbidden-citation", "m", &["RULE-r"])];
    let mut warnings = Vec::new();
    route_stubbed_rules(&BTreeSet::new(), &mut errors, &mut warnings);
    assert_eq!(errors.len(), 1);
    assert!(warnings.is_empty());
}
