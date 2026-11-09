//! A `fetch` key that names no executable (§FS-fetch.2.1): the one test of the value, and the
//! key's name and repair every site that reports it prints. It reads nothing but the string,
//! so a reader of the key asks it without touching the disk.

use std::path::Path;

use super::discovery::config_file_under;
use super::record::Config;
use crate::model::{format_path, relative_from_base, remedies};

/// §FS-fetch.2.1: whether a `fetch` value names no executable, being empty or only whitespace.
pub(crate) fn names_no_executable(value: &str) -> bool {
    value.trim().is_empty()
}

/// §FS-fetch.2.1: whether `grund fetch` has nothing to run for this value, its one question of
/// the disk: a blank value still runs a file of that very name at the config root.
pub(crate) fn fetch_runs_nothing(value: &str, integration: &Path) -> bool {
    names_no_executable(value) && !integration.is_file()
}

/// §FS-fetch.2.1: the key as a message names it, in the spelling of the version that wrote it.
pub(crate) fn fetch_key_name(version: u32, kind: &str) -> String {
    if version == 1 {
        format!("[[kinds]] `{kind}` fetch")
    } else {
        format!("`[schema.kinds.{kind}] fetch`")
    }
}

/// §FS-fetch.2.1: the one repair, from the *Fix* template of
/// §FS-config-v2.checks.config-fetch-names-nothing. Only the setting that needs the key differs
/// by version, and version 1's is `resolve`.
pub(crate) fn fetch_key_repair(version: u32, kind: &str) -> String {
    let repair = remedies::config_fetch_names_nothing(format!("`[rules.resolution] {kind}`"));
    if version == 1 {
        repair.replace(&format!("`[rules.resolution] {kind}`"), "`resolve`")
    } else {
        repair
    }
}

// Scaffold for #1322's specify commit, where the catalog's case is written ahead of the findings
// that give it (§FS-check.3.1.3.2): implement removes it when `check` calls these two.
const _: [fn(String) -> String; 2] = [
    remedies::dangling_fetch_names_nothing,
    remedies::missing_snapshot_fetch_names_nothing,
];

/// §FS-fetch.2.1: what `grund fetch` says in place of the operating system's reason: the key's
/// line in its config file, as a report path from `report_base`, what is wrong and the repair.
pub(crate) fn fetch_names_nothing_reason(
    config: &Config,
    kind: &str,
    report_base: &Path,
) -> String {
    let project = config.project();
    let source = project
        .schema
        .rows
        .iter()
        .find(|row| row.name == kind)
        .and_then(|row| row.fetch_key())
        .and_then(|(_, source)| source);
    let place = source.map_or_else(String::new, |at| {
        let file = config_file_under(&config.root, &at.path);
        let file = format_path(&relative_from_base(report_base, &file));
        format!("{file}:{}: ", at.line)
    });
    format!(
        "{place}{} names no executable; {}",
        fetch_key_name(project.version, kind),
        fetch_key_repair(project.version, kind)
    )
}

#[cfg(test)]
mod tests {
    use super::{fetch_key_repair, fetch_runs_nothing, names_no_executable};

    /// §FS-fetch.2.1: empty and whitespace-only values name nothing, and any other character
    /// names what it spells, stray spaces and all.
    #[test]
    fn only_an_empty_or_whitespace_value_names_no_executable() {
        for value in ["", " ", "   ", "\t", " \t "] {
            assert!(names_no_executable(value), "{value:?}");
        }
        for value in ["scripts/fetch", " scripts/fetch "] {
            assert!(!names_no_executable(value), "{value:?}");
        }
    }

    /// §FS-fetch.2.1: a blank value runs nothing unless a file stands where it would run.
    #[test]
    fn a_blank_value_has_nothing_to_run_unless_a_file_stands_there() {
        let file = std::env::current_exe().expect("the test binary");
        let nothing = file.with_extension("absent");
        assert!(fetch_runs_nothing(" ", &nothing));
        assert!(!fetch_runs_nothing(" ", &file));
        assert!(!fetch_runs_nothing("scripts/fetch", &nothing));
    }

    /// §FS-check.3.1.3.2: the repair's bytes, its setting spelled by the version that wrote it.
    #[test]
    fn the_repair_names_the_setting_of_its_version() {
        let tail = "name the program, or if this kind is not fetched remove the key and any";
        let errors = "and its missing IDs are then errors";
        assert_eq!(
            fetch_key_repair(1, "AR"),
            format!("{tail} `resolve`, {errors}")
        );
        assert_eq!(
            fetch_key_repair(2, "AR"),
            format!("{tail} `[rules.resolution] AR`, {errors}")
        );
    }
}
