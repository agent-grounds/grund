//! Read input-authored facts before engine/host preparation; never generate
//! expectations from answers (§REQ-one-answer.proof.evidence,
//! §FS-scan.text.buffer, §FS-show.1.8).

use super::agreement_expected_bytes as bytes;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const TREE: &[&str] = &[
    "check",
    "list",
    "list-size",
    "cover",
    "cover-lines",
    "fmt",
    "show-batch",
];

pub fn coordinates(variant: &Value) -> Result<BTreeSet<String>, String> {
    let list = variant["coordinates"]
        .as_array()
        .ok_or("missing authored coordinate inventory")?;
    let coordinates: BTreeSet<_> = list
        .iter()
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .ok_or("invalid coordinate")
        })
        .collect::<Result<_, _>>()?;
    if coordinates.len() != list.len() || coordinates.is_empty() {
        return Err("coordinates must be nonempty and unique (include an authored absent query for an empty tree)".into());
    }
    Ok(coordinates)
}

pub fn obligations(variant: &Value) -> Result<BTreeSet<(String, String)>, String> {
    if variant["absent"].is_string() {
        return Ok(BTreeSet::new());
    }
    let mut questions: BTreeSet<String> = TREE.iter().map(|s| s.to_string()).collect();
    questions.insert("fmt-write".into());
    let coordinates = coordinates(variant)?;
    for coordinate in &coordinates {
        questions.extend([format!("show:{coordinate}"), format!("refs:{coordinate}")]);
    }
    let mut required: BTreeSet<_> = questions
        .into_iter()
        .flat_map(|key| {
            ["cli", "python", "node"]
                .into_iter()
                .map(move |p| (key.clone(), p.into()))
        })
        .collect();
    required.insert(("editor:snapshot".into(), "editor".into()));
    let snapshot_refused = variant["calls"]["editor:snapshot"]["editor"]
        .get("failure")
        .is_some();
    if snapshot_refused {
        if variant["editorUnavailable"]["reason"] != "snapshot refuses"
            || variant["editorUnavailable"]["point"] != "§FS-lsp.2.2.1"
        {
            return Err(
                "refused snapshot must explicitly justify unavailable editor coordinate questions"
                    .into(),
            );
        }
    } else {
        if variant.get("editorUnavailable").is_some() {
            return Err("healthy snapshot cannot skip editor queries".into());
        }
        required.extend(
            coordinates
                .into_iter()
                .map(|c| (format!("editor:show:{c}"), "editor".into())),
        );
    }
    Ok(required)
}

pub fn validate_variant(variant: &Value) -> Result<(), String> {
    if variant.get("absent").is_none() {
        return Err("missing explicit authored applicability".into());
    }
    if let Some(reason) = variant["absent"].as_str() {
        if !matches!(
            reason,
            "no input repository: command grammar/process case"
                | "no proper directory below the input root"
                | "no proper file or directory path below the input root"
        ) {
            return Err(format!("unsupported authored inapplicability: {reason}"));
        }
        if ["coordinates", "calls", "writtenChanges"]
            .iter()
            .any(|key| variant.get(key).is_some())
        {
            return Err(
                "inapplicable variant cannot hide authored questions or output facts".into(),
            );
        }
        return Ok(());
    }
    if !variant["absent"].is_null() {
        return Err("authored applicability must be a precise reason or null".into());
    }
    let fields: BTreeSet<_> = variant["context"]
        .as_object()
        .map(|v| v.keys().map(String::as_str).collect())
        .unwrap_or_default();
    if fields != super::agreement_truth::CONTEXT.iter().copied().collect()
        || !variant["writtenChanges"].is_object()
    {
        return Err("missing authored invocation context or written output patches".into());
    }
    for (path, entry) in variant["writtenChanges"].as_object().unwrap() {
        if !bytes::safe_relative(path)
            || (!bytes::valid_entry(entry) && entry != &serde_json::json!({"remove":true}))
        {
            return Err(format!("invalid written output patch {path}"));
        }
    }
    let required = obligations(variant)?;
    let calls = variant["calls"]
        .as_object()
        .ok_or("missing authored per-question facts")?;
    let mut supplied = BTreeSet::new();
    for (key, call) in calls {
        if !call["point"].as_str().is_some_and(|p| p.starts_with('§'))
            || !call["authoredFrom"]
                .as_str()
                .is_some_and(|s| !s.trim().is_empty())
        {
            return Err(format!("{key}: missing defining point/input provenance"));
        }
        for producer in ["cli", "python", "node", "editor"] {
            if call.get(producer).is_some() {
                supplied.insert((key.clone(), producer.into()));
            }
        }
    }
    if required != supplied {
        return Err(format!(
            "exact question/coordinate obligations differ: missing {:?}, extra {:?}",
            required.difference(&supplied).collect::<Vec<_>>(),
            supplied.difference(&required).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

/// Missing facts are construction errors, not applicability or empty answers.
/// Discovery happens before native builds and preserves the dynamic corpus cap.
pub fn load(cases: &Path) -> Result<BTreeMap<String, Value>, Vec<String>> {
    let facts = cases
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("integration/command_agreement_facts");
    let mut loaded = BTreeMap::new();
    let mut errors = Vec::new();
    let mut directories: Vec<_> = fs::read_dir(cases)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    directories.sort();
    for case in directories {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let file = facts.join(format!("{name}.json"));
        let document = fs::read(&file)
            .map_err(|e| format!("{name}: no input-authored manifest ({e})"))
            .and_then(|raw| {
                serde_json::from_slice::<Value>(&raw).map_err(|e| format!("{name}: {e}"))
            });
        let checked = document.and_then(|document| {
            if document["version"] != 1 || document["case"] != name {
                return Err(format!("{name}: wrong manifest version/case"));
            }
            if document["inputFiles"] != bytes::inventory(&case) {
                return Err(format!(
                    "{name}: authored inputs differ from fixture bytes/manifests/symlinks"
                ));
            }
            let variants = document["variants"]
                .as_object()
                .ok_or_else(|| format!("{name}: missing variants"))?;
            let expected: BTreeSet<_> = super::agreement_corpus::VARIANTS
                .iter()
                .map(|s| s.to_string())
                .collect();
            if variants.keys().cloned().collect::<BTreeSet<_>>() != expected {
                return Err(format!("{name}: exact variant inventory differs"));
            }
            for (variant, facts) in variants {
                validate_variant(facts).map_err(|e| format!("{name}/{variant}: {e}"))?;
            }
            Ok(document)
        });
        match checked {
            Ok(document) => {
                loaded.insert(name, document);
            }
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(loaded)
    } else {
        Err(errors)
    }
}
