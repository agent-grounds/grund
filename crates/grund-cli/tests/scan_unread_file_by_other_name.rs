//! Black-box: at a workspace root, a narrowed `grund check <path>` answers for an unreadable
//! file at or under the path by any of the file's names, so a directory link whose name sorts
//! first decides nothing about the exit (§FS-check.1.3.6.3, agent-grounds/grund#1299).
//! A path that holds the file under no name keeps the caution and its exit `0`. Git carries no
//! mode a fixture could keep unreadable, so the e2e corpus cannot hold this; the tests skip
//! where the mode does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const SCHEMA: &str = "[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.GOAL]\nfolders = [\"docs\"]\nindex = false\n";

/// The issue's tree: a workspace root with one member, `real/s.md` at mode 000, and a directory
/// link to `real` named `link`.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str, link: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1299-{name}-{}", std::process::id()));
        let _ = fs::set_permissions(root.join("real/s.md"), fs::Permissions::from_mode(0o644));
        let _ = fs::remove_dir_all(&root);
        for dir in ["docs", "m/docs", "real", "other"] {
            fs::create_dir_all(root.join(dir)).expect("create a directory");
        }
        for (path, text) in [
            (
                "grund.toml",
                format!(
                    "grund_config_version = 2\nproject_name = \"root\"\n\n{SCHEMA}\n\
                     [workspace]\nmembers = [\"m\"]\n"
                ),
            ),
            (
                "m/grund.toml",
                format!("grund_config_version = 2\nproject_name = \"m\"\n\n{SCHEMA}"),
            ),
            (
                "docs/GOAL-a.md",
                "# GOAL-a: A\n\nSee \u{a7}GOAL-b.\n".into(),
            ),
            (
                "docs/GOAL-b.md",
                "# GOAL-b: B\n\nSee \u{a7}GOAL-a.\n".into(),
            ),
            (
                "m/docs/GOAL-c.md",
                "# GOAL-c: C\n\nSee \u{a7}GOAL-d.\n".into(),
            ),
            (
                "m/docs/GOAL-d.md",
                "# GOAL-d: D\n\nSee \u{a7}GOAL-c.\n".into(),
            ),
            ("real/s.md", "# A note\n".into()),
            ("other/o.md", "# Other\n".into()),
        ] {
            fs::write(root.join(path), text).expect("write a fixture file");
        }
        std::os::unix::fs::symlink("real", root.join(link)).expect("make the directory link");
        fs::set_permissions(root.join("real/s.md"), fs::Permissions::from_mode(0o000))
            .expect("chmod 000");
        Self { root }
    }

    /// Whether the mode bars this process: it does not bar root, which proves nothing here.
    fn barred(&self) -> bool {
        fs::read(self.root.join("real/s.md")).is_err()
    }

    fn check(&self, path: &str) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["check", path])
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let code = output.status.code().expect("exited with a status");
        (code, String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(
            self.root.join("real/s.md"),
            fs::Permissions::from_mode(0o644),
        );
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// An `error:` line about the unreadable file, under whichever name: the spelling is not judged.
fn has_error(stderr: &str) -> bool {
    stderr
        .lines()
        .any(|line| line.starts_with("error: ") && line.contains("s.md: permission denied"))
}

fn has_caution(stderr: &str) -> bool {
    stderr.lines().any(|line| {
        line.starts_with("warning: ")
            && line.contains("s.md: permission denied")
            && line.contains("outside the report scope")
    })
}

/// Both narrowed runs over the file's own directory and the file itself exit `2` with an error,
/// and say nothing of the file being outside the path.
fn assert_answers_for_the_file(fixture: &Fixture) {
    for path in ["real", "real/s.md"] {
        let (code, stderr) = fixture.check(path);
        assert!(
            code == 2 && has_error(&stderr) && !has_caution(&stderr),
            "grund check {path}: s.md is under `{path}`, so the run must exit 2 with an error \
             about it and no outside-the-report-scope caution; it exited {code}:\n{stderr}"
        );
    }
}

/// The control: a link whose name sorts after `real` leaves `real/s.md` the surviving name, so
/// the fixture reaches the failure and only which name survives is in question.
#[test]
fn a_link_sorting_after_leaves_the_narrowed_run_red() {
    let fixture = Fixture::new("zz", "zz");
    if !fixture.barred() {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_answers_for_the_file(&fixture);
}

/// The ticket: the link `aa` sorts first, so the walk keeps `aa/s.md`, and the run must still
/// count `real/s.md` as at or under `real`.
#[test]
#[ignore = "red until #1299 lets the scan-failure split ask every name of the file; implement removes this"]
fn a_link_sorting_before_leaves_the_narrowed_run_red() {
    let fixture = Fixture::new("aa", "aa");
    if !fixture.barred() {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_answers_for_the_file(&fixture);
}

/// The guard: `other` holds the file under no name, so the read failure stays a caution with
/// exit `0`; a fix that made every read failure an error would turn this red.
#[test]
fn a_path_holding_the_file_under_no_name_keeps_the_caution() {
    for link in ["aa", "zz"] {
        let fixture = Fixture::new(&format!("other-{link}"), link);
        if !fixture.barred() {
            eprintln!(
                "skipped: the mode does not bar this user, so nothing here can be unreadable"
            );
            return;
        }
        let (code, stderr) = fixture.check("other");
        assert!(
            code == 0 && has_caution(&stderr) && !has_error(&stderr),
            "grund check other (link {link}): s.md is outside `other`, so the run must keep the \
             caution and exit 0; it exited {code}:\n{stderr}"
        );
    }
}
