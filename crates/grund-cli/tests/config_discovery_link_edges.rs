//! Black-box: the starts and spellings lexical config discovery keeps or moves
//! (§FS-config.1.5). A looping link given as the path starts at its directory
//! (§FS-scan.scope.links.unresolvable.named); a link whose target lookup is refused is
//! placed as far as the disk answers, so it still reaches its project
//! (§FS-scan.scope.links.owed.refused); a path through a directory link inside the root
//! is spelled from the path given (§FS-cli.3.4, §FS-config.3.6.1,
//! §FS-scan.identity.link-path); and a relative path that does not exist still climbs
//! the working directory's parents (§FS-config.1). The refused cases skip where mode 000
//! does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

const V2: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";
const V1: &str = "grund_config_version = 1\n\n[id]\nnamed_sections = true\n";
const UNKNOWN: &str = "# Notes\n\nSee \u{a7}FS-nope.\n";

struct Tree {
    base: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: i32,
    text: String,
}

impl Tree {
    fn new(test: &str) -> Self {
        let base = std::env::temp_dir().join(format!("grund-1326-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        Self {
            base,
            locked: Vec::new(),
        }
    }

    /// A version-2 project at `p`: `docs/a.md` declares and cites `FS-a`, and the hidden
    /// `.r/sub/x.md` cites an unknown `FS-nope`, so only a read through a link into `.r`
    /// finds anything.
    fn project(test: &str) -> Self {
        let tree = Self::new(test);
        tree.write("p/grund.toml", V2);
        tree.write("p/docs/a.md", "# FS-a: A\n\nText, see \u{a7}FS-a.\n");
        tree.write("p/.r/sub/x.md", UNKNOWN);
        tree
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.base.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create the parent");
        fs::write(path, text).expect("write the fixture");
    }

    /// `rel -> target`, the target written relative to the link.
    fn link(&self, rel: &str, target: &str) {
        let path = self.base.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create the parent");
        symlink(target, path).expect("create the link");
    }

    /// Bars `rel` and says whether the mode bars this user; it does not bar root.
    fn bar(&mut self, rel: &str) -> bool {
        let path = self.base.join(rel);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so no lookup is refused");
        }
        barred
    }

    /// `grund <args>` from `cwd`, relative to the base.
    fn run(&self, cwd: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.base.join(cwd))
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            text: format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        }
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.base.join(rel)).expect("read the fixture")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.base);
    }
}

impl Run {
    fn expect(&self, code: i32, line_start: &str, what: &str) {
        assert!(
            self.code == code && self.text.lines().any(|line| line.starts_with(line_start)),
            "{what}: expected exit {code} and a line starting {line_start:?}; got exit {}\n{}",
            self.code,
            self.text
        );
    }
}

/// §FS-scan.scope.links.unresolvable.named, §FS-config.1.5: a looping link given as the
/// path starts discovery at its directory, so under the invocation base the link is
/// `self` and the config `../grund.toml`, the spellings of a start at `docs`.
#[test]
fn a_looping_link_given_as_the_path_starts_at_its_directory() {
    let tree = Tree::project("loop");
    tree.link("p/docs/self", "self");
    let base = "--path-base=invocation";
    tree.run("p", &["check", "docs/self", base]).expect(
        2,
        "error: self: unreadable symlink: filesystem loop",
        "check",
    );
    tree.run("p", &["config", "migrate", "--to", "2", "docs/self", base])
        .expect(
            0,
            "../grund.toml is already version 2; nothing to migrate",
            "migrate",
        );
}

/// §FS-scan.scope.links.owed.refused, §FS-cli.3.4: a link given as the path whose target
/// lookup is refused fails at the link, and the invocation base is the working directory,
/// because a refused lookup cannot tell a file from a directory, so the link is `docs/x`.
#[test]
fn a_refused_link_given_as_the_path_is_spelled_from_the_working_directory() {
    let mut tree = Tree::project("refused-path");
    tree.link("p/docs/x", "../.r/sub");
    if !tree.bar("p/.r") {
        return;
    }
    tree.run("p", &["check", "docs/x", "--path-base=invocation"])
        .expect(
            2,
            "error: docs/x: unreadable symlink: permission denied",
            "check",
        );
}

