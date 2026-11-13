//! Black-box: a rewrite that resolves a qualified citation against another project needs that
//! project's proof (§FS-fmt.7.4.4). Where the project's scan met an error, nothing is rewritten and
//! the run names the path, on every route to the rewrite: a narrowed run with a clean walk, a link
//! only an ignore file covers, and a whole run under §FS-fmt.3.1 that goes on to exit 2. A wrong
//! write is the defect, so each refusal reads the file back. The refusal belongs to the rewrite,
//! so a run with none to make behaves as it did. Git carries no mode, so the e2e corpus cannot hold
//! these trees, and every test skips where mode 000 does not bar the user (root).
#![cfg(unix)]

#[path = "support/tmpdir_leftovers.rs"]
mod tmpdir_leftovers;

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

/// `§m/FS-001` names one holder where `m/docs/secret` is barred, and two where it can be read.
const SHORTHAND: &str = "See \u{a7}m/FS-001 here.";

fn config(name: &str, format: &str, shorthand: &str, cross_refs: bool, rest: &str) -> String {
    format!(
        "grund_config_version = 1\nproject_name = \"{name}\"\n\n\
         [reference]\nstrict = true\n{shorthand}\n\
         [id]\nformat = \"{format}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [scan]\ninclude = [\"docs\"]\n\n\
         [fmt.cross_refs]\nenabled = {cross_refs}\n{rest}"
    )
}

struct Tree {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: i32,
    stderr: String,
}

impl Tree {
    /// A root citing `body` in `docs/FS-root.md`, and a member `m` holding `FS-001-login` and, in
    /// `m/docs/secret`, a second holder of number one.
    fn new(test: &str, body: &str, cross_refs: bool) -> Self {
        let root = std::env::temp_dir().join(format!("grund-alias-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the home");
        fs::create_dir_all(root.join("m/docs/secret")).expect("create the member");
        let members = "\n[workspace]\nmembers = [\"m\"]\n";
        let write = |path: &str, text: String| fs::write(root.join(path), text).expect("write");
        write(
            "grund.toml",
            config("root", "{kind}-{slug}", "", cross_refs, members),
        );
        write(
            "m/grund.toml",
            config(
                "m",
                "{kind}-{number}-{slug}",
                "shorthand = \"canonical\"\n",
                cross_refs,
                "",
            ),
        );
        write("docs/FS-root.md", format!("# FS-root: Root\n\n{body}\n"));
        write(
            "m/docs/FS-001-login.md",
            "# FS-001-login: Login\n\nBody.\n".to_string(),
        );
        write(
            "m/docs/secret/FS-001-hidden.md",
            "# FS-001-hidden: Second holder\n\nBody.\n".to_string(),
        );
        Self {
            root,
            locked: Vec::new(),
        }
    }

    /// Bars the member's directory and says whether the mode bars this user.
    fn bar(&mut self) -> bool {
        let path = self.root.join("m/docs/secret");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so nothing here fails to open");
        }
        barred
    }

    fn free(&mut self) {
        for path in self.locked.drain(..) {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("free it");
        }
    }

