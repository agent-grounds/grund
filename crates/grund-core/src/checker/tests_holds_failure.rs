//! Test module: the one answer to whether a scan failure is inside a narrowed run's path, and
//! which name it is printed under (§FS-check.1.3.6.3, §FS-check.2.4.2, §AR-scanner.1.entry).
//! Unix-only: every tree here is reached through a link.

use std::fs;
use std::path::{Path, PathBuf};

use super::reference_scope::{ScanScope, partition_scan_failures, path_report_scope};
use crate::config::Config;
use crate::model::FailureNames;
use crate::scanner::ScanError;
use crate::testing::{physical_test_root, symlink};

const DENIED: &str = "permission denied";

/// `e2e -> realE`, with `realE/cases` and `docs` real.
fn tree(name: &str) -> PathBuf {
    let root = physical_test_root(name);
    fs::create_dir_all(root.join("realE/cases")).expect("create the directory");
    fs::create_dir_all(root.join("docs")).expect("create the home");
    symlink("realE", &root.join("e2e"));
    root
}

fn scope(root: &Path, rel: &str) -> ScanScope {
    let config = Config::default_for(root.to_path_buf());
    path_report_scope(config.schema(), config.frame(), &root.join(rel), true)
        .expect("resolve the scope")
        .expect("a path below the root narrows")
}

/// The failure the whole run names `survivor`, and the names it also answers to.
fn failure(root: &Path, survivor: &str, others: &[&str]) -> (ScanError, FailureNames) {
    let mut names = FailureNames::default();
    for other in others {
        names.keep(&root.join(survivor), DENIED, &root.join(other));
    }
    (ScanError::unread_entry(root.join(survivor), DENIED), names)
}

#[test]
fn a_failure_is_inside_under_each_of_its_names() {
    let root = tree("a_failure_is_inside_under_each_of_its_names");
    let (failure, names) = failure(&root, "e2e/cases", &["realE/cases"]);

    for (path, name) in [
        ("realE", "realE/cases"),
        ("realE/cases", "realE/cases"),
        ("e2e", "e2e/cases"),
        ("e2e/cases", "e2e/cases"),
    ] {
        assert_eq!(
            scope(&root, path).holds_failure(&failure, &names),
            Some(root.join(name)),
            "check {path}"
        );
    }
}

#[test]
fn a_failure_no_name_of_which_is_under_the_path_is_outside() {
    let root = tree("a_failure_no_name_of_which_is_under_the_path_is_outside");
    let (failure, names) = failure(&root, "e2e/cases", &["realE/cases"]);

    assert_eq!(scope(&root, "docs").holds_failure(&failure, &names), None);
}

/// `e2e` also holds `realE` through its canonical form, and the survivor is spelled there, so the
/// preference for the path as given is all that names it `e2e/cases`.
#[test]
fn the_name_under_the_path_as_given_is_preferred_to_the_first_the_scope_holds() {
    let root = tree("the_name_under_the_path_as_given_is_preferred");
    let (failure, names) = failure(&root, "realE/cases", &["e2e/cases"]);

    assert_eq!(
        scope(&root, "e2e").holds_failure(&failure, &names),
        Some(root.join("e2e/cases"))
    );
}

/// With no name under the path as given, the first the scope holds through its canonical form.
#[test]
fn with_no_name_as_given_the_first_the_scope_holds_is_used() {
    let root = tree("with_no_name_as_given_the_first_the_scope_holds");
    let (failure, names) = failure(&root, "realE/cases", &[]);

    assert_eq!(
        scope(&root, "e2e").holds_failure(&failure, &names),
        Some(root.join("realE/cases"))
    );
}

/// One path, two failures: the name one lost is no name of the other, whose side stays its own.
#[test]
fn a_kept_name_says_nothing_about_another_failure_at_that_path() {
    let root = tree("a_kept_name_says_nothing_about_another_failure");
    let (listing, names) = failure(&root, "e2e/cases", &["realE/cases"]);
    let parse = ScanError::unread_entry(root.join("e2e/cases"), "error parsing glob");

    let (inside, outside) = partition_scan_failures(
        [listing, parse.clone()],
        &names,
        Some(&scope(&root, "realE")),
    );

    assert_eq!(
        inside,
        [ScanError::unread_entry(root.join("realE/cases"), DENIED)]
    );
    assert_eq!(outside, [parse]);
}

/// A run over the config root narrows nothing, so every failure keeps the whole run's name.
#[test]
fn no_scope_leaves_every_failure_inside_under_its_own_name() {
    let root = tree("no_scope_leaves_every_failure_inside");
    let (failure, names) = failure(&root, "e2e/cases", &["realE/cases"]);

    let (inside, outside) = partition_scan_failures([failure.clone()], &names, None);

    assert_eq!(inside, [failure]);
    assert!(outside.is_empty());
}
