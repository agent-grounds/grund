//! Black-box: `grund init <path>` calls its target missing only where the lookup says so
//! (§FS-init.1). Behind a directory the user cannot search the lookup is refused, which says
//! nothing about the target (§FS-scan.scope.homes.lookup.path), so `init` refuses it at the path as
//! given with `permission denied`, exit `2`, an empty stdout and nothing written, and never says
//! `target directory does not exist`. The control keeps that line for a target that is not there.
//! The barred case skips where mode 000 does not bar the user (root).
#![cfg(unix)]

#[path = "../../crates/grund-cli/tests/support/fixture_cleanup.rs"]
mod fixture_cleanup;

use fixture_cleanup::remove_fixture_tree;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `<base>` holding `.r/sub`, with `.r` at mode 000 while the value lives.
struct Barred {
    base: PathBuf,
}

impl Drop for Barred {
    fn drop(&mut self) {
        // The mode first: a barred directory cannot be emptied.
        let _ = fs::set_permissions(self.base.join(".r"), fs::Permissions::from_mode(0o755));
        remove_fixture_tree(&self.base, "remove fixture");
    }
}

fn init(base: &Path, target: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["init", target])
        .env("NO_COLOR", "1")
        .current_dir(base)
        .output()
        .expect("run grund init")
}

/// Every entry under `dir`, recursively, as paths relative to it.
fn entries(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).expect("list the fixture").flatten() {
        let path = entry.path();
        found.push(
            path.strip_prefix(dir)
                .expect("under the fixture")
                .to_path_buf(),
        );
        if path.is_dir() {
            found.extend(entries(&path).into_iter().map(|rest| {
                path.strip_prefix(dir)
                    .expect("under the fixture")
                    .join(rest)
            }));
        }
    }
    found.sort();
    found
}

#[test]
fn a_target_behind_a_barred_directory_is_a_permission_failure_not_a_missing_one() {
    let base = std::env::temp_dir().join(format!("grund-1354-init-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join(".r/sub")).expect("create the target");
    let barred = Barred { base };
    let before = entries(&barred.base);
    let parent = barred.base.join(".r");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).expect("set the mode");
    if fs::metadata(parent.join("sub")).is_ok() {
        eprintln!("skipped: the mode does not bar this user, so no lookup is refused");
        return;
    }
    let output = init(&barred.base, ".r/sub");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).expect("restore the mode");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        output.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        stderr, "error: .r/sub: permission denied; make the path readable\n",
        "never `target directory does not exist`"
    );
    assert_eq!(
        entries(&barred.base),
        before,
        "a refused init writes nothing"
    );
}

/// Control: a target that is not there keeps the missing-target line.
#[test]
fn a_target_that_is_not_there_does_not_exist() {
    let base = std::env::temp_dir().join(format!("grund-1354-init-absent-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("create the base");
    let barred = Barred { base };
    let output = init(&barred.base, "nope/sub");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: target directory does not exist: nope/sub\n"
    );
}
