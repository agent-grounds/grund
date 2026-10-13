//! Fact projections over common domains (§REQ-one-answer.proof.matrix). The
//! Rust envelope is an agreement peer, never the independent truth oracle.

use super::agreement_host::Host;
use super::agreement_oracles::{entries, rows, text, without_keys, witnesses};
use super::agreement_questions::Question;
use grund_core::{node_embedding_call, node_request};
use serde_json::{Value, json};
use std::path::Path;
use std::process::Output;

fn equal<T: PartialEq + std::fmt::Debug>(
    name: &str,
    actual: T,
    expected: T,
    errors: &mut Vec<String>,
) {
    if actual != expected {
        errors.push(format!("{name}: {actual:?} against {expected:?}"));
    }
}

/// Exact lossless host envelopes accompany command-specific CLI projections.
pub fn hosts(
    host: &mut Host,
    context: &Value,
    question: &Question,
    cli: &Output,
    errors: &mut Vec<String>,
) -> Value {
    let cwd = text(context, "cwd");
    let request = question.python_request(&cwd);
    let python = host.call("python", request.clone());
    let python_context = question.context(context, "python");
    let independent = match witnesses(
        &python_context,
        &fact_rows(question, &python),
        question.point,
    ) {
        Ok(witness) => witness,
        Err(error) => {
            errors.push(error);
            json!({"oracleError":"physical-source-bytes"})
        }
    };
    super::agreement_evidence::witness(independent.clone());
    let node_req = question.node_request(&cwd);
    let node = host.call("node", node_req.clone());
    if matches!(question.surface, "show" | "show-batch") {
        let node_rust = node_embedding_call(node_request(
            question.node_operation,
            node_req["args"].as_array().unwrap().clone(),
            Path::new(&cwd),
        ));
        equal("Node complete metadata envelope", &node, &node_rust, errors);
    }
    equal(
        "actual host complete common envelopes",
        super::agreement_host_projection::node_common(node.clone(), question.node_operation),
        super::agreement_host_projection::from_python(python.clone(), question.node_operation),
        errors,
    );
    let node_independent = match witnesses(
        &question.context(context, "node"),
        &fact_rows(question, &node),
        question.point,
    ) {
        Ok(witness) => witness,
        Err(error) => {
            errors.push(error);
            json!({"oracleError":"physical-source-bytes"})
        }
    };
    super::agreement_evidence::witness(node_independent);
    let cli_context = question.context(context, "cli");
    let projected = super::agreement_sites::output(question.surface, cli, &cli_context);
    compare_cli(
        question,
        &projected,
        &super::agreement_sites::host(question.surface, python.clone(), &python_context),
        errors,
    );
    compare_cli(
        question,
        &projected,
        &super::agreement_sites::host(
            question.surface,
            node.clone(),
            &question.context(context, "node"),
        ),
        errors,
    );
    json!({"python":python,"node":node,"witness":independent,"request":request})
}

