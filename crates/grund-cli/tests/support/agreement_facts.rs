//! Cross-question common facts, beyond host conversion equality
//! (§REQ-one-answer.identity.declarations, §REQ-one-answer.sections.map,
//! §FS-scan.path.cover). Known stub placement and section omissions keep their
//! ruled answers; other omissions remain failures.

use super::agreement_evidence as evidence;
use super::agreement_oracles::{entries, physical, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn declarations(
    context: &Value,
    answers: &BTreeMap<&str, Value>,
    loaded: Option<&Value>,
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
        if let Some(size) = peers
            .iter()
            .find(|size| row["path"] == size["path"] && row["line"] == size["line"])
        {
            if row["duplicate"] != size["duplicate"] {
                errors.push("list/size duplicate verdict differs".into());
            }
            continue;
        }
        if let Some(size) = peers.iter().find(|size| {
            row["defines"]
                .as_str()
                .zip(size["path"].as_str())
                .is_some_and(|(defines, path)| {
                    physical(context, defines) == physical(context, path)
                        && healthy_unscanned(context, &row, &physical(context, path), loaded)
                        && row["duplicate"] == size["duplicate"]
                        && loaded.is_some_and(|loaded| {
                            loaded["snapshot"]["stubs"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|stub| {
                                    super::agreement_paths::saved_path(
                                        context,
                                        Path::new(stub["target_path"].as_str().unwrap()),
                                    ) == physical(context, path)
                                        && stub["target_line"] == size["line"]
                                })
                        })
                })
        }) {
            // §REQ-one-answer.differences.stubs: Placement keeps the stub
            // home until #650, while size reads the target.
            evidence::receipt(
                "known-difference",
                Path::new(&text(context, "saved")),
                json!({"list":row,"size":size}),
                json!({"row":"Placement"}),
            );
        } else {
            errors.push(format!("list/size physical site: {row} against {peers:?}"));
        }
    }
}

/// §REQ-one-answer.differences.stubs: the real healthy stub and an unscanned
/// physical target are both preconditions, never just two returned row flags.
pub fn healthy_unscanned(
    context: &Value,
    row: &Value,
    target: &Path,
    loaded: Option<&Value>,
) -> bool {
    let Some(snapshot) = loaded.map(|s| &s["snapshot"]) else {
        return false;
    };
    if row["stub"] != true || !target.is_file() {
        return false;
    }
    let canonical = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let target = canonical(target);
    if snapshot["scanned_files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| {
            canonical(&super::agreement_paths::saved_path(
                context,
                Path::new(path.as_str().unwrap()),
            )) == target
        })
    {
        return false;
    }
    let Some(stub_path) = row["path"].as_str() else {
        return false;
    };
    let stub_path = canonical(&physical(context, stub_path));
    snapshot["stubs"].as_array().unwrap().iter().any(|stub| {
        let stub_file =
            super::agreement_paths::saved_path(context, Path::new(stub["path"].as_str().unwrap()));
        let target_file = super::agreement_paths::saved_path(
            context,
            Path::new(stub["target_path"].as_str().unwrap()),
        );
        canonical(&stub_file) == stub_path
            && canonical(&target_file) == target
            && stub["line"] == row["line"]
            && stub["query_id"].as_str().is_some_and(|query| {
                query.rsplit('/').next()
                    == row["id"].as_str().map(|id| id.rsplit('/').next().unwrap())
            })
            && std::fs::read_to_string(&stub_file)
                .ok()
                .is_some_and(|bytes| {
                    bytes
                        .lines()
                        .nth(row["line"].as_u64().unwrap_or(0).saturating_sub(1) as usize)
                        .is_some_and(|line| {
                            line.contains("](") && line.contains(row["id"].as_str().unwrap())
                        })
                })
            && std::fs::read_to_string(&target_file)
                .ok()
                .is_some_and(|bytes| {
                    bytes
                        .lines()
                        .nth(stub["target_line"].as_u64().unwrap_or(0).saturating_sub(1) as usize)
                        .is_some_and(|line| {
                            line.contains(row["id"].as_str().unwrap().rsplit('/').next().unwrap())
                        })
                })
    })
}

pub fn sections(
    context: &Value,
    coordinate: &str,
    show: &Value,
    batch: &Value,
    listed: &Value,
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
    let path = body["path"].as_str().unwrap();
    let target = physical(context, path);
    let target = target.canonicalize().unwrap_or(target);
    let outside_scan = entries(listed, "entries").iter().any(|entry| {
        entry["defines"].as_str().is_some_and(|defines| {
            let defined = physical(context, defines);
            defined.canonicalize().unwrap_or(defined) == target
                && healthy_unscanned(context, entry, &target, loaded)
        })
    });
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
            if outside_scan {
                evidence::receipt(
                    "known-difference",
                    &target,
                    json!({"coordinate":coordinate,"section":section_path,
                    "outsideScan":true}),
                    json!({"row":"U03a"}),
                );
            } else {
                errors.push(format!(
                    "batch omits {coordinate}.{section_path} direct show returns"
                ));
            }
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
                if outside_scan {
                    evidence::receipt(
                        "known-difference",
                        &target,
                        json!({"coordinate":coordinate,"section":section_path,
                        "outsideScan":true}),
                        json!({"row":"U03c"}),
                    );
                } else {
                    errors.push(format!(
                        "editor omits {coordinate}.{section_path} direct show returns"
                    ));
                }
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
