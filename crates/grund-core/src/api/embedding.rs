//! Additive language-neutral disk API (§FS-distribution.3.1, §AR-bindings.2).
//! Existing exhaustively constructible options/results and CLI defaults stay intact.

use super::embedding_data::Data;
use super::embedding_failure::{envelope, error_data, failure};
use super::{embedding_config, embedding_queries, embedding_writers};
use crate::*;
use serde_json::{Value, json};
use std::path::PathBuf;

/// One per-call request. Host frontends validate their own types; options use
/// schema names, never internal Config layouts (§FS-distribution.3.3.5).
pub struct EmbeddingRequest {
    pub operation: String,
    pub root: PathBuf,
    pub explicit: bool,
    pub args: Vec<Value>,
    pub options: Value,
}

impl EmbeddingRequest {
    pub(super) fn flag(&self, name: &str) -> bool {
        self.options[name].as_bool().unwrap_or(false)
    }
    pub(super) fn string(&self, name: &str) -> Option<String> {
        self.options[name].as_str().map(str::to_owned)
    }
    pub(super) fn strings(&self, name: &str) -> Vec<String> {
        self.options[name]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|s| s.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub(super) fn arg(&self, index: usize) -> &str {
        self.args
            .get(index)
            .and_then(Value::as_str)
            .unwrap_or_default()
    }
}

/// Complete results or structured failures, never streams/process state
/// (§FS-distribution.3.3.1, §FS-distribution.3.3.2, §FS-distribution.3.3.4).
pub fn embedding_call(request: EmbeddingRequest) -> Value {
    crate::config::with_embedding_base(&request.root, || envelope(run(&request)))
}

fn run(r: &EmbeddingRequest) -> Result<Value, Value> {
    // §FS-distribution.3.3.5: empty input succeeds without config discovery.
    if r.operation == "show_batch"
        && r.args
            .first()
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        return Ok(json!({"records": [], "run_cautions": []}));
    }
    let tree = !matches!(
        r.operation.as_str(),
        "integrations" | "agent_setup_instructions"
    );
    if tree {
        if r.operation != "init" {
            std::fs::metadata(&r.root).map_err(|error| {
                let mut source = OperationDiagnostic::new("filesystem", "io", error.to_string());
                source.path = Some(crate::model::format_path(&r.root));
                error_data(anyhow::Error::new(error).context(source), &[])
            })?;
        }
        if r.root.exists() {
            crate::workspace::preflight_embedding_paths(&r.root).map_err(|e| error_data(e, &[]))?;
        }
    }
    match r.operation.as_str() {
        "check" => check_data(r),
        "scan" => embedding_queries::scan_data(r),
        "show" | "show_batch" | "refs" | "list_ids" | "list_sizes" | "cover" | "propose_id"
        | "complete_ids" => embedding_queries::query(r),
        "effective_config" | "validate_config" | "reference_style" => embedding_config::config(r),
        "fmt" | "init" | "fetch" | "integrations" => embedding_writers::write(r),
        "agent_setup_instructions" => Ok(
            json!({"text": canonical_template_text(AGENT_SETUP_INSTRUCTIONS), "run_cautions": []}),
        ),
        _ => Err(failure(
            "operation",
            "unsupported-operation",
            format!("unknown operation {}", r.operation),
            &[],
        )),
    }
}

fn check_data(r: &EmbeddingRequest) -> Result<Value, Value> {
    let mut selection = CheckFindingSelection::default();
    for code in r.strings("only") {
        selection.add_only(&code).map_err(|e| error_data(e, &[]))?;
    }
    for code in r.strings("ignore") {
        selection
            .add_ignore(&code)
            .map_err(|e| error_data(e, &[]))?;
    }
    if r.flag("only_rule") {
        selection.scope_to_trial_rule();
    }
    let (cautions, output) = check_with_run_warnings(CheckOpts {
        path: r.root.clone(),
        path_provided: r.explicit,
        require_grounding: r.flag("require_grounding"),
        include_suggestions: r.flag("suggestions"),
        full: r.flag("full"),
        rule: r.string("rule"),
    });
    let output = output.map_err(|e| error_data(e, &cautions))?;
    let select = |values: &[Finding]| {
        values
            .iter()
            .filter(|f| selection.retains(f.code, &f.authority))
            .cloned()
            .collect::<Vec<_>>()
    };
    let selected = Report {
        errors: select(&output.report.errors),
        warnings: select(&output.report.warnings),
        suggestions: select(&output.report.suggestions),
    };
    Ok(
        json!({"report": output.report.data(), "selected_report": selected.data(),
        "had_scan_errors": output.had_scan_errors, "output_format": output.output_format,
        "run_cautions": cautions.data()}),
    )
}
