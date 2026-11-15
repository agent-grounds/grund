//! Black-box regressions for §FS-config-v2.checks.config-value-home-invalid: a version-2 value
//! row's later entry that fails a check its first entry is refused for is refused at load from
//! grund 1.0.0, at the line that lists it, before any walk, so 0.22.0's scan error is never
//! reached. These two trees cannot be e2e cases: git carries no mode a fixture could keep
//! unsearchable, and the pair is kept together.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    root: PathBuf,
    locked: Option<PathBuf>,
}

impl Fixture {
    /// A version-2 project whose `CONST` value row lists `places`, its first home
    /// `values/a.json` or `values/one/c.json` declaring `CONST-a`, cited from the README.
    fn new(name: &str, places: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1206-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("values/one")).unwrap();
        fs::write(
            root.join("grund.toml"),
            format!(
                "grund_config_version = 2\nproject_name = \"g1206\"\n\n[schema.kinds.CONST]\n\
                 form = \"value\"\n{places}\n"
            ),
        )
        .unwrap();
        fs::write(root.join("values/a.json"), "{\"CONST-a\": [\"one\"]}\n").unwrap();
        fs::write(root.join("values/one/c.json"), "{\"CONST-a\": [\"one\"]}\n").unwrap();
        fs::write(root.join("README.md"), "Cites §CONST-a.\n").unwrap();
        Self { root, locked: None }
    }

    fn check(&self) -> (Option<i32>, String) {
        let Output { status, stderr, .. } = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        (status.code(), String::from_utf8(stderr).unwrap())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(path) = &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A later `folders` entry that is a file is refused at the `folders` line, in one line with no
/// `hint:`, and the scan that failed in 0.22.0 never runs.
#[test]
fn later_folder_entry_that_is_a_file_is_refused_at_load() {
    let fixture = Fixture::new(
        "folder-is-file",
        "folders = [\"values/one\", \"values/a.json\"]",
    );
    let (code, stderr) = fixture.check();
    assert_eq!(code, Some(2), "{stderr}");
    let refusal = "error: grund.toml:6: value home for kind `CONST` must be an existing \
        folder: values/a.json; make it an existing folder, or drop the entry; this became an \
        error in grund 1.0.0\n";
    assert_eq!(stderr, refusal);
}

/// A later `files` entry under a directory this user cannot search is refused at the `files`
/// line by its reason, never as absent (§FS-fetch.4.3), in one line with no `hint:`.
#[test]
fn later_file_entry_that_cannot_be_looked_up_is_refused_at_load() {
    let mut fixture = Fixture::new(
        "lookup",
        "files = [\"values/a.json\", \"values/locked/b.json\"]",
    );
    let locked = fixture.root.join("values/locked");
    fs::create_dir_all(&locked).unwrap();
    fs::write(locked.join("b.json"), "{\"CONST-b\": [\"two\"]}\n").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    fixture.locked = Some(locked.clone());
    if fs::canonicalize(locked.join("b.json")).is_ok() {
        eprintln!("skipping: the mode does not bar this user");
        return;
    }
    let (code, stderr) = fixture.check();
    assert_eq!(code, Some(2), "{stderr}");
    let refusal = "error: grund.toml:6: could not look up value home for kind `CONST`: \
        values/locked/b.json: permission denied; make the home and every directory above it \
        accessible; this became an error in grund 1.0.0\n";
    assert_eq!(stderr, refusal);
}
