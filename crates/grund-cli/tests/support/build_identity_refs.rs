//! Git-valid Unix ref names retain their bytes and keep naming this build's
//! exact commit, including after a symbolic alias moves (§FS-distribution.4.1.1).
//! The real-Cargo harness also holds both no-op builds after each transition.

use super::*;
use std::os::unix::ffi::OsStrExt;

fn assert_clean(ws: &Workspace) {
    assert_eq!(
        ws.git(
            &ws.root,
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=no",
            ],
        ),
        ""
    );
}

#[test]
fn a_git_valid_non_utf8_branch_keeps_its_commit() {
    let ws = Workspace::new("non-utf8-branch");
    let root = &ws.root;
    let branch = OsStr::from_bytes(b"refs/heads/nonutf/branch-\xff");
    ws.git(root, &[OsStr::new("check-ref-format"), branch]);
    ws.git(
        root,
        &[OsStr::new("update-ref"), branch, OsStr::new("HEAD")],
    );
    ws.git(
        root,
        &[OsStr::new("symbolic-ref"), OsStr::new("HEAD"), branch],
    );
    assert_clean(&ws);
    let first = format!("{DEV}+{}", ws.head(root));
    ws.expect("non-UTF-8 branch", root, &[], &first, Rerun::Must);
    ws.settled("non-UTF-8 branch", root, &[], &first);

    ws.git(root, &["commit", "-q", "--allow-empty", "-m", "two"]);
    assert_clean(&ws);
    let second = format!("{DEV}+{}", ws.head(root));
    assert_ne!(first, second);
    ws.expect("non-UTF-8 ref moves", root, &[], &second, Rerun::Must);
    ws.settled("non-UTF-8 ref moves", root, &[], &second);

    ws.git(root, &["pack-refs", "--all"]);
    assert!(!root.join(".git").join(branch).exists());
    ws.expect("non-UTF-8 ref packed", root, &[], &second, Rerun::Either);
    ws.settled("non-UTF-8 ref packed", root, &[], &second);
    ws.git(root, &["commit", "-q", "--allow-empty", "-m", "three"]);
    assert_clean(&ws);
    let third = format!("{DEV}+{}", ws.head(root));
    assert_ne!(second, third);
    ws.expect("packed non-UTF-8 ref moves", root, &[], &third, Rerun::Must);
    ws.settled("packed non-UTF-8 ref moves", root, &[], &third);
}

#[test]
fn retargeting_a_non_utf8_symbolic_ref_updates_the_label() {
    let ws = Workspace::new("non-utf8-alias");
    let root = &ws.root;
    let first = format!("{DEV}+{}", ws.head(root));
    ws.git(root, &["commit", "-q", "--allow-empty", "-m", "two"]);
    let second = format!("{DEV}+{}", ws.head(root));
    assert_ne!(first, second);
    ws.git(root, &["branch", "other"]);
    ws.git(root, &["reset", "--hard", "HEAD~1"]);
    let alias = OsStr::from_bytes(b"refs/heads/aliases/alias-\xff");
    ws.git(root, &[OsStr::new("check-ref-format"), alias]);
    ws.git(
        root,
        &[
            OsStr::new("symbolic-ref"),
            alias,
            OsStr::new("refs/heads/main"),
        ],
    );
    ws.git(
        root,
        &[OsStr::new("symbolic-ref"), OsStr::new("HEAD"), alias],
    );
    assert_clean(&ws);
    ws.expect("non-UTF-8 alias", root, &[], &first, Rerun::Must);
    ws.settled("non-UTF-8 alias", root, &[], &first);

    let head = fs::read(root.join(".git/HEAD")).unwrap();
    let main = fs::read(root.join(".git/refs/heads/main")).unwrap();
    let other = fs::read(root.join(".git/refs/heads/other")).unwrap();
    let head_mtime = fs::metadata(root.join(".git/HEAD"))
        .unwrap()
        .modified()
        .unwrap();
    ws.git(
        root,
        &[
            OsStr::new("symbolic-ref"),
            alias,
            OsStr::new("refs/heads/other"),
        ],
    );
    assert_eq!(head, fs::read(root.join(".git/HEAD")).unwrap());
    assert_eq!(
        head_mtime,
        fs::metadata(root.join(".git/HEAD"))
            .unwrap()
            .modified()
            .unwrap()
    );
    assert_eq!(main, fs::read(root.join(".git/refs/heads/main")).unwrap());
    assert_eq!(other, fs::read(root.join(".git/refs/heads/other")).unwrap());
    assert_clean(&ws);
    ws.expect(
        "non-UTF-8 alias retargeted",
        root,
        &[],
        &second,
        Rerun::Must,
    );
    ws.settled("non-UTF-8 alias retargeted", root, &[], &second);

    ws.git(root, &["pack-refs", "--all"]);
    assert!(!root.join(".git/refs/heads/main").exists());
    assert!(!root.join(".git/refs/heads/other").exists());
    ws.expect("alias terminals packed", root, &[], &second, Rerun::Either);
    ws.settled("alias terminals packed", root, &[], &second);
    ws.git(
        root,
        &[
            OsStr::new("symbolic-ref"),
            alias,
            OsStr::new("refs/heads/main"),
        ],
    );
    assert_clean(&ws);
    ws.expect("alias to packed main", root, &[], &first, Rerun::Must);
    ws.settled("alias to packed main", root, &[], &first);
}
