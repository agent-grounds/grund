//! Black-box: which workspace and which member a path through a directory link belongs
//! to (§FS-config.1.5). `grund init` teaches the members of the workspace on the target's
//! own parents, never those of a stranger workspace above the link's target
//! (§FS-init.2.3.4.15.1), and a narrowed run under a member, written so, runs as that
//! member whether or not the member has a config of its own (§FS-scan.projects.discovery).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

const UNKNOWN: &str = "# Notes\n\nSee \u{a7}FS-nope.\n";

struct Tree(PathBuf);

struct Run {
    code: i32,
    text: String,
}

impl Tree {
    fn new(test: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1326-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ws")).expect("create the workspace");
        let init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.join("ws"))
            .output()
            .expect("initialize the workspace repository");
        assert!(init.status.success(), "{init:?}");
        Self(root)
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create the parent");
        fs::write(path, text).expect("write the fixture");
    }

    /// `rel -> target`, the target written relative to the link.
    fn link(&self, rel: &str, target: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create the parent");
        symlink(target, path).expect("create the link");
    }

    /// `grund <args>` from the workspace root.
    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.0.join("ws"))
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
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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

/// §FS-init.2.3.4.15.1, §FS-config.1.5: `grund init m`, with `m` linked out of the
/// workspace `ws` into a stranger workspace that also lists an `m`, finds `ws` on the
/// target's own parents, so the written guidance never lists the stranger's member `q`.
#[test]
fn init_through_a_linked_target_teaches_no_stranger_workspace() {
    let tree = Tree::new("init-stranger");
    let workspace = |name: &str, members: &str| {
        format!(
            "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
             [workspace]\nmembers = [{members}]\n"
        )
    };
    tree.write("ws/grund.toml", &workspace("ws", "\"m\", \"n\""));
    tree.write(
        "ws/n/grund.toml",
        "grund_config_version = 2\nproject_name = \"n\"\n",
    );
    tree.write("outside/grund.toml", &workspace("stranger", "\"m\", \"q\""));
    tree.write(
        "outside/q/grund.toml",
        "grund_config_version = 2\nproject_name = \"q\"\n",
    );
    fs::create_dir_all(tree.0.join("outside/m")).expect("create the target");
    tree.link("ws/m", "../outside/m");
    let init = tree.run(&["init", "m", "--name", "m", "--no-vcs"]);
    assert_eq!(init.code, 0, "{}", init.text);
    let agents = fs::read_to_string(tree.0.join("outside/m/AGENTS.md")).expect("AGENTS.md");
    assert!(
        !agents.contains("[`q`](../q/)"),
        "init taught the stranger workspace's member:\n{agents}"
    );
}

/// §FS-scan.projects.discovery, §FS-config.1.5: a file under member `m`, reached through
/// `m/cases` linked out of the workspace, runs as `m` with or without `m`'s own config,
/// and so is spelled from the member.
#[test]
fn a_path_linked_out_of_the_workspace_runs_as_its_member() {
    for own_config in [false, true] {
        let tree = Tree::new(&format!("member-out-{own_config}"));
        tree.write(
            "ws/grund.toml",
            "project_name = \"ws\"\n\n[workspace]\nmembers = [\"m\"]\n",
        );
        tree.write("ws/m/docs/FS-a.md", "# FS-a: A\n\nText.\n");
        tree.write("outside/cases/b/x.md", UNKNOWN);
        tree.link("ws/m/cases", "../../outside/cases");
        if own_config {
            tree.write("ws/m/grund.toml", "project_name = \"m\"\n");
        }
        tree.run(&["check", "m/cases/b/x.md"]).expect(
            1,
            "cases/b/x.md:3: error:",
            &format!("own config: {own_config}"),
        );
    }
}

/// §FS-scan.projects.discovery: a file reached through `m/x -> ../n/y`, a link from
/// member `m` into member `n`, runs as `m`, the member its path is written in, as it does
/// where `m` has a config of its own; it is spelled `x/x.md`, never `n`'s `y/x.md`.
#[test]
fn a_path_linked_into_another_member_runs_as_the_member_it_is_written_in() {
    for own_config in [false, true] {
        let tree = Tree::new(&format!("member-into-{own_config}"));
        tree.write(
            "ws/grund.toml",
            "project_name = \"ws\"\n\n[workspace]\nmembers = [\"m\", \"n\"]\n",
        );
        tree.write("ws/m/docs/FS-a.md", "# FS-a: A\n\nText.\n");
        tree.write("ws/n/y/x.md", UNKNOWN);
        tree.link("ws/m/x", "../n/y");
        if own_config {
            tree.write("ws/m/grund.toml", "project_name = \"m\"\n");
        }
        tree.run(&["check", "m/x/x.md"]).expect(
            1,
            "x/x.md:3: error:",
            &format!("own config: {own_config}"),
        );
    }
}
