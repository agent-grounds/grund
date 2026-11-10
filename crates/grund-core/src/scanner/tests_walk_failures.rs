//! Test module: the walk's one step over its failures (§FS-check.2.4.2, §AR-scanner.1.entry):
//! what compares as one directory entry, the names a survivor keeps, and the guard that only a
//! run with two or more failures resolves anything for it (§GOAL-fast-feedback). Unix-only:
//! every tree here is about a link or a mode.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::scan_error::{FailureRoots, FileKind, ScanFailure, publish_file_failure};
use super::tree::ScanError;
use super::walk_failures::{one_report_per_entry, whole_run_failures};
use super::walk_reporting::walk_scannable_files_reporting;
use crate::config::Config;
use crate::model::FailureNames;
use crate::sources::realpaths_during;
use crate::testing::{physical_test_root, symlink, write};

/// The cases-root pass can report an ignore file the walk already reported under another
/// directory spelling. It keeps the whole-run name and adds its alias (§FS-check.2.4.2).
#[test]
fn issue_1337_an_ignore_failure_keeps_the_walks_name_and_the_passes_alias() {
    let root = linked_tree("ignore_failure_pass_alias");
    write(&root.join("realE/cases/.ignore"), "{bad\n");
    let config = Config::default_for(root.clone());
    let whole = ScanError::new(
        root.join("realE/cases/.ignore"),
        "error parsing glob '{bad'".into(),
        ScanFailure::Unparsed(FileKind::Ignore),
    );
    let mut alias = whole.clone();
    alias.path = root.join("e2e/cases/.ignore");
    let mut failures = vec![whole.clone()];
    let mut names = FailureNames::default();

    publish_file_failure(alias.clone(), config.frame(), &mut names, &mut failures);
    publish_file_failure(alias.clone(), config.frame(), &mut names, &mut failures);

    assert_eq!(failures, [whole.clone()]);
    assert_eq!(names_of(&names, &whole), [whole.path, alias.path]);
}

/// Choosing an ordinary-root name does not turn a loop met below the passed path into a
/// failure at that path (§FS-check.2.4.3). CLI keeps the encountered entry's remedy.
#[test]
fn issue_1337_selecting_a_whole_run_name_keeps_the_encounters_failure_place() {
    let root = physical_test_root("whole_run_failure_place");
    fs::create_dir_all(root.join("a/sub")).expect("create the directory");
    symlink("..", &root.join("a/sub/up"));
    symlink("a", &root.join("b"));
    let passed = root.join("b/sub/up");
    let at = FailureRoots::of(
        vec![passed.clone(), root.clone()],
        Some(passed),
        root.clone(),
    );
    let first = report(&root, "b/sub/up/sub/up", "loop");
    let ordinary = report(&root, "a/sub/up", "loop");
    let mut expected = first.clone().placed(&at);
    expected.path = ordinary.path.clone();
    let mut failures = vec![first.clone(), ordinary];

    let names = whole_run_failures(&mut failures, 1, &at);

    assert_eq!(failures, [expected.clone()]);
    assert_eq!(names_of(&names, &expected), [expected.path, first.path]);
}

/// `<root>/realE/cases` and `<root>/real`, with `e2e -> realE` and `ext -> real`.
fn linked_tree(name: &str) -> PathBuf {
    let root = physical_test_root(name);
    fs::create_dir_all(root.join("realE/cases")).expect("create the directory");
    fs::create_dir_all(root.join("real")).expect("create the target");
    symlink("realE", &root.join("e2e"));
    symlink("real", &root.join("ext"));
    root
}

fn report(root: &Path, rel: &str, reason: &str) -> ScanError {
    ScanError::unread_entry(root.join(rel), reason)
}

fn names_of(names: &FailureNames, failure: &ScanError) -> Vec<PathBuf> {
    names
        .of(&failure.path, &failure.reason)
        .map(Path::to_path_buf)
        .collect()
}

#[test]
fn one_entry_for_one_reason_is_one_report_and_the_survivor_keeps_the_other_name() {
    let root = linked_tree("one_entry_for_one_reason");
    let first = report(&root, "e2e/cases", "denied");
    let second = report(&root, "realE/cases", "denied");
    let mut failures = vec![first.clone(), second.clone()];

    let names = one_report_per_entry(&mut failures);

    assert_eq!(
        failures,
        [first.clone()],
        "the first report is the survivor"
    );
    assert_eq!(names_of(&names, &first), [first.path, second.path]);
}