/// §FS-config.1.5, §FS-scan.scope.links.owed.refused: from outside every project, a link
/// into one whose target lookup is refused is placed as far as the disk answers, so the
/// run takes that project's config and fails at the link rather than passing under the
/// defaults, and `config explain` reads the same project.
#[test]
fn a_refused_link_from_outside_every_project_keeps_the_project_it_reaches() {
    let mut tree = Tree::project("refused-outside");
    tree.link("w/into", "../p/.r/sub");
    if !tree.bar("p/.r") {
        return;
    }
    let check = tree.run("w", &["check", "into"]);
    assert!(
        check.code == 2
            && check.text.lines().any(|line| {
                line.starts_with("error: ")
                    && line.contains("/w/into: unreadable symlink: permission denied")
            })
            && !check.text.contains("nothing to scan"),
        "check fails at the link: exit {}\n{}",
        check.code,
        check.text
    );
    tree.run(
        "w",
        &["config", "explain", "project_name", "--path", "into"],
    )
    .expect(0, "grund.toml is read as version 2.", "explain");
}

/// §FS-cli.3.4, §FS-config.3.6.1, §FS-scan.identity.link-path: a path through a directory
/// link inside the root is spelled from the path given under `--path-base=invocation` and
/// under `relative_paths = false` alike, never from the link's target as `../link/x.md`.
#[test]
fn an_in_root_directory_link_is_spelled_from_the_path_given() {
    let tree = Tree::new("in-root-link");
    tree.write("p/grund.toml", V1);
    tree.write("p/docs/real/x.md", UNKNOWN);
    tree.link("p/docs/link", "real");
    let base = "--path-base=invocation";
    tree.run("p", &["check", "docs/link"]).expect(
        1,
        "docs/link/x.md:3: error:",
        "control, root base",
    );
    for args in [
        ["check", "docs/link", base],
        ["check", "docs/link/x.md", base],
    ] {
        tree.run("p", &args)
            .expect(1, "x.md:3: error:", &args.join(" "));
    }
    tree.write(
        "p/grund.toml",
        &format!("{V1}\n[output]\nrelative_paths = false\n"),
    );
    tree.run("p", &["check", "docs/link"])
        .expect(1, "x.md:3: error:", "relative_paths = false");
}

/// §FS-config.1, §FS-config.1.5: a relative path that does not exist is made absolute
/// against the working directory and climbs its parents, so from a project subdirectory
/// `config explain` reads the project's config and `config migrate` previews it without
/// writing, and `check` under a config that refuses to load prints that refusal.
#[test]
fn a_missing_relative_path_climbs_the_working_directorys_parents() {
    let tree = Tree::new("missing-path");
    let v1 = format!("{V1}\n[scan]\ninclude = [\"docs\"]\n");
    tree.write("v1/grund.toml", &v1);
    tree.write("v1/docs/notes.md", "# Notes\n\nText.\n");
    tree.write("v1/sub/.keep", "");
    tree.write(
        "bad/grund.toml",
        "grund_config_version = 2\n\n[schema]\nnope = 1\n",
    );
    tree.write("bad/sub/.keep", "");
    tree.run(
        "v1/sub",
        &["config", "explain", "project_name", "--path", "missing"],
    )
    .expect(0, "grund.toml is read as version 1.", "explain");
    tree.run("v1/sub", &["config", "migrate", "--to", "2", "missing"])
        .expect(
            0,
            "grund.toml: version 1 → 2, preview, nothing written",
            "migrate",
        );
    assert_eq!(tree.read("v1/grund.toml"), v1, "the preview writes nothing");
    tree.run("bad/sub", &["check", "missing"]).expect(
        2,
        "error: grund.toml:4: unknown key `nope` in [schema]",
        "check",
    );
}
