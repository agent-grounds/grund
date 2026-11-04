//! Issue #1180's corpus cases, run alone: a home that already holds the fetched output comes out
//! byte-identical (§FS-fetch.6.1). The red ones are held out of the full pass until the fix
//! lands; implement removes the hold and the marker, and moves them into the unmarked list.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1180: the full pass skips these and the ignored contract runs them.
pub const RED_UNTIL_1180: [&str; 3] = [
    "fetch-unchanged-output-leading-blank-line",
    "fetch-unchanged-output-trailing-blank-line-before-sibling",
    "fetch-unchanged-output-folder-home-nbsp-indent",
];

/// The corpus without the cases the full pass holds out until #1180 lands.
pub fn without_red(mut cases: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
    cases.retain(|case| !RED_UNTIL_1180.iter().any(|red| case.ends_with(red)));
    cases
}

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
    run("fetch unchanged output", &["fetch-unchanged-output-plain"]);
}

#[test]
#[ignore = "red until #1180 leaves a home that already holds the output as it is; implement removes this"]
fn fetch_unchanged_output_red_cases() {
    run("fetch unchanged output (red)", &RED_UNTIL_1180);
}