pub fn fact_rows(question: &Question, envelope: &Value) -> Vec<Value> {
    match question.surface {
        "list" | "list-size" => entries(envelope, "entries"),
        "refs" => entries(envelope, "hits"),
        "show" => {
            serde_json::from_str::<Value>(envelope["result"]["json"].as_str().unwrap_or("null"))
                .ok()
                .filter(Value::is_object)
                .into_iter()
                .collect()
        }
        "cover" => entries(envelope, "entries")
            .into_iter()
            .flat_map(|row| row["citations"].as_array().cloned().unwrap_or_default())
            .collect(),
        "cover-lines" => entries(envelope, "records"),
        "check" => ["errors", "warnings", "suggestions"]
            .into_iter()
            .flat_map(|key| {
                envelope["result"]["selected_report"][key]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
            })
            .collect(),
        "show-batch" => entries(envelope, "records")
            .into_iter()
            .filter_map(|row| {
                serde_json::from_str::<Value>(row["result"]["json"].as_str().unwrap_or("null"))
                    .ok()
                    .filter(Value::is_object)
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn columns(rows: Vec<Value>, names: &[&str]) -> Vec<Value> {
    let mut projected: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let mut result = json!({});
            for name in names {
                result[*name] = row[*name].clone();
            }
            result
        })
        .collect();
    projected.sort_by_key(Value::to_string);
    projected
}

pub fn compare_cli(question: &Question, cli: &Output, host: &Value, errors: &mut Vec<String>) {
    let actual = rows(cli);
    if !host["failure"].is_null() {
        super::agreement_refusals::compare(cli, &host["failure"], errors);
        return;
    }
    let result = &host["result"];
    match question.surface {
        "check" => {
            let expected = fact_rows(question, host);
            let names = ["severity", "path", "line", "code", "message"];
            equal(
                "check findings",
                columns(actual, &names),
                columns(expected, &names),
                errors,
            );
        }
        "list" => {
            let names = [
                "project",
                "id",
                "section",
                "kind",
                "path",
                "line",
                "title",
                "stub",
                "defines",
                "refs",
                "duplicate",
            ];
            equal(
                "list declarations",
                columns(actual, &names),
                columns(entries(host, "entries"), &names),
                errors,
            );
        }
        "list-size" => {
            let mut expected = entries(host, "entries");
            for row in &mut expected {
                for measurement in row["measurements"].as_array().cloned().unwrap_or_default() {
                    let unit = measurement["unit"].as_str().unwrap();
                    row[format!("lead_{unit}")] = measurement["lead"].clone();
                    row[format!("full_{unit}")] = measurement["full"].clone();
                }
            }
            let names = [
                "project",
                "id",
                "section",
                "kind",
                "path",
                "line",
                "stub",
                "defines",
                "duplicate",
                "lead_words",
                "full_words",
                "lead_lines",
                "full_lines",
                "lead_bytes",
                "full_bytes",
            ];
            equal(
                "sizes",
                columns(actual, &names),
                columns(expected, &names),
                errors,
            );
        }
        "refs" => {
            let names = [
                "project", "path", "line", "column", "id", "section", "marker", "text",
            ];
            equal(
                "citation sites",
                columns(actual, &names),
                columns(entries(host, "hits"), &names),
                errors,
            );
        }
        "show" => {
            let expected: Value =
                serde_json::from_str(result["json"].as_str().unwrap()).expect("host show JSON");
            equal(
                "show body, span, map and anchor",
                actual,
                vec![expected],
                errors,
            );
        }
        "show-batch" => {
            let expected = entries(host, "records");
            equal(
                "batch coordinate count",
                actual.len(),
                expected.len(),
                errors,
            );
            for (actual, expected) in actual.iter().zip(expected) {
                equal(
                    "batch coordinate",
                    &actual["query"],
                    &expected["query"],
                    errors,
                );
                if expected["failure"].is_null() {
                    let body: Value =
                        serde_json::from_str(expected["result"]["json"].as_str().unwrap()).unwrap();
                    equal("batch body", &actual["result"], &body, errors);
                } else {
                    equal(
                        "batch refused coordinate",
                        &actual["ok"],
                        &json!(false),
                        errors,
                    );
                    equal(
                        "batch refusal reason and sites",
                        super::agreement_refusals::fact(&actual["error"]),
                        super::agreement_refusals::fact(&expected["failure"]),
                        errors,
                    );
                }
            }
        }
        "cover" => {
            let names = ["project", "path", "citations"];
            // CLI and host differ in optional project metadata on each citation;
            // compare the complete source/citation coordinate on their domain.
            let simplify = |rows: Vec<Value>| -> Vec<Value> {
                rows.into_iter()
                    .map(|mut row| {
                        row["citations"] = json!(columns(
                            row["citations"].as_array().cloned().unwrap_or_default(),
                            &["path", "line", "column", "id", "section", "marker", "text"]
                        ));
                        row
                    })
                    .collect()
            };
            equal(
                "cover scan and citations",
                columns(simplify(actual), &names),
                columns(simplify(entries(host, "entries")), &names),
                errors,
            );
        }
        "cover-lines" => equal(
            "line ownership",
            without_keys(actual, &["project"]),
            without_keys(entries(host, "records"), &["project"]),
            errors,
        ),
        "fmt" => {
            // §FS-fmt.3.1: a complete dry run exits 0 even with changes;
            // incomplete scans exit 2, while refused outside-root writes do not.
            let expected_exit = if result["scan_errors"]
                .as_array()
                .is_some_and(|errors| !errors.is_empty())
            {
                2
            } else {
                0
            };
            equal("fmt exit", cli.status.code(), Some(expected_exit), errors);
        }
        _ => unreachable!(),
    }
}
