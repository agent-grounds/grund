//! Node's deliberate record projection over the shared embedding engine
//! (§FS-distribution.3.2.1, §FS-distribution.3.2.2.2).

use super::embedding::{EmbeddingRequest, run};
use super::embedding_data::Data;
use super::embedding_failure::{envelope, error_data};
use crate::*;
use serde_json::{Value, json};
use std::path::Path;

/// Normalize the public argument framing for the common Rust oracle. This is
/// transport only; hosts validate their input types (§FS-distribution.3.0.3).
pub fn node_request(operation: &str, args: Vec<Value>, cwd: &Path) -> EmbeddingRequest {
    let positional = matches!(operation, "check" | "scan" | "init" | "referenceStyle");
    let option_index = match operation {
        "proposeId" => 2,
        "show" | "showBatch" | "refs" | "fetch" | "check" | "init" => 1,
        _ => 0,
    };
    let mut options = args
        .get(option_index)
        .filter(|v| v.is_object())
        .cloned()
        .unwrap_or_else(|| json!({}));
    let path = if positional {
        args.first().and_then(Value::as_str)
    } else {
        options["root"].as_str()
    };
    let explicit = path.is_some();
    let root = cwd.join(path.unwrap_or("."));
    let operands = match operation {
        "show" | "showBatch" | "refs" | "fetch" => args.into_iter().take(1).collect(),
        "proposeId" => args.into_iter().take(2).collect(),
        "completeIds" => vec![options["prefix"].as_str().unwrap_or("").into()],
        "integrations" => vec![options["client"].as_str().unwrap_or("").into()],
        _ => Vec::new(),
    };
    for (host, wire) in [
        ("requireGrounding", "require_grounding"),
        ("onlyRule", "only_rule"),
        ("crossRefs", "cross_refs"),
        ("dryRun", "dry_run"),
        ("noVcs", "no_vcs"),
        ("conversationTarget", "conversation_target"),
    ] {
        if let Some(v) = options.as_object_mut().and_then(|o| o.remove(host)) {
            options[wire] = v;
        }
    }
    EmbeddingRequest {
        operation: operation.into(),
        root,
        explicit,
        args: operands,
        options,
    }
}

/// Same-source native envelope. No host-specific scanner or rendered-message
/// classification is needed (§FS-distribution.3.2.2.1).
pub fn node_embedding_call(mut r: EmbeddingRequest) -> Value {
    let operation = r.operation.clone();
    crate::config::with_embedding_base(&r.root.clone(), || {
        if operation == "check" {
            let mut selection = CheckFindingSelection::default();
            for (key, values) in [("only", r.strings("only")), ("ignore", r.strings("ignore"))] {
                for code in values {
                    let result = if key == "only" {
                        selection.add_only(&code)
                    } else {
                        selection.add_ignore(&code)
                    };
                    if let Err(error) = result {
                        let mut f = super::embedding_failure::failure(
                            "input",
                            "invalid-argument",
                            error.to_string(),
                            &[],
                        );
                        normalize_failure(&mut f, &operation);
                        return json!({"failure":f,"result":null,"run_cautions":[]});
                    }
                }
            }
        }
        if operation == "scan" {
            let mut v = envelope(super::embedding_snapshot::scan(&r));
            normalize(&mut v, &operation);
            return v;
        }
        if matches!(operation.as_str(), "effectiveConfig" | "validateConfig") {
            let mut v = envelope(config(&r));
            normalize(&mut v, &operation);
            return v;
        }
        r.operation = match operation.as_str() {
            "showBatch" => "show_batch",
            "list" => "list_ids",
            "listSizes" => "list_sizes",
            "proposeId" => "propose_id",
            "completeIds" => "complete_ids",
            "referenceStyle" => "reference_style",
            "agentSetupInstructions" => "agent_setup_instructions",
            other => other,
        }
        .into();
        if operation == "init" {
            r.options["write"] = json!(!r.flag("dry_run") && !r.flag("check"));
        }
        if operation == "fetch" {
            r.options["write"] = json!(true);
        }
        let operand = r.args.first().cloned().unwrap_or(Value::Null);
        let write = r.flag("write");
        // §FS-distribution.3.2.2.2: Node's delegate adds show metadata; the rest is shared.
        let mut value = envelope(run(&r, super::embedding_node_show::query));
        if operation == "integrations" {
            let output = if value["failure"].is_null() {
                value.get_mut("result")
            } else {
                value
                    .get_mut("failure")
                    .and_then(|f| f.get_mut("partial_output"))
            };
            // §FS-distribution.3.2.2.1: completed writes use the same typed partial record.
            if let Some(output) = output.filter(|v| v.is_object()) {
                super::embedding_integrations::project(
                    output,
                    operand.as_str().filter(|s| !s.is_empty()),
                    write,
                );
            }
        }
        if operation == "fetch" && value["failure"].is_null() {
            value["result"]["id"] = operand;
        }
        normalize(&mut value, &operation);
        value
    })
}

