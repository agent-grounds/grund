//! §FS-config.1.4 — a file argument with no directory part has the working
//! directory as its parent, so `grund <command> NAME` and `grund <command> ./NAME`
//! are one run: same config root, same workspace member, same path base. Each
//! case runs the built binary from the file's own directory, the only place the
//! two spellings differ. The `fmt` cases pin §FS-fmt.6.2.5 the same way: the
//! link a bare name gets is the link `./NAME` gets.

use std::fs;
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
    assert_spellings_agree_with(&[command], bare, cwd)
}

/// [`assert_spellings_agree`] for a command line of several words, the file
/// argument appended last.
fn assert_spellings_agree_with(command: &[&str], bare: &str, cwd: &Path) -> Output {
    let dotted = format!("./{bare}");
    let bare_run = run_grund(&[command, &[bare]].concat(), cwd);
    let dotted_run = run_grund(&[command, &[dotted.as_str()]].concat(), cwd);
    let shown = command.join(" ");
    assert_eq!(
        render(&bare_run),
        render(&dotted_run),
        "`grund {shown} {bare}` and `grund {shown} {dotted}` from {} must be one run",
        cwd.display()
    );
    bare_run
}

/// A fresh copy of fixture `name` under Cargo's scratch directory for
/// integration tests, for a run that writes: the committed fixture is never
/// touched.
fn fixture_copy(name: &str, copy: &str) -> PathBuf {
    let to = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(copy)
        .join("repo");
    let _ = fs::remove_dir_all(&to);
    copy_dir(&fixture(name), &to);
    to
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|err| panic!("create {}: {err}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|err| panic!("read {}: {err}", from.display())) {
        let source = entry.unwrap().path();
        let target = to.join(source.file_name().unwrap());
        if source.is_dir() {
            copy_dir(&source, &target);
        } else {
            fs::copy(&source, &target).unwrap_or_else(|err| {
                panic!("copy {} to {}: {err}", source.display(), target.display())
            });
        }
    }
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
    assert!(
        stdout.contains("  docs/FS-thing.md:3"),
        "{}",
        render(&output)
    );
    assert!(
        !stdout.contains("member/docs/FS-thing.md"),
        "{}",
        render(&output)
    );
}

/// §FS-fmt.6.2.5: from the repository root, `fmt --write NOTES.md` wraps the
/// citation with the link `./NOTES.md` gets, `docs/fs/...`, not one `../` per
/// directory above the repository, and reports the file as `NOTES.md`, not by
/// an absolute path (§FS-config.3.6). Each spelling writes its own copy.
#[test]
fn bare_file_fmt_write_writes_the_link_dotted_spelling_writes() {
    // `\u{a7}` is the marker, escaped so this repository does not read a citation.
    let expected = "See [\u{a7}FS-demo](docs/fs/FS-demo.md#fs-demo-demo).\n";
    let mut runs = Vec::new();
    for (copy, arg) in [
        ("bare_file_argument_fmt_bare", "NOTES.md"),
        ("bare_file_argument_fmt_dotted", "./NOTES.md"),
    ] {
        let repo = fixture_copy("fmt", copy);
        let output = run_grund(&["fmt", "--write", arg], &repo);
        let written = fs::read_to_string(repo.join("NOTES.md")).expect("read NOTES.md");
        assert_eq!(output.status.code(), Some(0), "{arg}: {}", render(&output));
        assert_eq!(
            written, expected,
            "`grund fmt --write {arg}` wrote the wrong link"
        );
        runs.push(render(&output));
    }
    assert_eq!(
        runs[0], runs[1],
        "`NOTES.md` and `./NOTES.md` must report alike"
    );
    assert_eq!(
        runs[0],
        "exit Some(0)\n--- stdout\nrewrote 1 line:\n  NOTES.md (1)\n--- stderr\n"
    );
}

/// §FS-fmt.6.2.5: a file that already holds the canonical link passes
/// `fmt --check` by its bare name, as it does by `./LINKED.md`.
#[test]
fn bare_file_fmt_check_passes_an_already_canonical_link() {
    let output = assert_spellings_agree_with(&["fmt", "--check"], "LINKED.md", &fixture("fmt"));
    assert_eq!(output.status.code(), Some(0), "{}", render(&output));
}

/// §FS-fmt.6.2.5: the preview (neither `--check` nor `--write`) of an
/// already-canonical file is empty for the bare name too, and the preview of
/// a pending wrap names the file by the same path under both spellings.
#[test]
fn bare_file_fmt_preview_matches_the_dotted_spelling() {
    let repo = fixture("fmt");
    let linked = assert_spellings_agree("fmt", "LINKED.md", &repo);
    assert_eq!(linked.status.code(), Some(0), "{}", render(&linked));
    let notes = assert_spellings_agree("fmt", "NOTES.md", &repo);
    assert_eq!(notes.status.code(), Some(1), "{}", render(&notes));
}
