//! A case manifest whose name is a link whose target does not exist is a manifest that cannot be
//! read, not an absent one: `check` fails the case at the link's own path with the broken-link
//! reason and exits 2, whole-tree and narrowed to the case; a dangling `expected.exit` still names
//! a case that fails there (§FS-declarations.e2e-case.id), and a link that loops carries the
//! walk's unreadable-link reason (§FS-declarations.e2e-case.failure,
//! §FS-scan.scope.links.unresolvable, §FS-check.2.4, §FS-check.1.3.6.3). These are not goldens
//! because a golden has no skip and a dangling link is not portable in git; the tree is built in
//! the temp directory instead. Unix-only: every tree here is about a link.
#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
                      slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.E2E]\n\
                      folders = [\"e2e/cases\"]\nindex = false\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// Cases `a`, `b` and `c`, each with `expected.exit`, and `b/<manifest>` a dangling link.
    fn dangling(name: &str, manifest: &str) -> Self {
        Self::linked(name, manifest, &format!("{manifest}.gone"))
    }

    /// Cases `a`, `b` and `c`, each with `expected.exit`, and `b/<manifest>` a link to `target`,
    /// in place of any file of that name.
    fn linked(name: &str, manifest: &str, target: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1330-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        for case in ["a", "b", "c"] {
            let dir = root.join("e2e/cases").join(case);
            fs::create_dir_all(&dir).expect("create the case");
            fs::write(dir.join("expected.exit"), "0\n").expect("write expected.exit");
        }
        let link = root.join("e2e/cases/b").join(manifest);
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(target, link).expect("create the link");
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `check <args>` exits 2 and says exactly the broken-link line at `e2e/cases/b/<manifest>`.
fn assert_fails_at_the_link(name: &str, manifest: &str, args: &[&str]) {
    let tree = Tree::dangling(name, manifest);
    assert_says(
        &tree,
        args,
        manifest,
        "broken symlink: the target does not exist",
    );
}

/// `check <args>` over `tree` exits 2 and says exactly `reason` at `e2e/cases/b/<manifest>`.
fn assert_says(tree: &Tree, args: &[&str], manifest: &str, reason: &str) {
    let output = tree.run(args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = format!(
        "error: e2e/cases/b/{manifest}: {reason}; \
         make the path readable, or keep it out of the scan\n"
    );
    assert_eq!(stderr, expected, "grund {args:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "",
        "grund {args:?}"
    );
    assert_eq!(output.status.code(), Some(2), "grund {args:?}");
}

#[test]
fn a_dangling_spec_refs_fails_the_whole_tree_check_at_the_link() {
    assert_fails_at_the_link("spec-refs-tree", "spec.refs", &["check"]);
}

#[test]
fn a_dangling_spec_refs_fails_the_check_narrowed_to_its_case() {
    assert_fails_at_the_link("spec-refs-case", "spec.refs", &["check", "e2e/cases/b"]);
}

#[test]
fn a_dangling_command_args_fails_the_whole_tree_check_at_the_link() {
    assert_fails_at_the_link("command-args-tree", "command.args", &["check"]);
}

#[test]
fn a_dangling_command_args_fails_the_check_narrowed_to_its_case() {
    assert_fails_at_the_link(
        "command-args-case",
        "command.args",
        &["check", "e2e/cases/b"],
    );
}

// §FS-declarations.e2e-case.id: a dangling `expected.exit` still names a case, which fails at it.
#[test]
fn a_dangling_expected_exit_fails_the_whole_tree_check_at_the_link() {
    assert_fails_at_the_link("expected-exit-tree", "expected.exit", &["check"]);
}

#[test]
fn a_dangling_expected_exit_fails_the_check_narrowed_to_its_case() {
    assert_fails_at_the_link(
        "expected-exit-case",
        "expected.exit",
        &["check", "e2e/cases/b"],
    );
}

/// `list` is not silent about the case either: like an `expected.exit` at mode 000, it names the
/// link, exits 2 and lists the other cases, the failing one declaring nothing.
#[test]
fn listing_a_case_whose_expected_exit_dangles_names_the_link() {
    let tree = Tree::dangling("expected-exit-list", "expected.exit");
    let output = tree.run(&["list", "--kind", "E2E"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: e2e/cases/b/expected.exit: broken symlink: the target does not exist; \
         make the path readable, or keep it out of the scan\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "E2E-a  e2e/cases/a:1\nE2E-c  e2e/cases/c:1\n"
    );
    assert_eq!(output.status.code(), Some(2));
}

/// A manifest link that is a loop carries the reason the walk gives the same link
/// (§FS-scan.scope.links.unresolvable).
#[test]
fn a_looping_spec_refs_carries_the_walks_unreadable_link_reason() {
    let tree = Tree::linked("spec-refs-loop", "spec.refs", "spec.refs");
    assert_says(
        &tree,
        &["check"],
        "spec.refs",
        "unreadable symlink: filesystem loop or indirection limit (e.g. symlink loop)",
    );
}
