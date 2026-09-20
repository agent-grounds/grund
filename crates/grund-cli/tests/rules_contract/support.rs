//! Test-only repository and process scaffold for §FS-rules. It deliberately
//! knows no production parser, fact, scanner, or diagnostic type.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn fixture() -> PathBuf {
    repo_root().join("tests/e2e/cases/check-rules-chapter-scope/repo")
}

pub fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund")
}

pub fn text(output: &[u8]) -> String {
    String::from_utf8(output.to_vec()).expect("UTF-8 command output")
}

pub fn assert_run(output: &Output, exit: i32, stdout: &str, stderr: &str) {
    assert_eq!(output.status.code(), Some(exit), "unexpected exit");
    assert_eq!(text(&output.stdout), stdout, "unexpected stdout");
    assert_eq!(text(&output.stderr), stderr, "unexpected stderr");
}

pub fn scratch(name: &str) -> PathBuf {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = repo_root()
        .join("target/rules-contract-work")
        .join(format!("{name}-{}-{serial}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale test repository");
    }
    copy_tree(&fixture(), &root);
    root
}

pub fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, contents).expect("write fixture file");
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create fixture copy");
    for entry in fs::read_dir(source).expect("read fixture") {
        let entry = entry.expect("fixture entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(source_path, target_path).expect("copy fixture file");
        }
    }
}
