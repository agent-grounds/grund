//! Issue #1290's cases, run alone: a path-shaped `[scan] exclude` entry is read as the version 2
//! glob it resembles, so its refusal names and counts what that glob would stop among the files
//! version 1 reads, and on a tree the scan cannot walk an entry that asks the scan fails the run
//! (§FS-config-migrate.3.1.4.1, §FS-config-migrate.7). Each golden holds every `refused:` line and
//! the exit, and each count was measured with `grund list` on the case's tree. Implement removes
//! the hold, and with it this module and its `mod` line in `e2e.rs`, because the full pass then
//! runs all nineteen cases.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;
use std::path::PathBuf;

/// Red until #1290: the full pass skips these; implement removes this hold, and the full pass
/// then runs them as it runs the pins.
const RED_UNTIL_1290: [&str; 16] = [
    "config-migrate-refuse-exclude-glob-anchored",
    "config-migrate-refuse-exclude-glob-below-a-place",
    "config-migrate-refuse-exclude-glob-directory-match",
    "config-migrate-refuse-exclude-glob-disk-name",
    "config-migrate-refuse-exclude-glob-double-star",
    "config-migrate-refuse-exclude-glob-names-a-place",
    "config-migrate-refuse-exclude-glob-opted-out-block",
    "config-migrate-refuse-exclude-glob-read-alone",
    "config-migrate-refuse-exclude-glob-report",
    "config-migrate-refuse-exclude-glob-spelling-not-normalized",
    "config-migrate-refuse-exclude-glob-star-stops-at-slash",
    "config-migrate-refuse-exclude-glob-trailing-space",
    "config-migrate-refuse-exclude-glob-unanchored-directory",
    "config-migrate-refuse-exclude-glob-unanchored-unwalkable-tree",
    "config-migrate-refuse-exclude-glob-unread-files",
    "config-migrate-refuse-exclude-glob-unwalkable-tree",
];

/// What a fix could get wrong and today is right: a glob that names a scan root stops nothing
/// there, and an entry that stops nothing neither fails the run nor asks the scan.
const PINS: [&str; 3] = [
    "config-migrate-refuse-exclude-glob-glob-names-a-place",
    "config-migrate-refuse-exclude-glob-stops-nothing",
    "config-migrate-refuse-exclude-glob-stops-nothing-unwalkable-tree",
];

/// The full pass's cases, without [`RED_UNTIL_1290`].
pub fn without_red(cases: Vec<PathBuf>) -> Vec<PathBuf> {
    cases
        .into_iter()
        .filter(|case| !RED_UNTIL_1290.iter().any(|red| case.ends_with(red)))
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
#[ignore = "red until #1290 reads a path-shaped entry as the version 2 glob; implement removes this"]
fn exclude_glob_reading_contract() {
    run_alone("exclude glob reading", &RED_UNTIL_1290);
}

/// Pass today: every one is also in the full pass, and a fix that overreaches fails them.
#[test]
fn exclude_glob_reading_pins() {
    run_alone("exclude glob reading pins", &PINS);
}
