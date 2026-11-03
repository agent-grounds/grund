//! Issue #1133's cases, run alone: a YAML value source is reported as a JSON one is, by every
//! reader of the catalog, and the code that held it apart is spent (§FS-values.2.2.4,
//! §FS-values.5.3). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 28] = [
    "check-yaml-value-source-unparsable",
    "check-yaml-value-source-unparsable-json",
    "list-yaml-value-source-unparsable",
    "show-yaml-value-source-unparsable",
    "show-yaml-value-source-unparsable-own-id",
    "refs-yaml-value-source-unparsable",
    "check-yaml-value-source-unparsable-cover",
    "id-yaml-value-source-unparsable",
    "check-yaml-value-source-nested-too-deep",
    "check-yaml-value-source-malformed-utf8",
    "list-yaml-value-source-malformed-utf8",
    "check-yaml-value-source-anchor-refused",
    "list-yaml-value-source-invalid-member-keeps-sibling",
    "check-yaml-value-source-wrong-shape",
    "check-yaml-value-source-root-not-mapping",
    "check-yaml-value-source-invalid-component",
    "check-yaml-value-source-refused-escape",
    "check-yaml-value-source-repeated-key",
    "check-yaml-value-source-json-duplicate",
    "list-yaml-value-source-json-duplicate",
    "check-yaml-value-source-issue-example",
    "check-yaml-value-source-missing-index-entry-json",
    "check-yaml-value-source-clean-uncited-unused",
    "check-yaml-value-source-fetched-kind-read",
    "list-yaml-value-source-fetched-kind-read",
    "config-explain-yaml-value-source-deferred",
    "config-explain-codes",
    "check-only-yaml-value-source-deferred-spent",
];

#[test]
fn yaml_value_window_closed_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("the closed YAML value-source window", &outcomes);
}
