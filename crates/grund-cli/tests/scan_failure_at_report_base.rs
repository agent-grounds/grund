//! Black-box: a scan failure at the directory that is the report base names that directory `.`
//! (§FS-cli.3.4, §FS-check.2.4), in the text line and in JSON `path`, from every command that
//! meets it (§FS-workspace.8.7.3). Git carries no mode a fixture could keep unreadable, so the
//! e2e corpus cannot hold this; the tests skip where the mode does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.SPEC]\nfolders = [\"docs/open\"]\n";

/// The config root is no path handed in, so its fix is `readable` (§FS-check.2.4.3).
const AT_BASE: &str = "error: .: permission denied; make the path readable";

/// `docs/locked` is the path the command was given, so its fix is `passed` (§FS-check.2.4.3).
const PASSED_AT_BASE: &str = "error: .: permission denied; make the path readable, or keep it out \
    of the scan by not passing it, because exclude does not prune a path the command is given";

const JSON_AT_BASE: &str = "{\"severity\":\"error\",\"path\":\".\",\"line\":null,\"code\":\"io\",\
    \"message\":\"permission denied; make the path readable, or keep it out of the scan by not \
    passing it, because exclude does not prune a path the command is given\",\
    \"sites\":null,\"authority\":null}";

/// A v2 project with one readable kind home, `docs/open`, and a directory beside it,
/// `docs/locked`, that this user may not list.
struct Fixture {
    root: PathBuf,
    locked: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1178-{name}-{}", std::process::id()));
        let _ = fs::set_permissions(&root, fs::Permissions::from_mode(0o755));
        let _ = fs::set_permissions(root.join("docs/locked"), fs::Permissions::from_mode(0o755));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/open")).expect("create the kind home");
        fs::create_dir_all(root.join("docs/locked")).expect("create the locked directory");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join("docs/open/a.md"), "# SPEC-a: A\n\nPlain text.\n")
            .expect("write the declaration");
        let locked = root.join("docs/locked");
        Self { root, locked }
    }

    fn lock(&self, dir: &Path, mode: u32) {
        fs::set_permissions(dir, fs::Permissions::from_mode(mode)).expect("set the mode");
    }

    fn run(&self, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let code = output.status.code().expect("exited with a status");
        (code, String::from_utf8_lossy(&output.stderr).into_owned())
    }

    /// `grund <args> docs/locked --path-base=invocation`: the directory argument is the base.
    fn at_base(&self, args: &[&str]) -> (i32, String) {
        let mut all = args.to_vec();
        all.extend(["docs/locked", "--path-base=invocation"]);
        self.run(&all)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.root, fs::Permissions::from_mode(0o755));
        let _ = fs::set_permissions(&self.locked, fs::Permissions::from_mode(0o755));
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Whether the mode bars this process: it does not bar root, which proves nothing here.
fn barred(dir: &Path) -> bool {
    fs::read_dir(dir).is_err()
}

fn has_line(stderr: &str, line: &str) -> bool {
    stderr.lines().any(|candidate| candidate == line)
}

/// The control: under the project base the same failure names `docs/locked`, so the fixture
/// reaches the scan failure and only its spelling at the base is in question.
#[test]
fn the_project_base_names_the_unreadable_directory() {
    let fixture = Fixture::new("control");
    fixture.lock(&fixture.locked, 0o000);
    if !barred(&fixture.locked) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    let (code, stderr) = fixture.run(&["check", "docs/locked", "--path-base=project"]);
    assert_eq!(
        (code, stderr),
        (
            2,
            "error: docs/locked: permission denied; make the path readable, or keep it out of \
             the scan by not passing it, because exclude does not prune a path the command is \
             given\n"
                .to_string()
        )
    );
}

#[test]
fn check_names_an_unreadable_invocation_base_dot() {
    let fixture = Fixture::new("check");
    fixture.lock(&fixture.locked, 0o000);
    if !barred(&fixture.locked) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_eq!(
        fixture.at_base(&["check"]),
        (2, format!("{PASSED_AT_BASE}\n"))
    );
}

#[test]
fn json_path_of_an_unreadable_invocation_base_is_dot() {
    let fixture = Fixture::new("json");
    fixture.lock(&fixture.locked, 0o000);
    if !barred(&fixture.locked) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    let (code, stderr) = fixture.at_base(&["check", "--format", "json"]);
    assert_eq!(code, 2, "{stderr}");
    assert!(
        has_line(&stderr, JSON_AT_BASE),
        "no `\"path\":\".\"` io finding in:\n{stderr}"
    );
}

/// Every command that meets the failure prints the line `check` prints (§FS-workspace.8.7.3),
/// so a fix at the `check` site alone stays red here.
#[test]
fn every_command_names_an_unreadable_invocation_base_dot() {
    let fixture = Fixture::new("commands");
    fixture.lock(&fixture.locked, 0o000);
    if !barred(&fixture.locked) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    for args in [&["list"][..], &["show", "SPEC-a"]] {
        assert_eq!(
            fixture.at_base(args),
            (2, format!("{PASSED_AT_BASE}\n")),
            "grund {args:?}"
        );
    }
    for args in [
        &["refs", "SPEC-a"][..],
        &["cover"],
        &["show", "--batch", "--glob", "SPEC-*", "--format", "json"],
        &["fmt", "--check"],
    ] {
        let (code, stderr) = fixture.at_base(args);
        assert_eq!(code, 2, "grund {args:?}: {stderr}");
        assert!(
            has_line(&stderr, PASSED_AT_BASE),
            "grund {args:?} names no `.`:\n{stderr}"
        );
    }
    // `grund id` prints the line without the Fix text (a separate defect); only the path is here.
    let (code, stderr) = fixture.at_base(&["id", "SPEC", "new thing"]);
    assert_eq!(code, 2, "grund id: {stderr}");
    assert!(
        stderr
            .lines()
            .any(|line| line.starts_with("error: .: permission denied")),
        "grund id names no `.`:\n{stderr}"
    );
}

/// No flag: the config root is the default report base, so an unlistable config root is the
/// base itself and reads `.` too.
#[test]
fn check_names_an_unlistable_config_root_dot() {
    let fixture = Fixture::new("root");
    fixture.lock(&fixture.root, 0o311);
    if !barred(&fixture.root) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    let (code, stderr) = fixture.run(&["check"]);
    assert_eq!(code, 2, "{stderr}");
    assert!(
        has_line(&stderr, AT_BASE),
        "no `error: .:` line in:\n{stderr}"
    );
    assert!(
        !stderr.lines().any(|line| line.starts_with("error: : ")),
        "an empty path in:\n{stderr}"
    );
}