    fn fmt(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("fmt")
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn body(&self) -> String {
        fs::read_to_string(self.root.join("docs/FS-root.md")).expect("read the file back")
    }

    /// Asserts that `grund fmt <args>` refuses at exit 2, names the member's directory, and leaves
    /// the file byte-identical.
    fn assert_refused(&self, args: &[&str]) {
        self.assert_refused_naming(args, "m/docs/secret: permission denied");
    }

    /// Asserts that `grund fmt <args>` refuses at exit 2 naming `named` first, and leaves the file
    /// byte-identical.
    fn assert_refused_naming(&self, args: &[&str], named: &str) {
        let before = self.body();
        let run = self.fmt(args);
        assert_eq!(run.code, 2, "grund fmt {args:?}:\n{}", run.stderr);
        assert!(
            run.stderr
                .contains(&format!("nothing was rewritten: {named}")),
            "grund fmt {args:?} does not name {named}:\n{}",
            run.stderr
        );
        assert_eq!(
            self.body(),
            before,
            "grund fmt {args:?} wrote through the hole"
        );
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        self.free();
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// No link at all: the walk of `docs` meets nothing, so nothing but the proof stops this write.
#[test]
fn a_narrowed_run_with_no_link_refuses_and_leaves_the_file() {
    let mut tree = Tree::new("no-link", SHORTHAND, true);
    if !tree.bar() {
        return;
    }
    tree.assert_refused(&["--write", "docs"]);
    tree.assert_refused(&["--write", "docs/FS-root.md"]);
    tree.assert_refused(&["--check", "docs"]);
}

/// A full ID and a stale link change no token, so only the link pass can write here, and it asks
/// the same proof before it does (§FS-fmt.7.4.4).
#[test]
fn a_link_the_pass_would_write_into_a_member_with_a_hole_refuses_and_leaves_the_file() {
    for (test, body) in [
        ("full-id", "See \u{a7}m/FS-001-login here."),
        (
            "stale-wrap",
            "See [\u{a7}m/FS-001-login](../m/docs/old.md) here.",
        ),
    ] {
        let mut tree = Tree::new(test, body, true);
        if !tree.bar() {
            return;
        }
        tree.assert_refused(&["--write", "docs"]);
        tree.assert_refused(&["--check", "docs"]);
    }
}

/// The proof asks whether the member's scan met an error, not which: a broken link is a path the
/// walk could not read, and no mode bars anything here, so this one runs as root too (§FS-fmt.7.4.4).
#[test]
fn a_broken_link_in_the_member_refuses_a_narrowed_rewrite() {
    let tree = Tree::new("broken-link", SHORTHAND, true);
    fs::remove_file(tree.root.join("m/docs/secret/FS-001-hidden.md")).expect("drop the holder");
    symlink("nowhere", tree.root.join("m/docs/x.md")).expect("make the link");
    tree.assert_refused_naming(&["--write", "docs"], "m/docs/x.md: broken symlink");
    tree.assert_refused_naming(&["--check", "docs"], "m/docs/x.md: broken symlink");
    fs::remove_file(tree.root.join("m/docs/x.md")).expect("remove the link");
    assert_eq!(tree.fmt(&["--write", "docs"]).code, 0);
    assert!(
        tree.body().contains("\u{a7}m/FS-001-login"),
        "{}",
        tree.body()
    );
}

/// A link out of the member is no hole in its set (§FS-scan.scope.links.owed.fence), so its walk
/// meets no error, the proof stands, and a whole run writes.
#[test]
fn a_link_out_of_the_member_leaves_its_proof_standing() {
    let mut tree = Tree::new("member-outward", SHORTHAND, true);
    fs::remove_file(tree.root.join("m/docs/secret/FS-001-hidden.md")).expect("drop the holder");
    let outside = tree.root.with_extension("outside");
    fs::create_dir_all(&outside).expect("create the target");
    symlink(&outside, tree.root.join("m/docs/ext")).expect("make the link");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o000)).expect("set the mode");
    tree.locked.push(outside.clone());
    if fs::read_dir(&outside).is_ok() {
        return;
    }
    let run = tree.fmt(&["--write"]);
    tree.free();
    fs::remove_dir_all(&outside).expect("remove the target");
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(
        tree.body().contains("\u{a7}m/FS-001-login"),
        "{}",
        tree.body()
    );
}

/// A conflict block changes no recognition, so it leaves the member's set short of nothing: a
/// narrowed rewrite into it lands, and a whole run still refuses on the block (§FS-fmt.7.4.4).
#[test]
fn a_conflict_block_in_the_member_refuses_a_whole_run_and_no_narrowed_rewrite() {
    let tree = Tree::new("conflict", SHORTHAND, true);
    let second_holder = tree.root.join("m/docs/secret/FS-001-hidden.md");
    fs::remove_file(second_holder).expect("remove the second holder");
    fs::write(
        tree.root.join("m/docs/FS-002-c.md"),
        "# FS-002-c: C\n\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> b\n",
    )
    .expect("write the block");
    let before = tree.body();
    let whole = tree.fmt(&["--write"]);
    assert_eq!(whole.code, 2, "{}", whole.stderr);
    assert!(
        whole.stderr.contains("unresolved merge conflict"),
        "{}",
        whole.stderr
    );
    assert_eq!(
        tree.body(),
        before,
        "a whole run wrote past a conflict block"
    );
    let narrowed = tree.fmt(&["--write", "docs"]);
    assert_eq!(narrowed.code, 0, "{}", narrowed.stderr);
    assert!(
        tree.body().contains("[\u{a7}m/FS-001-login]"),
        "the narrowed rewrite did not land:\n{}",
        tree.body()
    );
}

/// With the directory readable, the same citation is ambiguous and there is no rewrite to make, so
/// the bar decides what a wrong write would have said.
#[test]
fn the_same_tree_read_in_full_has_nothing_to_rewrite() {
    let tree = Tree::new("read-in-full", SHORTHAND, true);
    let before = tree.body();
    let run = tree.fmt(&["--write", "docs"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(tree.body(), before);
}

/// A link into the member is reported at the link, on the narrowed run and on the run over one
/// file alike, and no file is written.
#[test]
fn a_narrowed_run_over_a_link_into_the_member_refuses_and_leaves_the_file() {
    let mut tree = Tree::new("direct-link", SHORTHAND, true);
    symlink("../m/docs/secret", tree.root.join("docs/ext")).expect("create the link");
    if !tree.bar() {
        return;
    }
    for scope in ["docs", "docs/FS-root.md"] {
        tree.assert_refused_naming(&["--write", scope], "docs/ext: unreadable symlink");
    }
}

/// Only `.gitignore` covers the link, so no report at the link could have stopped this write.
#[test]
fn a_narrowed_run_over_a_gitignored_link_refuses_and_leaves_the_file() {
    let mut tree = Tree::new("ignored-link", SHORTHAND, true);
    let init = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&tree.root)
        .status()
        .expect("run git");
    assert!(init.success());
    fs::write(tree.root.join(".gitignore"), "docs/ext\n").expect("write the ignore file");
    symlink("../m/docs/secret", tree.root.join("docs/ext")).expect("create the link");
    if !tree.bar() {
        return;
    }
    tree.assert_refused(&["--write", "docs"]);
}

/// Cross-references off: a whole run writes through a scan failure under §FS-fmt.3.1, but not a
/// rewrite whose declaration set has a hole in it. It exited 2 with the wrong write in place.
#[test]
fn a_whole_run_with_cross_refs_off_refuses_and_leaves_the_file() {
    let mut tree = Tree::new("cross-refs-off", SHORTHAND, false);
    if !tree.bar() {
        return;
    }
    tree.assert_refused(&["--write"]);
    tree.assert_refused(&["--write", "docs"]);
}

/// An import's tree is held to the same proof: a whole run exited 0 with the wrong write in place.
#[test]
fn a_rewrite_into_an_import_with_a_hole_refuses_and_leaves_the_file() {
    let base = std::env::temp_dir().join(format!("grund-alias-import-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let mut tree = Tree {
        root: base.join("w"),
        locked: Vec::new(),
    };
    fs::create_dir_all(tree.root.join("docs")).expect("create the home");
    fs::create_dir_all(base.join("lib/docs/secret")).expect("create the import");
    let write = |path: PathBuf, text: String| fs::write(path, text).expect("write");
    write(
        tree.root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"w\"\n\n\
         [schema]\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [schema.kinds.FS]\nfolders = [\"docs\"]\n\n\
         [workspace.imports.lib]\npath = \"../lib\"\nproject = \"lib\"\n"
            .to_string(),
    );
    write(
        base.join("lib/grund.toml"),
        config(
            "lib",
            "{kind}-{number}-{slug}",
            "shorthand = \"canonical\"\n",
            true,
            "",
        ),
    );
    write(
        tree.root.join("docs/FS-root.md"),
        "# FS-root: Root\n\nSee \u{a7}lib/FS-001 here.\n".to_string(),
    );
    write(
        base.join("lib/docs/FS-001-login.md"),
        "# FS-001-login: Login\n\nBody.\n".to_string(),
    );
    let secret = base.join("lib/docs/secret");
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).expect("set the mode");
    tree.locked.push(secret.clone());
    if fs::read_dir(&secret).is_ok() {
        return;
    }
    let before = tree.body();
    let run = tree.fmt(&["--write"]);
    tree.free();
    let _keep = fs::remove_dir_all(base.join("lib"));
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("nothing was rewritten: ../lib/docs/secret: permission denied"),
        "{}",
        run.stderr
    );
    assert_eq!(tree.body(), before);
}

/// Asserts a narrowed run behaves as it did where its rewrites touch no failed project, or the
/// citation already reads what the rewrite would write.
fn assert_untouched(test: &str, body: &str) {
    let mut tree = Tree::new(test, body, true);
    if !tree.bar() {
        return;
    }
    let before = tree.body();
    let run = tree.fmt(&["--write", "docs"]);
    assert_eq!(run.code, 0, "{body}:\n{}", run.stderr);
    assert_eq!(tree.body(), before, "{body} was rewritten");
}

/// A link the project's own hand already wrote right is no rewrite, so it needs no proof.
#[test]
fn a_wrap_already_right_is_no_rewrite_and_is_not_refused() {
    assert_untouched(
        "already-right",
        "See [\u{a7}m/FS-001-login](../m/docs/FS-001-login.md#fs-001-login-login) here.",
    );
}

/// A never-rewrite zone holds no rewrite either.
#[test]
fn a_citation_in_inline_code_is_not_refused() {
    assert_untouched("inline-code", "See `\u{a7}m/FS-001` in code.");
}

/// Nothing visible holds the number, so there is no rewrite to refuse.
#[test]
fn a_shorthand_no_visible_declaration_holds_is_not_refused() {
    assert_untouched("no-holder", "See \u{a7}m/FS-009 here.");
}

/// A rewrite of the project's own citations touches no failed project, and it still lands.
#[test]
fn a_rewrite_that_names_no_failed_project_still_lands() {
    let mut tree = Tree::new("own-citation", "See \u{a7}FS-root here.", true);
    if !tree.bar() {
        return;
    }
    let run = tree.fmt(&["--write", "docs"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(
        tree.body().contains("[\u{a7}FS-root]"),
        "the project's own citation was not wrapped:\n{}",
        tree.body()
    );
}

/// The refusal is not a report of the unreadable path by itself: freeing the directory ends it.
#[test]
fn making_the_directory_readable_ends_the_refusal() {
    let mut tree = Tree::new("freed", SHORTHAND, true);
    if !tree.bar() {
        return;
    }
    tree.assert_refused(&["--write", "docs"]);
    tree.free();
    let run = tree.fmt(&["--write", "docs"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// A passing import test leaves nothing in its temporary directory: the fixture removes the
/// whole base it built, `lib` beside the scanned `w` included, and not only the tree it scans
/// (§AR-ci-fixture-cleanup).
#[test]
#[ignore = "red until #1392 removes the fixture's root; implement removes this"]
fn the_import_test_leaves_nothing_in_its_temporary_directory() {
    let left = tmpdir_leftovers::left_by(&[
        "a_rewrite_into_an_import_with_a_hole_refuses_and_leaves_the_file",
    ]);
    assert!(
        left.is_empty(),
        "the import test passed and left {left:?} in its temporary directory"
    );
}
