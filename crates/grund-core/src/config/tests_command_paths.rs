//! Test module: how a command for the user to paste spells a path
//! (§FS-config-migrate.6.5.1) — from the working directory, links resolved,
//! `..` where it climbs, empty or absent at the directory itself, discovery's
//! path without a working directory, and one POSIX shell word with `./` before
//! a leading `-` (and `:`, for `git`).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use std::path::Path;

use super::*;
use crate::testing::{test_root, write};

/// §FS-config-migrate.6.5.1 item 1: the working directory itself is the empty
/// path, which a directory argument leaves out.
#[test]
fn the_working_directory_is_the_empty_path() {
    let root = test_root("command-path-here");
    write(&root.join("grund.toml"), "");
    assert_eq!(command_path(Some(&root), &root), "");
    assert_eq!(command_directory(Some(&root), &root), None);
    assert_eq!(
        command_path(Some(&root), &root.join("grund.toml")),
        "grund.toml"
    );
}

/// §FS-config-migrate.6.5.1 item 1: a target below is named through the
/// directory, one above or beside it is reached with `..`.
#[test]
fn a_target_elsewhere_is_named_from_the_directory() {
    let root = test_root("command-path-elsewhere");
    write(&root.join("apps/api/grund.toml"), "");
    write(&root.join("docs/x.md"), "");
    let docs = root.join("docs");
    let api = root.join("apps/api");
    assert_eq!(command_path(Some(&root), &api), "apps/api");
    assert_eq!(command_path(Some(&docs), &root), "..");
    assert_eq!(command_path(Some(&docs), &api), "../apps/api");
    assert_eq!(
        command_directory(Some(&docs), &api).as_deref(),
        Some("../apps/api")
    );
    assert_eq!(command_path(Some(&api), &docs), "../../docs");
}

/// §FS-config-migrate.6.5.1 item 1: a link on either side is resolved before
/// the two are compared, so it adds no `..`, and a target reached through a
/// link is named where it is.
#[cfg(unix)]
#[test]
fn links_on_either_side_are_resolved() {
    let root = test_root("command-path-links");
    write(&root.join("project/docs/x.md"), "");
    write(&root.join("services/api/grund.toml"), "");
    std::fs::create_dir_all(root.join("far/away")).unwrap();
    let docs = root.join("far/away/docs");
    std::os::unix::fs::symlink(root.join("project/docs"), &docs).unwrap();
    assert_eq!(command_path(Some(&docs), &root.join("project")), "..");

    std::fs::create_dir_all(root.join("apps")).unwrap();
    std::os::unix::fs::symlink("../services/api", root.join("apps/api")).unwrap();
    assert_eq!(
        command_path(Some(&root), &root.join("apps/api")),
        "services/api"
    );
    assert_eq!(
        command_path(Some(&root), &root.join("apps/api/grund.toml")),
        "services/api/grund.toml"
    );
}

/// §FS-config-migrate.6.5.1 item 1: a target that is itself a link to a file
/// keeps its own name, the file git tracks, while its directory resolves.
#[cfg(unix)]
#[test]
fn a_file_link_is_not_followed() {
    let root = test_root("command-path-file-link");
    write(&root.join("real.toml"), "");
    std::os::unix::fs::symlink("real.toml", root.join("grund.toml")).unwrap();
    assert_eq!(
        command_path(Some(&root), &root.join("grund.toml")),
        "grund.toml"
    );
}

/// §FS-config-migrate.6.5.1 item 1: with no working directory, the target as
/// the caller reached it, formatted with `/`, relative or absolute as given.
#[test]
fn no_working_directory_keeps_the_path_as_reached() {
    assert_eq!(command_path(None, Path::new("ws/apps/api")), "ws/apps/api");
    assert_eq!(command_path(None, Path::new("../sub")), "../sub");
    assert_eq!(command_path(None, Path::new("/srv/sub")), "/srv/sub");
    assert_eq!(
        command_directory(None, Path::new("my app")).as_deref(),
        Some("'my app'")
    );
}

/// §FS-config-migrate.6.5.1 item 3: a directory argument that would start with
/// `-` gets `./`, and any other unsafe character is quoted.
#[test]
fn a_directory_argument_is_one_word_that_reads_as_a_path() {
    let root = test_root("command-path-words");
    for dir in ["-x", "my app", "-my x"] {
        write(&root.join(dir).join("grund.toml"), "");
    }
    let word = |dir: &str| command_directory(Some(&root), &root.join(dir)).unwrap();
    assert_eq!(word("-x"), "./-x");
    assert_eq!(word("my app"), "'my app'");
    assert_eq!(word("-my x"), "'./-my x'");
}

/// §FS-config-migrate.6.5.1 item 3: a directory argument that starts with `:`
/// stays as it is, since grund reads no pathspec magic. Unix only: `:` cannot
/// be in a Windows file name.
#[cfg(unix)]
#[test]
fn a_leading_colon_directory_argument_stays_as_it_is() {
    let root = test_root("command-path-colon");
    write(&root.join(":x").join("grund.toml"), "");
    assert_eq!(
        command_directory(Some(&root), &root.join(":x")).as_deref(),
        Some(":x")
    );
}

/// §FS-config-migrate.6.5.1 item 3: the undo defuses a leading `:` as well,
/// which `git` would read as pathspec magic; only a leading one is.
#[test]
fn a_path_word_defuses_only_the_leading_characters_it_is_given() {
    let undo = ['-', ':'];
    assert_eq!(path_word("-x/grund.toml", &undo), "./-x/grund.toml");
    assert_eq!(path_word(":x/grund.toml", &undo), "./:x/grund.toml");
    assert_eq!(path_word(":my x/grund.toml", &undo), "'./:my x/grund.toml'");
    assert_eq!(path_word("x:/grund.toml", &undo), "x:/grund.toml");
    assert_eq!(path_word(":x", &['-']), ":x");
    assert_eq!(path_word("../grund.toml", &undo), "../grund.toml");
    assert_eq!(
        path_word("-x", &[]),
        "-x",
        "with nothing to defuse, as given"
    );
    assert_eq!(path_word("my app", &[]), "'my app'");
}

/// §FS-config-migrate.6.5.1 item 3: a path made only of the safe characters is
/// written as it is.
#[test]
fn safe_characters_stay_bare() {
    let every_safe = "AZaz09_-./,:@%+=";
    assert_eq!(shell_word(every_safe), every_safe);
    assert_eq!(shell_word("apps/api"), "apps/api");
    assert_eq!(shell_word("D:/project/grund.toml"), "D:/project/grund.toml");
}

/// §FS-config-migrate.6.5.1 item 3: anything outside the safe set, the empty
/// word included, is single-quoted, and a `'` inside becomes `'\''`.
#[test]
fn other_words_are_single_quoted() {
    assert_eq!(shell_word("my app"), "'my app'");
    assert_eq!(shell_word("it's"), r"'it'\''s'");
    assert_eq!(shell_word(""), "''");
    for unsafe_text in [
        "$HOME", "a*b", "a;b", "~/x", "a\"b", "a\\b", "é", "a\tb", "!x",
    ] {
        let word = shell_word(unsafe_text);
        assert!(
            word.starts_with('\'') && word.ends_with('\''),
            "{unsafe_text:?} -> {word}"
        );
    }
}
