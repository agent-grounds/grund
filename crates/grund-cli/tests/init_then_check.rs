//! Integration: what `grund init` writes is what `grund check` holds it to
//! (§FS-check.3.5.4, §FS-init.2.3.5.8). After every `init` end-to-end case that
//! writes, `check` reports no `agents-init` finding about a block `init` wrote
//! — a member's fixture block a root run leaves alone is not one — and
//! `init --check` exits 0,
//! and `check`, `check --full` and `check <path>` report the same blocks
//! (§FS-check.1.3.6.1).
#![allow(dead_code)]
#[path = "support/case_runner.rs"]
mod case_runner;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The cases whose one command is a successful write of the whole copy.
fn writing_init_cases(root: &Path) -> Vec<PathBuf> {
    let read = |case: &Path, file: &str| fs::read_to_string(case.join(file)).unwrap_or_default();
    case_runner::discover_e2e_cases(root)
        .into_iter()
        .filter(|case| {
            let name = case.file_name().unwrap().to_string_lossy();
            let args = read(case, "command.args");
            let words: Vec<&str> = args.split_whitespace().collect();
            name.starts_with("init-")
                && words.first() == Some(&"init")
                && words.contains(&"{repo_copy}")
                && !words.iter().any(|w| matches!(*w, "--check" | "--dry-run"))
                && read(case, "expected.exit").trim() == "0"
        })
        .collect()
}

fn grund(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run grund")
}

/// Every `agents-init` finding a `check` run with `args` reports, as its path
/// and `path:line: message`, in report order.
fn agents_init(repo: &Path, args: &[&str]) -> Vec<(String, String)> {
    let out = grund(repo, &[&["check", "--format", "json"], args].concat());
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|f| f["code"] == "agents-init")
        .map(|f| {
            let path = f["path"].as_str().unwrap_or_default().to_string();
            let line = format!(
                "{path}:{}: {}",
                f["line"],
                f["message"].as_str().unwrap_or_default()
            );
            (path, line)
        })
        .collect()
}

/// Whether `init` left the file at `path` as the fixture had it.
fn untouched(case: &Path, repo: &Path, path: &str) -> bool {
    fs::read(case.join("repo").join(path))
        .is_ok_and(|fixture| fs::read(repo.join(path)).is_ok_and(|now| now == fixture))
}

#[test]
fn every_init_case_leaves_blocks_check_holds_current() {
    let root = manifest_dir();
    let cases = writing_init_cases(&root);
    assert!(cases.len() > 40, "the sweep found {} cases", cases.len());
    let mut wrong = Vec::new();
    for case in &cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let Some(run) = case_runner::run_in_copy(&root, case) else {
            continue;
        };
        assert_eq!(run.output.status.code(), Some(0), "{name}: init");
        let repo = run.repo();
        let whole = agents_init(repo, &[]);
        let written: Vec<_> = whole
            .iter()
            .filter(|(path, _)| !untouched(case, repo, path))
            .map(|(_, line)| line)
            .collect();
        if !written.is_empty() {
            wrong.push(format!("{name}: check reports {written:#?}"));
        }
        let path = repo.to_string_lossy().into_owned();
        for (how, args) in [("--full", vec!["--full"]), ("<path>", vec![path.as_str()])] {
            let other = agents_init(repo, &args);
            if other != whole {
                wrong.push(format!(
                    "{name}: check {how} reports {other:#?}, check {whole:#?}"
                ));
            }
        }
        let current = grund(repo, &["init", ".", "--check"]);
        if current.status.code() != Some(0) {
            wrong.push(format!(
                "{name}: init --check exits {:?}: {}",
                current.status.code(),
                String::from_utf8_lossy(&current.stdout)
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
