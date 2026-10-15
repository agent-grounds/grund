//! Test module: the rule that keeps the walk from a stub's target
//! (§FS-declarations.stubs.target.3, §AR-scanner.1.13).

use std::path::PathBuf;

use super::read_predicate::excluding_rule;
use crate::config::{Config, load_config};
use crate::model::ExcludingRule;
use crate::testing::{test_root, write};

/// A project whose `[scan]` table is `scan`, with `notes/gen/a.md` on disk.
fn project(name: &str, scan: &str) -> Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
             [scan]\n{scan}\n"
        ),
    );
    write(&root.join("notes/gen/a.md"), "# FS-x: X\n");
    load_config(&root).expect("load config")
}

fn rule(config: &Config, roots: &[&str], relative: &str) -> Option<ExcludingRule> {
    let root = config.frame().root();
    let roots: Vec<PathBuf> = roots.iter().map(|path| root.join(path)).collect();
    excluding_rule(
        config.schema(),
        config.frame(),
        &roots,
        &root.join(relative),
    )
}

#[test]
fn an_exclude_entry_on_the_path_is_named_as_written() {
    let config = project("excluding_rule_entry", "exclude = [\"gen\"]");
    assert_eq!(
        rule(&config, &["docs"], "notes/gen/a.md"),
        Some(ExcludingRule::Exclude("gen".to_string()))
    );
}

#[test]
fn an_ignore_file_is_named_from_the_config_root_with_its_line() {
    let config = project("excluding_rule_ignore", "respect_gitignore = true");
    write(&config.frame().root().join("notes/.ignore"), "gen/\n");
    assert_eq!(
        rule(&config, &["docs"], "notes/gen/a.md"),
        Some(ExcludingRule::IgnoreFile {
            file: "notes/.ignore".to_string(),
            pattern: "gen/".to_string(),
        })
    );
}

#[test]
fn at_one_directory_the_exclude_entry_comes_before_the_ignore_file() {
    let config = project(
        "excluding_rule_precedence",
        "exclude = [\"gen\"]\nrespect_gitignore = true",
    );
    write(&config.frame().root().join(".ignore"), "notes/gen/\n");
    assert_eq!(
        rule(&config, &["docs"], "notes/gen/a.md"),
        Some(ExcludingRule::Exclude("gen".to_string()))
    );
}

#[test]
fn an_ignore_file_is_no_reason_where_the_walk_does_not_respect_it() {
    let config = project("excluding_rule_no_respect", "respect_gitignore = false");
    write(&config.frame().root().join(".ignore"), "notes/gen/\n");
    assert_eq!(rule(&config, &["docs"], "notes/gen/a.md"), None);
}

#[test]
fn a_scan_root_below_the_excluded_name_outruns_it() {
    let config = project("excluding_rule_root_outruns", "exclude = [\"gen\"]");
    assert_eq!(rule(&config, &["notes/gen"], "notes/gen/a.md"), None);
}

#[test]
fn a_hidden_directory_is_no_reason_in_this_release() {
    let config = project("excluding_rule_hidden", "exclude = [\".gen\"]");
    write(
        &config.frame().root().join("notes/.gen/a.md"),
        "# FS-x: X\n",
    );
    assert_eq!(rule(&config, &["docs"], "notes/.gen/a.md"), None);
}

#[test]
fn a_target_outside_the_config_root_is_no_reason() {
    let config = project("excluding_rule_outside", "exclude = [\"gen\"]");
    let outside = test_root("excluding_rule_outside_target").join("gen/a.md");
    write(&outside, "# FS-x: X\n");
    let root = config.frame().root();
    assert_eq!(
        excluding_rule(
            config.schema(),
            config.frame(),
            &[root.join("docs")],
            &outside
        ),
        None
    );
}

/// §FS-declarations.stubs.target.3: a root outruns rules only under the spelling
/// walked, not through every alias of the same physical directory.
#[cfg(unix)]
#[test]
fn an_explicit_root_does_not_clear_a_differently_filtered_alias() {
    for ignore in [false, true] {
        let config = project(
            &format!("excluding_rule_alias_root_{ignore}"),
            if ignore {
                "respect_gitignore = true"
            } else {
                "exclude = [\"gen\"]"
            },
        );
        let root = config.frame().root();
        std::os::unix::fs::symlink("gen", root.join("notes/readable")).unwrap();
        if ignore {
            write(&root.join(".ignore"), "notes/gen/\n");
        }
        assert!(rule(&config, &["notes/readable"], "notes/gen/a.md").is_some());
        assert_eq!(rule(&config, &["notes/gen"], "notes/gen/a.md"), None);
        assert_eq!(rule(&config, &["docs"], "notes/readable/a.md"), None);
    }
}
