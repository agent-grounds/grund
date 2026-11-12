//! A reported path has one spelling and a scan failure one rendering (grund#1207). A file named
//! inside a scan failure's message is spelled from the report base like the failure's location,
//! never absolute, so the same tree prints the same bytes wherever it sits (§FS-errors.4,
//! §FS-cli.3.4); the line is the one `check` prints whichever command meets the failure
//! (§FS-errors.2.2.5, §FS-workspace.8.7.3); and a path inside a message that names the report base
//! reads `.` (§FS-cli.3.4). These build trees in the temp directory because the bytes under test
//! depend on where the tree sits and on the directory a run starts from. Unix-only: one case
//! makes a directory unreadable.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
                      slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.E2E]\n\
                      folders = [\"e2e/cases\"]\nindex = false\n";

/// What `check` prints after `error: <path>: ` for case `a`'s unparsable `expected.exit`, with
/// `manifest` the manifest as the report spells it.
fn unparsable(manifest: &str) -> String {
    format!(
        "parse {manifest}: invalid digit found in string; \
         write the case's expected exit status in the file as one integer"
    )
}

/// A scratch directory removed on drop; `name` keeps concurrent tests apart.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1207-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the scratch directory");
        Self(root)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, text).expect("write the file");
}

/// grund.249's tree at `root`: one E2E case, `a`, whose `expected.exit` is not an integer.
fn unparsable_exit_tree(root: &Path, config: &str) {
    write(&root.join("grund.toml"), config);
    write(&root.join("e2e/cases/a/expected.exit"), "not-an-exit\n");
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run grund")
}

/// The `error:` lines a run wrote to stderr, each with its newline, and nothing else it wrote
/// there: `refs`' note and a member's empty-scan caution are not what these tests are about.
fn error_lines(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| line.starts_with("error:") || line.starts_with('{'))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// grund.249 from the project root: text and JSON name the manifest from the report base, and
/// the same tree at two depths prints the same bytes (§FS-errors.4).
#[test]
fn an_unparsable_expected_exit_reads_the_same_wherever_the_tree_sits() {
    let scratch = Scratch::new("249-depth");
    let shallow = scratch.0.join("one/p");
    let deep = scratch.0.join("two/much/deeper/p");
    for root in [&shallow, &deep] {
        unparsable_exit_tree(root, CONFIG);
    }
    let text = format!(
        "error: e2e/cases/a/expected.exit: {}\n",
        unparsable("e2e/cases/a/expected.exit")
    );
    let json = format!(
        "{{\"severity\":\"error\",\"path\":\"e2e/cases/a/expected.exit\",\"line\":null,\
         \"code\":\"unparsable-file\",\"message\":\"{}\",\"sites\":null,\"authority\":null}}\n",
        unparsable("e2e/cases/a/expected.exit")
    );
    for root in [&shallow, &deep] {
        let output = run(root, &["check"]);
        assert_eq!(String::from_utf8_lossy(&output.stderr), text, "{root:?}");
        assert_eq!(output.status.code(), Some(2), "{root:?}");
        let output = run(root, &["check", "--format", "json"]);
        assert_eq!(String::from_utf8_lossy(&output.stderr), json, "{root:?}");
        assert_eq!(output.status.code(), Some(2), "{root:?}");
    }
}

/// grund.249 from a subdirectory under `--path-base=invocation`: the manifest climbs out of the
/// subdirectory the way the location beside it does (§FS-cli.3.4, §FS-config.3.6.1).
#[test]
fn an_unparsable_expected_exit_is_spelled_from_the_invocation_base() {
    let scratch = Scratch::new("249-invocation");
    unparsable_exit_tree(&scratch.0, CONFIG);
    let sub = scratch.0.join("sub");
    fs::create_dir_all(&sub).expect("create the subdirectory");
    let output = run(&sub, &["check", "--path-base=invocation"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!(
            "error: ../e2e/cases/a/expected.exit: {}\n",
            unparsable("../e2e/cases/a/expected.exit")
        )
    );
    assert_eq!(output.status.code(), Some(2));
}

/// grund.249 at a workspace root: the member's manifest is spelled from the workspace root, as
/// its location is, never from the member that scanned it (§FS-workspace.8.7.3).
#[test]
fn an_unparsable_expected_exit_in_a_member_is_spelled_from_the_workspace_root() {
    let scratch = Scratch::new("249-workspace");
    write(
        &scratch.0.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"ws\"\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    unparsable_exit_tree(&scratch.0.join("m"), &CONFIG.replace("\"p\"", "\"m\""));
    let output = run(&scratch.0, &["check"]);
    assert_eq!(
        error_lines(&output),
        format!(
            "error: m/e2e/cases/a/expected.exit: {}\n",
            unparsable("m/e2e/cases/a/expected.exit")
        )
    );
    assert_eq!(output.status.code(), Some(2));
}

