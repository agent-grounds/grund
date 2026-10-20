//! §FS-config-explain.3, §FS-config-explain.4: every key the gates admit is
//! worded by the embedded schema, every code's row has a meaning, and every
//! table a version's cell names is one that version admits. Each audit runs on
//! the embedded copy and, as its negative control, on a doctored one, so an
//! audit that could not fail does not pass.

use serde_json::Value;

use super::{schema, target};
use crate::config::vocabulary::admitted;
use crate::model::catalog::{self, CodeSeverity, Row};

/// Each admitted key `schema` gives no meaning or no values for, named.
fn unworded_keys(schema: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    for version in [1, 2] {
        for (header, keys) in admitted(version) {
            for key in keys {
                let node = schema::key_node(schema, version, &header, &key);
                let worded = node.is_some_and(|node| {
                    schema::meaning(node).is_some_and(|meaning| !meaning.is_empty())
                        && !schema::value_words(schema, node).is_empty()
                });
                if !worded {
                    problems.push(format!("version {version} `{header} {key}`"));
                }
            }
        }
    }
    problems
}

/// Each row without a meaning, and each version's cell that names a table its
/// version does not admit, named by code and cell.
fn catalog_problems(rows: &[Row]) -> Vec<String> {
    let mut problems = Vec::new();
    for row in rows {
        if row.meaning.trim().is_empty() {
            problems.push(format!("`{}`: no Meaning", row.code));
        }
        for version in [1, 2] {
            let tables = admitted(version);
            let side = row.side(version);
            let severity = match &side.severity {
                CodeSeverity::Strength(text) | CodeSeverity::Setting(text) => Some(text),
                _ => None,
            };
            let cells = [(
                "Severity",
                severity.cloned().into_iter().collect::<Vec<_>>(),
            )]
            .into_iter()
            .chain([(
                "Enabled by",
                side.enabled
                    .clauses()
                    .iter()
                    .map(|clause| clause.text.clone())
                    .collect(),
            )]);
            for (cell, texts) in cells {
                for spelled in texts.iter().flat_map(|text| target::spelled_tables(text)) {
                    if !target::admits(&tables, &spelled) {
                        problems.push(format!("`{}` v{version} {cell}: names {spelled}", row.code));
                    }
                }
            }
        }
    }
    problems
}

#[test]
fn every_admitted_key_is_worded_by_the_embedded_schema() {
    assert_eq!(unworded_keys(schema::embedded()), Vec::<String>::new());
}

#[test]
fn a_projection_missing_a_keys_description_names_the_key() {
    let mut doctored = schema::embedded().clone();
    let strict =
        &mut doctored["definitions"]["v1"]["properties"]["reference"]["properties"]["strict"];
    strict.as_object_mut().unwrap().remove("description");
    assert_eq!(unworded_keys(&doctored), ["version 1 `[reference] strict`"]);
}

#[test]
fn every_row_has_a_meaning_and_names_only_its_versions_tables() {
    assert_eq!(catalog_problems(catalog::rows()), Vec::<String>::new());
}

#[test]
fn a_catalog_missing_a_codes_meaning_names_the_code() {
    let rendered = include_str!("../../../assets/explain/check-codes.json");
    let mut doctored: Value = serde_json::from_str(rendered).unwrap();
    let dangling = doctored
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["code"] == "dangling")
        .unwrap();
    dangling["meaning"] = Value::String(String::new());
    let rows = catalog::parse(&doctored.to_string()).unwrap();
    assert_eq!(catalog_problems(&rows), ["`dangling`: no Meaning"]);
}

#[test]
fn a_catalog_naming_a_version_1_setting_in_a_version_2_cell_names_the_cell() {
    let rendered = include_str!("../../../assets/explain/check-codes.json");
    let mut doctored: Value = serde_json::from_str(rendered).unwrap();
    let lead = doctored
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["code"] == "oversized-lead")
        .unwrap();
    lead["2"]["enabled"]["clauses"][0]["text"] =
        Value::String("[reference] lead_size_warning is set".to_string());
    let rows = catalog::parse(&doctored.to_string()).unwrap();
    assert_eq!(
        catalog_problems(&rows),
        ["`oversized-lead` v2 Enabled by: names [reference]"]
    );
}
