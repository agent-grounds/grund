//! Test module: what `home_requirement` asks of each home (§AR-config.1.6), per form and per
//! reading — a rule kind's homes by how the configuration reads them
//! (§FS-config-v2.schema.sources.rules), a value kind's by whether they exist
//! (§FS-config.3.4.9) — for `files` against `folders` entries, and the first entry against a
//! later one.

use super::*;
use crate::config::load_config;
use crate::config::project::Sources;
use crate::testing::{TemporaryRoot, test_root, write};

/// The sources a project with `head` (a `grund.toml` up to its kinds) reads, at its root.
fn project(name: &str, toml: &str) -> (TemporaryRoot, Sources) {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    let config = load_config(&root).expect("the control project loads");
    (root, config.schema().sources.clone())
}

fn v2(languages: &str, extensions: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"g1206\"\n[schema.sources]\n\
         languages = [{languages}]\n{extensions}"
    )
}

const V1: &str = "grund_config_version = 1\n[scan]\ninclude = [\"docs\"]\n";

fn kind(rules: bool) -> KindConfig {
    KindConfig {
        kind: "K".to_string(),
        folder: None,
        file: None,
        title: None,
        index: crate::config::KindIndex::Default,
        citable: true,
        scan: true,
        require_grounding: None,
        grounding_level: None,
        values: !rules,
        value_chapter: None,
        rules,
        format: None,
        resolve: None,
        fetch: None,
    }
}

fn home(path: &str, file: bool, first: bool) -> Home<'_> {
    Home {
        path,
        file,
        first,
        line: 7,
    }
}

fn warns(verdict: &Verdict, code: Code) -> bool {
    matches!(verdict, Verdict::Warns(warned, _) if *warned == code)
}

/// §FS-config-v2.schema.sources.rules: a rule `files` entry by its reading, in any position — a
/// `document` language holds, a `source` one warns, a `data` one or none refuses the row.
#[test]
fn rule_file_homes_by_reading_in_every_position() {
    let toml = v2(
        "\"markdown\", \"rust\", \"json\"",
        "[schema.sources.extensions]\ntxt = \"markdown\"\n",
    );
    let (root, sources) = project("home_requirement_rule_files", &toml);
    let k = kind(true);
    for first in [true, false] {
        let judge =
            |path| home_requirement(Ask::Rule, &k, home(path, true, first), &root, &sources);
        assert!(matches!(judge("r/a.md"), Verdict::Holds));
        assert!(matches!(judge("r/a.txt"), Verdict::Holds));
        assert!(warns(&judge("r/a.rs"), Code::CONFIG_RULE_FILE_NOT_MARKDOWN));
        assert!(matches!(judge("r/a.json"), Verdict::RefusesRow));
        assert!(matches!(judge("r/a.unclaimed"), Verdict::RefusesRow));
    }
}

/// §FS-config-v2.checks.config-rule-folder-not-markdown: a rule `folders` entry holds while any
/// admitted name reads Markdown, and warns where none can, as the first entry or a later one.
#[test]
fn rule_folder_homes_by_whether_any_name_reads_markdown() {
    let k = kind(true);
    let (root, sources) = project("home_requirement_rule_folder", &v2("\"markdown\"", ""));
    let (bare_root, bare) = project("home_requirement_rule_folder_bare", &v2("\"rust\"", ""));
    for first in [true, false] {
        let folder = home("rules", false, first);
        assert!(matches!(
            home_requirement(Ask::Rule, &k, folder, &root, &sources),
            Verdict::Holds
        ));
        assert!(warns(
            &home_requirement(Ask::Rule, &k, folder, &bare_root, &bare),
            Code::CONFIG_RULE_FOLDER_NOT_MARKDOWN
        ));
    }
}

/// §FS-config.3.4.12: version 1 reads a rule `file` by its name, and accepts a folder on its
/// presence, so its verdicts do not move.
#[test]
fn rule_homes_in_version_1_read_the_name() {
    let (root, sources) = project("home_requirement_rule_v1", V1);
    let k = kind(true);
    let judge =
        |path, file| home_requirement(Ask::Rule, &k, home(path, file, true), &root, &sources);
    assert!(matches!(judge("r/a.md", true), Verdict::Holds));
    assert!(matches!(judge("r/a.rs", true), Verdict::RefusesRow));
    assert!(matches!(judge("rules", false), Verdict::Holds));
}

/// §FS-config.3.4.9, §FS-config-v2.checks.config-value-home-absent: an absent first value home is
/// refused and a later one warns, for a file and a folder alike; one that exists holds.
#[test]
fn value_homes_first_against_later() {
    let (root, sources) = project("home_requirement_value", &v2("\"markdown\"", ""));
    write(&root.join("values/a.md"), "");
    let k = kind(false);
    let judge = |path, file, first| {
        home_requirement(Ask::Value, &k, home(path, file, first), &root, &sources)
    };
    for (path, file) in [("values/absent.md", true), ("absent", false)] {
        assert!(
            matches!(judge(path, file, true), Verdict::Refused(_)),
            "{path}"
        );
        assert!(
            warns(&judge(path, file, false), Code::CONFIG_VALUE_HOME_ABSENT),
            "{path}"
        );
    }
    assert!(matches!(judge("values/a.md", true, false), Verdict::Holds));
    assert!(matches!(judge("values", false, true), Verdict::Holds));
    // §FS-config-v2.checks.config-value-home-invalid: a later home in the wrong shape warns, the
    // first is refused.
    assert!(matches!(judge("values", true, true), Verdict::Refused(_)));
    assert!(warns(
        &judge("values", true, false),
        Code::CONFIG_VALUE_HOME_INVALID
    ));
    assert!(warns(
        &judge("values/a.md", false, false),
        Code::CONFIG_VALUE_HOME_INVALID
    ));
}

/// §FS-config-v2.checks.config-value-home-invalid: a later value home that cannot be looked up
/// warns, and the walk reads it as before; the first is refused with the hint.
#[cfg(unix)]
#[test]
fn value_home_lookup_failure_first_against_later() {
    use std::os::unix::fs::PermissionsExt;
    let (root, sources) = project("home_requirement_value_lookup", &v2("\"markdown\"", ""));
    write(&root.join("values/locked/b.json"), "{}");
    let locked = root.join("values/locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let k = kind(false);
    let judge = |first| {
        home_requirement(
            Ask::Value,
            &k,
            home("values/locked/b.json", true, first),
            &root,
            &sources,
        )
    };
    // A user the mode does not stop (root) can look it up, so there is nothing to judge.
    let blocked = std::fs::canonicalize(locked.join("b.json")).is_err();
    let (first, later) = (judge(true), judge(false));
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    if !blocked {
        return;
    }
    assert!(matches!(&first, Verdict::Refused(message) if message.contains("\nhint: ")));
    assert!(warns(&later, Code::CONFIG_VALUE_HOME_INVALID));
}
