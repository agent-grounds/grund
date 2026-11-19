//! The spelling of migrate's undo and re-preview (§FS-config-migrate.6.5.1):
//! one POSIX shell word per path, a leading `-` or `:` defused, `PATH` as given.
//! Where the config lies relative to the working directory — `..`, links, the
//! absolute fallback — is the core's `MigrateOpts::config_from`, tested there,
//! and so is the shell word itself, the core's `path_word`.

use super::*;

/// The two commands' variable parts for a config spelled `restore` from the
/// working directory and `given` as the `PATH` argument.
fn spelled(restore: &str, given: Option<&str>) -> (String, String) {
    let commands = MigrateCommands::spelled(Some(restore), given);
    (commands.restore.unwrap(), commands.path)
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
}

/// §FS-config-migrate.6.5.1 item 3: a path outside the safe set is
/// single-quoted, and a `'` inside becomes `'\''`.
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

/// §FS-config-migrate.6.5.1 item 3: an undo path that would start with `:` gets
/// `./` too, since `git` reads a leading `:` as pathspec magic and would restore
/// another file; grund reads no such magic, so `PATH` stays as given.
#[test]
fn a_leading_colon_is_not_pathspec_magic() {
    assert_eq!(
        spelled(":x/grund.toml", Some(":x")),
        ("./:x/grund.toml".into(), " :x".into())
    );
    assert_eq!(
        spelled(":my x/grund.toml", Some(":my x")),
        ("'./:my x/grund.toml'".into(), " ':my x'".into())
    );
    assert_eq!(
        spelled("x:/grund.toml", None).0,
        "x:/grund.toml",
        "only a leading `:` is magic"
    );
}

/// §FS-config-migrate.6.5.1 item 1: where discovery no longer finds a config,
/// the undo names no file; it is never spelled from the report base.
#[test]
fn no_config_found_prints_no_undo() {
    let commands = MigrateCommands::spelled(None, Some("apps/api"));
    let report = MigrateReport {
        wrote: true,
        ..MigrateReport::default()
    };
    let mut sink = Vec::new();
    render_migrate_advice(&report, &commands, &mut sink).unwrap();
    let advice = String::from_utf8(sink).unwrap();
    assert!(!advice.contains("git restore"), "{advice}");
    assert!(
        advice.contains("run grund config migrate --to 2 apps/api\n"),
        "{advice}"
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
