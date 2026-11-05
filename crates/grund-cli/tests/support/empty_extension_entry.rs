//! Issue #1237's cases, run alone: `config migrate` says what an empty string in a version 1
//! `[scan] extensions` reads and what removing it costs (§FS-config-migrate.3.1.9), its other
//! refusals name what the walk reads under that entry, and `config explain` says the same
//! (§FS-scan.scope.extensions.empty). They are held out of the full pass until the fix lands.
//! Implement removes the hold, and with it this module and its `mod` line in `e2e.rs`, because the
//! full pass then runs them. The other `*-empty-extension-*` cases are guards that pass today, so
//! they stay in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;
use std::path::PathBuf;

/// Red until #1237: the full pass skips these; implement removes this.
const RED_UNTIL_1237: [&str; 4] = [
    "config-explain-v1-extensions-empty-entry",
    "config-migrate-refuse-empty-extension",
    "config-migrate-refuse-nested-homes-empty-extension",
    "config-migrate-refuse-path-shaped-exclude-empty-extension",
];

/// The full pass's cases, without [`RED_UNTIL_1237`].
pub fn without_red(cases: Vec<PathBuf>) -> Vec<PathBuf> {
    cases
        .into_iter()
        .filter(|case| !RED_UNTIL_1237.iter().any(|red| case.ends_with(red)))
        .collect()
}

#[test]
#[ignore = "red until #1237 lengthens the entry's refusal and asks one test; implement removes this"]
fn empty_extension_entry_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = RED_UNTIL_1237
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("empty extension entry", &outcomes);
}
