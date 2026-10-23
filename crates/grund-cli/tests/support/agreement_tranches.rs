//! Each tranche of S4 is held to the dynamic whole-family boundary
//! (§REQ-one-answer.proof.matrix.family), complete independent facts and actual
//! completed calls (§REQ-one-answer.proof.evidence, §FS-distribution.3.3.1), by
//! one check every tranche calls with the families it delivers.

use super::{agreement_expectations as expectations, agreement_families as families};
use serde_json::json;
use std::collections::BTreeSet;

/// The families tranche 1 delivered with the compact fact model.
const TRANCHE_1: &[&str] = &["cover", "scan", "stub"];
/// The family tranche 2 delivered.
const TRANCHE_2: &[&str] = &["basic"];
/// The families tranche 3 delivers: every family of three cases or fewer.
const TRANCHE_3: &[&str] = &[
    "agent",
    "cross",
    "declare",
    "deterministic",
    "e2e",
    "external",
    "guide",
    "haskell",
    "json",
    "line",
    "markdown",
    "marked",
    "named",
    "path",
    "relative",
    "remote",
    "report",
    "rust",
];

fn members(cases: &BTreeSet<String>, family: &str) -> BTreeSet<String> {
    cases
        .iter()
        .filter(|case| families::family(case) == family)
        .cloned()
        .collect()
}

#[test]
fn s4_basic_family_runs_against_complete_authored_facts() {
    assert_covered_whole(TRANCHE_2, TRANCHE_1);
}

#[test]
#[ignore = "red until #674 agent, cross, declare, deterministic, e2e, external, guide, haskell, \
            json, line, markdown, marked, named, path, relative, remote, report and rust have \
            authored facts; implement removes this"]
fn s4_tranche_3_families_run_against_complete_authored_facts() {
    assert_covered_whole(TRANCHE_3, &[TRANCHE_1, TRANCHE_2].concat());
}

/// §REQ-one-answer.proof.matrix.family: every member of each `delivered` family
/// enters the same full receipt acceptance as the retained seed and covered
/// families, and every `retained` family stays covered. Preflight is
/// deliberately before scope checks and native preparation, so missing facts
/// cannot be hidden by leaving a family outside COVERED.
fn assert_covered_whole(delivered: &[&str], retained: &[&str]) {
    let cases = families::cases_root().canonicalize().expect("corpus");
    let discovered = families::discovered();
    let mut asked = BTreeSet::new();
    for family in delivered {
        let found = members(&discovered, family);
        assert!(!found.is_empty(), "the {family} family must exist");
        asked.extend(found);
    }
    let manifests = expectations::load(&cases, &asked).unwrap_or_else(|errors| {
        panic!(
            "independent truth for {delivered:?} is unfinished for dynamically discovered \
             cases {asked:?}:\n{}\nNo native preparation or matrix execution attempted.",
            errors.join("\n")
        )
    });
    assert_eq!(manifests.keys().cloned().collect::<BTreeSet<_>>(), asked);

    // §REQ-one-answer.proof.evidence: the input closure and question closure
    // must reject deleting a physical declaration/section/citation/alias and
    // all questions together. These retained controls use authored input bytes.
    super::agreement_query_controls::declaration_section_alias_subsets_fail_even_with_all_their_calls_deleted();
    super::agreement_query_controls::physical_bytes_and_membership_reject_deleting_or_rewriting_the_inventory_input();

    let scope = families::scope(&discovered);
    assert!(
        asked.is_subset(&scope),
        "facts exist but cases are absent from completed-call acceptance: {:?}",
        asked.difference(&scope).collect::<Vec<_>>()
    );
    for family in delivered {
        assert!(super::COVERED.contains(family), "{family} is not COVERED");
        assert!(
            !super::WIDENING.iter().any(|(row, _)| row == family),
            "{family} is still WIDENING"
        );
    }
    for family in retained {
        assert!(super::COVERED.contains(family), "lost {family} proof");
    }
    assert_new_members_require_facts(&discovered, delivered);

    // This asserts genuine CLI/editor/package calls, exact applicability and
    // authored question receipts, not a count or a family label. Once shares
    // the execution with the ordinary acceptance without duplicating the matrix.
    super::agreement_acceptance::s4_executes_the_covered_families_and_the_seed();
}

