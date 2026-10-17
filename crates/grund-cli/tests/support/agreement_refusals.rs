//! Refusals have reasons and sites, not just an unsuccessful exit
//! (§FS-output-shapes.1, §FS-output-shapes.4.1, §REQ-one-answer.proof.matrix).

use serde_json::{Value, json};
use std::process::Output;

pub fn fact(finding: &Value) -> Value {
    let mut sites = finding["sites"].as_array().cloned().unwrap_or_default();
    sites.sort_by_key(Value::to_string);
    json!({"code":finding["code"],"message":finding["message"],"path":finding["path"],
        "line":finding["line"],"column":finding["column"],"sites":sites,
        "authority":finding["authority"].as_array().cloned().unwrap_or_default()})
}

pub fn compare(cli: &Output, failure: &Value, errors: &mut Vec<String>) {
    if cli.status.success() {
        errors.push(format!("CLI succeeds but host refuses: {failure}"));
        return;
    }
    let stderr = String::from_utf8_lossy(&cli.stderr);
    let findings: Vec<Value> = stderr
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|row: &Value| row["severity"] == "error")
        .collect();
    if !findings.is_empty() {
        if findings.len() != 1 || fact(&findings[0]) != fact(failure) {
            errors.push(format!(
                "CLI/host refusal facts: {findings:?} against {failure}"
            ));
        }
        if findings.iter().any(|f| {
            matches!(
                f["code"].as_str(),
                Some(
                    "not-found"
                        | "missing-section"
                        | "section-not-found"
                        | "invalid-id"
                        | "broken-stub"
                        | "ambiguous"
                        | "ambiguous-section"
                        | "unknown-project"
                        | "query-failed"
                )
            )
        }) && failure["kind"] != "query"
        {
            errors.push(format!(
                "query refusal has unrelated host family: {failure}"
            ));
        }
    } else {
        // The CLI's text channel does not expose every host-only field. Compare
        // its complete reason, retaining multiline scope notes; never claim an
        // unavailable family/code/site was independently checked by text.
        let reason = stderr
            .lines()
            .filter(|line| !line.starts_with("warning:") && !line.starts_with("hint:"))
            .collect::<Vec<_>>()
            .join("\n");
        let reason = reason.strip_prefix("error: ").unwrap_or(&reason).trim_end();
        if reason != failure["message"].as_str().unwrap_or("") {
            errors.push(format!(
                "CLI/host refusal reason: {reason:?} against {failure}"
            ));
        }
        let query_code = [
            ("ID not found:", "not-found"),
            ("section not found:", "missing-section"),
            ("invalid ID", "invalid-id"),
            ("ambiguous ID:", "ambiguous"),
            ("ambiguous section:", "ambiguous-section"),
            ("broken stub:", "broken-stub"),
            ("unknown project alias", "unknown-project"),
            ("unqualified ID requires", "query-failed"),
        ]
        .into_iter()
        .find_map(|(prefix, code)| reason.starts_with(prefix).then_some(code));
        if let Some(code) = query_code
            && (failure["kind"] != "query" || failure["code"] != code)
        {
            errors.push(format!(
                "text query refusal family/code: query/{code} against {failure}"
            ));
        }
    }
}
