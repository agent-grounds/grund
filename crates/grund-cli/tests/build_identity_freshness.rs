//! A build's label never names a stale state, and costs a rerun only when it has to
//! (§FS-distribution.4.1.1). A temporary workspace laid out like this one carries
//! the real build script; each step changes the tree, rebuilds it through Cargo
//! and runs the binary it built.
//!
//! Whether the build script ran is read from `cargo build -v`, which prints a
//! ``Running `…/build-script-build` `` line exactly when Cargo executes the script
//! and prints nothing about it when the script's output is fresh.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const DEV: &str = "0.20.1-dev";

/// The build script and every file it includes, copied from this crate.
const SCRIPT_FILES: &[&str] = &["build.rs", "build_identity.rs"];

/// Git's location variables, which a pre-commit hook sets.
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Rerun {
    /// The step changed an input, so the script must run again.
    Must,
    /// Nothing the label depends on changed, so the script must stay fresh.
    MustNot,
    /// Either is correct.
    Either,
}

struct Workspace {
    scratch: PathBuf,
    root: PathBuf,
}

impl Workspace {
    /// A committed workspace with the real build script, outside every other tree.
    fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let scratch = std::env::temp_dir().join(format!(
            "grund-1043-freshness-{}-{}-{name}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&scratch);
        let root = scratch.join("ws");
        let manifest = format!(
            "[workspace]\nmembers = [\"crates/grund-cli\"]\nresolver = \"3\"\n\n\
             [workspace.package]\nversion = \"{DEV}\"\nedition = \"2024\"\n"
        );
        write(&root.join("Cargo.toml"), &manifest);
        write(
            &root.join("crates/grund-cli/Cargo.toml"),
            "[package]\nname = \"grund\"\nversion.workspace = true\n\
             edition.workspace = true\npublish = false\n",
        );
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        for file in SCRIPT_FILES {
            fs::copy(source.join(file), root.join("crates/grund-cli").join(file)).unwrap();
        }
        write(
            &root.join("crates/grund-cli/src/main.rs"),
            "fn main() {\n    println!(\"{}\", env!(\"GRUND_BUILD_VERSION\"));\n}\n",
        );
        write(&root.join("docs/notes.md"), "notes\n");
        write(&root.join(".gitignore"), "/target/\n/Cargo.lock\n");
        let workspace = Self { scratch, root };
        workspace.git(&workspace.root, &["init", "-q"]);
        workspace.commit_all(&workspace.root, "one");
        workspace
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
            .env(
                "GIT_CONFIG_GLOBAL",
                self.scratch.join("no-global-gitconfig"),
            )
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

    fn commit_all(&self, dir: &Path, message: &str) {
        self.git(dir, &["add", "-A"]);
        self.git(dir, &["commit", "-q", "-m", message]);
    }

    fn head(&self, dir: &Path) -> String {
        self.git(dir, &["rev-parse", "HEAD"]).trim()[..10].to_string()
    }

    /// Build the workspace at `dir` through Cargo with `env` added.
    fn cargo(&self, dir: &Path, env: &[(&str, &str)]) -> Output {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let mut command = Command::new(cargo);
        for name in GIT_LOCATION
            .iter()
            .chain(&["GRUND_BUILD_COMMIT", "CARGO_BUILD_TARGET_DIR"])
        {
            command.env_remove(name);
        }
        command
            .args(["build", "-v", "--offline"])
            .current_dir(dir)
            .env("CARGO_TARGET_DIR", dir.join("target"))
            .env("CARGO_LOG", "cargo::core::compiler::fingerprint=info")
            .envs(env.iter().copied())
            .output()
            .expect("run cargo")
    }

    /// Build `dir` with `env` added, then run it.
    /// Returns the label the binary printed and whether the build script ran.
    fn build(&self, dir: &Path, env: &[(&str, &str)]) -> (String, bool, String) {
        let output = self.cargo(dir, env);
        let log = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            output.status.success(),
            "cargo build in {}:\n{log}",
            dir.display()
        );
        let reran = log.lines().any(|line| {
            line.trim_start().starts_with("Running `") && line.contains("build-script-build")
        });
        let binary = dir.join(format!(
            "target/debug/grund{}",
            std::env::consts::EXE_SUFFIX
        ));
        let ran = Command::new(&binary)
            .output()
            .expect("run the built binary");
        assert!(ran.status.success(), "{}: {ran:?}", binary.display());
        let label = String::from_utf8(ran.stdout)
            .unwrap()
            .trim_end()
            .to_string();
        (label, reran, log)
    }

    /// One step: build `dir` with `env`, and hold its label and its rerun.
    fn expect(&self, step: &str, dir: &Path, env: &[(&str, &str)], label: &str, rerun: Rerun) {
        let (built, reran, log) = self.build(dir, env);
        let why = || {
            log.lines()
                .filter(|line| line.contains("dirty") || line.contains("stale"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(
            built,
            label,
            "{step}: the build names the wrong state\n{}",
            why()
        );
        match rerun {
            Rerun::Must => assert!(
                reran,
                "{step}: an input changed but the script stayed fresh"
            ),
            Rerun::MustNot => assert!(!reran, "{step}: the script reran\n{}", why()),
            Rerun::Either => {}
        }
    }

    /// One step that must fail the build: returns what Cargo printed.
    fn refused(&self, step: &str, dir: &Path, env: &[(&str, &str)]) -> String {
        let output = self.cargo(dir, env);
        let log = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            !output.status.success(),
            "{step}: the build succeeded\n{log}"
        );
        log
    }

    /// Two no-op builds after a step: each keeps the label and reruns nothing.
    fn settled(&self, step: &str, dir: &Path, env: &[(&str, &str)], label: &str) {
        for round in ["first", "second"] {
            let step = format!("{step}, {round} no-op build");
            self.expect(&step, dir, env, label, Rerun::MustNot);
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
#[ignore = "red until #1043 the build script names the commit and watches what changes it; implement removes this"]
fn the_label_follows_head_and_the_dirty_state() {
    let ws = Workspace::new("head");
    let root = ws.root.clone();
    let one = ws.head(&root);
    ws.expect("clean", &root, &[], &format!("{DEV}+{one}"), Rerun::Either);
    ws.settled("clean", &root, &[], &format!("{DEV}+{one}"));

    write(&root.join("docs/notes.md"), "edited, not staged\n");
    let dirty = format!("{DEV}+{one}.dirty");
    ws.expect("unstaged edit", &root, &[], &dirty, Rerun::Must);
    ws.settled("unstaged edit", &root, &[], &dirty);

    write(&root.join("docs/notes.md"), "notes\n");
    ws.expect(
        "reverted by hand",
        &root,
        &[],
        &format!("{DEV}+{one}"),
        Rerun::Must,
    );
    ws.settled("reverted by hand", &root, &[], &format!("{DEV}+{one}"));

    ws.git(&root, &["pack-refs", "--all"]);
    assert!(!root.join(".git/refs/heads/main").exists());
    ws.expect("packed", &root, &[], &format!("{DEV}+{one}"), Rerun::Either);
    ws.settled("packed", &root, &[], &format!("{DEV}+{one}"));
    write(&root.join("docs/notes.md"), "two\n");
    ws.commit_all(&root, "two");
    let two = ws.head(&root);
    ws.expect(
        "commit on a packed ref",
        &root,
        &[],
        &format!("{DEV}+{two}"),
        Rerun::Must,
    );
    ws.settled(
        "commit on a packed ref",
        &root,
        &[],
        &format!("{DEV}+{two}"),
    );

    // A pre-commit hook's location variables point at another repository.
    let foreign = Workspace::new("foreign");
    let (dir, index) = (foreign.root.join(".git"), foreign.root.join(".git/index"));
    let hook = [
        ("GIT_DIR", dir.to_str().unwrap()),
        ("GIT_INDEX_FILE", index.to_str().unwrap()),
    ];
    write(&root.join("docs/notes.md"), "three\n");
    ws.commit_all(&root, "three");
    let three = ws.head(&root);
    ws.expect(
        "commit under a hook's variables",
        &root,
        &hook,
        &format!("{DEV}+{three}"),
        Rerun::Must,
    );

    let tree = ws.scratch.join("wt");
    ws.git(
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
    ws.expect(
        "a worktree",
        &tree,
        &[],
        &format!("{DEV}+{three}"),
        Rerun::Either,
    );
    ws.settled("a worktree", &tree, &[], &format!("{DEV}+{three}"));
    ws.git(&tree, &["checkout", "-q", "--detach", &two]);
    ws.expect(
        "HEAD moved in a worktree",
        &tree,
        &[],
        &format!("{DEV}+{two}"),
        Rerun::Must,
    );
    ws.settled(
        "HEAD moved in a worktree",
        &tree,
        &[],
        &format!("{DEV}+{two}"),
    );
    ws.expect(
        "the main worktree after",
        &root,
        &[],
        &format!("{DEV}+{three}"),
        Rerun::Either,
    );

    write(&root.join("docs/untracked.md"), "new\n");
    ws.expect(
        "untracked file",
        &root,
        &[],
        &format!("{DEV}+{three}"),
        Rerun::MustNot,
    );
    ws.settled("untracked file", &root, &[], &format!("{DEV}+{three}"));
}

#[test]
#[ignore = "red until #1043 the build script names the commit and watches what changes it; implement removes this"]
fn a_tree_that_loses_its_git_builds_bare_until_the_variable_or_git_names_it() {
    let ws = Workspace::new("gitless");
    let root = ws.root.clone();
    let one = format!("{DEV}+{}", ws.head(&root));
    ws.expect("clean", &root, &[], &one, Rerun::Either);
    ws.settled("clean", &root, &[], &one);

    fs::rename(root.join(".git"), ws.scratch.join("git-away")).unwrap();
    ws.expect(".git removed, variable unset", &root, &[], DEV, Rerun::Must);
    let set = [("GRUND_BUILD_COMMIT", "0123456789")];
    ws.expect(
        "variable set",
        &root,
        &set,
        &format!("{DEV}+0123456789"),
        Rerun::Must,
    );
    let changed = [("GRUND_BUILD_COMMIT", "abcdefabcd.dirty")];
    let given = format!("{DEV}+abcdefabcd.dirty");
    ws.expect("variable changed", &root, &changed, &given, Rerun::Must);

    fs::rename(ws.scratch.join("git-away"), root.join(".git")).unwrap();
    ws.expect(
        ".git restored, variable set",
        &root,
        &changed,
        &one,
        Rerun::Must,
    );
    ws.settled(".git restored, variable set", &root, &changed, &one);
    ws.expect("variable unset", &root, &[], &one, Rerun::Either);
    ws.settled("variable unset", &root, &[], &one);

    // A malformed variable fails the build while git wins, so it is watched there too.
    let malformed = [("GRUND_BUILD_COMMIT", "ABCDEF0123")];
    let log = ws.refused(
        "a malformed variable while git qualifies",
        &root,
        &malformed,
    );
    assert!(
        log.contains("GRUND_BUILD_COMMIT"),
        "the refusal names no variable:\n{log}"
    );
    ws.expect(
        "variable unset after the refusal",
        &root,
        &[],
        &one,
        Rerun::Either,
    );
    ws.settled("variable unset after the refusal", &root, &[], &one);
}

#[test]
#[ignore = "red until #1043 the build script names the commit and watches what changes it; implement removes this"]
fn only_paths_that_exist_are_watched() {
    let ws = Workspace::new("existing");
    let root = ws.root.clone();
    assert!(!root.join(".git/packed-refs").exists());
    let one = format!("{DEV}+{}", ws.head(&root));
    ws.expect("no packed-refs", &root, &[], &one, Rerun::Either);
    ws.settled("no packed-refs", &root, &[], &one);

    ws.git(&root, &["checkout", "-q", "-b", "fix/nested"]);
    ws.expect("a nested branch", &root, &[], &one, Rerun::Either);
    ws.settled("a nested branch", &root, &[], &one);
    write(&root.join("docs/notes.md"), "two\n");
    ws.commit_all(&root, "two");
    let two = format!("{DEV}+{}", ws.head(&root));
    ws.expect("commit on the nested branch", &root, &[], &two, Rerun::Must);
    ws.git(&root, &["pack-refs", "--all"]);
    assert!(
        !root.join(".git/refs/heads/fix").exists(),
        "pack-refs left refs/heads/fix/"
    );
    ws.expect("the nested branch packed", &root, &[], &two, Rerun::Either);
    ws.settled("the nested branch packed", &root, &[], &two);
    // An empty commit moves only the ref, which packing left with no loose file.
    ws.git(&root, &["commit", "-q", "--allow-empty", "-m", "three"]);
    let three = ws.head(&root);
    let clean = format!("{DEV}+{three}");
    ws.expect("commit after packing", &root, &[], &clean, Rerun::Must);
    ws.settled("commit after packing", &root, &[], &clean);

    fs::remove_file(root.join("docs/notes.md")).unwrap();
    let dirty = format!("{DEV}+{three}.dirty");
    ws.expect("a deleted tracked file", &root, &[], &dirty, Rerun::Must);
    ws.settled("a deleted tracked file", &root, &[], &dirty);
    ws.git(&root, &["checkout", "--", "docs/notes.md"]);
    ws.expect("the deleted file restored", &root, &[], &clean, Rerun::Must);
    ws.git(&root, &["mv", "docs/notes.md", "docs/moved.md"]);
    ws.expect("a renamed tracked file", &root, &[], &dirty, Rerun::Must);
    ws.settled("a renamed tracked file", &root, &[], &dirty);
}
