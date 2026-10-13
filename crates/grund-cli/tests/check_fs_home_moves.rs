//! End-to-end contract for the `fs-home-moves` warning (§FS-check.checks.fs-home-moves):
//! a project with no config file whose FS declarations live in `requirements.md`
//! hears, one release early, that grund 0.19.0 moves the built-in FS home.
//!
//! The case is a tree with no config in either discovery form, so it cannot be an
//! e2e case under `tests/e2e/cases/`: discovery would walk up into this
//! repository's own `grund.toml` (§FS-config.1). Each tree is built under the
//! system temp directory instead.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// §FS-check.checks.fs-home-moves: the exact message, at the first FS declaration.
const WARNING: &str = "requirements.md:3: warning: FS-001-login is in requirements.md, the FS \
                       home only by default, and grund 0.19.0 moves that default to \
                       docs/functional-spec/ — run `grund init` to write a grund.toml that \
                       keeps FS in requirements.md, or move the FS declarations into \
                       docs/functional-spec/\n";

/// Two FS declarations that cite each other, so the tree is otherwise clean.
const REQUIREMENTS: &str = "# Requirements\n\n## FS-001-login: Login\n\n\
                            A user logs in, before §FS-002-logout.\n\n\
                            ## FS-002-logout: Logout\n\nA user logs out, after §FS-001-login.\n";

fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("grund-fs-home-moves-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("create fixture parent");
        fs::write(path, text).expect("write fixture");
    }
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("run grund {args:?}: {err}"))
}

fn transcript(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// §FS-check.checks.fs-home-moves: one warning, at the first FS declaration, in
/// place of `success`, and the exit status stays `0`.
#[test]
fn a_zero_config_tree_with_fs_in_requirements_md_is_warned_once() {
    let root = tree("text", &[("requirements.md", REQUIREMENTS)]);
    let output = run(&root, &["check"]);
    assert_eq!(output.status.code(), Some(0), "{}", transcript(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        WARNING,
        "{}",
        transcript(&output)
    );
}

/// §FS-check.checks.fs-home-moves: JSON carries the same site, code and message.
#[test]
fn the_warning_is_one_json_record_with_its_code() {
    let root = tree("json", &[("requirements.md", REQUIREMENTS)]);
    let output = run(&root, &["check", "--format", "json"]);
    assert_eq!(output.status.code(), Some(0), "{}", transcript(&output));
    let message = WARNING
        .trim_end()
        .strip_prefix("requirements.md:3: warning: ")
        .unwrap()
        .replace('"', "\\\"");
    let expected = format!(
        "{{\"severity\":\"warning\",\"path\":\"requirements.md\",\"line\":3,\
         \"code\":\"fs-home-moves\",\"message\":\"{message}\",\"sites\":null,\
         \"authority\":null}}\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected,
        "{}",
        transcript(&output)
    );
}

/// §FS-check.checks.fs-home-moves: the code selects by the ordinary
/// `--only` / `--ignore` rules (§FS-check.1.4).
#[test]
fn the_code_is_selectable() {
    let root = tree("select", &[("requirements.md", REQUIREMENTS)]);
    let ignored = run(&root, &["check", "--ignore", "fs-home-moves"]);
    assert_eq!(ignored.status.code(), Some(0), "{}", transcript(&ignored));
    assert_eq!(
        String::from_utf8_lossy(&ignored.stdout),
        "success\n",
        "{}",
        transcript(&ignored)
    );
    let only = run(&root, &["check", "--only", "fs-home-moves"]);
    assert_eq!(only.status.code(), Some(0), "{}", transcript(&only));
    assert_eq!(
        String::from_utf8_lossy(&only.stdout),
        WARNING,
        "{}",
        transcript(&only)
    );
}

/// §FS-check.checks.fs-home-moves.1: every case the move does not touch stays
/// silent. These pass before the check exists too; they hold its scope after.
#[test]
fn trees_the_move_does_not_touch_stay_silent() {
    let homeless_fs = "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\n\
                       title = \"What: behavior\"\n";
    let explicit_fs = "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\n\
                       file = \"requirements.md\"\ntitle = \"What: behavior\"\n";
    let cases: [(&str, Vec<(&str, &str)>); 6] = [
        (
            "explicit-fs-home",
            vec![
                ("requirements.md", REQUIREMENTS),
                ("grund.toml", explicit_fs),
            ],
        ),
        (
            "kinds-omitted",
            vec![
                ("requirements.md", REQUIREMENTS),
                ("grund.toml", "grund_config_version = 1\n"),
            ],
        ),
        (
            "kinds-empty",
            vec![
                ("requirements.md", REQUIREMENTS),
                ("grund.toml", "grund_config_version = 1\nkinds = []\n"),
            ],
        ),
        (
            "fs-row-without-home",
            vec![
                ("requirements.md", REQUIREMENTS),
                ("grund.toml", homeless_fs),
            ],
        ),
        (
            "no-fs-declaration",
            vec![(
                "requirements.md",
                "# Requirements\n\nNothing declared yet.\n",
            )],
        ),
        ("no-requirements-md", vec![("docs/notes.md", "# Notes\n")]),
    ];
    let mut failures = Vec::new();
    for (name, files) in cases {
        let root = tree(name, &files);
        let output = run(&root, &["check", "--format", "json"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("fs-home-moves") || stdout.contains("moves that default") {
            failures.push(format!("{name}: warned\n{}", transcript(&output)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-check.checks.fs-home-moves.1: a config `grund init` generates names the
/// FS home explicitly, so running it is one of the two edits that clear the warning.
#[test]
fn a_generated_config_clears_the_warning() {
    let root = tree("generated", &[("requirements.md", REQUIREMENTS)]);
    fs::create_dir_all(root.join(".git")).unwrap();
    let init = run(&root, &["init", "."]);
    assert_eq!(init.status.code(), Some(0), "{}", transcript(&init));
    let output = run(&root, &["check", "--format", "json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("fs-home-moves"), "{}", transcript(&output));
}
