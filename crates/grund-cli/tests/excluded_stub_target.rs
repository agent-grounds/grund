//! A stub whose target git ignores stays healthy and warns, for one release, that
//! grund 0.19.0 reports it as `broken-stub` (§FS-declarations.checks.excluded-stub-target,
//! §FS-declarations.stubs.target.3). The e2e cases cover `[scan] exclude` and `.ignore`;
//! these need a git repository of their own, which a committed fixture cannot carry, for
//! `.gitignore` and `.git/info/exclude` to be ignore files at all.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 1\n\n\
     [reference]\nstrict = true\nrequire_grounding = false\n\n\
     [id]\nformat = \"{kind}-{slug}\"\n\n\
     [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
     [scan]\ninclude = [\"docs\", \"notes\"]\nrespect_gitignore = true\n";

/// The tree of the ticket's example: `docs/fs/x.md` stubs `notes/gen/a.md`, which
/// declares `FS-x`, and `docs/refs.md` cites it so nothing else is reported.
fn tree(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/excluded-stub-target")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    let git = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&root)
        .output()
        .expect("spawn git init");
    assert!(git.status.success(), "git init {}", root.display());
    write(&root, "grund.toml", CONFIG);
    write(&root, "docs/refs.md", "Keeps \u{a7}FS-x live.\n");
    write(&root, "docs/fs/x.md", "# FS-x: [x](../../notes/gen/a.md)\n");
    write(&root, "notes/gen/a.md", "# FS-x: X\n\nLead.\n");
    root
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
    fs::write(path, body).expect("write fixture");
}

fn check(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .args(extra)
        .current_dir(root)
        .output()
        .expect("run grund check")
}

fn warning(ignore_file: &str, pattern: &str) -> String {
    format!(
        "docs/fs/x.md:1: warning: FS-x is a stub to ../../notes/gen/a.md, which the scan does \
         not read because {ignore_file} ignores it by {pattern}, and grund 0.19.0 reports this \
         stub as broken-stub — move the target out of what {ignore_file} ignores, or stop \
         ignoring it there, to keep the stub\n"
    )
}

/// The ticket's example: `notes/gen` is git-ignored, the stub stays healthy and exit `0`,
/// and the one line `check` prints is the warning naming `.gitignore` and its pattern.
#[test]
fn a_git_ignored_target_warns_at_the_stub_and_stays_healthy() {
    let root = tree("gitignore");
    write(&root, ".gitignore", "notes/gen/\n");
    let output = check(&root, &[]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        warning(".gitignore", "notes/gen/"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "the exit status does not move"
    );
    let _ = fs::remove_dir_all(root);
}

/// `.git/info/exclude` is an ignore file too, and is named by its path from the
/// config root.
#[test]
fn a_target_in_git_info_exclude_names_that_file() {
    let root = tree("info-exclude");
    write(&root, ".git/info/exclude", "gen/\n");
    let output = check(&root, &[]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        warning(".git/info/exclude", "gen/"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "the exit status does not move"
    );
    let _ = fs::remove_dir_all(root);
}

/// With `respect_gitignore = false` the ignore files are no rule of the walk, so they
/// are no reason either: the run is silent.
#[test]
fn ignore_files_are_no_reason_when_the_walk_does_not_respect_them() {
    let root = tree("no-respect");
    write(&root, ".gitignore", "notes/gen/\n");
    write(
        &root,
        "grund.toml",
        &CONFIG.replace("respect_gitignore = true", "respect_gitignore = false"),
    );
    let output = check(&root, &[]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "success\n");
    assert_eq!(output.status.code(), Some(0));
    let _ = fs::remove_dir_all(root);
}
