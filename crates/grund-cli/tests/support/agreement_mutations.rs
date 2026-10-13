//! Fmt writes are compared over independent copies, including host conversions
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.writes.anchors, §FS-fmt.6.2).

use super::agreement_corpus::{compare_tree, invoke};
use super::agreement_evidence as evidence;
use super::agreement_host::Host;
use super::agreement_oracles::text;
use grund_core::{EmbeddingRequest, embedding_call, node_embedding_call, node_request};
use serde_json::{Value, json};
use std::path::Path;

fn clone(host: &mut Host, context: &Value, name: &str) -> Value {
    let destination = host.request(json!({"op":"clone","source":context["saved"],"name":name}));
    let original = text(context, "saved");
    let mut cloned = context.clone();
    for field in ["saved", "cwd", "path"] {
        cloned[field] =
            json!(text(context, field).replacen(&original, destination.as_str().unwrap(), 1));
    }
    compare_tree(
        Path::new(&original),
        Path::new(cloned["saved"].as_str().unwrap()),
    )
    .expect("mutable input is an independent byte-identical copy");
    cloned
}

fn normalized(mut value: Value, context: &Value) -> Value {
    fn visit(value: &mut Value, root: &str) {
        match value {
            Value::String(s) => *s = s.replace(root, "<repo>"),
            Value::Array(values) => values.iter_mut().for_each(|v| visit(v, root)),
            Value::Object(values) => values.values_mut().for_each(|v| visit(v, root)),
            _ => {}
        }
    }
    visit(&mut value, &text(context, "saved"));
    value
}

pub fn execute(host: &mut Host, context: &Value, errors: &mut Vec<String>) {
    let cli = clone(host, context, "fmt-cli");
    let python = clone(host, context, "fmt-python");
    let node = clone(host, context, "fmt-node");
    let rust_python = clone(host, context, "fmt-rust-python");
    let rust_node = clone(host, context, "fmt-rust-node");
    let mut args = vec!["fmt".into(), "--write".into(), "--cross-refs".into()];
    if context["explicit"] == true {
        args.push(text(&cli, "path"));
    }
    let output = invoke(Path::new(&text(&cli, "cwd")), &args, None);
    let witness = json!({"oracle":"independent-mutation-input-bytes",
        "point":"§REQ-one-answer.writes.anchors","before":context["savedInventory"]});
    evidence::witness(witness.clone());
    let request = json!({"operation":"fmt","cwd":python["cwd"],"root":python["path"],
        "explicit":context["explicit"],"args":[],"options":{"write":true,"cross_refs":true}});
    let actual = host.call("python", request);
    evidence::witness(witness.clone());
    let expected = embedding_call(EmbeddingRequest {
        operation: "fmt".into(),
        root: text(&rust_python, "path").into(),
        explicit: context["explicit"] == true,
        args: Vec::new(),
        options: json!({"write":true,"cross_refs":true}),
    });
    if normalized(actual.clone(), &python) != normalized(expected, &rust_python) {
        errors.push("Python fmt write envelope differs from its independent-copy peer".into());
    }
    let options = if context["explicit"] == true {
        json!({"root":node["path"],"write":true,"crossRefs":true})
    } else {
        json!({"write":true,"crossRefs":true})
    };
    let actual_node = host.call(
        "node",
        json!({"operation":"fmt","cwd":node["cwd"],"args":[options]}),
    );
    evidence::witness(witness);
    let options = if context["explicit"] == true {
        json!({"root":rust_node["path"],"write":true,"crossRefs":true})
    } else {
        json!({"write":true,"crossRefs":true})
    };
    let expected = node_embedding_call(node_request(
        "fmt",
        vec![options],
        Path::new(&text(&rust_node, "cwd")),
    ));
    if normalized(actual_node.clone(), &node) != normalized(expected, &rust_node) {
        errors.push("Node fmt write envelope differs from its independent-copy peer".into());
    }
    for peer in [&python, &node, &rust_python, &rust_node] {
        if let Err(error) = compare_tree(
            Path::new(&text(&cli, "saved")),
            Path::new(&text(peer, "saved")),
        ) {
            errors.push(format!("fmt write common-domain bytes: {error}"));
        }
    }
    let complete = |answer: &Value| {
        answer["failure"].is_null()
            && answer["result"]["scan_errors"]
                .as_array()
                .is_some_and(Vec::is_empty)
    };
    if output.status.success() != complete(&actual)
        || output.status.success() != complete(&actual_node)
    {
        errors.push("fmt write refusal differs between the CLI and hosts".into());
    }
}