fn assert_new_members_require_facts(discovered: &BTreeSet<String>, delivered: &[&str]) {
    let mut new_cases = BTreeSet::new();
    let mut extended = discovered.clone();
    for family in delivered {
        let new_case = format!("{family}-new-member-without-authored-facts");
        assert!(!discovered.contains(&new_case), "control name became real");
        extended.insert(new_case.clone());
        assert!(members(&extended, family).contains(&new_case));
        assert!(families::scope(&extended).contains(&new_case));
        new_cases.insert(new_case);
    }

    // Isolated corpus with the real directory layout used by the fact loader;
    // no mutation of the repository's fixture tree or native setup is needed.
    let configs: Vec<String> = new_cases
        .iter()
        .map(|case| format!("tests/e2e/cases/{case}/repo/grund.toml"))
        .collect();
    let mut files: Vec<(&str, &str)> = configs
        .iter()
        .map(|path| (path.as_str(), super::agreement_tree::CONFIG_SLUG))
        .collect();
    files.push(("tests/integration/command_agreement_facts/.keep", ""));
    let control = super::agreement_tree::Tree::new("grund-674-new-members", &files);
    let errors = expectations::load(&control.0.join("tests/e2e/cases"), &new_cases)
        .expect_err("new members without truth must fail before native setup");
    for new_case in &new_cases {
        assert!(
            errors
                .iter()
                .any(|error| error.starts_with(&format!("{new_case}: no input-authored manifest"))),
            "missing new member {new_case} was not diagnosed: {errors:?}"
        );
    }
}

/// §REQ-one-answer.proof.evidence: Node metadata cannot disappear with a
/// common-domain projection. Expectations below are authored control inputs,
/// never captured engine answers or a substitute for corpus facts.
#[test]
fn s4_node_show_metadata_corruption_is_rejected() {
    let tree = super::agreement_tree::Tree::new("grund-674-node-metadata", &[]);
    let context = json!({"base":tree.0,"saved":tree.0,"reportBase":tree.0,
        "path":tree.0,"explicit":false,"scopes":{},"files":[],"textFiles":[]});
    let shown = json!({"id":"FS-alpha","section":null,"kind_title":"Features",
        "manifest":null,"path":"<repo>/docs/fs.md","line":1,
        "body":"Authored body.\n","sections":[],"json":json!({
            "id":"FS-alpha","section":null,"kind_title":"Features",
            "path":"<repo>/docs/fs.md","line":1,"anchor":"fs-alpha-alpha",
            "body":"Authored body.\n","sections":[]}).to_string()});
    for question in super::agreement_questions::tree_questions(&context)
        .into_iter()
        .chain(super::agreement_questions::coordinate_questions(
            &context, "FS-alpha",
        ))
        .filter(|q| matches!(q.surface, "show" | "show-batch"))
    {
        let result = if question.surface == "show" {
            shown.clone()
        } else {
            json!({"records":[{"query":{"id":"FS-alpha","section":null},
                "ok":true,"result":shown}],"note":null})
        };
        let expected = json!({"failure":null,"result":result,"run_cautions":[]});
        let mut actual = expected.clone();
        let shown_pointer = if question.surface == "show" {
            "/result"
        } else {
            "/result/records/0/result"
        };
        let shown = actual.pointer_mut(shown_pointer).unwrap();
        shown["path"] = json!("docs/fs.md");
        let mut object: serde_json::Value =
            serde_json::from_str(shown["json"].as_str().unwrap()).unwrap();
        object["path"] = json!("docs/fs.md");
        shown["json"] = json!(object.to_string());
        let key = super::agreement_truth::key(&question);
        let mut bound = context.clone();
        bound["truth"]["calls"][&key] = json!({"point":question.point,
            "authoredFrom":"explicit control envelope and input body","node":expected});
        let response = |answer: &serde_json::Value| {
            let mut errors = Vec::new();
            super::agreement_truth::host(&bound, &question, "node", answer, &mut errors);
            errors
        };
        assert!(response(&actual).is_empty(), "correct envelope rejected");
        for field in ["id", "section", "kind_title", "manifest"] {
            let pointer = if question.surface == "show" {
                format!("/result/{field}")
            } else {
                format!("/result/records/0/result/{field}")
            };
            let mut wrong = actual.clone();
            *wrong.pointer_mut(&pointer).expect("authored metadata") = json!("corrupt");
            assert_eq!(
                super::agreement_host_projection::node_common(
                    wrong.clone(),
                    question.node_operation
                ),
                super::agreement_host_projection::node_common(
                    actual.clone(),
                    question.node_operation
                ),
                "control must leave the common projection equal"
            );
            assert!(
                !response(&wrong).is_empty(),
                "{} changed {field} escaped independent truth",
                question.surface
            );
        }
    }
}
