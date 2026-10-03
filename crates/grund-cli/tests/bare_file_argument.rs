//! §FS-config.1.4 — a file argument with no directory part has the working
//! directory as its parent, so `grund <command> NAME` and `grund <command> ./NAME`
//! are one run: same config root, same workspace member, same path base. Each
//! case runs the built binary from the file's own directory, the only place the
//! two spellings differ.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/bare_file_argument")
        .join(name)
        .join("repo")
}

fn run_grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

fn render(output: &Output) -> String {
    format!(
        "exit {:?}\n--- stdout\n{}--- stderr\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Runs `command` once with `bare` and once with `./bare` from `cwd`, and
/// requires the two runs to print the same bytes and exit the same way.
fn assert_spellings_agree(command: &str, bare: &str, cwd: &Path) -> Output {
    let dotted = format!("./{bare}");
    let bare_run = run_grund(&[command, bare], cwd);
    let dotted_run = run_grund(&[command, &dotted], cwd);
    assert_eq!(
        render(&bare_run),
        render(&dotted_run),
        "`grund {command} {bare}` and `grund {command} {dotted}` from {} must be one run",
        cwd.display()
    );
    bare_run
}

/// §FS-config.1.4: from `docs/functional-spec/`, a bare name finds the
/// `grund.toml` two levels up, whose `[id] format` and folder `FS` home both
/// disagree with the built-in defaults, so a run on the defaults cannot pass.
#[test]
fn bare_file_check_from_a_subdirectory_loads_the_config_above_it() {
    let cwd = fixture("project").join("docs/functional-spec");
    let output = assert_spellings_agree("check", "FS-widget.md", &cwd);
    assert_eq!(output.status.code(), Some(0), "{}", render(&output));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "success\n");
}

/// §FS-config.1.4: the bare name renders its path from the config root, as
/// `./FS-widget.md` does, not from the working directory.
#[test]
fn bare_file_list_from_a_subdirectory_renders_paths_from_the_config_root() {
    let cwd = fixture("project").join("docs/functional-spec");
    let output = assert_spellings_agree("list", "FS-widget.md", &cwd);
    assert_eq!(output.status.code(), Some(0), "{}", render(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("docs/functional-spec/FS-widget.md:3"),
        "{}",
        render(&output)
    );
}

/// §FS-config.1.4 with §FS-workspace.5.1: inside a member that has no
/// `grund.toml` of its own, a bare name is rooted at that member, exactly as
/// `./FS-thing.md` is, so its path renders from the member's root.
#[test]
fn bare_file_list_inside_a_configless_member_is_rooted_at_the_member() {
    let cwd = fixture("workspace").join("member/docs");
    let output = assert_spellings_agree("list", "FS-thing.md", &cwd);
    assert_eq!(output.status.code(), Some(0), "{}", render(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("  docs/FS-thing.md:3"), "{}", render(&output));
    assert!(!stdout.contains("member/docs/FS-thing.md"), "{}", render(&output));
}
