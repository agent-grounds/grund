//! The label a build of the CLI reports, derived over fixture trees by the build
//! script's own function, without Cargo (§FS-distribution.4.1.1): which trees have
//! qualifying git, when `.dirty` is earned, when `GRUND_BUILD_COMMIT` is taken or
//! refused, and that every path the script would watch exists.

#[path = "../build_identity.rs"]
mod build_identity;

use build_identity::{BuildIdentity, MalformedCommit, build_identity};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const DEV: &str = "0.20.1-dev";

/// Git's location variables, which a pre-commit hook sets: no fixture command may
/// follow them into the repository the suite runs in.
const GIT_LOCATION: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_CONFIG",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
];

/// A scratch directory under the system temp dir, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1043-identity-{}-{}-{name}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    /// A workspace laid out like this one, at `relative`, not yet under git.
    fn workspace(&self, relative: &str) -> PathBuf {
        let root = self.0.join(relative);
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/grund-cli\"]\n",
        );
        write(
            &root.join("crates/grund-cli/Cargo.toml"),
            "[package]\nname = \"grund\"\n",
        );
        write(&root.join("crates/grund-cli/src/main.rs"), "fn main() {}\n");
        write(&root.join("docs/notes.md"), "notes\n");
        write(&root.join(".gitignore"), "/target/\n");
        root
    }

    /// A workspace committed as its own repository: qualifying git.
    fn checkout(&self, relative: &str) -> PathBuf {
        let root = self.workspace(relative);
        self.git(&root, &["init", "-q"]);
        self.commit_all(&root, "one");
        root
    }

    fn commit_all(&self, dir: &Path, message: &str) {
        self.git(dir, &["add", "-A"]);
        self.git(dir, &["commit", "-q", "-m", message]);
    }

    fn git(&self, dir: &Path, args: &[&str]) -> String {
        let mut command = Command::new("git");
        for name in GIT_LOCATION {
            command.env_remove(name);
        }
        let output = command
            .args([
                "-c",
                "init.defaultBranch=main",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(["-c", "core.autocrlf=false"])
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", self.0.join("no-global-gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.com")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.com")
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} in {}: {output:?}",
            dir.display()
        );
        String::from_utf8(output.stdout).unwrap()
    }

    /// The first ten hexadecimal digits of HEAD's full SHA, read here independently.
    fn head(&self, dir: &Path) -> String {
        self.git(dir, &["rev-parse", "HEAD"]).trim()[..10].to_string()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn crate_dir(workspace: &Path) -> PathBuf {
    workspace.join("crates/grund-cli")
}

fn derive_with(
    version: &str,
    crate_dir: &Path,
    commit: Option<&str>,
    git: &OsStr,
) -> Result<BuildIdentity, MalformedCommit> {
    build_identity(version, crate_dir, commit.map(OsStr::new), git)
}

fn derive(crate_dir: &Path, commit: Option<&str>) -> BuildIdentity {
    derive_with(DEV, crate_dir, commit, OsStr::new("git"))
        .unwrap_or_else(|refused| panic!("{}: refused: {refused}", crate_dir.display()))
}

/// The tree has qualifying git and names `expected`; every watched path exists, so
/// Cargo reruns the script only when one of them changes (§FS-distribution.4.1.1).
fn assert_names(crate_dir: &Path, expected: &str) {
    let identity = derive(crate_dir, None);
    assert_eq!(identity.version, expected, "{}", crate_dir.display());
    assert!(
        !identity.watched.is_empty(),
        "qualifying git watches nothing"
    );
    for path in &identity.watched {
        let path = crate_dir.join(path);
        assert!(
            path.exists(),
            "watched path does not exist: {}",
            path.display()
        );
    }
}

/// The tree has no qualifying git: bare without the variable, the variable as given
/// with it, in both of its spellings.
fn assert_no_git(crate_dir: &Path, git: &OsStr) {
    let bare = derive_with(DEV, crate_dir, None, git).unwrap();
    assert_eq!(bare.version, DEV, "{}", crate_dir.display());
    for given in ["0123456789", "abcdef0123.dirty"] {
        let taken = derive_with(DEV, crate_dir, Some(given), git).unwrap();
        assert_eq!(
            taken.version,
            format!("{DEV}+{given}"),
            "{}",
            crate_dir.display()
        );
    }
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_clean_checkout_names_the_first_ten_digits_of_its_commit() {
    let scratch = Scratch::new("clean");
    let root = scratch.checkout("ws");
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn an_unstaged_or_a_staged_edit_to_a_tracked_file_is_dirty() {
    let scratch = Scratch::new("dirty");
    let root = scratch.checkout("ws");
    let dirty = format!("{DEV}+{}.dirty", scratch.head(&root));
    write(&root.join("docs/notes.md"), "edited outside the crate\n");
    assert_names(&crate_dir(&root), &dirty);
    scratch.git(&root, &["add", "docs/notes.md"]);
    assert_names(&crate_dir(&root), &dirty);
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn untracked_and_ignored_files_leave_the_build_clean() {
    let scratch = Scratch::new("untracked");
    let root = scratch.checkout("ws");
    write(&root.join("docs/untracked.md"), "new\n");
    write(&root.join("target/debug/ignored"), "built\n");
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_worktree_names_its_own_head_clean_and_dirty() {
    let scratch = Scratch::new("worktree");
    let root = scratch.checkout("ws");
    let tree = scratch.0.join("wt");
    scratch.git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "side",
            tree.to_str().unwrap(),
        ],
    );
    write(&tree.join("docs/notes.md"), "on the side\n");
    scratch.commit_all(&tree, "side");
    let head = scratch.head(&tree);
    assert_ne!(head, scratch.head(&root));
    assert_names(&crate_dir(&tree), &format!("{DEV}+{head}"));
    write(&tree.join("docs/notes.md"), "edited on the side\n");
    assert_names(&crate_dir(&tree), &format!("{DEV}+{head}.dirty"));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_repository_with_no_packed_refs_watches_only_paths_that_exist() {
    let scratch = Scratch::new("no-packed-refs");
    let root = scratch.checkout("ws");
    assert!(!root.join(".git/packed-refs").exists());
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_packed_branch_ref_names_its_commit_and_watches_only_paths_that_exist() {
    let scratch = Scratch::new("packed");
    let root = scratch.checkout("ws");
    scratch.git(&root, &["pack-refs", "--all"]);
    assert!(!root.join(".git/refs/heads/main").exists());
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));

    scratch.git(&root, &["checkout", "-q", "-b", "fix/nested"]);
    scratch.git(&root, &["pack-refs", "--all"]);
    assert!(
        !root.join(".git/refs/heads/fix").exists(),
        "pack-refs left refs/heads/fix/"
    );
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_deleted_or_renamed_tracked_file_is_dirty_and_watches_only_paths_that_exist() {
    let scratch = Scratch::new("deleted");
    let root = scratch.checkout("ws");
    let dirty = format!("{DEV}+{}.dirty", scratch.head(&root));
    fs::remove_file(root.join("docs/notes.md")).unwrap();
    assert_names(&crate_dir(&root), &dirty);
    scratch.git(&root, &["checkout", "--", "docs/notes.md"]);
    scratch.git(&root, &["mv", "docs/notes.md", "docs/moved.md"]);
    assert_names(&crate_dir(&root), &dirty);
}

#[test]
#[ignore = "red until #1043 the build script takes GRUND_BUILD_COMMIT; implement removes this"]
fn a_tree_with_no_git_takes_the_variable_or_reports_the_bare_version() {
    let scratch = Scratch::new("no-git");
    let root = scratch.workspace("ws");
    assert_no_git(&crate_dir(&root), OsStr::new("git"));
}

#[test]
#[ignore = "red until #1043 the build script takes GRUND_BUILD_COMMIT; implement removes this"]
fn an_unborn_head_has_no_qualifying_git() {
    let scratch = Scratch::new("unborn");
    let root = scratch.workspace("ws");
    scratch.git(&root, &["init", "-q"]);
    scratch.git(&root, &["add", "-A"]);
    assert_no_git(&crate_dir(&root), OsStr::new("git"));
}

#[test]
#[ignore = "red until #1043 the build script takes GRUND_BUILD_COMMIT; implement removes this"]
fn a_tree_inside_a_foreign_repository_has_no_qualifying_git() {
    let scratch = Scratch::new("foreign");
    let outer = scratch.0.join("outer");
    write(&outer.join("README.md"), "another project\n");
    scratch.git(&outer, &["init", "-q"]);
    scratch.commit_all(&outer, "outer");
    let inner = scratch.workspace("outer/vendor/ws");
    assert_no_git(&crate_dir(&inner), OsStr::new("git"));
    scratch.commit_all(&outer, "vendor the workspace");
    assert_no_git(&crate_dir(&inner), OsStr::new("git"));
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn a_crate_copy_nested_inside_a_checkout_has_no_qualifying_git() {
    let scratch = Scratch::new("nested-copy");
    let root = scratch.checkout("ws");
    assert_names(&crate_dir(&root), &format!("{DEV}+{}", scratch.head(&root)));
    // An unpacked `.crate` two levels inside the checkout, as under its `tmp/`.
    let copy = root.join("tmp/grund-0.21.1-dev");
    write(&copy.join("Cargo.toml"), "[package]\nname = \"grund\"\n");
    write(&copy.join("src/main.rs"), "fn main() {}\n");
    assert_no_git(&copy, OsStr::new("git"));
}

#[test]
#[ignore = "red until #1043 the build script takes GRUND_BUILD_COMMIT; implement removes this"]
fn missing_git_counts_as_no_qualifying_git() {
    let scratch = Scratch::new("missing-git");
    let root = scratch.checkout("ws");
    let missing = scratch.0.join("no-such-git");
    assert_no_git(&crate_dir(&root), missing.as_os_str());
}

/// A stub git that records each call to `log` and refuses like a checkout git
/// calls of dubious ownership.
#[cfg(unix)]
fn refusing_git(scratch: &Scratch, log: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let stub = scratch.0.join("refusing-git");
    let script = format!(
        "#!/bin/sh\necho \"$@\" >> '{}'\n\
         echo \"fatal: detected dubious ownership in repository\" >&2\nexit 128\n",
        log.display()
    );
    fs::write(&stub, script).unwrap();
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
    stub
}

#[cfg(unix)]
#[test]
#[ignore = "red until #1043 the build script takes GRUND_BUILD_COMMIT; implement removes this"]
fn git_refusing_the_checkout_counts_as_no_qualifying_git() {
    let scratch = Scratch::new("refusing-git");
    let root = scratch.checkout("ws");
    let log = scratch.0.join("git-calls");
    assert_no_git(&crate_dir(&root), refusing_git(&scratch, &log).as_os_str());
}

#[test]
#[ignore = "red until #1043 the build script reads qualifying git; implement removes this"]
fn qualifying_git_wins_over_a_well_formed_variable() {
    let scratch = Scratch::new("git-wins");
    let root = scratch.checkout("ws");
    let identity = derive(&crate_dir(&root), Some("0123456789"));
    assert_eq!(identity.version, format!("{DEV}+{}", scratch.head(&root)));
}

#[test]
#[ignore = "red until #1043 the build script refuses a malformed GRUND_BUILD_COMMIT; implement removes this"]
fn a_malformed_variable_fails_a_development_build_whether_or_not_git_wins() {
    let scratch = Scratch::new("malformed");
    let with_git = scratch.checkout("with-git");
    let without = scratch.workspace("without-git");
    let forty = "0123456789abcdef0123456789abcdef01234567";
    let malformed = [
        "",
        forty,
        "012345678",
        "ABCDEF0123",
        ".dirty",
        "abcdef0123.DIRTY",
        "abcdef0123+dirty",
        "abcdef0123.dirty.dirty",
        "abcdef0123 ",
    ];
    for root in [&with_git, &without] {
        for given in malformed {
            let refused = derive_with(DEV, &crate_dir(root), Some(given), OsStr::new("git"));
            let expected = MalformedCommit {
                value: given.to_string(),
            };
            assert_eq!(refused, Err(expected), "{given:?} in {}", root.display());
            let message = refused.unwrap_err().to_string();
            assert!(message.contains("GRUND_BUILD_COMMIT"), "{message}");
        }
    }
}

#[test]
fn a_release_version_reads_neither_git_nor_the_variable() {
    let scratch = Scratch::new("release");
    let root = scratch.checkout("ws");
    write(&root.join("docs/notes.md"), "a dirty tree\n");
    for version in ["0.20.0", "1.0.0-rc.1"] {
        for given in [None, Some("garbage"), Some("0123456789")] {
            let identity = derive_with(version, &crate_dir(&root), given, OsStr::new("git"));
            assert_eq!(identity.unwrap().version, version, "{given:?}");
        }
    }
    #[cfg(unix)]
    {
        let log = scratch.0.join("git-calls");
        let git = refusing_git(&scratch, &log);
        let identity = derive_with(
            "0.20.0",
            &crate_dir(&root),
            Some("garbage"),
            git.as_os_str(),
        );
        assert_eq!(identity.unwrap().version, "0.20.0");
        assert!(
            !log.exists(),
            "a release build ran git: {:?}",
            fs::read_to_string(&log)
        );
    }
}
