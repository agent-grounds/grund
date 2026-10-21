//! §FS-check.3.18.5.2 — the entry a not-listed `missing-index-entry` prints is
//! the repair: pasted into the index, it clears the finding, raises no
//! `unlinked-index-entry` (§FS-check.3.17), and the next `grund fmt --write`
//! leaves it alone, because it is the link `fmt --cross-refs` would write.
//!
//! The e2e goldens pin the words; they cannot show the words are *true*. This
//! takes the literal out of the message the real binary printed and writes it
//! back, under both `strict` settings and for an index file that does not exist
//! yet (§FS-check.3.18.7), whose repair is a file holding that entry.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// A project whose `FS` kind is a folder with the default `README.md` index
/// (§FS-config.3.4) and two declarations, one of them listed. `index` is the
/// index file's whole text, or `None` for a folder with no index at all.
fn build_fixture(name: &str, strict: bool, index: Option<&str>) -> PathBuf {
    let dir = manifest_dir()
        .join("target/index-literal-round-trip")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("docs/specs")).expect("create specs dir");
    fs::write(
        dir.join("grund.toml"),
        format!(
            "grund_config_version = 1\nproject_name = \"literal-round-trip\"\n\n\
             [reference]\nmarker = \"§\"\nstrict = {strict}\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\n\n\
             [scan]\ninclude = [\"docs\"]\n\n\
             [fmt.cross_refs]\nenabled = true\nanchor_format = \"github\"\n"
        ),
    )
    .expect("write grund.toml");
    fs::write(
        dir.join("docs/specs/FS-001-login.md"),
        "# FS-001-login: a user logs in\n\nLead.\n",
    )
    .expect("write first declaration");
    fs::write(
        dir.join("docs/specs/FS-002-session.md"),
        "# FS-002-session: the session a login opens\n\nLead.\n",
    )
    .expect("write second declaration");
    if let Some(index) = index {
        fs::write(dir.join("docs/specs/README.md"), index).expect("write README.md");
    }
    dir
}

/// The entries the not-listed findings print, in order: the text between
/// "; write `" and "` in " on each `missing-index-entry` line.
fn printed_entries(check: &str) -> Vec<String> {
    check
        .lines()
        .filter(|line| line.contains(" is not listed in "))
        .map(|line| {
            let (_, rest) = line.split_once("; write `").unwrap_or_else(|| {
                panic!("§FS-check.3.18.5.2: a not-listed finding names no entry: {line}")
            });
            let (entry, _) = rest
                .split_once("` in docs/specs/README.md")
                .unwrap_or_else(|| panic!("the entry is not said to go in the index: {line}"));
            entry.to_string()
        })
        .collect()
}

/// The `missing-index-entry` and `unlinked-index-entry` findings a JSON `check`
/// reports, read by code: the unused warning's explanation of a discounted
/// index entry (§FS-check.4.1.2) also says "index" and is not one of them.
fn index_findings(dir: &Path, case: &str) -> Vec<String> {
    let output = run_grund(&["check", "--format", "json", "."], dir);
    assert_eq!(output.status.code(), Some(0), "{case}: JSON check failed");
    stdout(&output)
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let finding: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|err| panic!("{case}: not a JSON finding ({err}): {line}"));
            let code = finding["code"]
                .as_str()
                .unwrap_or_else(|| panic!("{case}: a JSON finding carries no code: {line}"));
            matches!(code, "missing-index-entry" | "unlinked-index-entry").then(|| line.to_string())
        })
        .collect()
}

/// Run `check`, paste every printed entry into the index as its own list item,
/// and require a clean second `check` and an index `fmt --write` leaves alone.
fn assert_printed_entries_repair(dir: &Path, case: &str, expected: usize) {
    let before = stdout(&run_grund(&["check", "."], dir));
    let entries = printed_entries(&before);
    assert_eq!(entries.len(), expected, "{case}: {before}");

    let index = dir.join("docs/specs/README.md");
    let mut text = fs::read_to_string(&index).unwrap_or_else(|_| "# Specs\n\n".to_string());
    for entry in &entries {
        text.push_str(&format!("- {entry}\n"));
    }
    fs::write(&index, &text).expect("write the printed entries");

    let after = run_grund(&["check", "."], dir);
    assert_eq!(
        after.status.code(),
        Some(0),
        "{case}: the printed entry did not repair the index: {}",
        stdout(&after)
    );
    let left = index_findings(dir, case);
    assert!(
        left.is_empty(),
        "{case}: the printed entry left an index finding: {left:?}"
    );

    let fmt = run_grund(&["fmt", "--write", "."], dir);
    assert_eq!(fmt.status.code(), Some(0), "{case}: fmt failed");
    assert_eq!(
        fs::read_to_string(&index).expect("read index"),
        text,
        "{case}: the printed entry is not the link `fmt` writes: {}",
        stdout(&fmt)
    );
}

const LISTS_LOGIN: &str =
    "# Specs\n\n- [§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in)\n";

#[test]
fn the_printed_entry_repairs_the_index_under_strict_mode() {
    let dir = build_fixture("strict", true, Some(LISTS_LOGIN));
    assert_printed_entries_repair(&dir, "strict = true", 1);
}

#[test]
fn the_printed_entry_repairs_the_index_off_strict_mode() {
    let dir = build_fixture("loose", false, Some(LISTS_LOGIN));
    assert_printed_entries_repair(&dir, "strict = false", 1);
}

#[test]
fn the_printed_entries_are_the_missing_index_file() {
    let dir = build_fixture("no_index", true, None);
    assert_printed_entries_repair(&dir, "no index file", 2);
}
