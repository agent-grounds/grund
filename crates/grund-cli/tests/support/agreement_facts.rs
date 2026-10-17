//! Cross-question common facts, beyond host conversion equality
//! (§REQ-one-answer.identity.declarations, §REQ-one-answer.sections.map,
//! §FS-scan.path.cover). Every closed row is an expectation of agreement, so an
//! omission or a second site is a failure (§REQ-one-answer.proof.evidence.closed).

use super::agreement_oracles::{entries, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// `list` and `list --size` place each declaration at one site. Placement closed
/// with §DF-stubs-by-home-and-at-the-target, so a stub's home against its target
/// is a divergence like any other; the context and snapshot that once qualified
/// that exception are no longer read.
pub fn declarations(
    _context: &Value,
    answers: &BTreeMap<&str, Value>,
    _loaded: Option<&Value>,
    errors: &mut Vec<String>,
) {
    let lists = entries(&answers["list"], "entries");
    let sizes: Vec<_> = entries(&answers["list-size"], "entries")
        .into_iter()
        .filter(|row| row["section"].is_null())
        .collect();
    let identity = |row: &Value| (row["project"].to_string(), row["id"].to_string());
    let one: BTreeSet<_> = lists.iter().map(identity).collect();
    let two: BTreeSet<_> = sizes.iter().map(identity).collect();
    if one != two {
        errors.push(format!(
            "list/size declaration set: {one:?} against {two:?}"
        ));
    }
    for row in lists {
        let peers: Vec<_> = sizes
            .iter()
            .filter(|size| identity(size) == identity(&row))
            .collect();
        match peers
            .iter()
            .find(|size| row["path"] == size["path"] && row["line"] == size["line"])
        {
            Some(size) if row["duplicate"] != size["duplicate"] => {
                errors.push("list/size duplicate verdict differs".into());
            }
            Some(_) => {}
            None => errors.push(format!("list/size physical site: {row} against {peers:?}")),
        }
    }
}

/// Every section a direct `show` returns is in the exhaustive batch and offered
/// by the editor. U03a and U03c closed with §DF-stubs-by-home-and-at-the-target,
/// so an outside target's section is held like any other; the context and the
/// listing that once qualified that exception are no longer read.
pub fn sections(
    _context: &Value,
    coordinate: &str,
    show: &Value,
    batch: &Value,
    _listed: &Value,
    loaded: Option<&Value>,
    errors: &mut Vec<String>,
) {
    let Some(serialized) = show["result"]["json"].as_str() else {
        return;
    };
    let body: Value = serde_json::from_str(serialized).expect("show JSON");
    if !body["section"].is_null() {
        return;
    }
    let batch_records = entries(batch, "records");
    let local_coordinate = coordinate.rsplit('/').next().unwrap();
    let current_local = batch_records.iter().any(|record| {
        record["query"]["id"] == local_coordinate
            && record["query"]["section"].is_null()
            && record["result"]["json"]
                .as_str()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .is_some_and(|result| result["path"] == body["path"])
    });
    let batch_coordinates: BTreeSet<_> = batch_records
        .into_iter()
        .map(|record| {
            (
                record["query"]["id"].to_string(),
                record["query"]["section"].to_string(),
            )
        })
        .collect();
    for section in body["sections"].as_array().into_iter().flatten() {
        let section_path = section["path"].as_str().unwrap();
        let explicit = batch_coordinates.contains(&(
            json!(coordinate).to_string(),
            json!(section_path).to_string(),
        )) || current_local
            && batch_coordinates.contains(&(
                json!(local_coordinate).to_string(),
                json!(section_path).to_string(),
            ));
        // Exhaustive batch canonicalizes inline coordinates into id+section.
        if !explicit {
            errors.push(format!(
                "batch omits {coordinate}.{section_path} direct show returns"
            ));
        }
        if let Some(snapshot) = loaded {
            let offered = snapshot["snapshot"]["sections"]
                .as_array()
                .unwrap()
                .iter()
                .any(|site| {
                    site["query_id"]
                        == format!(
                            "{coordinate}{}{section_path}",
                            site["section_separator"].as_str().unwrap()
                        )
                });
            if !offered {
                errors.push(format!(
                    "editor omits {coordinate}.{section_path} direct show returns"
                ));
            }
        }
    }
}

/// Cover bounds scans; refs may resolve beyond that domain. Hold every citation
/// they both read, using physical sites to avoid alias/display-path differences.
pub fn citations(context: &Value, refs: &Value, cover: &Value, errors: &mut Vec<String>) {
    let entries = entries(cover, "entries");
    let paths: BTreeSet<_> = entries
        .iter()
        .filter_map(|entry| entry["path"].as_str())
        .collect();
    let covered: BTreeSet<_> = entries
        .iter()
        .flat_map(|entry| entry["citations"].as_array().into_iter().flatten())
        .map(|citation| {
            (
                citation["path"].to_string(),
                citation["line"].to_string(),
                citation["column"].to_string(),
                citation["id"].to_string(),
                citation["section"].to_string(),
            )
        })
        .collect();
    for hit in super::agreement_oracles::entries(refs, "hits") {
        if hit["path"]
            .as_str()
            .is_some_and(|path| paths.contains(path))
        {
            let site = (
                hit["path"].to_string(),
                hit["line"].to_string(),
                hit["column"].to_string(),
                hit["id"].to_string(),
                hit["section"].to_string(),
            );
            if !covered.contains(&site) {
                errors.push(format!(
                    "{} refs site absent from cover's common scan: {hit}",
                    text(context, "case")
                ));
            }
        }
    }
}
