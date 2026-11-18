//! The spelling of migrate's undo and re-preview (§FS-config-migrate.6.5.1):
//! one POSIX shell word per path, a leading `-` defused, `PATH` as given.
//! Where the config lies relative to the working directory — `..`, links, the
//! absolute fallback — is the core's `MigrateOpts::config_from`, tested there.

use super::*;

/// The two commands' variable parts for a config spelled `restore` from the
/// working directory and `given` as the `PATH` argument.
fn spelled(restore: &str, given: Option<&str>) -> (String, String) {
    let commands = MigrateCommands::spelled(restore, given);
    (commands.restore, commands.path)
}

/// §FS-config-migrate.6.5.1 item 5: at the project root with no `PATH`, the
/// bytes §FS-config-migrate.6.5 shows, the deprecated location included.
#[test]
fn the_project_root_keeps_its_bytes() {
    assert_eq!(spelled("grund.toml", None), ("grund.toml".into(), String::new()));
    assert_eq!(
        spelled(".agents/grund.toml", None),
        (".agents/grund.toml".into(), String::new())
    );
}

/// §FS-config-migrate.6.5.1 items 1-2: a relative spelling, `..` included, and
/// the `PATH` exactly as typed, trailing `/` and leading `./` kept.
#[test]
fn paths_of_safe_characters_stay_bare() {
    assert_eq!(
        spelled("apps/api/grund.toml", Some("apps/api")),
        ("apps/api/grund.toml".into(), " apps/api".into())
    );
    assert_eq!(
        spelled("../grund.toml", None),
        ("../grund.toml".into(), String::new())
    );
    assert_eq!(
        spelled("./apps/api/grund.toml", Some("./apps/api/")),
        ("./apps/api/grund.toml".into(), " ./apps/api/".into())
    );
    let every_safe = "AZaz09_-./,:@%+=";
    assert_eq!(shell_word(every_safe), every_safe);
}

/// §FS-config-migrate.6.5.1 item 3: anything outside the safe set, the empty
/// word included, is single-quoted, and a `'` inside becomes `'\''`.
#[test]
fn other_paths_are_single_quoted() {
    assert_eq!(
        spelled("my app/grund.toml", Some("my app")),
        ("'my app/grund.toml'".into(), " 'my app'".into())
    );
    assert_eq!(
        spelled("it's/grund.toml", Some("it's")),
        (r"'it'\''s/grund.toml'".into(), r" 'it'\''s'".into())
    );
    assert_eq!(shell_word(""), "''");
    for unsafe_char in ["$HOME", "a*b", "a;b", "~/x", "a\"b", "a\\b", "é", "a\tb", "!x"] {
        let word = shell_word(unsafe_char);
        assert!(
            word.starts_with('\'') && word.ends_with('\''),
            "{unsafe_char:?} -> {word}"
        );
    }
}

/// §FS-config-migrate.6.5.1 item 3: an undo path that would start with `-`
/// gets `./`, quoted after it when it must be; `PATH` is never rewritten.
#[test]
fn a_leading_dash_is_not_an_option() {
    assert_eq!(
        spelled("-x/grund.toml", Some("./-x")),
        ("./-x/grund.toml".into(), " ./-x".into())
    );
    assert_eq!(
        spelled("-my x/grund.toml", None),
        ("'./-my x/grund.toml'".into(), String::new())
    );
}

/// §FS-config-migrate.6.5.1 item 1: the absolute path, where the core finds no
/// relative spelling, is written like any other path.
#[test]
fn an_absolute_path_is_one_word_too() {
    assert_eq!(
        spelled("/srv/project/grund.toml", None).0,
        "/srv/project/grund.toml"
    );
    assert_eq!(spelled("D:/project/grund.toml", None).0, "D:/project/grund.toml");
    assert_eq!(
        spelled("/srv/my project/grund.toml", None).0,
        "'/srv/my project/grund.toml'"
    );
}
