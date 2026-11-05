//! Issue #1214's cases, run alone: a `folder`-home fetch of an ID declared in a file outside that
//! home refuses, with exit 2, the file named from the report base and the tree byte-identical
//! (§FS-fetch.5.3). Refusal and guard cases also run in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const REFUSED: [&str; 26] = [
    "fetch-folder-elsewhere-beside-home-declaration-refused",
    "fetch-folder-elsewhere-beside-home-other-content-refused",
    "fetch-folder-elsewhere-beside-home-source-refused",
    "fetch-folder-elsewhere-broken-stub-another-home-refused",
    "fetch-folder-elsewhere-link-title-outside-homes-refused",
    "fetch-folder-elsewhere-markdown-h2-refused",
    "fetch-folder-elsewhere-markdown-home-absent-refused",
    "fetch-folder-elsewhere-markdown-refused",
    "fetch-folder-elsewhere-source-go-refused",
    "fetch-folder-elsewhere-source-python-function-refused",
    "fetch-folder-elsewhere-source-python-refused",
    "fetch-folder-elsewhere-source-rust-refused",
    "fetch-folder-elsewhere-stub-h2-refused",
    "fetch-folder-elsewhere-stub-other-name-refused",
    "fetch-folder-elsewhere-stub-relative-link-refused",
    "fetch-folder-elsewhere-stub-report-link-refused",
    "fetch-folder-elsewhere-stub-to-markdown-refused",
    "fetch-folder-elsewhere-stub-under-another-name-refused",
    "fetch-folder-elsewhere-two-files-refused",
    "fetch-folder-elsewhere-undecodable-beside-declaration-refused",
    "fetch-folder-elsewhere-v1-markdown-refused",
    "fetch-folder-elsewhere-v1-source-refused",
    "fetch-folder-elsewhere-v1-stub-refused",
    "fetch-folder-elsewhere-v1-stub-target-unscanned-refused",
    "fetch-folder-elsewhere-workspace-member-refused",
    "fetch-folder-elsewhere-workspace-member-stub-refused",
];

/// What must not stop the fetch, and what it must still replace where it stands.
const GUARDS: [&str; 16] = [
    "fetch-folder-elsewhere-broken-stub-lacks-replaced",
    "fetch-folder-elsewhere-broken-stub-missing-replaced",
    "fetch-folder-elsewhere-citation-markdown-created",
    "fetch-folder-elsewhere-citation-source-created",
    "fetch-folder-elsewhere-excluded-created",
    "fetch-folder-elsewhere-fenced-created",
    "fetch-folder-elsewhere-home-markdown-replaced",
    "fetch-folder-elsewhere-language-not-admitted-created",
    "fetch-folder-elsewhere-other-id-created",
    "fetch-folder-elsewhere-stub-outside-home-replaced",
    "fetch-folder-elsewhere-stub-target-undecodable-replaced",
    "fetch-folder-elsewhere-undecodable-outside-created",
    "fetch-folder-elsewhere-v1-outside-include-created",
    "fetch-folder-elsewhere-workspace-member-root-declares-created",
    "fetch-folder-elsewhere-workspace-other-member-created",
    "fetch-folder-elsewhere-workspace-root-member-declares-created",
];

fn run_alone(label: &str, names: &[&str]) {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = names
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed(label, &outcomes);
}

#[test]
fn fetch_folder_elsewhere_contract() {
    run_alone("fetch folder elsewhere", &REFUSED);
}

/// Pass today: every one is also in the full pass, and a fix that refuses too much fails them.
#[test]
fn fetch_folder_elsewhere_guards() {
    run_alone("fetch folder elsewhere guards", &GUARDS);
}
