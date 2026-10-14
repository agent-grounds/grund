//! Exact authored facts for generic calls, independently of returned rows
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.proof.evidence,
//! §FS-show.2.4, §FS-list.3).

use super::{
    agreement_evidence as evidence, agreement_expected_bytes as bytes, agreement_paths as paths,
    agreement_questions::Question,
};
use serde_json::{Value, json};
use std::{path::Path, process::Output};

pub const CONTEXT: &[&str] = &[
    "relative",
    "path",
    "cwd",
    "editorPath",
    "explicit",
    "overlays",
    "overlayRelative",
    "heldBytes",
    "bufferOnly",
    "scopes",
];

pub fn bind(context: &mut Value, facts: &Value) {
    if !context["absent"].is_null() {
        // Platform symlink applicability is still physically verified by the
        // materialization oracle, rather than declared absent in a fact file.
        if context["symlinkError"]["winerror"] != 1314 {
            assert_eq!(
                facts["absent"], context["absent"],
                "authored applicability differs"
            );
        }
        return;
    }
    assert!(
        facts["absent"].is_null(),
        "authored applicability cannot discard a real input"
    );
    assert_eq!(
        super::agreement_context::identity(context).expect("unambiguous invocation identity"),
        facts["context"],
        "variant invocation/configuration/held-edit facts differ"
    );
    context["truth"] = facts.clone();
}

pub fn key(question: &Question) -> String {
    match question.surface {
        "show" | "refs" => format!(
            "{}:{}",
            question.surface,
            question.args[0].as_str().unwrap()
        ),
        other => other.into(),
    }
}

pub fn attach(context: &Value, key: &str, producer: &str, actual: Value, errors: &mut Vec<String>) {
    let definition = &context["truth"]["calls"][key];
    let expected = definition
        .get(producer)
        .expect("preflight requires this exact authored question");
    if &actual != expected {
        errors.push(format!(
            "input-authored {key}/{producer}: {actual} against {expected}"
        ));
    }
    evidence::witness(json!({"oracle":"input-authored-facts","complete":true,
        "key":key,"producer":producer,"point":definition["point"],
        "authoredFrom":definition["authoredFrom"],"facts":expected}));
}

fn stream(bytes: &[u8]) -> Value {
    let text = String::from_utf8_lossy(bytes);
    let rows = text
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>();
    rows.map_or_else(|_| json!(text), |rows| json!(rows))
}

pub fn cli_fact(surface: &str, output: &Output, context: &Value) -> Value {
    let projected = super::agreement_sites::output(surface, output, context);
    paths::logical_value(
        json!({"exit":output.status.code(),
        "stdout":stream(&projected.stdout),"stderr":stream(&projected.stderr)}),
        context,
    )
}

pub fn cli(context: &Value, question: &Question, output: &Output, errors: &mut Vec<String>) {
    assert_eq!(
        context["truth"]["calls"][key(question)]["point"],
        question.point,
        "expected fact must cite this question's defining point"
    );
    attach(
        context,
        &key(question),
        "cli",
        cli_fact(question.surface, output, context),
        errors,
    );
}

pub fn host(
    context: &Value,
    question: &Question,
    producer: &str,
    result: &Value,
    errors: &mut Vec<String>,
) {
    assert_eq!(
        context["truth"]["calls"][key(question)]["point"],
        question.point
    );
    let actual = super::agreement_sites::host(question.surface, result.clone(), context);
    attach(
        context,
        &key(question),
        producer,
        paths::logical_value(actual, context),
        errors,
    );
}

pub fn editor(context: &Value, key: &str, response: &Value, errors: &mut Vec<String>) {
    // Structured snapshots contain physical paths; serialized hover JSON is
    // decoded by logical_value. Section-map coordinates remain coordinates.
    let actual = paths::logical_value(response.clone(), context);
    attach(context, key, "editor", actual, errors);
}

pub fn expected_written(context: &Value) -> Value {
    let before = bytes::inventory(Path::new(context["saved"].as_str().unwrap()));
    bytes::resulting(&before, &context["truth"]["writtenChanges"])
        .expect("preflight requires authored resulting bytes")
}

pub fn writer(
    context: &Value,
    producer: &str,
    answer: Value,
    expected_files: &Value,
    errors: &mut Vec<String>,
) {
    let actual_files = bytes::inventory(Path::new(context["saved"].as_str().unwrap()));
    if &actual_files != expected_files {
        errors.push(format!(
            "input-authored fmt-write/{producer} anchors/resulting bytes differ"
        ));
    }
    attach(context, "fmt-write", producer, answer, errors);
    // Attach output bytes to that actual writer's receipt, including refusal or
    // unchanged-byte expectations. Equality with other writers is additional.
    evidence::witness(json!({"oracle":"input-authored-facts","complete":true,
        "key":"fmt-write","producer":producer,"point":"§REQ-one-answer.writes.anchors",
        "facts":context["truth"]["calls"]["fmt-write"][producer],"writtenFiles":expected_files,
        "authoredFrom":context["truth"]["calls"]["fmt-write"]["authoredFrom"]}));
}