fn config(r: &EmbeddingRequest) -> Result<Value, Value> {
    let (cautions, config) =
        super::embedding_config::load(&r.root, r.operation == "validateConfig");
    let config = config.map_err(|e| error_data(e, &cautions))?;
    let mut values = super::embedding_config::schema(&config);
    let version = values
        .as_object_mut()
        .unwrap()
        .remove("grund_config_version")
        .unwrap();
    values["version"] = version;
    Ok(json!({"root":crate::model::format_path(&config.root),
        "config_file":config.config_file.as_ref().map(|p|crate::model::format_path(p)),
        "values":values,"run_cautions":cautions.data()}))
}

fn normalize(value: &mut Value, operation: &str) {
    if overflow(value) {
        let cautions = value["run_cautions"].clone();
        let mut f = super::embedding_failure::failure(
            "operation",
            "numeric-overflow",
            "native result exceeds JavaScript's safe integer range".into(),
            &[],
        );
        normalize_failure(&mut f, operation);
        *value = json!({"result":null,"failure":f,"run_cautions":cautions});
        return;
    }
    if !value["failure"].is_null() {
        normalize_failure(&mut value["failure"], operation);
        return;
    }
    let out = &mut value["result"];
    match operation {
        "completeIds" => rename(out, "candidates", "ids"),
        "agentSetupInstructions" => rename(out, "text", "instructions"),
        "init" => rename(out, "has_pending_changes", "pending_changes"),
        "refs" => {
            refs_totals(out);
            for key in ["site_total", "file_total", "file_summaries"] {
                out.as_object_mut().unwrap().remove(key);
            }
        }
        "showBatch" => {
            for record in out["records"].as_array_mut().into_iter().flatten() {
                let ok = record["failure"].is_null();
                record["ok"] = json!(ok);
                if ok {
                    record.as_object_mut().unwrap().remove("failure");
                    record["result"]
                        .as_object_mut()
                        .unwrap()
                        .remove("run_cautions");
                } else {
                    let cautions = record["failure"]["run_cautions"].clone();
                    normalize_failure(&mut record["failure"], "show");
                    record["failure"]["run_cautions"] = cautions;
                    record.as_object_mut().unwrap().remove("result");
                }
            }
        }
        _ => {}
    }
    out.as_object_mut().unwrap().remove("run_cautions");
}

pub(super) fn normalize_failure(f: &mut Value, operation: &str) {
    f["operation"] = json!(operation);
    if f["kind"] == "filesystem" {
        f["kind"] = json!("io");
    }
    if f["kind"] == "path-encoding" {
        f["kind"] = json!("input");
    }
    if f["code"] == "config" {
        f["code"] = json!("invalid-config");
    }
    if f["kind"] == "io" {
        f["code"] = json!("filesystem");
    }
    if f["code"] == "operation" {
        f["code"] = json!("operation-failed");
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
    // §FS-distribution.3.2.2.1: inspecting an absent partial must preserve null.
    if let Some(result) = f.get_mut("partial").and_then(|p| p.get_mut("result")) {
        if operation == "init" {
            rename(result, "has_pending_changes", "pending_changes");
        }
        // §FS-distribution.3.2.2.2: a refused refs keeps the folds its result would carry.
        if operation == "refs" && result.is_object() {
            refs_totals(result);
        }
        if let Some(result) = result.as_object_mut() {
            result.remove("run_cautions");
        }
    }
    f.as_object_mut().unwrap().remove("run_cautions");
}

fn refs_totals(out: &mut Value) {
    out["summaries"] = super::embedding_refs::summaries(&out["hits"]);
    out["totals"] = json!({"sites":out["hits"].as_array().map_or(0,Vec::len),
        "files":out["summaries"].as_array().map_or(0,Vec::len)});
}

fn rename(out: &mut Value, old: &str, new: &str) {
    if let Some(v) = out.as_object_mut().and_then(|o| o.remove(old)) {
        out[new] = v;
    }
}

// §FS-distribution.3.2.2: validate before a host can round a native integer.
fn overflow(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            n.as_i64()
                .is_some_and(|n| n.unsigned_abs() > 9_007_199_254_740_991)
                || n.as_u64().is_some_and(|n| n > 9_007_199_254_740_991)
        }
        Value::Array(values) => values.iter().any(overflow),
        Value::Object(values) => values.values().any(overflow),
        _ => false,
    }
}
