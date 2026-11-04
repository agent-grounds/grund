//! Focused tests for the nested-declaration refusal of fetch's output
//! validator (§FS-fetch.3.1): a heading the scanner would read as a declaration
//! is refused by its ID at any depth below the requested one, independent of
//! the one-depth heading limit, while numbered and fenced subsections stay accepted.

use super::fetch::validate_fetched_declaration;
use crate::testing::{numbered_config, test_root};

fn validate(name: &str, text: &str) -> Result<(), String> {
    let config = numbered_config(test_root(name));
    validate_fetched_declaration(text, "AR-001-thing", 2, &config.grammar)
        .map_err(|failure| failure.message)
}

#[test]
fn nested_declaration_two_depths_below_is_refused_by_its_id_not_its_depth() {
    let refusal = validate(
        "nested_declaration_two_depths_below",
        "## AR-001-thing: Thing\n\n#### FS-002-other: Other\n",
    )
    .expect_err("a nested declaration must be refused");
    assert_eq!(
        refusal,
        "fetch output nests declaration `FS-002-other` inside requested `AR-001-thing`; \
         fetch output must hold only the requested declaration"
    );
}

#[test]
fn nested_requested_id_is_refused() {
    let refusal = validate(
        "nested_requested_id",
        "## AR-001-thing: Thing\n\n### AR-001-thing: Again\n",
    )
    .expect_err("the requested ID nested again must be refused");
    assert!(
        refusal.contains("nests declaration `AR-001-thing`"),
        "{refusal}"
    );
}

#[test]
fn numbered_and_fenced_subsections_stay_accepted() {
    validate(
        "nested_subsection_accepted",
        "## AR-001-thing: Thing\n\n### 1. Notes\n\nprose\n",
    )
    .expect("a numbered subsection is not a declaration");
    validate(
        "nested_fenced_heading_accepted",
        "## AR-001-thing: Thing\n\n```\n### AR-002-other: Other\n```\n",
    )
    .expect("a fenced heading is content");
}

#[test]
fn nested_declaration_after_a_tab_is_refused() {
    let refusal = validate(
        "nested_declaration_after_tab",
        "## AR-001-thing: Thing\n\n###\tAR-002-other: Other\n",
    )
    .expect_err("a tab ends the hashes for the scanner, so this heading declares");
    assert!(
        refusal.contains("nests declaration `AR-002-other`"),
        "{refusal}"
    );
}

#[test]
fn sibling_declaration_after_a_tab_is_refused_as_a_second_declaration() {
    let refusal = validate(
        "sibling_declaration_after_tab",
        "## AR-001-thing: Thing\n\n##\tAR-002-other: Other\n",
    )
    .expect_err("a same-depth sibling after a tab is a second declaration");
    assert_eq!(
        refusal,
        "fetch output must contain exactly one declaration and no leading content"
    );
}

#[test]
fn nested_near_miss_declarations_are_refused() {
    for (name, heading, token) in [
        (
            "nested_near_miss_case",
            "### AR-002-Other: Other",
            "AR-002-Other",
        ),
        ("nested_near_miss_short", "### AR-002: Other", "AR-002"),
    ] {
        let refusal = validate(name, &format!("## AR-001-thing: Thing\n\n{heading}\n"))
            .expect_err("the scanner records a near miss as a declaration");
        assert!(
            refusal.contains(&format!("nests declaration `{token}`")),
            "{heading}: {refusal}"
        );
    }
}
