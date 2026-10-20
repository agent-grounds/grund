//! Additive language-neutral disk API (§FS-distribution.3.1, §AR-bindings.2).
//! Existing exhaustively constructible options/results and CLI defaults stay intact.

use super::embedding_data::Data;
use super::embedding_failure::{envelope, error_data, failure};
use super::{embedding_config, embedding_queries, embedding_writers};
use crate::sources::{path_exists, path_metadata};
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
    crate::config::with_embedding_base(&request.root, || {
        envelope(run(&request, embedding_queries::query))
    })
}

/// The read/query delegate a host projection may replace; every other step is
/// shared, so Node's show metadata never reaches Python (§FS-distribution.3.2.2.2).
pub(super) type Query = fn(&EmbeddingRequest) -> Result<Value, Value>;

pub(super) fn run(r: &EmbeddingRequest, query: Query) -> Result<Value, Value> {
    // §FS-distribution.3.3.5: empty input succeeds without config discovery.
    if r.operation == "show_batch"
        && r.args
            .first()
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        return Ok(embedding_queries::batch_result(Vec::new(), None, json!([])));
    }
    // §FS-coordinate-globs.errors.shape: a broken `grund.toml` hides no refused pattern.
    embedding_queries::refuse_before_discovery(r)?;
    let tree = !matches!(
        r.operation.as_str(),
        "integrations" | "agent_setup_instructions"
    );
    if tree {
        if r.operation != "init" {
            path_metadata(&r.root).map_err(|error| {
                let mut source = OperationDiagnostic::new("filesystem", "io", error.to_string());
                source.path = Some(crate::model::format_path(&r.root));
                error_data(anyhow::Error::new(error).context(source), &[])
            })?;
        }
        if path_exists(&r.root) {
            crate::workspace::preflight_embedding_paths(&r.root).map_err(|e| error_data(e, &[]))?;
        }
    }
    match r.operation.as_str() {
        "check" => check_data(r),
        "scan" => embedding_queries::scan_data(r),
        "show" | "show_batch" | "refs" | "list_ids" | "list_sizes" | "cover" | "propose_id"
        | "complete_ids" => query(r),
        "effective_config" | "validate_config" | "reference_style" => embedding_config::config(r),
        "fmt" | "init" | "fetch" | "integrations" => embedding_writers::write(r),
        // §FS-distribution.3.2.1, §FS-distribution.3.3.5: both bindings return the whole
        // guide, the text `--full` prints; `Full` names no section, so it is never `None`.
        "agent_setup_instructions" => {
            Ok(json!({"text": setup_instructions(SetupPart::Full), "run_cautions": []}))
        }
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
    let (output, armed) = output.map_err(|e| error_data(e, &cautions))?;
    // §FS-check.1.4.1: the CLI's one selection, off lines included.
    let selected = selection.select(&output.report, &armed);
    Ok(
        json!({"report": output.report.data(), "selected_report": selected.data(),
        "had_scan_errors": output.had_scan_errors, "output_format": output.output_format,
        "run_cautions": cautions.data()}),
    )
}
