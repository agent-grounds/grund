//! Lossless failure envelopes for hosts (§FS-distribution.3.3.2).

use super::embedding_data::Data;
use crate::{Finding, OperationDiagnostic, RefsOutput, ShowQueryError};
use serde_json::{Value, json};

pub(super) fn failure(kind: &str, code: &str, message: String, cautions: &[Finding]) -> Value {
    json!({"kind": kind, "code": code, "message": message,
        "path": null, "line": null, "column": null, "sites": [], "authority": [],
        "causes": [], "run_cautions": cautions.to_vec().data(),
        "partial_output": null, "details": {}})
}

pub(super) fn error_data(error: anyhow::Error, cautions: &[Finding]) -> Value {
    let mut result = failure("operation", "operation", format!("{error:#}"), cautions);
    if let Some(source) = error.downcast_ref::<OperationDiagnostic>() {
        result["kind"] = json!(source.class);
        result["code"] = json!(source.code);
        result["path"] = json!(source.path);
        result["line"] = json!(source.line);
        result["column"] = json!(source.column);
        result["details"] = source.details.clone();
    }
    if let Some(query) = error.downcast_ref::<ShowQueryError>() {
        result["kind"] = json!("query");
        result["code"] = json!(query.code);
        result["sites"] = query.sites.data();
    }
    if let Some(io) = error.downcast_ref::<std::io::Error>() {
        if result["kind"] == "operation" {
            result["kind"] = json!("filesystem");
        }
        result["details"]["os_error"] = json!(io.raw_os_error());
    }
    if let Some(refs) = error.downcast_ref::<RefsOutput>() {
        result["run_cautions"] = refs.warnings.data();
        result["partial_output"] = json!({"scan_errors": refs.scan_errors.data(),
            "output_format": refs.output_format, "workspace": refs.workspace});
    }
    result["causes"] = json!(
        error
            .chain()
            .skip(1)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    if let Some(context) = error.downcast_ref::<crate::model::OperationContext>() {
        result["message"] = json!(context.message);
        result["run_cautions"] = context.cautions.clone();
        result["partial_output"] = context.partial_output.clone();
    }
    result
}

pub(super) fn envelope(outcome: Result<Value, Value>) -> Value {
    match outcome {
        Ok(result) => json!({"failure": null,
            "run_cautions": result["run_cautions"], "result": result}),
        Err(failure) => json!({"result": null,
            "run_cautions": failure["run_cautions"], "failure": failure}),
    }
}
