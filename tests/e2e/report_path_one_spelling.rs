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

/// `init` stops on grund.249's failure with `check`'s message for it, fix included
/// (§FS-errors.2.2.5). A rule kind is what makes `init` scan the tree. Where `init` locates the
/// failure is not held here: it still names it by its absolute path, which is a fix of its own.
#[test]
fn init_prints_checks_message_with_its_fix() {
    let scratch = Scratch::new("249-init");
    let rule_kind = "\n[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nform = \"rule\"\n";
    unparsable_exit_tree(&scratch.0, &format!("{CONFIG}{rule_kind}"));
    let output = run(&scratch.0, &["init", "--no-vcs"]);
    let said = error_lines(&output);
    let message = unparsable("e2e/cases/a/expected.exit");
    let tail = format!("/{}\n", message.trim_start_matches("parse "));
    assert!(
        said.starts_with("error: ") && said.ends_with(&tail) && said.lines().count() == 1,
        "expected one line ending {tail}got {said}"
    );
    assert_eq!(output.status.code(), Some(2));
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

/// The ignore file named after `stop ignoring it in ` in `line`, up to the comma or line end.
fn ignore_file_named(line: &str) -> String {
    let (_, rest) = line
        .split_once("stop ignoring it in ")
        .expect("the rule's way out");
    rest.split(", and fetch again")
        .next()
        .expect("a name")
        .trim_end()
        .to_owned()
}

/// An ignore file outside the loaded root, here the global excludes file, is named by `fetch`'s
/// refusal as `check`'s tail names it for the same home, with the broken-stub line's fallback
/// rather than a `..` path that climbs out of the root (§FS-fetch.4.2.1, §FS-check.3.1.3.1,
/// §FS-cli.3.4, §AR-config.1.7).
#[test]
fn fetch_and_check_name_an_ignore_file_outside_the_root_alike() {
    let scratch = Scratch::new("outside-ignore");
    let (home, xdg, root) = (
        scratch.0.join("home"),
        scratch.0.join("xdg"),
        scratch.0.join("p"),
    );
    fs::create_dir_all(&home).expect("create the home");
    // A global excludes file applies in a Git repository (§FS-scan.scope.ignore-files.global).
    fs::create_dir_all(root.join(".git")).expect("mark the repository");
    write(&xdg.join("git/ignore"), "T-1.md\n");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.kinds.T]\n\
         folders = [\"docs/t\"]\nid_format = \"{kind}-{number}\"\nindex = false\n\
         fetch = \"scripts/fetch\"\n",
    );
    write(&root.join("notes.md"), "# Notes\n\nSee §T-1.\n");
    let script = root.join("scripts/fetch");
    write(&script, "#!/bin/sh\nprintf '# T-1: Fetched\\n'\n");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("make it executable");
    let run_isolated = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&root)
            .env("NO_COLOR", "1")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", &xdg)
            .output()
            .expect("run grund")
    };
    let fetch = error_lines(&run_isolated(&["fetch", "T-1"]));
    let check = String::from_utf8_lossy(&run_isolated(&["check"]).stdout).into_owned();
    let check = check
        .lines()
        .find(|line| line.contains("the scan does not read docs/t/T-1.md"))
        .unwrap_or_else(|| panic!("check names the unread snapshot:\n{check}"))
        .to_owned();
    let (fetched, checked) = (ignore_file_named(&fetch), ignore_file_named(&check));
    assert_eq!(fetched, checked, "fetch:\n{fetch}check:\n{check}");
    assert!(
        Path::new(&fetched).is_absolute(),
        "an escaping spelling: {fetch}"
    );
    assert_eq!(
        fs::canonicalize(&fetched).expect("the named file exists"),
        fs::canonicalize(xdg.join("git/ignore")).expect("the ignore file exists"),
    );
    assert!(
        fetch.starts_with("error: kind `T` cannot fetch into docs/t/T-1.md: "),
        "fetch:\n{fetch}"
    );
}
