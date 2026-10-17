//! The empty-scan caution's hidden-file branch (§FS-check.2.2.2, §AR-scanner.1.13).

use super::scope_cautions::empty_scan_warning;
use crate::config::{Config, load_config_at};
use crate::testing::{test_root, write};

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
