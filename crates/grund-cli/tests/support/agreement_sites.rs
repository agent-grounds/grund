//! Normalize physical report sites against the producing question's one base
//! (§FS-cli.3.4, §FS-config.3.6.1, §REQ-one-answer.proof.matrix).

use serde_json::{Value, json};
use std::path::{Component, PathBuf};
use std::process::Output;

fn location(row: &mut Value, key: &str, context: &Value) {
    if let Some(field) = row.get_mut(key)
        && let Some(path) = field.as_str()
    {
        let path = super::agreement_paths::physical(context, path);
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }
        *field = json!(super::agreement_paths::logical_path(
            &normalized.to_string_lossy(),
            context
        ));
    }
}

fn finding(row: &mut Value, context: &Value) {
    location(row, "path", context);
    if let Some(sites) = row.get_mut("sites").and_then(Value::as_array_mut) {
        for site in sites {
            location(site, "path", context);
        }
    }
}

pub fn rows(surface: &str, rows: &mut [Value], context: &Value) {
    for row in rows {
        match surface {
            "show-batch" => {
                for key in ["result", "error"] {
                    if let Some(value) = row.get_mut(key) {
                        finding(value, context);
                    }
                }
            }
            "cover" => {
                location(row, "path", context);
                if let Some(citations) = row.get_mut("citations").and_then(Value::as_array_mut) {
                    for citation in citations {
                        location(citation, "path", context);
                    }
                }
            }
            _ => {
                finding(row, context);
                location(row, "defines", context);
            }
        }
    }
}

pub fn host(surface: &str, mut host: Value, context: &Value) -> Value {
    if let Some(failure) = host.get_mut("failure") {
        finding(failure, context);
    }
    if let Some(out) = host.get_mut("result") {
        result(surface, out, context);
    }
    host
}

fn result(surface: &str, out: &mut Value, context: &Value) {
    match surface {
        "show" => show(out, context),
        "show-batch" => {
            for record in out
                .get_mut("records")
                .and_then(Value::as_array_mut)
                .into_iter()
                .flatten()
            {
                if let Some(result) = record.get_mut("result") {
                    show(result, context);
                }
                if let Some(failure) = record.get_mut("failure") {
                    finding(failure, context);
                }
            }
        }
        "check" => {
            for report in ["report", "selected_report"] {
                if let Some(report) = out.get_mut(report) {
                    for key in ["errors", "warnings", "suggestions"] {
                        if let Some(findings) = report.get_mut(key).and_then(Value::as_array_mut) {
                            rows(surface, findings, context);
                        }
                    }
                }
            }
        }
        _ => {
            for key in ["entries", "records", "hits"] {
                if let Some(entries) = out.get_mut(key).and_then(Value::as_array_mut) {
                    rows(surface, entries, context);
                }
            }
        }
    }
}

fn show(out: &mut Value, context: &Value) {
    location(out, "path", context);
    if let Some(serialized) = out.get_mut("json")
        && let Some(text) = serialized.as_str()
    {
        let mut body: Value = serde_json::from_str(text).unwrap();
        finding(&mut body, context);
        *serialized = json!(body.to_string());
    }
}

pub fn output(surface: &str, output: &Output, context: &Value) -> Output {
    let stream = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .split_inclusive('\n')
            .map(|line| {
                if let Ok(row) = serde_json::from_str::<Value>(line) {
                    let mut values = vec![row];
                    rows(surface, &mut values, context);
                    format!(
                        "{}{}",
                        values[0],
                        if line.ends_with('\n') { "\n" } else { "" }
                    )
                } else {
                    super::agreement_paths::logical_text(line, context)
                }
            })
            .collect::<String>()
            .into_bytes()
    };
    Output {
        status: output.status,
        stdout: stream(&output.stdout),
        stderr: stream(&output.stderr),
    }
}
