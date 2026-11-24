//! A version-1 config whose non-citable rows carry citation rules and ground nothing migrates
//! with both gates equal (§FS-config-migrate.5.3). Each such row's Citation directions bullet
//! names the setting that would make an uncited file a finding, spelled in the live config's
//! version (§FS-init.2.3.5), so the block version 1 renders and the candidate's differ in that
//! spelling alone. The rule does not move: the migrated tree still passes `grund check` with a file
//! in the folder home that cites nothing. Issue #1440 is the report: the guidance gate compared
//! the bytes and refused every such config with `repair: none`.
//!
//! The tree holds a folder home (`e2e`, `must`), a single-file home (`runbook`, `should`) and the
//! homeless kind (`code`, `should`), whose bullet names no setting. A several-file home is a
//! version-2 row, so no version-1 config renders one. The trees are built in the temp directory,
//! so run them with a `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The version-1 config; `{grounding}` is a line under each non-citable row, empty for the case.
const VERSION_1: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\
                         named_sections = true\n\n[[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n\n\
                         [[kinds]]\nkind = \"e2e\"\nfolder = \"tests/e2e\"\ncitable = false\n\
                         {grounding}\n[[kinds]]\nkind = \"runbook\"\nfile = \"docs/runbook.md\"\n\
                         citable = false\n{grounding}\n[scan]\n\
                         include = [\"docs\", \"tests\", \"src\"]\n\n[citations.e2e]\n\
                         must = [\"FS\"]\n\n[citations.runbook]\nshould = [\"FS\"]\n\n\
                         [citations.code]\nshould = [\"FS\"]\n";

/// Every file but the config: `FS-login`, and one file in each home that cites it.
const FILES: &[(&str, &str)] = &[
    ("docs/fs.md", "# FS-login: Login\n\nUsers log in.\n"),
    (
        "docs/runbook.md",
        "# Runbook\n\nRestart, as \u{a7}FS-login says.\n",
    ),
    (
        "tests/e2e/login.md",
        "# Login end to end\n\nAs \u{a7}FS-login says.\n",
    ),
    ("src/main.rs", "// \u{a7}FS-login\nfn main() {}\n"),
];

/// The complete report's line when both gates pass (§FS-config-migrate.5.4).
const COMPARED: &str = "\ncompared: the records are equal, and grund check --suggestions on this \
                        tree gives the same findings and the same exit before and after\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// [`VERSION_1`] with `grounding` under each non-citable row, [`FILES`], a git repository,
    /// and the block `grund init` writes, on which `grund check` is clean.
    fn new(name: &str, grounding: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1440-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let config = VERSION_1.replace("{grounding}", grounding);
        for (path, text) in [("grund.toml", config.as_str())].iter().chain(FILES) {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a parent")).expect("create a directory");
            fs::write(path, text).expect("write a file");
        }
        let tree = Self(root);
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&tree.0)
            .status()
            .expect("run git");
        assert!(git.success(), "git init");
        let init = tree.run(&["init"]);
        assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
        let check = tree.run(&["check"]);
        assert_eq!(check.status.code(), Some(0), "{}", streams(&check));
        tree
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

/// The two streams of `output`, for an assertion message.
fn streams(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Asserts that `grund config migrate --to 2`, with `extra` arguments, passes both gates.
fn assert_migrates(tree: &Tree, extra: &[&str]) {
    let mut args = vec!["config", "migrate", "--to", "2"];
    args.extend(extra);
    let output = tree.run(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(COMPARED) && !stderr.contains("refused:") && !stderr.contains("differs:"),
        "{}",
        streams(&output)
    );
    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
}

/// §FS-config-migrate.5.3: the control. With every row grounded the bullets name no setting, so
/// the block is the same in both grammars and the migration passes. This passes before the fix,
/// and shows the tree isolates the grounding sentence.
#[test]
fn config_migrate_passes_grounded_rows() {
    let tree = Tree::new("grounded", "require_grounding = true\n");
    assert_migrates(&tree, &[]);
}

/// §FS-config-migrate.5.3, §FS-config-migrate.5.2: rows that ground nothing migrate with both
/// gates equal, the folder home's `must` and the single-file home's `should` alike. Today the
/// guidance gate refuses the citation directions section with `repair: none`.
#[test]
fn config_migrate_passes_ungrounded_rows() {
    let tree = Tree::new("ungrounded", "");
    assert_migrates(&tree, &[]);
}

/// §FS-config-migrate.5.3, §FS-init.2.3.5: the migrated tree holds the same rule. After
/// `--write` and `grund init`, the block spells the setting in version 2, and `grund check` still
/// passes a file in the folder home that cites nothing. Today `--write` is refused and writes
/// nothing.
#[test]
fn migrated_ungrounded_rows_hold_the_same_rule() {
    let tree = Tree::new("written", "");
    assert_migrates(&tree, &["--write", "--full"]);
    let config = fs::read_to_string(tree.0.join("grund.toml")).expect("read the config");
    assert!(config.starts_with("grund_config_version = 2\n"), "{config}");
    assert!(!config.contains(".grounding]"), "{config}");
    let init = tree.run(&["init"]);
    assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
    let agents = fs::read_to_string(tree.0.join("AGENTS.md")).expect("read AGENTS.md");
    for row in ["e2e", "runbook"] {
        let setting =
            format!("`[rules.citations.{row}.grounding] must = \"file\"` makes it a finding.");
        assert!(agents.contains(&setting), "{agents}");
    }
    assert!(!agents.contains("require_grounding"), "{agents}");
    fs::write(
        tree.0.join("tests/e2e/none.md"),
        "# Cites nothing\n\nText.\n",
    )
    .expect("write");
    let check = tree.run(&["check"]);
    assert_eq!(check.status.code(), Some(0), "{}", streams(&check));
    assert_eq!(
        String::from_utf8_lossy(&check.stdout),
        "success\n",
        "{}",
        streams(&check)
    );
}
