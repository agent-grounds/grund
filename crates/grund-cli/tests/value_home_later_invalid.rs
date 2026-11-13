//! Black-box regressions for §FS-config-v2.checks.config-value-home-invalid: a version-2 value
//! row's later entry that grund 0.22.0 loaded but that fails a check its first entry is refused
//! for loads with that warning, and the walk reads it as before, so the run keeps 0.22.0's exit
//! and scan errors. These two trees cannot be e2e cases: one's scan error runs past the corpus's
//! stderr cap, and git carries no mode a fixture could keep unsearchable.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

const SCAN_REMEDY: &str = "make the path readable, or keep it out of the scan by removing it \
    from the [schema.kinds] row that lists it, because exclude does not prune scan roots";

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

/// A later `folders` entry that is a file failed 0.22.0's scan with exit 2; it still does, in
/// the same words, and the load adds the warning at the `folders` line instead of refusing.
#[test]
fn later_folder_entry_that_is_a_file_keeps_the_scan_error() {
    let fixture = Fixture::new(
        "folder-is-file",
        "folders = [\"values/one\", \"values/a.json\"]",
    );
    let (code, stderr) = fixture.check();
    assert_eq!(code, Some(2), "{stderr}");
    let scan = format!("error: values/a.json: read value home: not a directory; {SCAN_REMEDY}\n");
    assert!(stderr.contains(&scan), "expected\n{scan}got:\n{stderr}");
    let warning = "warning: grund.toml:6: value home for kind `CONST` must be an existing \
        folder: values/a.json; until then it is read as before grund 0.23.0; make it an existing \
        folder, or drop the entry; this becomes an error in grund 1.0.0\n";
    assert!(
        stderr.contains(warning),
        "expected\n{warning}got:\n{stderr}"
    );
    assert!(
        !stderr.contains("error: grund.toml"),
        "refused at load:\n{stderr}"
    );
}

/// A later `files` entry under a directory this user cannot search failed 0.22.0's scan with
/// exit 2; it still does, and the load warns at the `files` line instead of refusing.
#[test]
fn later_file_entry_that_cannot_be_looked_up_keeps_the_scan_error() {
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
    let scan = "error: values/locked/b.json: could not read value JSON source: permission denied";
    assert!(stderr.contains(scan), "expected\n{scan}\ngot:\n{stderr}");
    let warning = "warning: grund.toml:6: could not look up value home for kind `CONST`: \
        values/locked/b.json: permission denied; until then it is read as before grund 0.23.0; \
        make the home and every directory above it accessible; this becomes an error in grund \
        1.0.0\n";
    assert!(
        stderr.contains(warning),
        "expected\n{warning}got:\n{stderr}"
    );
    assert!(
        !stderr.contains("error: grund.toml"),
        "refused at load:\n{stderr}"
    );
}
