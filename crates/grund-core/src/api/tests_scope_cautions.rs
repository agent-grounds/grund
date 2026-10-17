//! The empty-scan caution's hidden-file branch (§FS-check.2.2.2, §AR-scanner.1.13),
//! and how it names the scope it read (§FS-workspace.5.3, §FS-config.3.6).

use super::scope_cautions::{display_scope, empty_scan_warning};
use crate::config::{Config, load_config_at};
use crate::testing::{test_root, write};
use std::path::Path;

/// A project at `name` whose `[scan] extensions` is `extensions`, holding a
/// hidden `.notes.txt`, loaded from its own `grund.toml`.
fn hidden_notes_project(name: &str, extensions: &str) -> Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!("[scan]\nextensions = [{extensions}]\n"),
    );
    write(&root.join(".notes.txt"), "# Notes\n");
    load_config_at(&root, &root).expect("load the project")
}

/// §FS-check.2.2.2, §AR-scanner.1.13: the hidden-file message is the read
/// predicate's answer, so it follows the extension list the run was given: a
/// hidden `.txt` file is named hidden once `txt` is listed, and keeps the
/// extension message, the second of its two reasons, while it is not.
#[test]
fn the_handed_hidden_file_branch_asks_the_read_predicate() {
    let unlisted = hidden_notes_project("hidden_file_branch_unlisted", "\"md\"");
    let hidden = unlisted.root.join(".notes.txt");
    let message = empty_scan_warning(&unlisted, &hidden, true, unlisted.display()).message;
    assert!(message.contains("matched grund's extensions"), "{message}");
    let listed = hidden_notes_project("hidden_file_branch_listed", "\"md\", \"txt\"");
    let hidden = listed.root.join(".notes.txt");
    let message = empty_scan_warning(&listed, &hidden, true, listed.display()).message;
    assert!(message.contains("is a hidden file"), "{message}");
}

/// §FS-workspace.5.3, §FS-config.3.6: the empty-scan caution never names its
/// scope by an absolute path: one under the report base is spelled from it, and
/// the base itself reads `.` — the single-project absolute argument and the
/// workspace root no golden can carry portably.
#[test]
fn the_empty_scan_caution_names_an_absolute_scope_from_the_report_base() {
    let config = hidden_notes_project("empty_scan_names_from_base", "\"md\"");
    std::fs::create_dir_all(config.root.join("empty")).expect("make the empty dir");
    let empty = config.root.join("empty");
    let message = empty_scan_warning(&config, &empty, true, config.display()).message;
    assert!(
        message.contains("no files under `empty` matched"),
        "{message}"
    );
    assert_eq!(display_scope(config.display(), &config.root), ".");
    assert_eq!(
        display_scope(config.display(), Path::new("typed/dir")),
        "typed/dir"
    );
}

/// §FS-workspace.5.3, §FS-config.3.6: a scope typed through a symlink to the
/// project is matched to the report base by canonical ancestor, so it is
/// spelled relatively like the findings beside it, not printed absolute.
#[cfg(unix)]
#[test]
fn the_empty_scan_caution_names_a_symlinked_scope_from_the_report_base() {
    let config = hidden_notes_project("empty_scan_names_through_link", "\"md\"");
    std::fs::create_dir_all(config.root.join("empty")).expect("make the empty dir");
    let link = config.root.with_extension("link");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&config.root, &link).expect("link the project");
    let message = empty_scan_warning(&config, &link.join("empty"), true, config.display()).message;
    assert!(
        message.contains("no files under `empty` matched"),
        "{message}"
    );
    assert_eq!(display_scope(config.display(), &link), ".");
    std::fs::remove_file(&link).expect("unlink the project");
}
