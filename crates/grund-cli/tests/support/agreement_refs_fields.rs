//! Verify the returned Python refs fields before common-schema projection
//! (§FS-distribution.3.3.1, §FS-distribution.3.3.2,
//! §REQ-one-answer.proof.evidence). Hit folds are consistency checks, not truth
//! about whether the engine omitted a citation.

use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn validate(envelope: &Value, errors: &mut Vec<String>) {
    let result = if envelope["failure"].is_null() {
        Some(&envelope["result"])
    } else {
        envelope["failure"]
            .get("partial_output")
            .filter(|v| !v.is_null())
    };
    let Some(result) = result else { return };
    let Some(hits) = result.get("hits").and_then(Value::as_array) else {
        errors.push("Python refs result/partial output has no hits array".into());
        return;
    };
    let mut files = BTreeMap::<(Option<String>, String), usize>::new();
    for hit in hits {
        let Some(path) = hit.get("path").and_then(Value::as_str) else {
            errors.push("Python refs hit has no path".into());
            return;
        };
        let project = match hit.get("project") {
            Some(Value::Null) => None,
            Some(Value::String(project)) => Some(project.clone()),
            _ => {
                errors.push("Python refs hit has no nullable project".into());
                return;
            }
        };
        *files.entry((project, path.into())).or_default() += 1;
    }
    for (field, expected) in [("site_total", hits.len()), ("file_total", files.len())] {
        if result.get(field) != Some(&json!(expected)) {
            errors.push(format!(
                "Python refs returned {field}: {} against {expected}",
                result[field]
            ));
        }
    }
    let expected: Vec<_> = files
        .into_iter()
        .map(|((project, path), sites)| json!({"project":project,"path":path,"sites":sites}))
        .collect();
    if result.get("file_summaries") != Some(&json!(expected)) {
        errors.push(format!(
            "Python refs returned file_summaries: {} against {expected:?}",
            result["file_summaries"]
        ));
    }
}
