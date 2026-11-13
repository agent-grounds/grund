//! #1387's reproducer, ported to the CLI harness: a candidate config line with no
//! version-1 line prints in `config migrate`'s `differs:` comparison as the
//! candidate's own line, the one `grund check` names once the candidate is
//! written, marked as the candidate's (§FS-config-migrate.5.2.1).

#[path = "support/fixture_cleanup.rs"]
mod fixture_cleanup;
use fixture_cleanup::remove_fixture_tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A version-1 file with no `[[kinds]]`, so v1's built-in `FS` row applies with its
/// folder `docs/functional-spec`, which does not exist.
const V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
                  named_sections = true\n[scan]\ninclude = [\"src\"]\n";
const CITING: &str = "// \u{a7}FS-typo\nclass A {}\n";
const QUIET: &str = "class A {}\n";

struct Fixture(PathBuf);

impl Fixture {
    /// A git tree holding `config` and `src/A.java` with `java`.
    fn new(name: &str, config: &str, java: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1387-{name}-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            remove_fixture_tree(&root, "remove stale fixture");
        }
        fs::create_dir_all(root.join("src")).unwrap();
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

/// The line `grund check --suggestions` names for the missing `FS` place once the
/// candidate is written: the candidate comes from the passing preview of the same
/// tree without the citation, and the citation is put back under it.
fn candidate_line() -> (String, usize) {
    let quiet = Fixture::new("quiet", V1, QUIET);
    let preview = grund(&quiet.0, &["config", "migrate", "--to", "2"]);
    assert_eq!(preview.status.code(), Some(0), "{}", text(&preview.stderr));
    let candidate = text(&preview.stdout);
    let written = Fixture::new("written", &candidate, CITING);
    let check = grund(&written.0, &["check", "--suggestions"]);
    let printed = text(&check.stdout);
    assert_eq!(check.status.code(), Some(1), "{printed}");
    let tail = "no place of kind FS exists: docs/functional-spec (grund.toml:";
    let at = printed.find(tail).unwrap_or_else(|| panic!("{printed}")) + tail.len();
    let digits: String = printed[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let line: usize = digits.parse().unwrap_or_else(|_| panic!("{printed}"));
    assert!(
        line <= candidate.lines().count(),
        "check names a line of the candidate: {line}\n{candidate}"
    );
    (candidate, line)
}

/// §FS-config-migrate.5.2.1: the `+ ` line names the line `grund check` names on
/// the written candidate, marked as the candidate's, and still refuses
/// (§FS-config-migrate.5.4).
#[test]
#[ignore = "red until #1387 prints the candidate's own line; implement removes this"]
fn migrate_names_an_unmapped_candidate_line_as_the_candidates_own() {
    let (_, line) = candidate_line();
    let fixture = Fixture::new("migrate", V1, CITING);
    let output = grund(&fixture.0, &["config", "migrate", "--to", "2"]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert_eq!(text(&output.stdout), "", "a refusal prints no candidate");
    let compared: Vec<&str> = stderr
        .lines()
        .skip_while(|l| !l.starts_with("differs: grund check --suggestions"))
        .skip(1)
        .take_while(|l| l.starts_with("- ") || l.starts_with("+ "))
        .collect();
    let added: Vec<&&str> = compared.iter().filter(|l| l.starts_with("+ ")).collect();
    assert_eq!(added.len(), 1, "one finding differs:\n{stderr}");
    let marked = format!(
        "no place of kind FS exists: docs/functional-spec (the candidate's grund.toml:{line})"
    );
    assert!(
        added[0].ends_with(&marked),
        "the `+ ` line names the candidate's line {line}:\n{stderr}"
    );
}

/// §FS-config-migrate.5.2.1: no comparison line names a config line that neither
/// the version-1 file nor the candidate has.
#[test]
#[ignore = "red until #1387 stops printing usize::MAX; implement removes this"]
fn migrate_never_names_a_config_line_neither_file_has() {
    let (candidate, _) = candidate_line();
    let fixture = Fixture::new("bounds", V1, CITING);
    let output = grund(&fixture.0, &["config", "migrate", "--to", "2"]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let longest = V1.lines().count().max(candidate.lines().count());
    for compared in stderr
        .lines()
        .filter(|l| l.starts_with("- ") || l.starts_with("+ "))
    {
        let mut rest = compared;
        while let Some(at) = rest.find("grund.toml:") {
            rest = &rest[at + "grund.toml:".len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(line) = digits.parse::<u128>() {
                assert!(
                    (1..=longest as u128).contains(&line),
                    "grund.toml:{line} is a line of neither file (at most {longest}):\n{stderr}"
                );
            }
        }
    }
}
