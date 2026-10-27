//! A target pinned at the rule's own nested member by its whole alias path
//! matches what the bare kind matches (§FS-rules.2, §FS-config.3.9.3), in both
//! relations. Every required finding that match produces is the code's ordinary
//! error since the window that warned closed in 0.19.0 (§FS-rules.7.9): a
//! prohibition, an inbound ceiling, a count and a per-target finding alike, with
//! no clause after the message. A numbered-section site counts as any other from
//! 0.18.0 (§FS-rules.7.8), so neither closed window's clause is left to print.

use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use crate::model::Diagnostic;
use std::collections::BTreeMap;

/// The required level's findings: the one list `evaluate` collapsed to once no
/// required finding can warn (§FS-rules.7.9).
pub(super) trait Required {
    fn required(self) -> Vec<Diagnostic>;
}

impl Required for Vec<Diagnostic> {
    fn required(self) -> Vec<Diagnostic> {
        self
    }
}

fn anchor(line: usize) -> RuleAnchor {
    RuleAnchor {
        path: "docs/api.md".into(),
        line,
        column: None,
    }
}

/// The facts of member `group/api` (§FS-workspace.6.1): `API-a` cites its own
/// `AR-b` at line 3 and sibling member `group/web`'s `AR-c` at line 4. Local
/// kinds are bare and foreign ones carry the alias path, as the adapter writes them.
pub(super) fn facts() -> RuleFacts {
    let node = |key: &str, label: &str, line| {
        (
            NodeKey(key.into()),
            NodeMeta {
                label: label.into(),
                anchor: anchor(line),
            },
        )
    };
    let site = |key: &str, label: &str, line| {
        (
            SiteKey(key.into()),
            SiteMeta {
                label: label.into(),
                anchor: RuleAnchor {
                    column: Some(3),
                    ..anchor(line)
                },
            },
        )
    };
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "group/api".into(),
            producer: "hand-built".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![
            (NodeKey("api".into()), "API".into()),
            (NodeKey("ar".into()), "AR".into()),
            (NodeKey("web".into()), "group/web/AR".into()),
        ],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: vec![
            (
                SiteKey("own".into()),
                NodeKey("api".into()),
                NodeKey("ar".into()),
            ),
            (
                SiteKey("sibling".into()),
                NodeKey("api".into()),
                NodeKey("web".into()),
            ),
        ],
        site_in: vec![
            (SiteKey("own".into()), NodeKey("api".into())),
            (SiteKey("sibling".into()), NodeKey("api".into())),
        ],
        nodes: BTreeMap::from([
            node("api", "API-a", 1),
            node("ar", "AR-b", 20),
            node("web", "group/web/AR-c", 30),
        ]),
        sites: BTreeMap::from([site("own", "AR-b", 3), site("sibling", "group/web/AR-c", 4)]),
    }
}

/// `Each API must cite …` over `targets`, with the given count.
pub(super) fn cites(targets: &[&str], cardinality: Cardinality) -> ParsedRule {
    ParsedRule {
        origin: "RULE-x".into(),
        anchor: anchor(40),
        subject: RuleSubject::Kind("API".into(), None),
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: targets.iter().map(|t| t.to_string()).collect(),
            mode: TargetMode::Aggregate,
        },
        cardinality,
    }
}

/// `Each API must not cite any …` over `targets`.
fn bans(targets: &[&str]) -> ParsedRule {
    ParsedRule {
        polarity: RulePolarity::Prohibiting,
        ..cites(targets, Cardinality::NONE)
    }
}

/// `Each AR must be cited by …` over `targets`, with the given count.
fn cited_by(targets: &[&str], cardinality: Cardinality) -> ParsedRule {
    ParsedRule {
        subject: RuleSubject::Kind("AR".into(), None),
        relation: RuleRelation::BeCitedBy,
        ..cites(targets, cardinality)
    }
}

const AT_LEAST_ONE: Cardinality = Cardinality {
    minimum: Some(1),
    maximum: None,
};