#[test]
fn another_entry_stays_apart_even_where_it_is_the_links_target() {
    let root = linked_tree("another_entry_stays_apart");
    let mut failures = vec![
        report(&root, "ext", "denied"),
        report(&root, "real", "denied"),
        report(&root, "realE/cases", "denied"),
    ];
    let before = failures.clone();

    let names = one_report_per_entry(&mut failures);

    assert_eq!(
        failures, before,
        "§FS-check.2.4.2: a link and its target are two entries"
    );
    assert_eq!(names, FailureNames::default());
}

#[test]
fn another_reason_stays_apart_and_a_name_answers_only_for_the_reason_that_lost_it() {
    let root = linked_tree("another_reason_stays_apart");
    let listing = report(&root, "e2e/cases", "denied");
    let parse = report(&root, "e2e/cases", "error parsing glob");
    let mut failures = vec![
        listing.clone(),
        parse.clone(),
        report(&root, "realE/cases", "denied"),
    ];

    let names = one_report_per_entry(&mut failures);

    assert_eq!(failures, [listing.clone(), parse.clone()]);
    assert_eq!(names_of(&names, &listing).len(), 2);
    assert_eq!(names_of(&names, &parse), [parse.path.clone()]);
}

/// Overlapping roots meet one failure under one name: one report, and no second name.
#[test]
fn one_name_met_twice_is_one_report_and_no_second_name() {
    let root = linked_tree("one_name_met_twice");
    let failure = report(&root, "realE/cases", "denied");
    let mut failures = vec![failure.clone(), failure.clone()];

    let names = one_report_per_entry(&mut failures);

    assert_eq!(failures, [failure]);
    assert_eq!(names, FailureNames::default());
}

/// No final name, or a parent that is gone: nothing to resolve, so the path compares as written.
/// Two that differ stay two, at the cost of a line, and neither is lost.
#[test]
fn a_path_with_no_key_compares_as_written_and_is_kept() {
    let root = physical_test_root("a_path_with_no_key");
    let (bare, gone, other) = (
        ScanError::unread_entry(PathBuf::from("/"), "denied"),
        report(&root, "gone/x", "denied"),
        report(&root, "other/x", "denied"),
    );
    let mut failures = vec![
        bare.clone(),
        bare.clone(),
        gone.clone(),
        gone.clone(),
        other.clone(),
    ];

    let names = one_report_per_entry(&mut failures);

    assert_eq!(failures, [bare, gone, other]);
    assert_eq!(names, FailureNames::default());
}

/// §AR-scanner.1.entry: nothing resolved for fewer than two reports, and each report's parent
/// for two, so this cannot pass by counting nothing.
#[test]
fn the_step_resolves_nothing_for_fewer_than_two_failures() {
    let root = linked_tree("the_step_resolves_nothing");
    let count =
        |mut failures: Vec<ScanError>| realpaths_during(|| one_report_per_entry(&mut failures)).1;

    assert_eq!(count(Vec::new()), 0);
    assert_eq!(count(vec![report(&root, "e2e/cases", "denied")]), 0);
    assert_eq!(
        count(vec![
            report(&root, "e2e/cases", "denied"),
            report(&root, "realE/cases", "denied"),
        ]),
        2
    );
}

/// The same through the walk: one tree, walked clean, with one barred directory, and with two.
/// The second failure is the first that costs a `realpath` the clean walk does not spend.
#[test]
fn a_clean_walk_and_a_one_failure_walk_spend_no_new_realpath() {
    let root = physical_test_root("a_clean_walk_and_a_one_failure_walk");
    for dir in ["docs/a", "docs/b"] {
        fs::create_dir_all(root.join(dir)).expect("create the directory");
    }
    fs::write(root.join("docs/notes.md"), "# Notes\n").expect("write the notes");
    let config = Config::default_for(root.clone());
    let walk = || {
        let (walked, spent) = realpaths_during(|| {
            walk_scannable_files_reporting(config.schema(), config.frame(), Some(&root), true)
                .expect("walk the tree")
        });
        (walked.errors.len(), spent)
    };
    let mode = |dir: &str, mode| {
        fs::set_permissions(root.join(dir), fs::Permissions::from_mode(mode)).expect("set the mode")
    };

    let (clean_failures, clean) = walk();
    mode("docs/a", 0o000);
    if fs::read_dir(root.join("docs/a")).is_ok() {
        mode("docs/a", 0o755);
        return;
    }
    let (one_failure, one) = walk();
    mode("docs/b", 0o000);
    let (two_failures, two) = walk();
    mode("docs/a", 0o755);
    mode("docs/b", 0o755);

    assert_eq!((clean_failures, one_failure, two_failures), (0, 1, 2));
    assert_eq!(
        one, clean,
        "a one-failure walk resolves what a clean one does"
    );
    assert!(
        two > one,
        "two failures are compared, so {two} must exceed {one}"
    );
}
