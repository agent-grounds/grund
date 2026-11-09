//! A `fetch` key that names no executable (§FS-fetch.2.1): the one test of the value, and the
//! key's name and repair every site that reports it prints. The test reads nothing but the
//! string; only whether a blank value still runs a file of that name asks the disk.

use std::path::{Path, PathBuf};

use super::discovery::config_file_under;
use super::observe_input;
use super::record::Config;
use crate::model::{format_path, relative_from_base, remedies};

/// §FS-fetch.2.1: whether a `fetch` value names no executable, being empty or only whitespace.
pub(crate) fn names_no_executable(value: &str) -> bool {
    value.trim().is_empty()
}

/// §FS-fetch.2.1: the path `grund fetch` runs for this value, from the project's config root
/// unless it is absolute. `check` asks of this same path before it names the command.
pub(crate) fn fetch_integration_path(root: &Path, value: &str) -> PathBuf {
    let configured = Path::new(value);
    if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        root.join(configured)
    }
}

/// §FS-fetch.2.1: whether `grund fetch` has nothing to run for this value, its one question of
/// the disk: a blank value still runs a file of that very name at the config root.
pub(crate) fn fetch_runs_nothing(value: &str, integration: &Path) -> bool {
    names_no_executable(value) && !integration.is_file()
}

/// §FS-fetch.2.1: whether a reader about to name `grund fetch` for the project at `root` would
/// name a command with nothing to run. The one file asked of is observed first, so `--watch`
/// reruns when it appears or goes (§FS-check.6.1.3).
pub(crate) fn fetch_key_runs_nothing(root: &Path, value: &str) -> bool {
    let integration = fetch_integration_path(root, value);
    if names_no_executable(value) {
        observe_input(&integration, false);
    }
    fetch_runs_nothing(value, &integration)
}

/// §FS-fetch.2.1: the key as a message names it, in the spelling of the version that wrote it.
pub(crate) fn fetch_key_name(version: u32, kind: &str) -> String {
    if version == 1 {
        format!("[[kinds]] `{kind}` fetch")
    } else {
        format!("`[schema.kinds.{kind}] fetch`")
    }
}

/// §FS-fetch.2.1: the setting that needs the key, in the spelling of the version that wrote it.
pub(crate) fn fetch_key_setting(version: u32, kind: &str) -> String {
    if version == 1 {
        "`resolve`".to_string()
    } else {
        format!("`[rules.resolution] {kind}`")
    }
}

/// §FS-fetch.2.1: the one repair, the `fetch-names-nothing` case of `dangling`'s *Fix*
/// (§FS-errors.5.5), its setting spelled by the version that wrote the key.
pub(crate) fn fetch_key_repair(version: u32, kind: &str) -> String {
    remedies::dangling_fetch_names_nothing(fetch_key_setting(version, kind))
}

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
    use std::path::Path;

    use super::{
        fetch_integration_path, fetch_key_repair, fetch_runs_nothing, names_no_executable,
    };

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

    /// §FS-fetch.2.1: a relative value runs from the config root, and an absolute one as written.
    #[test]
    fn the_integration_runs_from_the_root_unless_absolute() {
        let root = std::env::current_exe().expect("the test binary");
        let root = root.parent().expect("its folder");
        assert_eq!(
            fetch_integration_path(root, "scripts/fetch"),
            root.join("scripts/fetch")
        );
        assert_eq!(fetch_integration_path(root, " "), root.join(" "));
        let absolute = root.join("elsewhere");
        let absolute_value = absolute.to_str().expect("a UTF-8 test path");
        assert_eq!(
            fetch_integration_path(Path::new("unused"), absolute_value),
            absolute
        );
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
