//! Integration: `grund init --dry-run` and `--check` print the report a real run
//! prints on the same tree, each change under its `would-` verb, and the same
//! `next:` block: a preview judges the tree the run would leave (§FS-init.1,
//! §FS-init.2.2.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn grund(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run grund")
}

/// A fresh Git repository holding `files`, under a name of its own.
fn repo(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("grund-init-preview-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    assert!(git.status.success());
    for (path, contents) in files {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    dir
}

/// The preview's report with each `would-` verb put back as the real run's.
fn as_run(preview: &str) -> String {
    preview
        .lines()
        .map(|line| {
            for (would, done) in [
                ("would-write ", "wrote "),
                ("would-append ", "appended "),
                ("would-update ", "updated "),
            ] {
                if let Some(rest) = line.strip_prefix(would) {
                    return format!("{done}{rest}\n");
                }
            }
            format!("{line}\n")
        })
        .collect()
}

fn assert_preview_matches_run(name: &str, files: &[(&str, &str)]) {
    let previewed = repo(&format!("{name}-preview"), files);
    let dry = grund(&previewed, &["init", ".", "--dry-run"]);
    let check = grund(&previewed, &["init", ".", "--check"]);
    let _ = fs::remove_dir_all(&previewed);
    let ran = repo(&format!("{name}-run"), files);
    let run = grund(&ran, &["init", "."]);
    let _ = fs::remove_dir_all(&ran);

    let dry_err = String::from_utf8_lossy(&dry.stderr);
    let run_err = String::from_utf8_lossy(&run.stderr);
    assert_eq!(run.status.code(), Some(0), "{name}: {run_err}");
    assert_eq!(dry.status.code(), Some(0), "{name}: {dry_err}");
    assert_eq!(as_run(&dry_err), run_err, "{name}: the preview's report");
    assert_eq!(check.status.code(), Some(1), "{name}");
    assert_eq!(
        String::from_utf8_lossy(&check.stderr),
        dry_err,
        "{name}: --check prints the --dry-run report"
    );
}

/// The starter scans its config root, so the `AGENTS.md` the run writes is read and
/// neither the run nor its preview says `grund check` has nothing to scan.
#[test]
fn an_empty_repository_previews_the_run() {
    assert_preview_matches_run("empty", &[]);
}

#[test]
fn an_append_previews_the_run() {
    assert_preview_matches_run("append", &[("AGENTS.md", "# My agents\n")]);
}

/// `code` lists its folders, so the root `AGENTS.md` is outside every scope: both
/// the run and its preview keep the clause.
#[test]
fn a_tree_whose_scope_misses_the_written_file_previews_the_run() {
    let config = "grund_config_version = 2\nproject_name = \"p\"\n\n\
                  [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n\n\
                  [schema.kinds.code]\nfolders = [\"src\"]\n";
    assert_preview_matches_run("scoped", &[("grund.toml", config)]);
}
