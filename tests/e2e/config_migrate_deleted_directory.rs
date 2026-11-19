//! Issue #1457, ported from tool report grund.312: §FS-config-migrate.6.5.1
//! item 1 retains discovery's spelling without a working directory. These are
//! characterization tests; the defect is the documentation's absolute promise.

#[path = "../../crates/grund-cli/tests/support/temporary_root.rs"]
mod temporary_root;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use temporary_root::TemporaryRoot;

const UNDO: &str = "to get version 1 back: it is in version control, e.g. ";
const REPREVIEW: &str = "to see this candidate again: restore version 1 and run ";
const V1: &str = "grund_config_version = 1\n\n\
    [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
    [scan]\ninclude = [\"docs\"]\n";

struct Workspace(TemporaryRoot);

impl Workspace {
    fn new(name: &str) -> Self {
        let tree = Self(TemporaryRoot::new(&std::env::temp_dir(), name).physical());
        for dir in ["", "apps/api/"] {
            fs::create_dir_all(tree.0.join(format!("{dir}docs/fs"))).unwrap();
            fs::write(
                tree.0.join(format!("{dir}docs/fs/FS-demo.md")),
                "# FS-demo: Demo\n\nThe project.\n",
            )
            .unwrap();
            fs::write(tree.0.join(format!("{dir}grund.toml")), V1).unwrap();
        }
        fs::write(
            tree.0.join("grund.toml"),
            format!("{V1}\n[workspace]\nmembers = [\"apps/api\"]\n"),
        )
        .unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["add", "-A"],
            vec!["commit", "-qm", "version 1 root and member"],
        ] {
            succeeded(&tree.git().args(args).output().unwrap());
        }
        succeeded(
            &Command::new(env!("CARGO_BIN_EXE_grund"))
                .args(["config", "migrate", "--to", "2", "--write"])
                .current_dir(&*tree.0)
                .output()
                .unwrap(),
        );
        tree
    }

    fn git(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(&*self.0)
            .env("GIT_CONFIG_GLOBAL", self.0.join(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "user.name=grund",
                "-c",
                "user.email=grund@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"]);
        cmd
    }

    /// Only the child loses its working directory; parallel tests keep theirs.
    fn deleted(&self, program: &str, args: &[&str]) -> Output {
        let gone = self.0.join("gone");
        fs::create_dir(&gone).unwrap();
        Command::new("sh")
            .args([
                "-c",
                "rmdir \"$1\" || exit 97; shift; exec \"$@\"",
                "deleted-cwd",
            ])
            .arg(&gone)
            .arg(program)
            .args(args)
            .current_dir(&gone)
            .env("GIT_CONFIG_GLOBAL", self.0.join(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap()
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

fn succeeded(out: &Output) {
    assert!(out.status.success(), "{out:?}");
}

fn printed(stderr: &str, prefix: &str) -> String {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("no {prefix:?} in {stderr}"))
        .to_owned()
}

/// §FS-config-migrate.6.5.1 items 1-2: a reachable relative PATH stays relative.
/// The hint fails in the deleted directory and at the root, changing neither
/// config. It does not promise an undo that can be pasted at another root.
#[test]
fn deleted_directory_relative_path_retains_discovery_spelling() {
    let tree = Workspace::new("grund-1457-relative");
    let root = tree.read("grund.toml");
    let out = tree.deleted(
        env!("CARGO_BIN_EXE_grund"),
        &["config", "migrate", "--to", "2", "../apps/api", "--write"],
    );
    succeeded(&out);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let undo = printed(&stderr, UNDO);
    assert_eq!(undo, "git restore ../apps/api/grund.toml");
    assert_eq!(
        printed(&stderr, REPREVIEW),
        "grund config migrate --to 2 ../apps/api"
    );
    let member = tree.read("apps/api/grund.toml");
    assert!(root.starts_with("grund_config_version = 2\n"));
    assert!(member.starts_with("grund_config_version = 2\n"));
    assert_eq!(tree.read("grund.toml"), root);
    for deleted in [true, false] {
        let restore = if deleted {
            tree.deleted("sh", &["-c", &undo])
        } else {
            tree.git()
                .args(["restore", "../apps/api/grund.toml"])
                .output()
                .unwrap()
        };
        assert_eq!(restore.status.code(), Some(128), "{restore:?}");
        assert_eq!(tree.read("grund.toml"), root);
        assert_eq!(tree.read("apps/api/grund.toml"), member);
    }
}

/// §FS-config-migrate.6.5.1 item 1: an absolute PATH keeps the absolute control;
/// its printed undo restores only the member from an existing repository root.
#[test]
fn deleted_directory_absolute_path_restores_only_the_member() {
    let tree = Workspace::new("grund-1457-absolute");
    let root = tree.read("grund.toml");
    let member = tree.read("apps/api/grund.toml");
    let path = tree.0.join("apps/api");
    let out = tree.deleted(
        env!("CARGO_BIN_EXE_grund"),
        &[
            "config",
            "migrate",
            "--to",
            "2",
            path.to_str().unwrap(),
            "--write",
        ],
    );
    succeeded(&out);
    assert!(
        tree.read("apps/api/grund.toml")
            .starts_with("grund_config_version = 2\n")
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    let undo = printed(&stderr, UNDO);
    let file = undo.strip_prefix("git restore ").unwrap();
    assert!(Path::new(file).is_absolute(), "{undo}");
    assert_eq!(Path::new(file), path.join("grund.toml"));
    assert_eq!(
        printed(&stderr, REPREVIEW),
        format!("grund config migrate --to 2 {}", path.display())
    );
    // Paste the printed command itself, rather than reconstructing its path.
    succeeded(
        &Command::new("sh")
            .args(["-c", &undo])
            .current_dir(&*tree.0)
            .env("GIT_CONFIG_GLOBAL", tree.0.join(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap(),
    );
    assert_eq!(tree.read("grund.toml"), root);
    assert_eq!(tree.read("apps/api/grund.toml"), member);
}
