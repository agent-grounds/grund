//! Black-box: `grund id` refuses an incomplete scan at exit `2` with the line `check` prints for
//! the same tree, the reason followed by `; ` and `io`'s Fix (§FS-id.4, §FS-check.2.4,
//! §FS-workspace.8.7.3). Git carries no mode a fixture could keep unreadable, so the issue's
//! unreadable directory is built here; the malformed UTF-8 file is the same refusal for a user
//! the mode does not bar. Only the suffix is asserted: how a scoped run under
//! `--path-base=invocation` spells the path is a separate defect, #1178.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.SPEC]\nfolders = [\"docs/open\"]\n";

const FIX: &str = "; make the path readable, or keep it out of the scan\n";

/// The issue's tree: one readable declaration and `docs/locked`, which this user may not list.
struct Fixture {
    root: PathBuf,
    locked: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1189-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/open")).expect("create the kind home");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join("docs/open/a.md"), "# SPEC-a: A\n").expect("write the declaration");
        let locked = root.join("docs/locked");
        fs::create_dir_all(&locked).expect("create the locked directory");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("take its mode");
        Self { root, locked }
    }

    /// Whether the mode bars this process: it does not bar root, which proves nothing here.
    fn is_unreadable(&self) -> bool {
        fs::read_dir(&self.locked).is_err()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.locked, fs::Permissions::from_mode(0o755));
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn ask(root: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund");
    let code = output.status.code().expect("exited with a status");
    (code, String::from_utf8_lossy(&output.stderr).into_owned())
}

/// One `error:` line ending in io's Fix, the ending `check` gives the same failure.
fn assert_refused_with_the_fix(root: &Path, args: &[&str]) {
    let (code, stderr) = ask(root, args);
    assert_eq!(code, 2, "grund {args:?}: {stderr}");
    assert!(
        stderr.starts_with("error: ") && stderr.ends_with(FIX) && stderr.lines().count() == 1,
        "grund {args:?} printed {stderr:?}, not one error line ending {FIX:?}"
    );
}

#[test]
fn id_refuses_an_unreadable_directory_with_the_fix_check_prints() {
    let fixture = Fixture::new("locked");
    if !fixture.is_unreadable() {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    // The control: `check` already prints the Fix on this tree.
    assert_refused_with_the_fix(&fixture.root, &["check"]);
    for args in [
        &["id", "SPEC", "new thing"][..],
        &["id", "SPEC", "new thing", "docs"],
        &["id", "SPEC", "new thing", "."],
        &["id", "SPEC", "new thing", "docs/locked"],
        &["id", "SPEC", "new thing", "--path-base=invocation"],
        &[
            "id",
            "SPEC",
            "new thing",
            "docs/locked",
            "--path-base=invocation",
        ],
    ] {
        assert_refused_with_the_fix(&fixture.root, args);
    }
}

/// The same refusal for a file the walk opens and cannot decode, which no user's mode can read.
#[test]
fn id_refuses_an_undecodable_file_with_the_fix_check_prints() {
    let root = std::env::temp_dir().join(format!("grund-1189-utf8-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/open")).expect("create the kind home");
    fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
    fs::write(root.join("docs/open/a.md"), "# SPEC-a: A\n").expect("write the declaration");
    fs::write(root.join("docs/open/bad.md"), b"# SPEC-b: \xff\n").expect("write the bad file");
    let (_, check) = ask(&root, &["check"]);
    assert!(
        check.contains("docs/open/bad.md: ") && check.contains(FIX.trim_end()),
        "{check}"
    );
    let (code, id) = ask(&root, &["id", "SPEC", "new thing"]);
    let _ = fs::remove_dir_all(&root);
    let line = check
        .lines()
        .find(|line| line.starts_with("error: docs/open/bad.md: "))
        .expect("check names the file");
    assert_eq!(
        (code, id),
        (2, format!("{line}\n")),
        "grund id prints check's line"
    );
}
