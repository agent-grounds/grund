//! Normalize physical report sites against the producing question's one base
//! (§FS-cli.3.4, §FS-config.3.6.1, §REQ-one-answer.proof.matrix).

use serde_json::{Value, json};
use std::path::{Component, PathBuf};
use std::process::Output;

fn location(row: &mut Value, key: &str, context: &Value) {
    if let Some(path) = row[key].as_str() {
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
        row[key] = json!(super::agreement_paths::logical_path(
            &normalized.to_string_lossy(),
            context
        ));
    }
}

fn finding(row: &mut Value, context: &Value) {
    location(row, "path", context);
    for site in row["sites"].as_array_mut().into_iter().flatten() {
        location(site, "path", context);
    }
}

pub fn rows(surface: &str, rows: &mut [Value], context: &Value) {
    for row in rows {
        match surface {
            "show-batch" => {
                finding(&mut row["result"], context);
                finding(&mut row["error"], context);
            }
            "cover" => {
                location(row, "path", context);
                for citation in row["citations"].as_array_mut().into_iter().flatten() {
                    location(citation, "path", context);
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
    finding(&mut host["failure"], context);
    let out = &mut host["result"];
    match surface {
        "show" => show(out, context),
        "show-batch" => {
            for record in out["records"].as_array_mut().into_iter().flatten() {
                show(&mut record["result"], context);
                finding(&mut record["failure"], context);
            }
        }
        "check" => {
            for key in ["errors", "warnings", "suggestions"] {
                if let Some(findings) = out["selected_report"][key].as_array_mut() {
                    rows(surface, findings, context);
                }
            }
        }
        _ => {
            for key in ["entries", "records", "hits"] {
                if let Some(entries) = out[key].as_array_mut() {
                    rows(surface, entries, context);
                }
            }
        }
    }
    host
}

fn show(out: &mut Value, context: &Value) {
    if let Some(serialized) = out["json"].as_str() {
        let mut body: Value = serde_json::from_str(serialized).unwrap();
        finding(&mut body, context);
        out["json"] = json!(body.to_string());
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
