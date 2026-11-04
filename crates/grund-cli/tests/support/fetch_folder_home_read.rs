//! Issue #1185's cases, run alone: `grund fetch` finds the declaring file by the scan's walk
//! (§FS-fetch.5.1) and reads what stands at `<home>/<ID>.md` whatever that walk admits
//! (§FS-fetch.5.2). The cases are also in the full pass. What is refused is pinned in
//! `crates/grund-cli/tests/fetch_folder_home_read.rs`, not here: the refusal prints the
//! absolute path, which no golden of a `command.cwd` case can hold on another machine.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const GUARDS: [&str; 9] = [
    "fetch-folder-refetch-replaces",
    "fetch-folder-refetch-home-gitignored",
    "fetch-folder-refetch-home-excluded",
    "fetch-folder-other-name-unread-not-found",
    "fetch-folder-refetch-md-read-as-json",
    "fetch-folder-refetch-md-read-by-no-language",
    "fetch-folder-refetch-md-unlisted-v1",
    "fetch-folder-refetch-file-gitignored",
    "fetch-folder-refetch-file-excluded",
];

#[test]
fn fetch_folder_home_read_guards() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = GUARDS
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch folder home read guards", &outcomes);
}
