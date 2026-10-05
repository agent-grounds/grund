//! A resolved citation to a section that is no rule unit is a `cites` fact to
//! its nearest enclosing unit, on both ends; one that does not resolve is none
//! (§FS-rules.5.1, §FS-rules.3.4).

use super::facts::{NodeKey, RuleFacts};
use super::markdown::adapt_markdown;
use crate::testing::{numbered_config, scan_findings, test_root, write};

/// `FS-002-target` has a numbered section `.4` and a named `outcome` chapter
/// with a numbered child; `FS-003-twin` is declared twice, so it is ambiguous.
fn facts(name: &str, source_body: &str) -> RuleFacts {
    let root = test_root(name);
    write(
        &root.join("docs/functional-spec/FS-002-target.md"),
        "# FS-002-target: Target\n\nThe cited declaration.\n\n\
         ## 4. Numbered\n\nA numbered section.\n\n\
         ## outcome: Outcome\n\nA named chapter.\n\n\
         ### outcome.2: Detail\n\nA numbered child of the chapter.\n",
    );
    write(
        &root.join("docs/functional-spec/FS-003-twin.md"),
        "# FS-003-twin: Twin\n\nOne home.\n\n## 1. One\n\nBody.\n",
    );
    write(
        &root.join("docs/functional-spec/FS-004-twin.md"),
        "# FS-003-twin: Twin again\n\nThe other home.\n\n## 1. One\n\nBody.\n",
    );
    write(
        &root.join("docs/functional-spec/FS-001-source.md"),
        &format!("# FS-001-source: Source\n\nThe citing declaration.\n\n{source_body}"),
    );
    let mut config = numbered_config(root.clone());
    config.named_sections = true;
    config.rebuild_grammar().expect("rebuild named grammar");
    let findings = scan_findings(&config, &root);
    adapt_markdown(&findings, &config, true)
}

fn label(facts: &RuleFacts, node: &NodeKey) -> String {
    facts.nodes[node].label.clone()
}

/// Every `cites` fact as `(from, target)` labels, in fact order.
fn edges(facts: &RuleFacts) -> Vec<(String, String)> {
    facts
        .cites
        .iter()
        .map(|(_, from, target)| (label(facts, from), label(facts, target)))
        .collect()
}

#[test]
fn numbered_section_citation_targets_its_declaration() {
    let facts = facts(
        "rules_section_target_declaration",
        "## goal: Goal\n\nValidates §FS-002-target.4 here.\n",
    );
    assert_eq!(
        edges(&facts),
        [(
            "FS-001-source.goal".to_string(),
            "FS-002-target".to_string()
        )],
        "a resolved numbered section counts for its declaration"
    );
}

#[test]
fn numbered_child_of_a_chapter_targets_that_chapter() {
    let facts = facts(
        "rules_section_target_chapter",
        "## goal: Goal\n\nValidates §FS-002-target.outcome.2 here.\n",
    );
    assert_eq!(
        edges(&facts),
        [(
            "FS-001-source.goal".to_string(),
            "FS-002-target.outcome".to_string()
        )],
        "a numbered child counts for its nearest named ancestor chapter"
    );
}

#[test]
fn site_inside_a_numbered_section_comes_from_its_named_ancestor() {
    let facts = facts(
        "rules_section_target_from",
        "## goal: Goal\n\nThe aim.\n\n### goal.3: Third\n\nValidates §FS-002-target here.\n",
    );
    assert_eq!(
        edges(&facts),
        [(
            "FS-001-source.goal".to_string(),
            "FS-002-target".to_string()
        )],
        "the immediate `from` is the nearest named chapter, not the declaration"
    );
}

/// A guard: it holds before and after the change. The one resolved citation
/// beside them is the control that shows the others were read and dropped.
#[test]
fn unresolved_and_ambiguous_coordinates_contribute_no_fact() {
    let facts = facts(
        "rules_section_target_unresolved",
        "## goal: Goal\n\nCites §FS-002-target.9, §FS-002-target.outcome.7, \
         §FS-003-twin.1, §FS-009-missing.4, and §FS-002-target.outcome here.\n",
    );
    assert_eq!(
        edges(&facts),
        [(
            "FS-001-source.goal".to_string(),
            "FS-002-target.outcome".to_string()
        )],
        "only the resolved citation becomes a fact"
    );
    assert_eq!(facts.sites.len(), 1, "no site is recorded for the others");
}
