//! Independent truth beside agreement (§REQ-one-answer.proof.evidence): reviewed
//! command goldens and physical source-byte witnesses. No expected output is
//! regenerated from an engine call.

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

pub fn text(context: &Value, field: &str) -> String {
    context[field]
        .as_str()
        .unwrap_or_else(|| panic!("missing context field {field}"))
        .into()
}

pub fn normalize(value: &str, context: &Value) -> String {
    let name = text(context, "case");
    let original = if context["manifests"]["command.args"]
        .as_str()
        .is_some_and(|s| s.contains("{repo_copy}"))
        || context["manifests"]["command.cwd"] == "{repo_copy}\n"
    {
        format!("target/e2e-work/{name}/repo")
    } else {
        format!("tests/e2e/cases/{name}/repo")
    };
    value
        .replace("\r\n", "\n")
        .replace(&text(context, "base"), &original)
        .replace(&text(context, "saved"), &original)
}

pub fn golden(context: &Value, output: &Output) -> Result<Value, String> {
    let manifest = &context["manifests"];
    let expected_exit: i32 = manifest["expected.exit"]
        .as_str()
        .expect("every case has expected.exit")
        .trim()
        .parse()
        .expect("golden exit");
    let expected = |field| {
        let value = manifest[field]
            .as_str()
            .expect("every command has a reviewed output golden");
        if value == "\n" {
            String::new()
        } else {
            value.replace("\r\n", "\n")
        }
    };
    let out = normalize(&String::from_utf8_lossy(&output.stdout), context);
    let err = normalize(&String::from_utf8_lossy(&output.stderr), context);
    if output.status.code() != Some(expected_exit)
        || out != expected("expected.stdout")
        || err != expected("expected.stderr")
    {
        return Err(format!(
            "{} manifest golden: exit {:?}/{expected_exit}, stdout {out:?}/{:?}, stderr {err:?}/{:?}",
            text(context, "case"),
            output.status.code(),
            expected("expected.stdout"),
            expected("expected.stderr")
        ));
    }
    Ok(
        json!({"oracle":"reviewed-manifest-golden", "exit":expected_exit,
        "stdout":expected("expected.stdout"),"stderr":expected("expected.stderr"),
        "points":manifest["spec.refs"]}),
    )
}

