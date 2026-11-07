//! Test module: what one walk root remembers of the directory links it has recovered, which keeps
//! "nothing new" apart from "no link at all" (§AR-scanner.1.9). Only the second may reach the
//! walk root as a location, and each directory is listed once per walk root.

use std::fs;
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::walk_errors::LinkRecovery;
use crate::testing::test_root;

fn pathless(kind: io::ErrorKind) -> ignore::Error {
    ignore::Error::Io(io::Error::from(kind))
}

/// The links an error of `kind` brings to light in a walk inside `listing`, sorted.
fn recovered(
    recovery: &mut LinkRecovery,
    kind: io::ErrorKind,
    listing: &Path,
    root: &Path,
) -> Option<Vec<PathBuf>> {
    recovery
        .unopenable_links(&pathless(kind), Some(listing), root)
        .map(|links| {
            let mut paths: Vec<PathBuf> = links.into_iter().map(|(path, _)| path).collect();
            paths.sort();
            paths
        })
}

/// `<root>/docs` holding links `a` and `b` to a barred `<root>/real`; `None` where the mode does
/// not bar this user (root), so nothing here fails to open.
fn barred_tree(name: &str) -> Option<PathBuf> {
    let root = test_root(name);
    fs::create_dir_all(root.join("docs")).expect("create the home");
    fs::create_dir_all(root.join("real")).expect("create the target");
    symlink("../real", root.join("docs/a")).expect("create the link");
    symlink("../real", root.join("docs/b")).expect("create the link");
    fs::set_permissions(root.join("real"), fs::Permissions::from_mode(0o000)).expect("bar it");
    if fs::read_dir(root.join("real")).is_ok() {
        fs::set_permissions(root.join("real"), fs::Permissions::from_mode(0o755)).expect("free it");
        return None;
    }
    Some(root)
}

fn free(root: &Path) {
    fs::set_permissions(root.join("real"), fs::Permissions::from_mode(0o755)).expect("free it");
}

#[test]
fn a_recovered_link_is_handed_out_once_and_then_nothing_new_is_still_an_answer() {
    let Some(root) = barred_tree("link-recovery-once") else {
        return;
    };
    let docs = root.join("docs");
    let mut recovery = LinkRecovery::default();
    let denied = io::ErrorKind::PermissionDenied;
    let first = recovered(&mut recovery, denied, &docs, &root);
    let second = recovered(&mut recovery, denied, &docs, &root);
    free(&root);
    assert_eq!(first, Some(vec![docs.join("a"), docs.join("b")]));
    assert_eq!(second, Some(vec![]));
}

#[test]
fn an_error_of_another_kind_after_a_recovery_is_about_no_link() {
    let Some(root) = barred_tree("link-recovery-kind") else {
        return;
    };
    let docs = root.join("docs");
    let mut recovery = LinkRecovery::default();
    let first = recovered(&mut recovery, io::ErrorKind::PermissionDenied, &docs, &root);
    let other = recovered(&mut recovery, io::ErrorKind::OutOfMemory, &docs, &root);
    free(&root);
    assert!(first.is_some_and(|links| links.len() == 2));
    assert_eq!(other, None);
}

#[test]
fn a_directory_is_listed_once_per_walk_root() {
    let Some(root) = barred_tree("link-recovery-listed-once") else {
        return;
    };
    let docs = root.join("docs");
    let mut recovery = LinkRecovery::default();
    let denied = io::ErrorKind::PermissionDenied;
    let first = recovered(&mut recovery, denied, &docs, &root);
    symlink("../real", docs.join("later")).expect("create the link");
    let second = recovered(&mut recovery, denied, &docs, &root);
    let fresh = recovered(&mut LinkRecovery::default(), denied, &docs, &root);
    free(&root);
    assert!(first.is_some_and(|links| links.len() == 2));
    assert_eq!(
        second,
        Some(vec![]),
        "the directory was listed a second time"
    );
    assert!(fresh.is_some_and(|links| links.len() == 3));
}

#[test]
fn a_directory_with_no_such_link_and_a_walk_not_yet_inside_one_explain_nothing() {
    let root = test_root("link-recovery-none");
    fs::create_dir_all(root.join("docs")).expect("create the home");
    let mut recovery = LinkRecovery::default();
    let denied = io::ErrorKind::PermissionDenied;
    assert_eq!(
        recovered(&mut recovery, denied, &root.join("docs"), &root),
        None
    );
    assert!(
        recovery
            .unopenable_links(&pathless(denied), None, &root)
            .is_none()
    );
}
