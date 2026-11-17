//! Port of grund.298's explicit/bare reads and controls (§FS-show.1.4.1).
//! Whole-project resolution must survive validating an existing path (§FS-scan.path.show).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// Copy the breadcrumb workspace so controls may create paths without touching the corpus.
struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1422-workspace-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).expect("claim fixture directory");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/e2e/cases/workspace-show-path-breadcrumb/repo");
        for file in [
            "grund.toml",
            "docs/FS-root.md",
            "api/grund.toml",
            "api/docs/FS-001-api.md",
        ] {
            let target = root.join(file);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(fixture.join(file), target).unwrap();
        }
        assert!(!root.join("FS-gone").exists());
        assert!(!root.join("docs/gone.md").exists());
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove workspace fixture");
    }
}

/// Compare every byte of both streams and the exit status, as in the original reproducer.
fn expect(dir: &Path, args: &[&str], code: i32, stdout: &str, stderr: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run grund");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ),
        (Some(code), stdout.to_owned(), stderr.to_owned()),
        "grund {args:?}"
    );
}

const ERROR: &str = "error: path does not exist: FS-gone\n";
const HINT: &str = "hint: `show` reads one coordinate, so `FS-gone` is the path; read both with \
    `printf '%s\\n' '{\"id\":\"FS-root\"}' '{\"id\":\"FS-gone\"}' | grund show --batch";

fn missing(args: &[&str], stderr: &str) {
    let workspace = Workspace::new();
    expect(workspace.path(), args, 2, "", stderr);
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn explicit_coordinate_path_refused_with_hint() {
    missing(
        &["show", "FS-root", "FS-gone"],
        &format!("{ERROR}{HINT}`\n"),
    );
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn bare_coordinate_path_refused_with_hint() {
    missing(&["FS-root", "FS-gone"], &format!("{ERROR}{HINT}`\n"));
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn slash_path_refused_without_hint() {
    missing(
        &["show", "FS-root", "docs/gone.md"],
        "error: path does not exist: docs/gone.md\n",
    );
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn flag_path_refused_without_hint() {
    missing(&["show", "FS-root", "--path", "FS-gone"], ERROR);
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn json_coordinate_path_refused_with_json_batch_hint() {
    missing(
        &["show", "FS-root", "FS-gone", "--format=json"],
        &format!("{ERROR}{HINT} --format=json`\n"),
    );
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn json_bare_coordinate_path_refused_with_json_batch_hint() {
    missing(
        &["FS-root", "FS-gone", "--format=json"],
        &format!("{ERROR}{HINT} --format=json`\n"),
    );
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn json_slash_path_refused_without_hint() {
    missing(
        &["show", "FS-root", "docs/gone.md", "--format=json"],
        "error: path does not exist: docs/gone.md\n",
    );
}

#[test]
#[ignore = "red until #1422 validates missing workspace paths; implement removes this"]
fn json_flag_path_refused_without_hint() {
    missing(
        &["show", "FS-root", "--path", "FS-gone", "--format=json"],
        ERROR,
    );
}

/// A supplied path chooses discovery, never a filter on the declaration (§FS-scan.path.show).
#[test]
fn existing_paths_keep_project_wide_resolution() {
    let workspace = Workspace::new();
    let root = workspace.path();
    std::fs::create_dir(root.join("FS-existing")).unwrap();
    std::fs::create_dir(root.join("docs/narrow")).unwrap();
    for prefix in [&["show", "FS-root"][..], &["FS-root"][..]] {
        expect(root, prefix, 0, "Root lead.\n", "");
        for path in ["FS-existing", "docs/narrow"] {
            let mut positional = prefix.to_vec();
            positional.push(path);
            expect(root, &positional, 0, "Root lead.\n", "");
            let mut flagged = prefix.to_vec();
            flagged.extend(["--path", path]);
            expect(root, &flagged, 0, "Root lead.\n", "");
        }
    }
    expect(root, &["show", "api/FS-001-api"], 0, "Api lead.\n", "");
}

/// Retain the original check/list/member/single controls (§FS-show.1.4.1).
#[test]
fn other_routes_still_refuse_missing_paths() {
    let workspace = Workspace::new();
    let root = workspace.path();
    for command in ["check", "list"] {
        expect(root, &[command, "FS-gone"], 2, "", ERROR);
    }
    expect(
        &root.join("api"),
        &["show", "FS-001-api", "FS-gone"],
        2,
        "",
        ERROR,
    );
    let config = root.join("grund.toml");
    let text = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, text.split("[workspace]").next().unwrap()).unwrap();
    expect(
        root,
        &["show", "FS-root", "FS-gone"],
        2,
        "",
        &format!("{ERROR}{HINT}`\n"),
    );
}
