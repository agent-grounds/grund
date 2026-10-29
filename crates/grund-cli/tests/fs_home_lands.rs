//! End-to-end contract for the built-in `FS` home of grund 0.19.0
//! (§DF-fs-home-lands-in-0-19): a project with no config file and a `grund.toml`
//! that omits `[[kinds]]` give one answer, `folder = "docs/functional-spec"`
//! (§FS-config.3.4.4), and the `software-development` scaffold homes `FS` there
//! (§FS-init.2.4.10).
//!
//! The zero-config trees cannot be e2e cases under `tests/e2e/cases/`: discovery
//! would walk up into this repository's own `grund.toml` (§FS-config.1). Each
//! tree is built under the system temp directory instead.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// §FS-check.checks.config-version-1: the caution a version-1 config earns,
/// which a zero-config tree does not; the comparisons below drop it.
const VERSION_1_CAUTION: &str =
    "warning: grund.toml is version 1; migrate it with grund config migrate --to 2";

/// The same caution as one `--format json` finding line carries it.
const VERSION_1_JSON_CODE: &str = "\"code\":\"config-version-1\"";

/// A `grund.toml` that omits `[[kinds]]`, the side of each comparison that
/// already had the folder home before 0.19.0.
const KINDLESS_CONFIG: &str = "grund_config_version = 1\n";

/// Two FS declarations in `requirements.md` that cite each other.
const REQUIREMENTS: &str = "# Requirements\n\n## FS-001-login: Login\n\n\
                            A user logs in, before \u{a7}FS-002-logout.\n\n\
                            ## FS-002-logout: Logout\n\nA user logs out, after \u{a7}FS-001-login.\n";

/// One FS declaration in the folder home, listed in its index.
const FS_FOLDER: &[(&str, &str)] = &[
    (
        "docs/functional-spec/README.md",
        "# Functional Spec\n\n| ID | Subject |\n|---|---|\n\
         | [\u{a7}FS-001-login](FS-001-login.md) | Login |\n",
    ),
    (
        "docs/functional-spec/FS-001-login.md",
        "# FS-001-login: Login\n\nA user logs in.\n",
    ),
    ("docs/notes.md", "Login is \u{a7}FS-001-login.\n"),
];

fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("grund-fs-home-lands-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("create fixture parent");
        fs::write(path, text).expect("write fixture");
    }
    root
}

/// The same files twice: once with no config, once beside a `grund.toml` that
/// omits `[[kinds]]`.
fn pair(name: &str, files: &[(&str, &str)]) -> (PathBuf, PathBuf) {
    let zero = tree(&format!("{name}-zero"), files);
    let mut with_config = files.to_vec();
    with_config.push(("grund.toml", KINDLESS_CONFIG));
    let kindless = tree(&format!("{name}-kindless"), &with_config);
    (zero, kindless)
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

/// A run's exit and findings with the version-1 caution taken out, in text and
/// in JSON, and the `success` line it stands in for, so a zero-config run and a
/// kindless-config run can be compared whole.
fn answer(output: &Output) -> (Option<i32>, String, String) {
    let strip = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .lines()
            .filter(|line| {
                *line != VERSION_1_CAUTION
                    && *line != "success"
                    && !line.contains(VERSION_1_JSON_CODE)
            })
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    };
    (
        output.status.code(),
        strip(&output.stdout),
        strip(&output.stderr),
    )
}

/// The `FS` row `grund config show` prints, from `kind = "FS"` to the blank line.
fn fs_row(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let start = stdout
        .find("kind = \"FS\"")
        .unwrap_or_else(|| panic!("no FS row\n{}", transcript(output)));
    let row = &stdout[start..];
    row[..row.find("\n\n").unwrap_or(row.len())].to_string()
}

