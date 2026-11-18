//! A `grund.toml` that starts with a byte-order mark is refused at line 1 by name, before its
//! version is read (§FS-config.5): every command that loads the project exits 2 and
//! `config validate` exits 1 (§FS-config.4.3), each with the same line. The mark is written at
//! run time, so no fixture carries an invisible byte.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const REFUSAL: &str =
    "error: grund.toml:1: the file starts with a byte-order mark; save it as UTF-8 without one";

struct Tree(PathBuf);

impl Tree {
    fn new(test: &str, config: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grund-bom-config-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/functional-spec")).expect("create the kind folder");
        fs::write(root.join("grund.toml"), format!("\u{feff}{config}")).expect("write the config");
        Self(root)
    }

    fn run(&self, args: &[&str]) -> (Option<i32>, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
                .trim_end()
                .to_string(),
        )
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const V2: &str =
    "grund_config_version = 2\n\n[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n";

#[test]
fn a_byte_order_mark_before_the_version_is_named_by_check_and_validate() {
    let tree = Tree::new("v2", V2);
    assert_eq!(tree.run(&["check"]), (Some(2), REFUSAL.to_string()));
    assert_eq!(
        tree.run(&["config", "validate"]),
        (Some(1), REFUSAL.to_string())
    );
}

#[test]
fn a_byte_order_mark_before_a_comment_is_named_too() {
    let tree = Tree::new("comment", &format!("# The project's config.\n{V2}"));
    assert_eq!(tree.run(&["check"]), (Some(2), REFUSAL.to_string()));
}

#[test]
fn a_byte_order_mark_is_named_whatever_version_the_file_writes() {
    let tree = Tree::new(
        "v1",
        "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n",
    );
    assert_eq!(tree.run(&["check"]), (Some(2), REFUSAL.to_string()));
}

/// §FS-config-explain.2: a mark is one reason a file's version cannot be read.
#[test]
fn config_explain_names_a_byte_order_mark() {
    let tree = Tree::new("explain", V2);
    assert_eq!(
        tree.run(&["config", "explain", "oversized-lead"]),
        (Some(2), REFUSAL.to_string())
    );
}
