//! Review finding R1-01 of #1464: the paths `config migrate`'s report gate prints
//! follow `--path-base` like the heading above them (§FS-config-migrate.2.2), and
//! the config's own line keeps the candidate's mark there
//! (§FS-config-migrate.5.2), naming the finding as `grund check` names it from
//! the same place.

#[path = "support/fixture_cleanup.rs"]
mod fixture_cleanup;
use fixture_cleanup::remove_fixture_tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A version-1 file whose built-in `FS` row names the missing `docs/functional-spec`.
const V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
                  named_sections = true\n[scan]\ninclude = [\"src\"]\n";
const CITING: &str = "// \u{a7}FS-typo\nclass A {}\n";
const QUIET: &str = "class A {}\n";

struct Fixture(PathBuf);

impl Fixture {
    /// A git tree holding `config`, `src/A.java` with `java`, and an empty `docs`.
    fn new(name: &str, config: &str, java: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1464-gate-{name}-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            remove_fixture_tree(&root, "remove stale fixture");
        }
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("grund.toml"), config).unwrap();
        fs::write(root.join("src/A.java"), java).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        remove_fixture_tree(&self.0, "remove fixture");
    }
}

fn grund(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// The candidate line `grund check --suggestions` names for the missing `FS`
/// place once the candidate is written.
fn candidate_line() -> usize {
    let quiet = Fixture::new("quiet", V1, QUIET);
    let preview = grund(&quiet.0, &["config", "migrate", "--to", "2"]);
    assert_eq!(preview.status.code(), Some(0), "{}", text(&preview.stderr));
    let written = Fixture::new("written", &text(&preview.stdout), CITING);
    let printed = text(&grund(&written.0, &["check", "--suggestions"]).stdout);
    let tail = "no place of kind FS exists: docs/functional-spec (grund.toml:";
    let at = printed.find(tail).unwrap_or_else(|| panic!("{printed}")) + tail.len();
    let digits: String = printed[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or_else(|_| panic!("{printed}"))
}

/// The comparison lines of a refused preview run from `dir` with `args`.
fn compared(dir: &Path, args: &[&str]) -> Vec<String> {
    let output = grund(
        dir,
        &[&["config", "migrate", "--to", "2"][..], args].concat(),
    );
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    stderr
        .lines()
        .filter(|l| l.starts_with("- ") || l.starts_with("+ "))
        .map(str::to_owned)
        .collect()
}

/// §FS-config-migrate.2.2, §FS-config-migrate.5.2: from `docs` under
/// `--path-base invocation`, the finding and the config are spelled from `docs`,
/// as check spells them there, and the config line keeps the candidate's mark.
#[test]
fn gate_paths_follow_the_invocation_base() {
    let line = candidate_line();
    let fixture = Fixture::new("invocation", V1, CITING);
    let lines = compared(&fixture.0.join("docs"), &["--path-base", "invocation"]);
    let missing = "unknown reference FS-typo; no place of kind FS exists: docs/functional-spec";
    assert_eq!(
        lines,
        [
            "- error error dangling ../src/A.java:1:4 unknown reference FS-typo".to_owned(),
            format!(
                "+ error error dangling ../src/A.java:1:4 {missing} \
                 (the candidate's ../grund.toml:{line})"
            ),
        ]
    );
}

/// §FS-config-migrate.2.2: the default base spells the same lines from the root.
#[test]
fn gate_paths_keep_the_project_base() {
    let line = candidate_line();
    let fixture = Fixture::new("project", V1, CITING);
    let missing = "unknown reference FS-typo; no place of kind FS exists: docs/functional-spec";
    let expected = [
        "- error error dangling src/A.java:1:4 unknown reference FS-typo".to_owned(),
        format!(
            "+ error error dangling src/A.java:1:4 {missing} (the candidate's grund.toml:{line})"
        ),
    ];
    assert_eq!(compared(&fixture.0, &[]), expected);
    assert_eq!(compared(&fixture.0.join("docs"), &[]), expected);
}
