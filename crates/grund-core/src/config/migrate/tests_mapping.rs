//! Test module: the mapping cases of §FS-config-migrate.mapping, one or more per
//! row of the v1 lowering table (§AR-config.3), and the test that fails when a
//! row of `v1/mapping.rs` has none. Each case writes a small v1 tree, translates
//! it, and holds the outcome the table names: *mapped* and *written* pass the
//! records gate (§FS-config-migrate.5.1) with the spelling in the candidate, a
//! *notice* is reported, a *refused* key names its repair.

use std::collections::BTreeMap;

use super::{GateResult, Located, MigrateReport, Prepared, translate};
use crate::config::v1::mapping::{DEFAULTS, KEYS};
use crate::config::v2::{V1_TABLES, v1_homes};
use crate::testing::{test_root, write};

/// What a case expects of the translation.
pub(super) enum Outcome {
    /// Nothing refused, the records gate passes, and the candidate holds this.
    Mapped(&'static str),
    /// As `Mapped`, but the candidate must not hold this.
    Lacks(&'static str),
    /// A `written:` line names this default.
    Written(&'static str),
    /// A `mapped:` line of the key map spells its v2 key as this.
    Maps(&'static str),
    /// A notice says this, and the records gate passes.
    Notice(&'static str),
    /// A refusal says this.
    Refused(&'static str),
    /// The v1 reader itself rejects the file with this, before any mapping.
    Invalid(&'static str),
}
use Outcome::*;

/// One mapping case: the table row it covers, the v1 file after its version
/// line, the paths it needs beyond `docs/FS-a.md` (a trailing `/` is a folder),
/// and what it expects.
pub(super) struct Case {
    row: &'static str,
    toml: &'static str,
    paths: &'static [(&'static str, &'static str)],
    expect: &'static [Outcome],
}

pub(super) const fn case(
    row: &'static str,
    toml: &'static str,
    expect: &'static [Outcome],
) -> Case {
    Case {
        row,
        toml,
        paths: &[],
        expect,
    }
}

pub(super) const fn with(
    row: &'static str,
    toml: &'static str,
    paths: &'static [(&'static str, &'static str)],
    expect: &'static [Outcome],
) -> Case {
    Case {
        row,
        toml,
        paths,
        expect,
    }
}

pub(super) const FS: &str = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n";

/// `FS` after a section, so the section's keys stay in that section.
macro_rules! then_fs {
    ($toml:literal) => {
        concat!($toml, "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n")
    };
}

/// A section after `FS`, which names the row's keys.
macro_rules! fs_with {
    ($toml:literal) => {
        concat!("[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n", $toml)
    };
}

// The cases, one file per half of the table; they spell their files with the macros above.
mod tests_rows;
mod tests_settings;

/// Every case, the settings' and then the rows' and the rest.
fn cases() -> impl Iterator<Item = &'static Case> {
    tests_settings::CASES.iter().chain(tests_rows::CASES)
}

/// The v1 file of `toml`: the version line, the case, `[scan] include =
/// ["docs"]` unless the case speaks of include, and `[id] named_sections = true`
/// unless it speaks of that, since a missing default include entry and an unset
/// `named_sections` are each refused.
fn v1_file(toml: &str) -> String {
    let mut text = format!("grund_config_version = 1\n{toml}");
    if !toml.contains("include =") && !toml.contains("# default include") {
        if text.contains("[scan]\n") {
            text = text.replacen("[scan]\n", "[scan]\ninclude = [\"docs\"]\n", 1);
        } else {
            text.push_str("[scan]\ninclude = [\"docs\"]\n");
        }
    }
    if !toml.contains("named_sections") {
        if text.contains("[id]\n") {
            text = text.replacen("[id]\n", "[id]\nnamed_sections = true\n", 1);
        } else {
            text.push_str("[id]\nnamed_sections = true\n");
        }
    }
    text
}

/// Lay `case` down under a fresh root and translate it, or the reader's error.
fn translate_case(index: usize, case: &Case) -> Result<MigrateReport, String> {
    let root = test_root(&format!("migrate-mapping-{index}"));
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    for (path, text) in case.paths {
        if let Some(folder) = path.strip_suffix('/') {
            std::fs::create_dir_all(root.join(folder)).expect("create folder");
        } else {
            write(&root.join(path), text);
        }
    }
    write(&root.join("grund.toml"), &v1_file(case.toml));
    let located = Located {
        file: root.join("grund.toml"),
        root: root.clone(),
        config: "grund.toml".into(),
    };
    let prepared = translate(&located, false, false).map_err(|error| format!("{error:#}"));
    let _ = std::fs::remove_dir_all(&root);
    match prepared? {
        Prepared::Translated { report, .. } => Ok(*report),
        Prepared::AlreadyV2 => Err("read as version 2".into()),
    }
}

/// Why `report` (or the reader's error) misses `outcome`, or `None`.
fn missed(outcome: &Outcome, report: &Result<MigrateReport, String>) -> Option<String> {
    let report = match (outcome, report) {
        (Invalid(text), Err(error)) => {
            return (!error.contains(text)).then(|| format!("error {error:?}"));
        }
        (_, Err(error)) => return Some(format!("the reader refused it: {error}")),
        (Invalid(_), Ok(_)) => return Some("the reader accepted it".into()),
        (_, Ok(report)) => report,
    };
    let candidate = report.candidate.as_deref().unwrap_or_default();
    let refusals: Vec<&str> = report.refusals.iter().map(|r| r.what.as_str()).collect();
    if let Refused(text) = outcome {
        return (!refusals.iter().any(|what| what.contains(text)))
            .then(|| format!("refusals {refusals:?}"));
    }
    if !refusals.is_empty() {
        return Some(format!("refused: {refusals:?}"));
    }
    if report.gates != Some(GateResult::Equal) {
        return Some(format!("records gate: {:?}", report.gates));
    }
    let held = match outcome {
        Mapped(text) => candidate.contains(text),
        Lacks(text) => !candidate.contains(text),
        Written(text) => report.written.iter().any(|line| line.starts_with(text)),
        Maps(text) => report.mapped.iter().any(|key| key.spelling == *text),
        Notice(text) => report
            .notices
            .iter()
            .any(|notice| notice.why.contains(text)),
        Refused(_) | Invalid(_) => unreachable!("answered above"),
    };
    (!held).then(|| {
        format!(
            "candidate:\n{candidate}\nwritten: {:?}\nmapped: {:?}",
            report.written, report.mapped
        )
    })
}

/// §FS-config-migrate.mapping: every case translates to the outcome its row of
/// the table names.
#[test]
fn every_mapping_case_holds_its_outcome() {
    let mut failures = Vec::new();
    for (index, case) in cases().enumerate() {
        let report = translate_case(index, case);
        for outcome in case.expect {
            if let Some(why) = missed(outcome, &report) {
                failures.push(format!("{} ({:?}): {why}", case.row, case.toml));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-config-migrate.mapping: "a test fails when a row of the v1 lowering
/// table has none" — each row of `KEYS` and `DEFAULTS` has a case.
#[test]
fn every_lowering_row_has_a_mapping_case() {
    let missing: Vec<&str> = KEYS
        .iter()
        .chain(DEFAULTS)
        .map(|(name, _)| *name)
        .filter(|name| !cases().any(|case| case.row == *name))
        .collect();
    assert!(missing.is_empty(), "rows with no mapping case: {missing:?}");
}

/// The cases name only rows the table or the spec has, so a renamed row is
/// noticed here rather than passing with a stale case.
#[test]
fn every_mapping_case_names_a_row() {
    const SPEC_ONLY: &[&str] = &["any-comment declarations", "guidance at the root"];
    for case in cases() {
        let known = KEYS
            .iter()
            .chain(DEFAULTS)
            .any(|(name, _)| *name == case.row)
            || SPEC_ONLY.contains(&case.row);
        assert!(known, "case for unknown row {:?}", case.row);
    }
}

/// The v2 tables a `mapped:` spelling names: each `[…]` it writes, and the two
/// that `[scan]`'s keys name in words (`keymap.rs` `spelling`), the extensions
/// beside `[schema.sources] languages` and a definition by its name.
fn tables_in(spelling: &str) -> Vec<String> {
    let mut tables: Vec<String> = spelling
        .split('[')
        .skip(1)
        .filter_map(|rest| rest.split_once(']'))
        .map(|(table, _)| format!("[{table}]"))
        .collect();
    if spelling.contains("[schema.sources] languages and extensions") {
        tables.push("[schema.sources.extensions]".into());
    }
    if spelling.contains("definition ") || spelling.contains("definitions ") {
        tables.push("[schema.sources.definitions.*]".into());
    }
    tables
}

/// Whether the hint's `home` names `table`: `*` and a `<NAME>` stand for any one name.
fn names(home: &str, table: &str) -> bool {
    let home: Vec<&str> = home.trim_matches(['[', ']']).split('.').collect();
    let table: Vec<&str> = table.trim_matches(['[', ']']).split('.').collect();
    home.len() == table.len()
        && home
            .iter()
            .zip(&table)
            .all(|(home, table)| home == table || *home == "*" || home.starts_with('<'))
}

/// §FS-config-v2.reader.1.v1-tables: every v2 table the v2 reader's refusal of a
/// v1 table names is one §FS-config-migrate.mapping writes a key of that table
/// into, read off the `mapped:` lines of every case. `[output]` names run flags
/// instead, and no key of it is mapped into any table.
#[test]
fn every_table_a_v1_refusal_names_is_one_its_keys_are_mapped_into() {
    let mut written: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (index, case) in cases().enumerate() {
        let Ok(report) = translate_case(index, case) else {
            continue;
        };
        for mapped in &report.mapped {
            if let Some((header, _)) = mapped.key.split_once("] ") {
                let tables = written.entry(format!("{header}]")).or_default();
                tables.extend(tables_in(&mapped.spelling));
            }
        }
    }
    let mut failures = Vec::new();
    for (row, _) in V1_TABLES {
        let header = row.replace("<KIND>", "FS");
        let tables = written.get(&header).map_or(&[][..], Vec::as_slice);
        let Some(homes) = v1_homes(&header) else {
            failures.push(format!("{header} is a row of the map, and names nothing"));
            continue;
        };
        for home in homes {
            let held = if home.starts_with('[') {
                tables.iter().any(|table| names(&home, table))
            } else {
                tables.is_empty()
            };
            if !held {
                failures.push(format!(
                    "{header} names {home}; its keys map into {tables:?}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn v1_file_sets_include_unless_the_case_does() {
    assert!(v1_file("").contains("[scan]\ninclude = [\"docs\"]\n"));
    assert!(
        v1_file("[scan]\nextensions = [\"md\"]\n")
            .contains("[scan]\ninclude = [\"docs\"]\nextensions")
    );
    assert!(!v1_file("# default include\n").contains("include ="));
}

#[test]
fn v1_file_sets_named_sections_unless_the_case_does() {
    assert!(v1_file("").ends_with("[id]\nnamed_sections = true\n"));
    assert!(v1_file("[id]\nformat = \"x\"\n").contains("[id]\nnamed_sections = true\nformat"));
    assert!(!v1_file("# no named_sections\n").contains("named_sections = true"));
}
