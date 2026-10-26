//! §REQ-discoverable-offline.3: the built binary answers for every table, key
//! and check code, in each configuration version, from a directory outside any
//! checkout, and runs nothing the project configures (§FS-config-explain.3,
//! §FS-config-explain.4, §FS-config-explain.6). The tables and keys come from
//! the readers' gates and the codes from the catalog (§FS-errors.5.5.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use grund_core::explain_vocabulary;
use serde_json::Value;

#[path = "support/explain_catalog.rs"]
mod explain_catalog;
#[path = "support/explain_schema.rs"]
mod explain_schema;

use explain_catalog::{
    catalog, code_expectations, foreign_settings, printed, remedy_lines, repo_file, same_table,
    side, spelled_tables,
};
use explain_schema::{description_words, key_schema, value_words};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A project outside any checkout: its config version, or `None` for a
/// directory with no config, and the file its fetcher would leave.
struct Project {
    root: PathBuf,
    version: Option<u32>,
    marker: PathBuf,
}

impl Project {
    fn new(version: Option<u32>) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-explain-coverage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        let marker = root.join("fetch-ran");
        let config = match version {
            Some(1) => Some(
                "grund_config_version = 1\nproject_name = \"explain\"\n\n\
                 [[kinds]]\nkind = \"TICKET\"\nfile = \"docs/tickets.md\"\n\
                 format = \"{kind}-{number}\"\nfetch = \"scripts/fetch-ticket\"\n",
            ),
            Some(_) => Some(
                "grund_config_version = 2\nproject_name = \"explain\"\n\n\
                 [schema.kinds.TICKET]\nfiles = [\"docs/tickets.md\"]\n\
                 id_format = \"{kind}-{number}\"\nfetch = \"scripts/fetch-ticket\"\n\n\
                 [rules.resolution]\nTICKET = \"warn\"\n",
            ),
            None => None,
        };
        if let Some(config) = config {
            fs::write(root.join("grund.toml"), config).unwrap();
            let fetcher = format!("#!/bin/sh\ntouch '{}'\nexit 99\n", marker.display());
            executable(&root.join("scripts/fetch-ticket"), &fetcher);
        }
        Self {
            root,
            version,
            marker,
        }
    }

    /// The versions an answer here is given in, version 2 first with no file.
    fn versions(&self) -> Vec<u32> {
        self.version.map_or(vec![2, 1], |version| vec![version])
    }

    fn explain(&self, target: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["config", "explain", target])
            .current_dir(&self.root)
            .output()
            .expect("run grund")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn executable(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn executable(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// Every check of one answer: exit 0, every expected phrase present, and no
/// setting its version does not admit; each failure a line of `failures`.
fn judge(
    project: &Project,
    target: &str,
    expected: &[String],
    tables: Option<&[(String, Vec<String>)]>,
    failures: &mut Vec<String>,
) {
    let output = project.explain(target);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let at = format!("{} `{target}`", place(project));
    if output.status.code() != Some(0) || stdout.trim().is_empty() {
        failures.push(format!(
            "{at}: exit {:?}, stderr {:?}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
        return;
    }
    let flat = printed(&stdout);
    for phrase in expected {
        if !flat.contains(phrase.as_str()) {
            failures.push(format!("{at}: the answer lacks {phrase:?}"));
        }
    }
    if let Some(tables) = tables {
        for spelled in foreign_settings(&stdout, tables) {
            failures.push(format!(
                "{at}: names {spelled}, which its version does not admit"
            ));
        }
    }
}

fn place(project: &Project) -> String {
    match project.version {
        Some(version) => format!("version {version}"),
        None => "no config".to_string(),
    }
}

fn report(failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{} answers failed; the first ten:\n{}",
        failures.len(),
        failures
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// §REQ-discoverable-offline, §FS-config-explain.3, §FS-config-explain.4: each
/// table and key a version's reader admits, and each code, has an answer in
/// that version carrying what it accepts and means, or what it means, its
/// severity and what turns it on, and naming no setting the version refuses.
/// A table's answer names its keys (§FS-config-explain.1.2), and the version
/// is the file's, or both with no file (§FS-config-explain.2).
#[test]
fn every_table_key_and_code_is_answered_in_each_version() {
    let schema: Value = serde_json::from_str(&repo_file("schemas/grund.schema.json")).unwrap();
    let rows = catalog();
    let mut failures = Vec::new();
    for version in [Some(1), Some(2), None] {
        let project = Project::new(version);
        for answering in project.versions() {
            let (tables, codes) = explain_vocabulary(answering);
            let own = project.version.map(|_| tables.as_slice());
            for (header, keys) in &tables {
                if !header.is_empty() {
                    judge(&project, header, keys, own, &mut failures);
                }
                for key in keys {
                    let target = if header.is_empty() {
                        key.clone()
                    } else {
                        format!("{header} {key}")
                    };
                    let mut expected = vec![key.clone()];
                    match key_schema(&schema, answering, header, key) {
                        Some(node) => {
                            let meaning = description_words(node);
                            let values = value_words(&schema, node);
                            if meaning.is_none() || values.is_empty() {
                                failures.push(format!(
                                    "version {answering} `{target}`: the published schema \
                                     gives no meaning or no type"
                                ));
                            }
                            expected.extend(meaning);
                            expected.extend(values);
                        }
                        None => failures.push(format!(
                            "version {answering} `{target}`: the published schema has no node"
                        )),
                    }
                    judge(&project, &target, &expected, own, &mut failures);
                }
            }
            for code in codes {
                let Some(row) = rows.iter().find(|row| row.code == code) else {
                    failures.push(format!("`{code}`: no catalog row"));
                    continue;
                };
                let expected = code_expectations(row, answering);
                judge(&project, code, &expected, own, &mut failures);
            }
        }
        if project.marker.exists() {
            failures.push(format!("{}: the fetcher ran", place(&project)));
        }
    }
    report(&failures);
}

/// §FS-config-explain.4, §FS-errors.5.5.1: every code the rendered catalog ships
/// is answered, in version 1, in version 2 and with no config, outside any
/// checkout, with what clears its finding: at least one `Fix` line that is a
/// remedy or `none — <why>`, and exactly the lines its row's *Fix* and *Note*
/// cells give for that version.
#[test]
#[ignore = "red until #1027 prints each code's Fix and Note; implement removes this"]
fn every_code_answer_carries_its_fix_in_each_version() {
    let shipped: Value = serde_json::from_str(&repo_file(
        "crates/grund-core/assets/explain/check-codes.json",
    ))
    .unwrap();
    let codes: Vec<String> = shipped
        .as_array()
        .expect("the rendered catalog is a list of rows")
        .iter()
        .map(|row| row["code"].as_str().unwrap().to_string())
        .collect();
    assert!(!codes.is_empty(), "the rendered catalog ships no code");
    let rows = catalog();
    let mut failures = Vec::new();
    for version in [Some(1), Some(2), None] {
        let project = Project::new(version);
        for code in &codes {
            let output = project.explain(code);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let at = format!("{} `{code}`", place(&project));
            if output.status.code() != Some(0) {
                failures.push(format!("{at}: exit {:?}", output.status.code()));
                continue;
            }
            let fixes: Vec<&str> = stdout
                .lines()
                .filter(|line| line.starts_with("Fix: ") || line.starts_with("Fix, "))
                .collect();
            let remedies = project.versions().len();
            if fixes.len() < remedies
                || fixes.iter().any(|line| {
                    let words = line.split_once(": ").map_or("", |(_, words)| words);
                    words.trim().is_empty() || words.trim() == "none —" || words == "none"
                })
            {
                failures.push(format!("{at}: no Fix line for each version, in {fixes:?}"));
            }
            let Some(row) = rows.iter().find(|row| &row.code == code) else {
                failures.push(format!("{at}: no catalog row"));
                continue;
            };
            for answering in project.versions() {
                let Some(expected) = remedy_lines(&rows, row, answering) else {
                    failures.push(format!("{at}: the catalog row has no Fix cell"));
                    continue;
                };
                for line in expected {
                    if !stdout.lines().any(|printed| printed == line) {
                        failures.push(format!("{at}: version {answering} lacks {line:?}"));
                    }
                }
            }
        }
    }
    report(&failures);
}

/// §FS-config-explain.1.3, §FS-config-explain.4, §FS-config-explain.5: a
/// planned code and a misspelt one are a query that found nothing, exit 1,
/// naming how to list the codes.
#[test]
fn a_planned_or_misspelt_code_is_a_query_failure() {
    let planned: Vec<String> = catalog()
        .into_iter()
        .filter(|row| row.enabled.starts_with("planned — "))
        .map(|row| row.code)
        .collect();
    assert!(
        !planned.is_empty(),
        "the catalog marks its remote rows planned"
    );
    let mut failures = Vec::new();
    for version in [Some(1), Some(2), None] {
        let project = Project::new(version);
        for code in planned.iter().map(String::as_str).chain(["oversized-leed"]) {
            let output = project.explain(code);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() != Some(1)
                || !output.stdout.is_empty()
                || !stderr.contains("grund config explain --codes")
            {
                failures.push(format!(
                    "{} `{code}`: exit {:?}, stdout {:?}, stderr {:?}",
                    place(&project),
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout).trim(),
                    stderr.trim()
                ));
            }
        }
    }
    report(&failures);
}

/// §FS-errors.5.5.1: a catalog cell names, in each version, only tables that
/// version's reader admits, so the answer that prints it cannot send an agent
/// to a table its file refuses.
#[test]
fn catalog_cells_name_only_tables_their_version_admits() {
    let mut failures = Vec::new();
    for row in catalog() {
        if row.enabled.starts_with("planned — ") {
            continue;
        }
        for version in [2, 1] {
            let (tables, _) = explain_vocabulary(version);
            for cell in [&row.severity, &row.enabled] {
                let spoken = side(cell, version);
                for spelled in spelled_tables(&spoken) {
                    if !tables.iter().any(|(table, _)| same_table(&spelled, table)) {
                        failures.push(format!(
                            "`{}` names {spelled} in version {version}",
                            row.code
                        ));
                    }
                }
            }
        }
    }
    report(&failures);
}

/// The control for the setting check: the cell #832 replaced, which sent a
/// version-2 project to a version-1 table, is caught in version 2 and not in
/// version 1, and a line that labels its version is exempt.
#[test]
fn the_setting_check_catches_a_version_1_table_in_a_version_2_answer() {
    let (v2, _) = explain_vocabulary(2);
    let (v1, _) = explain_vocabulary(1);
    let old = "[reference] lead_size_warning = { max = <N>, unit = \"words\" } is set";
    assert_eq!(foreign_settings(old, &v2), ["[reference]"]);
    assert!(foreign_settings(old, &v1).is_empty());
    assert!(foreign_settings("The marker, as v1 [reference] marker.", &v2).is_empty());
    assert!(foreign_settings("[schema.kinds.<NAME>.fields.<field>] one_of", &v2).is_empty());
    assert_eq!(
        foreign_settings("[schema.leads.words] warn", &v1),
        ["[schema.leads.words]"]
    );
}

/// The control the marker check leans on: the fixture's fetcher, run by the one
/// command that runs it, does leave the marker.
#[cfg(unix)]
#[test]
fn the_fixture_fetcher_leaves_its_marker_when_fetch_runs_it() {
    for version in [1, 2] {
        let project = Project::new(Some(version));
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["fetch", "TICKET-1"])
            .current_dir(&project.root)
            .output()
            .expect("run grund fetch");
        assert!(
            project.marker.exists(),
            "version {version}: the fetcher left no marker; fetch said {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