/// Every command that meets grund.249's failure prints `check`'s line for it, fix included
/// (§FS-errors.2.2.5, §FS-workspace.8.7.3): `id` among them.
#[test]
fn every_command_prints_checks_line_for_a_failure_below_the_base() {
    let scratch = Scratch::new("249-commands");
    unparsable_exit_tree(&scratch.0, CONFIG);
    let line = format!(
        "error: e2e/cases/a/expected.exit: {}\n",
        unparsable("e2e/cases/a/expected.exit")
    );
    let mut differs = Vec::new();
    for args in [
        &["check"][..],
        &["list"],
        &["show", "E2E-a"],
        &["refs", "E2E-a"],
        &["show", "--batch", "--format", "json", "--glob", "E2E-*"],
        &["id", "E2E", "x"],
    ] {
        let output = run(&scratch.0, args);
        let said = error_lines(&output);
        if said != line || output.status.code() != Some(2) {
            differs.push(format!(
                "grund {args:?} exit {:?}: {said}",
                output.status.code()
            ));
        }
    }
    assert!(
        differs.is_empty(),
        "expected {line}differs:\n{}",
        differs.concat()
    );
}

/// A scan failure at the report base itself is `.` in every command's line, and every command
/// prints the same fix for it: the passed path's (§FS-cli.3.4, §FS-errors.2.2.5).
#[test]
fn every_command_prints_checks_line_for_a_failure_at_the_base() {
    let scratch = Scratch::new("base-commands");
    write(
        &scratch.0.join("grund.toml"),
        &CONFIG.replace("E2E", "FS").replace("e2e/cases", "docs"),
    );
    write(&scratch.0.join("docs/a.md"), "# FS-a: A\n\nLead.\n");
    write(&scratch.0.join("docs/locked/b.md"), "Text.\n");
    let locked = scratch.0.join("docs/locked");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("lock the directory");
    if fs::read_dir(&locked).is_ok() {
        // Root reads a mode-000 directory, so there is no failure to render.
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock");
        return;
    }
    let line = "error: .: permission denied; make the path readable, or keep it out of the scan \
                by not passing it, because exclude does not prune a path the command is given\n";
    let base = ["docs/locked", "--path-base=invocation"];
    let mut differs = Vec::new();
    for args in [
        &["check"][..],
        &["list"],
        &["show", "FS-a"],
        &["refs", "FS-a"],
        &["show", "--batch", "--format", "json", "--glob", "FS-*"],
        &["id", "FS", "x"],
    ] {
        let args = [args, &base[..]].concat();
        let output = run(&scratch.0, &args);
        let said = error_lines(&output);
        if said != line || output.status.code() != Some(2) {
            differs.push(format!(
                "grund {args:?} exit {:?}: {said}",
                output.status.code()
            ));
        }
    }
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock");
    assert!(
        differs.is_empty(),
        "expected {line}differs:\n{}",
        differs.concat()
    );
}

/// A `duplicate`'s `also declared at` that names an E2E case directory which is the report base
/// reads `.`, as the finding's `sites[].path` for it already does (§FS-cli.3.4). Only the message
/// text moves: the location and `sites` stay as they are.
#[test]
fn a_message_naming_the_report_base_reads_dot() {
    let scratch = Scratch::new("dup-dot");
    write(&scratch.0.join("grund.toml"), CONFIG);
    write(&scratch.0.join("e2e/cases/a/expected.exit"), "0\n");
    write(
        &scratch.0.join("e2e/cases/README.md"),
        "# E2E-a: A\n\nText.\n",
    );
    let case = scratch.0.join("e2e/cases/a");
    let output = run(&case, &["check", "--path-base=invocation"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "../README.md:1: error: duplicate declaration of E2E-a (also declared at .:1)\n"
    );
    assert_eq!(output.status.code(), Some(1));
    let output = run(
        &case,
        &["check", "--path-base=invocation", "--format", "json"],
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"severity\":\"error\",\"path\":\"../README.md\",\"line\":1,\"code\":\"duplicate\",\
         \"message\":\"duplicate declaration of E2E-a (also declared at .:1)\",\
         \"sites\":[{\"path\":\"../README.md\",\"line\":1},{\"path\":\".\",\"line\":1}],\
         \"authority\":null}\n"
    );
    assert_eq!(output.status.code(), Some(1));
}