pub(super) fn at_most(maximum: usize) -> Cardinality {
    Cardinality {
        minimum: None,
        maximum: Some(maximum),
    }
}

pub(super) fn codes(diagnostics: &[Diagnostic]) -> Vec<(&str, Option<usize>)> {
    diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.line))
        .collect()
}

/// The closing clauses the nested-alias window (§FS-rules.7.9) and the
/// counted-section window (§FS-rules.7.8) appended; neither is printed now.
const CLOSED_CLAUSES: [&str; 3] = [
    "; a target pinned at the rule's own nested alias path now matches",
    "; a citation to a numbered section now counts",
    "grund 0.",
];

/// Asserts that every finding is printed in its code's ordinary wording.
pub(super) fn assert_ordinary(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        let message = &diagnostic.message;
        for clause in CLOSED_CLAUSES {
            assert!(!message.contains(clause), "{clause:?} in {message}");
        }
    }
}

#[test]
fn an_own_nested_alias_pin_satisfies_a_cite_rule_like_the_bare_kind() {
    for target in ["AR", "group/api/AR"] {
        let errors = evaluate(&[cites(&[target], AT_LEAST_ONE)], &[], &facts()).required();
        assert!(errors.is_empty(), "{target}: {:?}", codes(&errors));
    }
}

#[test]
fn an_own_nested_alias_pin_satisfies_an_inbound_rule_like_the_bare_kind() {
    for target in ["API", "group/api/API"] {
        let errors = evaluate(&[cited_by(&[target], AT_LEAST_ONE)], &[], &facts()).required();
        assert!(errors.is_empty(), "{target}: {:?}", codes(&errors));
    }
}

#[test]
fn a_prohibition_only_the_own_nested_alias_pin_reaches_is_an_error() {
    let errors = evaluate(&[bans(&["group/api/AR"])], &[], &facts()).required();
    assert_eq!(codes(&errors), [("forbidden-citation", Some(3))]);
    assert_eq!(
        errors[0].message,
        "API must not cite group/api/AR (RULE-x) — re-point the citation or downgrade it to a plain Markdown link"
    );
}

#[test]
fn an_inbound_ceiling_only_the_own_nested_alias_pin_breaks_is_an_error() {
    let errors = evaluate(&[cited_by(&["group/api/API"], at_most(0))], &[], &facts()).required();
    assert_eq!(codes(&errors), [("uncited-unit", Some(20))]);
    assert_ordinary(&errors);
}

#[test]
fn a_count_the_corrected_match_breaks_is_an_error_with_or_without_it() {
    let targets = ["group/api/AR", "group/web/AR"];
    for maximum in [1, 0] {
        let errors = evaluate(&[cites(&targets, at_most(maximum))], &[], &facts()).required();
        assert_eq!(
            codes(&errors),
            [("citation-cardinality", Some(1))],
            "at most {maximum}"
        );
        assert_ordinary(&errors);
    }
}

#[test]
fn a_site_the_bare_kind_already_matched_stays_an_error() {
    let errors = evaluate(&[bans(&["AR", "group/api/AR"])], &[], &facts()).required();
    assert_eq!(codes(&errors), [("forbidden-citation", Some(3))]);
    assert_ordinary(&errors);
}

#[test]
fn bare_kinds_any_member_and_other_member_pins_judge_as_before() {
    for (target, lines) in [
        ("AR", vec![3]),
        ("*/AR", vec![3, 4]),
        ("group/web/AR", vec![4]),
    ] {
        let errors = evaluate(&[bans(&[target])], &[], &facts()).required();
        let found: Vec<_> = errors
            .iter()
            .map(|d| (d.code.as_str(), d.line.unwrap()))
            .collect();
        let expected: Vec<_> = lines.iter().map(|&l| ("forbidden-citation", l)).collect();
        assert_eq!(found, expected, "{target}");
    }
}