/// Resolve only authored path spellings, leaving symlinks to the filesystem.
/// A path outside the root is allowed: stub and parent-relative scans have those
/// semantics (§FS-scan.path.cover, §REQ-one-answer.stubs.outside-include).
pub fn physical(context: &Value, path: &str) -> PathBuf {
    let root = PathBuf::from(text(context, "saved"));
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// Every returned physical site has its independent authored byte witness.
/// This catches invented IDs, moved lines and token loss even if all frontends
/// agree. Full expected sets are supplied by reviewed manifests and the positive
/// controls; these witnesses do not claim a new reference parser.
pub fn witnesses(context: &Value, rows: &[Value], point: &str) -> Result<Value, String> {
    let mut witnesses = Vec::new();
    for row in rows {
        let Some(path) = row["path"].as_str() else {
            continue;
        };
        let physical = physical(context, path);
        if physical.is_dir() {
            // E2E declaration bodies are manifest-backed (§FS-show.2.4).
            let manifest = physical.join("command.args");
            if physical.join("expected.exit").is_file() {
                witnesses.push(json!({"path":path,
                    "manifest":if manifest.is_file() { fs::read_to_string(manifest).unwrap() } else { super::agreement_manifest::DEFAULT_COMMAND.into() },
                    "expectedExit":fs::read_to_string(physical.join("expected.exit")).unwrap()}));
                continue;
            }
            if row["code"].is_string() {
                witnesses.push(json!({"path":path,"directory":true,"code":row["code"]}));
                continue;
            }
        }
        if !physical.exists() && row["code"].is_string() {
            witnesses.push(json!({"path":path,"exists":false,"code":row["code"]}));
            continue;
        }
        let bytes = fs::read_to_string(&physical)
            .map_err(|error| format!("source witness {path}: {error}"))?;
        if let Some(line) = row["line"].as_u64() {
            let source = bytes
                .lines()
                .nth(line.saturating_sub(1) as usize)
                .or_else(|| {
                    (bytes.is_empty() && line == 1 && row["code"].is_string()).then_some("")
                })
                .ok_or_else(|| {
                    format!("source witness {path}:{line} is outside its authored bytes")
                })?;
            if let Some(id) = row["id"].as_str()
                && row["section"].is_null()
                && !source.contains(id.rsplit('/').next().unwrap())
                && !(physical
                    .extension()
                    .is_some_and(|extension| extension == "json")
                    && decoded_token(source, id.rsplit('/').next().unwrap()))
            {
                return Err(format!(
                    "{path}:{line} does not author returned ID {id}: {source:?}"
                ));
            }
            if let Some(token) = row["text"].as_str()
                && !token.is_empty()
                && !source.contains(token)
            {
                return Err(format!(
                    "{path}:{line} does not author returned token {token:?}"
                ));
            }
            let mut witness = json!({"path":path,"line":line,"source":source});
            // A plain Markdown leaf's expected body is authored bytes after its
            // heading, not another engine answer (§FS-list.3.4.1, §FS-list.3.4.2).
            if physical
                .extension()
                .is_some_and(|extension| extension == "md")
                && row["section"].is_null()
                && row["stub"] != true
                && source.starts_with('#')
            {
                let mut body: Vec<_> = bytes.lines().skip(line as usize).collect();
                if body.iter().all(|line| {
                    !line.trim_start().starts_with('#')
                        && !line.contains(['[', ']', '`', '<', '>'])
                        && !line.trim_start().starts_with('>')
                }) {
                    while body.first().is_some_and(|line| line.trim().is_empty()) {
                        body.remove(0);
                    }
                    while body.last().is_some_and(|line| line.trim().is_empty()) {
                        body.pop();
                    }
                    let body = if body.is_empty() {
                        String::new()
                    } else {
                        format!("{}\n", body.join("\n"))
                    };
                    if let Some(actual) = row["body"].as_str()
                        && actual != body
                    {
                        return Err(format!(
                            "{path}:{line} body against independent plain source: {actual:?}/{body:?}"
                        ));
                    }
                    witness["plainBody"] = json!(body);
                    for (unit, expected) in [
                        ("bytes", body.len()),
                        (
                            "lines",
                            body.split('\n')
                                .filter(|line| {
                                    !line.bytes().all(|b| matches!(b, b'\t'..=b'\r' | b' '))
                                })
                                .count(),
                        ),
                        (
                            "words",
                            body.split(|c: char| matches!(c, '\t'..='\r' | ' '))
                                .filter(|word| !word.is_empty())
                                .count(),
                        ),
                    ] {
                        for prefix in ["lead", "full"] {
                            let field = format!("{prefix}_{unit}");
                            let actual = row.get(&field).or_else(|| {
                                row["measurements"]
                                    .as_array()
                                    .and_then(|measurements| {
                                        measurements.iter().find(|m| m["unit"] == unit)
                                    })
                                    .and_then(|measurement| measurement.get(prefix))
                            });
                            if let Some(actual) = actual
                                && actual.as_u64() != Some(expected as u64)
                            {
                                return Err(format!(
                                    "{path}:{line} {field}: {actual} against authored-body {expected}"
                                ));
                            }
                            witness[field] = json!(expected);
                        }
                    }
                }
            }
            witnesses.push(witness);
        } else {
            witnesses.push(json!({"path":path,"bytes":bytes.len()}));
        }
    }
    Ok(json!({"oracle":"physical-source-bytes","point":point,"sites":witnesses}))
}

/// Standard JSON string decoding is independent of the declaration parser.
/// Preserve the exact authored line while handling escaped value/ID keys
/// (§FS-show.2.1, §REQ-one-answer.identity.declarations).
fn decoded_token(line: &str, expected: &str) -> bool {
    let mut start = None;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if start.is_some() && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            if let Some(begin) = start.take() {
                if serde_json::from_str::<String>(&line[begin..index + 1])
                    .is_ok_and(|value| value == expected)
                {
                    return true;
                }
            } else {
                start = Some(index);
            }
        }
    }
    false
}

pub fn rows(output: &Output) -> Vec<Value> {
    [&output.stdout, &output.stderr]
        .into_iter()
        .flat_map(|stream| {
            String::from_utf8_lossy(stream)
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn entries(envelope: &Value, key: &str) -> Vec<Value> {
    envelope["result"][key]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

pub fn without_keys(mut rows: Vec<Value>, keys: &[&str]) -> Vec<Value> {
    for row in &mut rows {
        if let Some(object) = row.as_object_mut() {
            for key in keys {
                object.remove(*key);
            }
        }
    }
    rows.sort_by_key(Value::to_string);
    rows
}
