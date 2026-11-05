//! Issue #1260's cases, run alone: `config migrate`'s `would stop reading` and `would change owner`
//! details, and the count beside the first, name exactly the files the version 1 scan reads, and
//! every refusal stands whatever they name (§FS-config-migrate.3.1.4.1,
//! §FS-config-migrate.3.1.5.1). Each case's golden holds every `refused:` line and the exit, so a
//! fix that drops a refusal fails here as surely as one that names a file version 1 does not read.
//! Implement removes the hold, and with it this module and its `mod` line in `e2e.rs`, because the
//! full pass then runs all eleven cases.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;
use std::path::PathBuf;

/// Red until #1260: the full pass skips these; implement removes this hold, and the full pass
/// then runs them as it runs the pins.
const RED_UNTIL_1260: [&str; 6] = [
    "config-migrate-refuse-hidden-and-ignored-named-files",
    "config-migrate-refuse-nested-homes-unread-files",
    "config-migrate-refuse-path-shaped-exclude-empty-extension-hidden",
    "config-migrate-refuse-path-shaped-exclude-outside-scope",
    "config-migrate-refuse-path-shaped-exclude-unread-files",
    "config-migrate-refuse-unlisted-extension-files",
];

/// What looks like the same mistake and is true, so a fix must keep naming it: files version 1
/// reads, a gitignored file home, `respect_gitignore = false`, and a hidden or ignored folder home.
const PINS: [&str; 5] = [
    "config-migrate-refuse-nested-homes-hidden-home",
    "config-migrate-refuse-nested-homes-ignored-file-home",
    "config-migrate-refuse-nested-homes-ignored-folder-home",
    "config-migrate-refuse-path-shaped-exclude-gitignore-off",
    "config-migrate-refuse-read-named-files",
];

/// The full pass's cases, without [`RED_UNTIL_1260`].
pub fn without_red(cases: Vec<PathBuf>) -> Vec<PathBuf> {
    cases
        .into_iter()
        .filter(|case| !RED_UNTIL_1260.iter().any(|red| case.ends_with(red)))
        .collect()
}

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
#[ignore = "red until #1260 names only the files version 1 reads; implement removes this"]
fn refusal_detail_reads_contract() {
    run_alone("refusal detail reads", &RED_UNTIL_1260);
}

/// Pass today: every one is also in the full pass, and a fix that names too little fails them.
#[test]
fn refusal_detail_reads_pins() {
    run_alone("refusal detail reads pins", &PINS);
}
