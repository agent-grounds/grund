//! Test module: a case manifest whose name is a link whose target does not resolve is a manifest
//! that cannot be read, failed at the link, never an absent one; and a name that is not there at
//! all stays absent (§FS-declarations.e2e-case.failure, §FS-scan.scope.links.unresolvable).
//! Unix-only: every tree here is about a link.

use std::path::{Path, PathBuf};

use super::e2e_case::{CaseFailure, read_e2e_case};
use super::scan_error::read_failure_reason;
use super::walk_errors::BROKEN_LINK;
use crate::config::Config;
use crate::testing::{physical_test_root, symlink, write};

/// A case directory `e2e/cases/b` holding only `expected.exit`.
fn case_dir(name: &str) -> (PathBuf, PathBuf) {
    let root = physical_test_root(name);
    let dir = root.join("e2e/cases/b");
    write(&dir.join("expected.exit"), "0\n");
    (root, dir)
}

/// The paths `read_e2e_case` failed at, or none when it read the case.
fn failed_paths(root: &Path, dir: &Path) -> Vec<PathBuf> {
    let config = Config::default_for(root.to_path_buf());
    match read_e2e_case(config.schema(), config.frame(), dir) {
        Ok(_) => Vec::new(),
        Err(failed) => failed
            .iter()
            .map(|failure| {
                assert!(
                    matches!(failure, CaseFailure::Unread(..)),
                    "a dangling manifest is a file that could not be read"
                );
                failure.path().to_path_buf()
            })
            .collect(),
    }
}

#[test]
fn a_dangling_spec_refs_fails_the_case_at_the_link() {
    let (root, dir) = case_dir("e2e_case_dangling_spec_refs");
    symlink("spec.refs.gone", &dir.join("spec.refs"));

    assert_eq!(failed_paths(&root, &dir), [dir.join("spec.refs")]);
}

#[test]
fn a_dangling_command_args_fails_the_case_at_the_link() {
    let (root, dir) = case_dir("e2e_case_dangling_command_args");
    symlink("command.args.gone", &dir.join("command.args"));

    assert_eq!(failed_paths(&root, &dir), [dir.join("command.args")]);
}

#[test]
fn an_absent_manifest_is_still_read_as_none() {
    let (root, dir) = case_dir("e2e_case_absent_manifests");

    assert_eq!(failed_paths(&root, &dir), Vec::<PathBuf>::new());
}

/// The reason the case pass gives the failure at `link`, read through `read_e2e_case`.
fn reason_at(root: &Path, dir: &Path, link: &Path) -> String {
    let config = Config::default_for(root.to_path_buf());
    let Err(failed) = read_e2e_case(config.schema(), config.frame(), dir) else {
        panic!("the case read");
    };
    failed
        .into_iter()
        .find_map(|failure| match failure {
            CaseFailure::Unread(path, err) if path == link => {
                Some(read_failure_reason(&path, &err))
            }
            _ => None,
        })
        .expect("a failure at the link")
}

#[test]
fn a_looping_manifest_carries_the_walks_unreadable_link_reason() {
    let (root, dir) = case_dir("e2e_case_looping_spec_refs");
    let link = dir.join("spec.refs");
    symlink("spec.refs", &link);

    assert_eq!(
        reason_at(&root, &dir, &link),
        "unreadable symlink: filesystem loop or indirection limit (e.g. symlink loop)"
    );
}

#[test]
fn a_dangling_manifest_carries_the_walks_broken_link_reason() {
    let (root, dir) = case_dir("e2e_case_dangling_reason");
    let link = dir.join("spec.refs");
    symlink("spec.refs.gone", &link);

    assert_eq!(reason_at(&root, &dir, &link), BROKEN_LINK);
}
