//! Fmt writes are compared over independent copies, including host conversions
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.writes.anchors, §FS-fmt.6.2).

use super::agreement_corpus::{compare_tree, invoke};
use super::agreement_evidence as evidence;
use super::agreement_host::Host;
use super::agreement_oracles::text;
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
    for surface in ["cli", "python", "node"] {
        for field in ["projectRoot", "reportBase", "invocationBase", "configFile"] {
            if let Some(path) = context["scopes"][surface][field].as_str()
                && let Ok(relative) = Path::new(path).strip_prefix(&original)
            {
                cloned["scopes"][surface][field] =
                    json!(Path::new(destination.as_str().unwrap()).join(relative));
            }
        }
    }
    compare_tree(
        Path::new(&original),
        Path::new(cloned["saved"].as_str().unwrap()),
    )
    .expect("mutable input is an independent byte-identical copy");
    cloned
}

fn normalized(value: Value, context: &Value) -> Value {
    super::agreement_paths::logical_value(value, context)
}

pub fn execute(host: &mut Host, context: &Value, errors: &mut Vec<String>) {
    let cli = clone(host, context, "fmt-cli");
    let python = clone(host, context, "fmt-python");
    let node = clone(host, context, "fmt-node");
    let mut args = vec!["fmt".into(), "--write".into(), "--cross-refs".into()];
    if context["explicit"] == true {
        args.push(text(&cli, "path"));
    }
    let output = invoke(Path::new(&text(&cli, "cwd")), &args, None);
    let witness = json!({"oracle":"independent-mutation-input-bytes","complete":false,
        "point":"§REQ-one-answer.writes.anchors","before":context["savedInventory"]});
    evidence::witness(witness.clone());
    let request = json!({"operation":"fmt","cwd":python["cwd"],"root":python["path"],
        "explicit":context["explicit"],"args":[],"options":{"write":true,"cross_refs":true},
        "scope":python["scopes"]["python"]});
    let actual = host.call("python", request);
    evidence::witness(witness.clone());
    let options = if context["explicit"] == true {
        json!({"root":node["path"],"write":true,"crossRefs":true})
    } else {
        json!({"write":true,"crossRefs":true})
    };
    let actual_node = host.call(
        "node",
        json!({"operation":"fmt","cwd":node["cwd"],"args":[options],"scope":node["scopes"]["node"]}),
    );
    evidence::witness(witness);
    let expected = super::agreement_host_projection::from_python(actual.clone(), "fmt");
    if normalized(actual_node.clone(), &node) != normalized(expected, &python) {
        errors.push("actual mutable host complete envelopes differ".into());
    }
    let projected = super::agreement_sites::output("fmt", &output, &cli);
    for (answer, copy) in [(&actual, &python), (&actual_node, &node)] {
        if !answer["failure"].is_null() {
            let normalized = normalized(answer.clone(), copy);
            super::agreement_refusals::compare(&projected, &normalized["failure"], errors);
        }
    }
    for peer in [&python, &node] {
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
