//! Issue #1180's corpus cases, run alone: a home that already holds the fetched output comes out
//! byte-identical (§FS-fetch.6.1).

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

fn run(name: &str, cases: &[&str]) {
    let root = repo_root();
    let dir = root.join("tests/e2e/cases");
    let outcomes = cases
        .iter()
        .map(|case| run_case(&root, &dir.join(case), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed(name, &outcomes);
}

#[test]
fn fetch_unchanged_output_cases() {
    run(
        "fetch unchanged output",
        &[
            "fetch-unchanged-output-plain",
            "fetch-unchanged-output-leading-blank-line",
            "fetch-unchanged-output-trailing-blank-line-before-sibling",
            "fetch-unchanged-output-folder-home-nbsp-indent",
        ],
    );
}
