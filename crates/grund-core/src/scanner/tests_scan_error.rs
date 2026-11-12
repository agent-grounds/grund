//! Test module: what a scan failure carries (§FS-check.2.4.3, §AR-scanner.1.failure): where a
//! failed path is, by location, and the code and *Fix* each failure answers. Unix-only: the second
//! spelling is a link.

use std::fs;
use std::path::PathBuf;

use super::scan_error::{
    FailureRoots, FileKind, Place, ScanError, ScanFailure, failure_place, scan_error_message,
};
use crate::model::ValueFormat;
use crate::testing::{physical_test_root, symlink};

const ENTRY: &str = "make the path readable, or keep it out of the scan";

fn roots(root: &std::path::Path, passed: Option<PathBuf>) -> FailureRoots {
    FailureRoots::of(
        vec![
            root.join("docs/fs"),
            root.join("spec.md"),
            root.to_path_buf(),
        ],
        passed,
        root.to_path_buf(),
    )
}

#[test]
fn a_failed_path_is_judged_by_where_it_is() {
    let root = physical_test_root("a_failed_path_is_judged_by_where_it_is");
    let at = roots(&root, None);
    assert_eq!(failure_place(&root.join("docs/fs"), &at), Place::Root);
    assert_eq!(failure_place(&root.join("spec.md"), &at), Place::Root);
    assert_eq!(failure_place(&root.join("docs/fs/x.md"), &at), Place::Entry);
    // §FS-check.2.4.3: the config root has no row, though it is one of the roots.
    assert_eq!(failure_place(&root, &at), Place::Readable);
}

#[test]
fn a_passed_path_wins_and_a_path_below_it_is_an_entry() {
    let root = physical_test_root("a_passed_path_wins_and_a_path_below_it_is_an_entry");
    let at = roots(&root, Some(root.join("docs/fs")));
    assert_eq!(failure_place(&root.join("docs/fs"), &at), Place::Passed);
    assert_eq!(failure_place(&root.join("docs/fs/x.md"), &at), Place::Entry);
}

#[test]
fn a_second_spelling_of_a_root_is_that_root() {
    let root = physical_test_root("a_second_spelling_of_a_root_is_that_root");
    fs::create_dir_all(root.join("docs/fs")).expect("create the root");
    symlink("docs", &root.join("alias"));
    assert_eq!(
        failure_place(&root.join("alias/fs"), &roots(&root, None)),
        Place::Root
    );
}

/// §FS-check.2.4.3: a link is judged as the path it is, so one below a root that points at a root,
/// the passed path or the config root keeps the `entry` fix, and so does a link naming a root.
#[test]
fn a_link_below_a_root_is_an_entry_whatever_it_points_at() {
    let root = physical_test_root("a_link_below_a_root_is_an_entry");
    fs::create_dir_all(root.join("docs/fs")).expect("create the root");
    symlink(".", &root.join("docs/fs/self"));
    symlink("../..", &root.join("docs/fs/up"));
    symlink("docs/fs", &root.join("alias"));
    let at = roots(&root, Some(root.join("docs/fs")));
    for link in ["docs/fs/self", "docs/fs/up", "alias"] {
        assert_eq!(failure_place(&root.join(link), &at), Place::Entry, "{link}");
    }
}

/// A loop can walk back to the entry the run was handed: the path is still spelled below it, so
/// it is below it, and `exclude` reaches it.
#[test]
fn a_loop_back_to_the_passed_link_is_an_entry() {
    let root = physical_test_root("a_loop_back_to_the_passed_link");
    fs::create_dir_all(root.join("a/sub")).expect("create the tree");
    symlink("..", &root.join("a/sub/up"));
    symlink("a", &root.join("b"));
    let at = FailureRoots::of(
        vec![root.join("b/sub/up")],
        Some(root.join("b/sub/up")),
        root.clone(),
    );
    assert_eq!(failure_place(&root.join("b/sub/up"), &at), Place::Passed);
    assert_eq!(
        failure_place(&root.join("b/sub/up/sub/up"), &at),
        Place::Entry
    );
}

/// A root spelled without a final name has none to dismiss by, so it is resolved on its own.
#[test]
fn a_root_with_no_final_name_is_resolved_on_its_own() {
    let root = physical_test_root("a_root_with_no_final_name");
    fs::create_dir_all(root.join("docs/fs")).expect("create the root");
    let at = FailureRoots::of(vec![root.join("docs/fs/..")], None, root.join("elsewhere"));
    assert_eq!(failure_place(&root.join("docs"), &at), Place::Root);
    assert_eq!(failure_place(&root.join("docs/fs"), &at), Place::Entry);
}

fn unread(place: Place, version: u32) -> (String, String) {
    let failure = ScanFailure::Unread { place, version };
    answer(failure)
}

fn answer(failure: ScanFailure) -> (String, String) {
    let error = ScanError::new(PathBuf::from("p"), "why".to_string(), failure);
    let code = error.code().as_str().to_string();
    let message = scan_error_message(&error, &super::scan_error::Absolute);
    let fix = message.strip_prefix("why; ").expect("the reason leads");
    (code, fix.to_string())
}

#[test]
fn each_place_answers_io_and_its_own_fix() {
    let code = |fix: &str| ("io".to_string(), fix.to_string());
    assert_eq!(unread(Place::Entry, 2), code(ENTRY));
    assert_eq!(unread(Place::Readable, 2), code("make the path readable"));
    assert!(unread(Place::Root, 2).1.contains("[schema.kinds] row"));
    assert!(
        unread(Place::Root, 1)
            .1
            .contains("[scan] include or [[kinds]] row")
    );
    assert!(unread(Place::Passed, 2).1.ends_with(
        "by not passing it, because exclude does not prune a path the command is given"
    ));
    assert!(
        unread(Place::Value, 2)
            .1
            .ends_with("because exclude does not reach a value source")
    );
    for place in [Place::Root, Place::Passed, Place::Value] {
        assert!(
            unread(place, 2).1.starts_with(ENTRY),
            "{place:?} keeps the prefix"
        );
    }
}

#[test]
fn each_kind_of_file_answers_unparsable_and_what_it_should_hold() {
    let parsed = |kind| answer(ScanFailure::Unparsed(kind));
    let code = "unparsable-file".to_string();
    assert_eq!(
        parsed(FileKind::Exit),
        (
            code.clone(),
            "write the case's expected exit status in the file as one integer".to_string()
        )
    );
    assert_eq!(
        parsed(FileKind::Value(ValueFormat::Json)),
        (code.clone(), "make the file valid JSON".to_string())
    );
    assert_eq!(
        parsed(FileKind::Value(ValueFormat::Yaml)),
        (code.clone(), "make the file valid YAML".to_string())
    );
    assert_eq!(
        parsed(FileKind::Ignore),
        (
            code,
            "write the line it names as a valid ignore pattern".to_string()
        )
    );
}
