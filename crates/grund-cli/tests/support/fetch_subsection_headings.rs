//! Issue #1159's cases, run alone: `grund fetch` accepts a subsection heading only when the
//! next scan reads it as a section of the declaration, and refuses any other heading at
//! subsection depth with exit 2 and the tree byte-identical (§FS-fetch.3.2,
//! §FS-declarations.checks.unmarked-heading). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 12] = [
    "fetch-subsection-prose-heading-refused",
    "fetch-subsection-unconfigured-kind-heading-refused",
    "fetch-subsection-escaped-id-heading-refused",
    "fetch-subsection-unmarked-heading-refused-v2",
    "fetch-subsection-folder-home-unmarked-heading-refused",
    "fetch-subsection-named-heading-without-named-sections-refused",
    "fetch-subsection-named-heading-accepted",
    "fetch-subsection-numbered-heading-without-named-sections-accepted",
    "fetch-subsection-relative-heading-accepted",
    "fetch-subsection-fenced-heading-accepted",
    "fetch-subsection-bold-label-accepted",
    "fetch-subsection-folder-home-numbered-heading-accepted",
];

#[test]
fn fetch_subsection_headings_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch subsection headings", &outcomes);
}
