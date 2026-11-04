//! Issue #1182's cases, run alone: a fetch whose replaced bytes hold a line the next scan records
//! as a declaration exits 2, names it and how to keep it, and leaves the tree byte-identical, and
//! output holding such a line is refused the same way (§FS-fetch.6.nested, §FS-fetch.3.1). The
//! controls are also in the full pass; the refused cases are held out of it until the fix lands,
//! and implement removes the hold.

use std::path::PathBuf;

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// The refused cases, red until #1182: the full pass skips them; implement removes this.
const RED_UNTIL_1182: [&str; 12] = [
    "fetch-home-nested-declaration-file-refused",
    "fetch-home-nested-declaration-folder-refused",
    "fetch-home-nested-declaration-indented-refused",
    "fetch-home-nested-declaration-comment-line-refused",
    "fetch-home-nested-declaration-before-sibling-refused",
    "fetch-home-nested-declaration-cited-refused",
    "fetch-home-nested-declaration-v2-refused",
    "fetch-home-nested-declaration-workspace-member-refused",
    "fetch-home-nested-near-miss-refused",
    "fetch-home-nested-requested-id-refused",
    "fetch-home-nested-first-of-two-refused",
    "fetch-nested-declaration-comment-line",
];

/// What a fix has to keep: these pass today, so a blanket refusal of deeper headings fails them.
const CONTROLS: [&str; 9] = [
    "fetch-home-subsections-replaced",
    "fetch-home-subsections-folder-replaced",
    "fetch-home-non-declarations-replaced",
    "fetch-home-unmarked-headings-replaced",
    "fetch-home-sibling-holds-nested-replaced",
    "fetch-home-sibling-holds-nested-folder-replaced",
    "fetch-home-insert-beside-nested",
    "fetch-home-insert-beside-nested-folder",
    "fetch-home-nested-output-refusal-first",
];

/// The corpus without the refused cases, for the full pass; implement removes this.
pub fn without_red(cases: Vec<PathBuf>) -> Vec<PathBuf> {
    cases
        .into_iter()
        .filter(|case| !RED_UNTIL_1182.iter().any(|red| case.ends_with(red)))
        .collect()
}

#[test]
#[ignore = "red until #1182 refuses a fetch whose replaced bytes hold a declaration the next scan records; implement removes this"]
fn fetch_home_nested_declarations_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = RED_UNTIL_1182
        .iter()
        .chain(&CONTROLS)
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch home nested declarations", &outcomes);
}