#[test]
fn a_recommended_prohibition_through_the_pin_is_an_ordinary_suggestion() {
    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..bans(&["group/api/AR"])
    };
    let suggestions = evaluate_suggestions(&[recommended], &[], &facts());
    assert_eq!(codes(&suggestions), [("discouraged-citation", Some(3))]);
    assert_ordinary(&suggestions);
}

/// `Each API must cite each …` over `targets`, with the given count.
pub(super) fn cites_each(targets: &[&str], cardinality: Cardinality) -> ParsedRule {
    let rule = cites(targets, cardinality);
    ParsedRule {
        targets: RuleTargets::Kinds {
            values: targets.iter().map(|t| t.to_string()).collect(),
            mode: TargetMode::PerTarget,
        },
        ..rule
    }
}

/// [`facts`] with `AR-c` declared in `group/api` itself and cited by nothing.
fn facts_with_uncited_own_target() -> RuleFacts {
    let mut facts = facts();
    facts.decl.push((NodeKey("ar-c".into()), "AR".into()));
    facts.nodes.insert(
        NodeKey("ar-c".into()),
        NodeMeta {
            label: "AR-c".into(),
            anchor: anchor(25),
        },
    );
    facts
}

/// [`facts`] with `API-a` citing `AR-b` twice more, once through its numbered
/// section `<§>AR-b.1`, a site counted as any other since §FS-rules.7.8 closed.
fn facts_with_section_site() -> RuleFacts {
    let mut facts = facts();
    for (key, line) in [("again", 5), ("section", 6)] {
        let site = SiteKey(key.into());
        facts
            .cites
            .push((site.clone(), NodeKey("api".into()), NodeKey("ar".into())));
        facts.site_in.push((site.clone(), NodeKey("api".into())));
        facts.sites.insert(
            site,
            SiteMeta {
                label: "AR-b".into(),
                anchor: anchor(line),
            },
        );
    }
    facts
}

#[test]
fn a_per_target_miss_is_an_error_through_the_pin_and_the_bare_kind() {
    for target in ["group/api/AR", "AR"] {
        let errors = evaluate(
            &[cites_each(&[target], AT_LEAST_ONE)],
            &[],
            &facts_with_uncited_own_target(),
        )
        .required();
        assert_eq!(
            codes(&errors),
            [("citation-cardinality", Some(1))],
            "{target}"
        );
        let message = &errors[0].message;
        assert!(message.contains("API-a cites AR-c 0 times"), "{message}");
        assert_ordinary(&errors);
    }
}

#[test]
fn a_per_target_bound_is_an_error_through_the_pin_and_the_bare_kind() {
    let exactly_once = Cardinality {
        minimum: Some(1),
        maximum: Some(1),
    };
    for target in ["group/api/AR", "AR"] {
        let errors = evaluate(
            &[cites_each(&[target], exactly_once)],
            &[],
            &facts_with_section_site(),
        )
        .required();
        assert_eq!(
            codes(&errors),
            [("citation-cardinality", Some(1))],
            "{target}"
        );
        let message = &errors[0].message;
        assert!(message.contains("API-a cites AR-b 3 times"), "{message}");
        assert_ordinary(&errors);
    }
}

/// Every mode of `Each API must cite at most 2 …` over `targets`.
fn at_most_two(targets: &[&str]) -> [ParsedRule; 2] {
    [cites(targets, at_most(2)), cites_each(targets, at_most(2))]
}

#[test]
fn a_bound_the_section_site_breaks_is_an_error_through_the_pin_and_the_bare_kind() {
    for target in ["group/api/AR", "AR"] {
        for rule in at_most_two(&[target]) {
            let mode = rule.targets.clone();
            let errors = evaluate(&[rule], &[], &facts_with_section_site()).required();
            assert_eq!(
                codes(&errors),
                [("citation-cardinality", Some(1))],
                "{mode:?}"
            );
            assert_ordinary(&errors);
        }
    }
}
