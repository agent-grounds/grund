//! §FS-config-migrate.3.1.7.2: the rewrite the repair of a value row names keeps its values
//! compared across the migration, on a row without `fetch`, and adds no generated section.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const MISMATCH: &str = "value mismatch for AR-001-power.1.1: bound `48`, declared `24`";

/// The marker and the citation are spelled apart, so this file's own scan reads neither.
fn rewritten() -> String {
    let marked = concat!("<!-- grund", ":value -->");
    let sign = '\u{a7}';
    format!(
        "# AR-001-power: Auxiliary power\n\n## 1. Auxiliary supply voltage {marked}\n\n\
         ### 1.1. 24\n### 1.2. V\n\n## 2. Use\n\nMarked: `48` ({sign}AR-001-power.1.1)\n"
    )
}

struct Tree(PathBuf);

impl Tree {
    fn new(config: &str, arch: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1232-rewrite-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs")).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        let tree = Self(root);
        tree.write(config, arch);
        tree
    }

    fn write(&self, config: &str, arch: &str) {
        fs::write(self.0.join("grund.toml"), config).unwrap();
        fs::write(self.0.join("docs/arch.md"), arch).unwrap();
    }

    fn grund(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn agents(&self) -> String {
        fs::read_to_string(Path::new(&self.0).join("AGENTS.md")).unwrap_or_default()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn config(key: &str) -> String {
    format!(
        "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n[[kinds]]\nkind = \"AR\"\n\
         file = \"docs/arch.md\"\n{key}\n[scan]\ninclude = [\"docs\"]\n"
    )
}

fn assert_mismatch(tree: &Tree, when: &str) {
    let out = tree.grund(&["check"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{when}: {out:?}");
    assert!(text.contains(MISMATCH), "{when}: {text}");
}

/// Refused as it stands; once rewritten, the wrong literal fails on both sides of the write.
fn assert_rewrite_keeps_comparing(key: &str, arch: &str, refused_at: &str) {
    let tree = Tree::new(&config(key), arch);
    let refused = tree.grund(&["config", "migrate", "--to", "2"]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(stderr.contains(refused_at), "{stderr}");

    tree.write(&config(""), &rewritten());
    assert_mismatch(&tree, "version 1, rewritten");
    let wrote = tree.grund(&["config", "migrate", "--to", "2", "--write"]);
    assert_eq!(wrote.status.code(), Some(0), "{wrote:?}");
    let init = tree.grund(&["init"]);
    assert_eq!(init.status.code(), Some(0), "{init:?}");
    let agents = tree.agents();
    assert!(agents.contains("grund"), "init wrote no block: {agents}");
    assert!(!agents.contains("### Declaration shapes"), "{agents}");
    assert_mismatch(&tree, "version 2, migrated");
}

#[test]
fn value_chapter_row_rewrite_keeps_comparing_across_the_migration() {
    assert_rewrite_keeps_comparing(
        "value_chapter = \"values\"\n",
        "# AR-001-power: Auxiliary power\n\n## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n#### values.aux-voltage.2: V\n\n## 2. Use\n\n\
         Chapter: `48` (\u{a7}AR-001-power.values.aux-voltage.1)\n",
        "value_chapter would become a field",
    );
}

#[test]
fn values_row_rewrite_keeps_comparing_across_the_migration() {
    assert_rewrite_keeps_comparing(
        "values = true\n",
        "# AR-001-power: Auxiliary supply voltage\n\n## 1. 24\n## 2. V\n",
        "values = true adds a \"Declaration shapes\" section",
    );
}
