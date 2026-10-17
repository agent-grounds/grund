//! Explicit host schema projections without another engine scan
//! (§FS-distribution.3.2.2.1, §FS-distribution.3.2.2.2). These are agreement
//! projections, never independent expected source facts.
//! Python's returned refs totals and every summary field must first pass
//! `agreement_refs_fields::validate` (§FS-distribution.3.3.1), including partials.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn failure(f: &mut Value, operation: &str) {
    f["operation"] = json!(operation);
    for (key, before, after) in [
        ("kind", "filesystem", "io"),
        ("kind", "path-encoding", "input"),
        ("code", "config", "invalid-config"),
        ("code", "operation", "operation-failed"),
    ] {
        if f[key] == before {
            f[key] = json!(after);
        }
    }
    if f["kind"] == "io" {
        f["code"] = json!("filesystem");
    }
    let partial = f
        .as_object_mut()
        .unwrap()
        .remove("partial_output")
        .unwrap_or(Value::Null);
    f["partial"] = if partial.is_null() {
        Value::Null
    } else {
        json!({"operation":operation,"result":partial})
    };
    if let Some(result) = f.get_mut("partial").and_then(|p| p.get_mut("result")) {
        if operation == "refs" && result.is_object() {
            folds(result);
        }
        if let Some(object) = result.as_object_mut() {
            object.remove("run_cautions");
        }
    }
    f.as_object_mut().unwrap().remove("run_cautions");
}

fn folds(result: &mut Value) {
    let mut files = BTreeMap::<String, (Value, usize, BTreeSet<u64>)>::new();
    for hit in result["hits"].as_array().into_iter().flatten() {
        let entry = files
            .entry(hit["path"].as_str().unwrap().to_string())
            .or_insert_with(|| (hit["project"].clone(), 0, BTreeSet::new()));
        entry.1 += 1;
        entry.2.insert(hit["line"].as_u64().unwrap());
    }
    result["summaries"] = json!(files.into_iter().map(|(path,(project,count,lines))|
        json!({"project":project,"path":path,"count":count,"lines":lines})).collect::<Vec<_>>());
    result["totals"] = json!({"sites":result["hits"].as_array().map_or(0,Vec::len),
        "files":result["summaries"].as_array().unwrap().len()});
}

pub fn from_python(mut value: Value, operation: &str, errors: &mut Vec<String>) -> Value {
    if operation == "refs" {
        super::agreement_refs_fields::validate(&value, errors);
    }
    // These Python fields duplicate the compared envelope channel. Assert them
    // before removing them; their absence from Node is not permission to ignore
    // a corrupt returned Python field (§FS-distribution.3.3.1).
    let cautions = value.get("run_cautions").cloned();
    let primary = if value["failure"].is_null() {
        &value["result"]
    } else {
        &value["failure"]
    };
    if primary.is_object() && primary.get("run_cautions") != cautions.as_ref() {
        errors.push("Python result/failure run_cautions differs from its envelope channel".into());
    }
    if !value["failure"].is_null() {
        failure(&mut value["failure"], operation);
        return value;
    }
    let Some(out) = value.get_mut("result").filter(|v| v.is_object()) else {
        return value;
    };
    if operation == "refs" {
        folds(out);
        for key in ["site_total", "file_total", "file_summaries"] {
            out.as_object_mut().unwrap().remove(key);
        }
    }
    if operation == "showBatch" {
        for record in out
            .get_mut("records")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            let ok = record["failure"].is_null();
            record["ok"] = json!(ok);
            if ok {
                record.as_object_mut().unwrap().remove("failure");
                if let Some(result) = record.get_mut("result").and_then(Value::as_object_mut) {
                    if result.get("run_cautions") != cautions.as_ref() {
                        errors.push(
                            "Python batch result run_cautions differs from batch channel".into(),
                        );
                    }
                    result.remove("run_cautions");
                }
            } else {
                let cautions = record["failure"]["run_cautions"].clone();
                failure(&mut record["failure"], "show");
                record["failure"]["run_cautions"] = cautions;
                record.as_object_mut().unwrap().remove("result");
            }
        }
    }
    out.as_object_mut().unwrap().remove("run_cautions");
    value
}

/// Node's additional show metadata retains its full native-peer assertion.
/// Common fields are compared after the explicit Python refs-field assertions.
pub fn node_common(mut value: Value, operation: &str) -> Value {
    let strip = |out: &mut Value| {
        if let Some(object) = out.as_object_mut() {
            for key in ["id", "section", "kind_title", "manifest"] {
                object.remove(key);
            }
        }
    };
    if operation == "show" {
        if let Some(result) = value.get_mut("result") {
            strip(result);
        }
    }
    if operation == "showBatch" {
        for record in value
            .get_mut("result")
            .and_then(|v| v.get_mut("records"))
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            if let Some(result) = record.get_mut("result") {
                strip(result);
            }
        }
    }
    value
}
