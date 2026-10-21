//! The basic-family tranche is held to the dynamic whole-family boundary
//! (§REQ-one-answer.proof.matrix.family), complete independent facts and actual
//! completed calls (§REQ-one-answer.proof.evidence, §FS-distribution.3.3.1).

use super::{agreement_expectations as expectations, agreement_families as families};
use serde_json::json;
use std::collections::BTreeSet;

fn basic_members(cases: &BTreeSet<String>) -> BTreeSet<String> {
    cases
        .iter()
        .filter(|case| families::family(case) == "basic")
        .cloned()
        .collect()
}

/// §REQ-one-answer.proof.matrix.family: every basic member enters the same
/// full receipt acceptance as the retained seed and covered families.
/// Preflight is deliberately before scope checks and native preparation, so
/// missing facts cannot be hidden by leaving basic outside COVERED.
#[test]
fn s4_basic_family_runs_against_complete_authored_facts() {
    let cases = families::cases_root().canonicalize().expect("corpus");
    let discovered = families::discovered();
    let basic = basic_members(&discovered);
    assert!(!basic.is_empty(), "the basic family must exist");
    let manifests = expectations::load(&cases, &basic).unwrap_or_else(|errors| {
        panic!(
            "basic-family independent truth is unfinished for dynamically discovered cases \
             {basic:?}:\n{}\nNo native preparation or matrix execution attempted.",
            errors.join("\n")
        )
    });
    assert_eq!(manifests.keys().cloned().collect::<BTreeSet<_>>(), basic);

    // §REQ-one-answer.proof.evidence: the input closure and question closure
    // must reject deleting a physical declaration/section/citation/alias and
    // all questions together. These retained controls use authored input bytes.
    super::agreement_query_controls::declaration_section_alias_subsets_fail_even_with_all_their_calls_deleted();
    super::agreement_query_controls::physical_bytes_and_membership_reject_deleting_or_rewriting_the_inventory_input();

    let scope = families::scope(&discovered);
    assert!(
        basic.is_subset(&scope),
        "basic facts exist but cases are absent from completed-call acceptance: {:?}",
        basic.difference(&scope).collect::<Vec<_>>()
    );
    assert!(super::COVERED.contains(&"basic"));
    assert!(!super::WIDENING.iter().any(|(family, _)| *family == "basic"));
    for retained in ["cover", "scan", "stub"] {
        assert!(super::COVERED.contains(&retained), "lost {retained} proof");
    }
    assert_new_member_requires_facts(&discovered);

    // This asserts genuine CLI/editor/package calls, exact applicability and
    // authored question receipts, not a count or a family label. Once shares
    // the execution with the ordinary acceptance without duplicating the matrix.
    super::agreement_acceptance::s4_executes_the_covered_families_and_the_seed();
}

fn assert_new_member_requires_facts(discovered: &BTreeSet<String>) {
    let new_case = "basic-new-member-without-authored-facts";
    assert!(!discovered.contains(new_case), "control name became real");
    let mut extended = discovered.clone();
    extended.insert(new_case.into());
    assert!(basic_members(&extended).contains(new_case));
    assert!(families::scope(&extended).contains(new_case));

    // Isolated corpus with the real directory layout used by the fact loader;
    // no mutation of the repository's fixture tree or native setup is needed.
    let control = super::agreement_tree::Tree::new(
        "grund-674-basic-new-member",
        &[
            (
                "tests/e2e/cases/basic-new-member-without-authored-facts/repo/grund.toml",
                super::agreement_tree::CONFIG_SLUG,
            ),
            ("tests/integration/command_agreement_facts/.keep", ""),
        ],
    );
    let errors = expectations::load(
        &control.0.join("tests/e2e/cases"),
        &BTreeSet::from([new_case.into()]),
    )
    .expect_err("new basic member without truth must fail before native setup");
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with(&format!("{new_case}: no input-authored manifest"))),
        "missing new member was not diagnosed: {errors:?}"
    );
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