/// §FS-config.3.4.4: the built-in `FS` row is `folder = "docs/functional-spec"`,
/// with or without a config file; §FS-config.3.4.4.4's split is gone.
#[test]
fn zero_config_and_a_kindless_config_print_the_same_fs_row() {
    let (zero, kindless) = pair("show", &[]);
    let zero = run(&zero, &["config", "show"]);
    let kindless = run(&kindless, &["config", "show"]);
    assert_eq!(zero.status.code(), Some(0), "{}", transcript(&zero));
    assert_eq!(fs_row(&zero), fs_row(&kindless), "{}", transcript(&zero));
    assert!(
        fs_row(&zero).contains("folder = \"docs/functional-spec\""),
        "{}",
        transcript(&zero)
    );
}

/// §FS-config.3.4.4: a zero-config tree checks its FS folder as a kindless
/// config does — the declaration is in its home, and the tree is clean.
#[test]
fn zero_config_checks_the_fs_folder_as_a_kindless_config_does() {
    let (zero, kindless) = pair("folder", FS_FOLDER);
    let zero = run(&zero, &["check"]);
    let kindless = run(&kindless, &["check"]);
    assert_eq!(answer(&zero), answer(&kindless), "{}", transcript(&zero));
    assert_eq!(zero.status.code(), Some(0), "{}", transcript(&zero));
    assert_eq!(
        String::from_utf8_lossy(&zero.stdout),
        "success\n",
        "{}",
        transcript(&zero)
    );
}

/// §FS-check.checks.fs-home-moves: FS declarations in a zero-config
/// `requirements.md` earn no `fs-home-moves` warning; the tree gets the answer
/// a kindless config gets (§DF-fs-home-lands-in-0-19).
#[test]
fn zero_config_requirements_md_gets_the_kindless_answer_and_no_warning() {
    let (zero, kindless) = pair("requirements", &[("requirements.md", REQUIREMENTS)]);
    for format in [&["check"][..], &["check", "--format", "json"][..]] {
        let zero_run = run(&zero, format);
        let kindless_run = run(&kindless, format);
        let text = transcript(&zero_run);
        assert!(!text.contains("fs-home-moves"), "{text}");
        assert!(!text.contains("0.19.0 moves that default"), "{text}");
        assert_eq!(answer(&zero_run), answer(&kindless_run), "{text}");
    }
}

/// §FS-init.2.4.10, §FS-init.2.1.3, §FS-init-fixtures.2: a fresh
/// `software-development` scaffold declares `FS` in `docs/functional-spec/`
/// and `REQ` in `docs/requirements/`, writes no `requirements.md`, and passes
/// `grund check` and `grund init --check` unmodified.
#[test]
fn the_scaffold_homes_fs_in_docs_functional_spec() {
    let root = tree("scaffold", &[]);
    // `grund init --check` wants a version-controlled tree, as a real one is.
    let git = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&root)
        .output()
        .expect("spawn git init");
    assert!(git.status.success(), "git init {}", root.display());
    let init = run(
        &root,
        &["init", "--template", "software-development", "--scaffold"],
    );
    assert_eq!(init.status.code(), Some(0), "{}", transcript(&init));
    let config = fs::read_to_string(root.join("grund.toml")).expect("grund.toml written");
    assert!(
        config.contains("[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n"),
        "{config}"
    );
    assert!(
        config.contains("[schema.kinds.REQ]\nfolders = [\"docs/requirements\"]\n"),
        "{config}"
    );
    assert!(
        root.join("docs/functional-spec/README.md").is_file(),
        "{}",
        transcript(&init)
    );
    assert!(
        !root.join("requirements.md").exists(),
        "{}",
        transcript(&init)
    );
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("AGENTS.md written");
    assert!(
        agents.contains("- [FS](docs/functional-spec): "),
        "{agents}"
    );
    let check = run(&root, &["check"]);
    assert_eq!(check.status.code(), Some(0), "{}", transcript(&check));
    let init_check = run(&root, &["init", "--check"]);
    assert_eq!(
        init_check.status.code(),
        Some(0),
        "{}",
        transcript(&init_check)
    );
}
