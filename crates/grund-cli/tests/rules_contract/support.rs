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

/// `assert_run` over a version 1 fixture, comparing stderr without the
/// `config-version-1` caution: that is a run-level line about the config
/// (§FS-check.checks.config-version-1), and these fixtures are version 1 because
/// the e2e cases sharing them are, not because the version is the test's subject.
pub fn assert_v1_run(output: &Output, exit: i32, stdout: &str, stderr: &str) {
    assert_eq!(output.status.code(), Some(exit), "unexpected exit");
    assert_eq!(text(&output.stdout), stdout, "unexpected stdout");
    assert_eq!(
        without_caution(&text(&output.stderr)),
        stderr,
        "unexpected stderr"
    );
}

/// Stderr with its `config-version-1` lines, text or JSON, dropped.
pub fn without_caution(stderr: &str) -> String {
    stderr
        .split_inclusive('\n')
        .filter(|line| {
            !line.contains("\"code\":\"config-version-1\"")
                && !line.contains(" is version 1; migrate it with grund config migrate --to 2")
        })
        .collect()
}

pub fn scratch(name: &str) -> PathBuf {
    scratch_from(&fixture(), name)
}

/// A writable copy of any fixture tree in this repository, so a contract about
/// more than one project can use a workspace fixture without a second harness.
pub fn scratch_from(source: &Path, name: &str) -> PathBuf {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = repo_root()
        .join("target/rules-contract-work")
        .join(format!("{name}-{}-{serial}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale test repository");
    }
    copy_tree(source, &root);
    root
}

/// The `repo/` tree of an e2e case, which is where this repository keeps the
/// fixtures a case and a contract test share.
pub fn case_repo(case: &str) -> PathBuf {
    repo_root().join("tests/e2e/cases").join(case).join("repo")
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

/// A scratch fixture whose `FS-demo` has lost its `requirements` chapter, so a
/// chapter-scoped subject over it selects nothing.
pub fn absent_chapter(name: &str) -> PathBuf {
    let root = unconfigured_rules(name);
    let declaration = root.join("docs/fs/FS-demo.md");
    let body = fs::read_to_string(&declaration).expect("fixture declaration");
    let chapter = body
        .find("## requirements: Requirements")
        .expect("fixture requirements chapter");
    fs::write(&declaration, format!("{}\n", body[..chapter].trim_end()))
        .expect("delete the requirements chapter");
    root
}

/// A scratch fixture whose configured rules are off, so a `--rule` run reports
/// that sentence and nothing else.
pub fn unconfigured_rules(name: &str) -> PathBuf {
    let root = scratch(name);
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("rules = true\n", ""),
    )
    .expect("disable configured rules");
    root
}

/// The fixture's config as version 2 without a `form = "rule"` kind, so a clean
/// run's stdout is `success` rather than the version 1 caution
/// (§FS-check.checks.config-version-1).
pub const NO_RULE_FORM_V2: &str = r#"grund_config_version = 2

[schema]
id_format = "{kind}-{slug}"

[schema.kinds.GOAL]
folders = ["docs/goals"]
index = false

[schema.kinds.REQ]
folders = ["docs/req"]
index = false

[schema.kinds.FS]
folders = ["docs/fs"]
index = false

[schema.kinds.AR]
folders = ["docs/ar"]
index = false

[schema.kinds.RULE]
folders = ["docs/rules"]
index = false

[schema.kinds.code]
folders = ["docs"]

[rules.citations.RULE]
must = ["GOAL"]

[presentation]
rules = "root"
"#;
