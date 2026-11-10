//! A case manifest whose name is a link whose target does not exist is a manifest that cannot be
//! read, not an absent one: `check` fails the case at the link's own path with the broken-link
//! reason and exits 2, whole-tree and narrowed to the case (§FS-declarations.e2e-case.failure,
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
        std::os::unix::fs::symlink(format!("{manifest}.gone"), link).expect("create the link");
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
    let output = tree.run(args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = format!(
        "error: e2e/cases/b/{manifest}: broken symlink: the target does not exist; \
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
#[ignore = "red until #1330 reads a dangling manifest as unread rather than absent; implement removes this"]
fn a_dangling_spec_refs_fails_the_whole_tree_check_at_the_link() {
    assert_fails_at_the_link("spec-refs-tree", "spec.refs", &["check"]);
}

#[test]
#[ignore = "red until #1330 reads a dangling manifest as unread rather than absent; implement removes this"]
fn a_dangling_spec_refs_fails_the_check_narrowed_to_its_case() {
    assert_fails_at_the_link("spec-refs-case", "spec.refs", &["check", "e2e/cases/b"]);
}

#[test]
#[ignore = "red until #1330 reads a dangling manifest as unread rather than absent; implement removes this"]
fn a_dangling_command_args_fails_the_whole_tree_check_at_the_link() {
    assert_fails_at_the_link("command-args-tree", "command.args", &["check"]);
}

#[test]
#[ignore = "red until #1330 reads a dangling manifest as unread rather than absent; implement removes this"]
fn a_dangling_command_args_fails_the_check_narrowed_to_its_case() {
    assert_fails_at_the_link(
        "command-args-case",
        "command.args",
        &["check", "e2e/cases/b"],
    );
}
