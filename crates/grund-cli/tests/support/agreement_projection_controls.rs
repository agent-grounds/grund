//! Replay real successful/refused host envelopes through the integrated
//! comparator (§FS-distribution.3.3.1, §REQ-one-answer.proof.matrix).
//! Recorded answers here are regression inputs, not the independent S4 oracle.

use super::{
    agreement_compare as compare, agreement_host_projection as projection,
    agreement_questions as questions, agreement_sites as sites,
};
use serde_json::{Value, json};
use std::process::{ExitStatus, Output};

fn record(surface: &str) -> Value {
    let bytes = match surface {
        "list" => include_str!("../fixtures/agreement_envelopes/list.json"),
        "list-size" => include_str!("../fixtures/agreement_envelopes/list-size.json"),
        "show-batch" => include_str!("../fixtures/agreement_envelopes/show-batch.json"),
        "refs" => include_str!("../fixtures/agreement_envelopes/refs.json"),
        "refs-nonempty" => include_str!("../fixtures/agreement_envelopes/refs-nonempty.json"),
        "show" => include_str!("../fixtures/agreement_envelopes/show.json"),
        _ => panic!("unrecorded control"),
    };
    serde_json::from_str(bytes).unwrap()
}

fn context() -> Value {
    json!({"base":"/fixture/repo","saved":"/fixture/repo","reportBase":"/fixture/repo",
        "path":"/fixture/repo","explicit":true,"files":[],"textFiles":[],"scopes":{}})
}

fn output(record: &Value) -> Output {
    #[cfg(unix)]
    let status = {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw((record["exit"].as_i64().unwrap() as i32) << 8)
    };
    #[cfg(windows)]
    let status = {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(record["exit"].as_u64().unwrap() as u32)
    };
    Output {
        status,
        stdout: record["stdout"].as_str().unwrap().as_bytes().to_vec(),
        stderr: record["stderr"].as_str().unwrap().as_bytes().to_vec(),
    }
}

fn compare(record: &Value) -> Vec<String> {
    let context = context();
    let surface = record["surface"].as_str().unwrap();
    let question = questions::tree_questions(&context)
        .into_iter()
        .chain(questions::coordinate_questions(&context, "FS-999-missing"))
        .find(|q| q.surface == surface)
        .unwrap();
    let cli = sites::output(surface, &output(&record["cli"]), &context);
    let mut errors = Vec::new();
    for host in ["python", "node"] {
        let projected = sites::host(surface, record[host].clone(), &context);
        compare::compare_cli(&question, &cli, &projected, &mut errors);
    }
    let python = projection::from_python(
        record["python"].clone(),
        question.node_operation,
        &mut errors,
    );
    if python != projection::node_common(record["node"].clone(), question.node_operation) {
        errors.push("common host envelopes differ".into());
    }
    errors
}

#[test]
fn successful_real_envelopes_preserve_nullable_fields_and_reject_wrong_sites() {
    for surface in ["list", "list-size", "show-batch", "refs", "show"] {
        let mut input = record(surface);
        assert!(
            compare(&input).is_empty(),
            "{surface}: {:?}",
            compare(&input)
        );
        for host in ["python", "node"] {
            let normalized = sites::host(surface, input[host].clone(), &context());
            assert_eq!(normalized["failure"], input[host]["failure"]);
            if surface == "show-batch" {
                assert_eq!(
                    normalized["result"]["records"][0].get("failure"),
                    input[host]["result"]["records"][0].get("failure")
                );
            }
        }
        if surface == "list" {
            input["python"]["result"]["entries"][0]["path"] = json!("wrong.md");
            assert!(!compare(&input).is_empty());
        }
        let mut wrong = record(surface);
        let primary = if wrong["python"]["failure"].is_null() {
            "result"
        } else {
            "failure"
        };
        wrong["python"][primary]["run_cautions"] =
            json!([{"message":"corrupted duplicate warning channel"}]);
        assert!(
            !compare(&wrong).is_empty(),
            "removed duplicate caution field escaped"
        );
    }
    for absent in [
        Value::Null,
        json!({}),
        json!({"result":null}),
        json!({"failure":null}),
    ] {
        assert_eq!(
            sites::host("show-batch", absent.clone(), &context()),
            absent
        );
        let mut rows = vec![absent.clone()];
        sites::rows("show-batch", &mut rows, &context());
        assert_eq!(rows, vec![absent]);
    }
    let refused = record("show")["node"].clone();
    assert_eq!(
        projection::node_common(refused.clone(), "showBatch"),
        refused,
        "batch setup refusal must retain result:null"
    );
    assert_eq!(projection::node_common(json!({}), "showBatch"), json!({}));
}

#[test]
fn mixed_real_batch_records_preserve_success_and_refusal_paths() {
    let mut input = record("show-batch");
    let refused = record("show");
    let mut cli: Vec<Value> = input["cli"]["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .take(1)
        .collect();
    let query = json!({"id":"FS-999-missing","section":null});
    cli.push(json!({"query":query,"ok":false,"result":null,
        "error":serde_json::from_str::<Value>(refused["cli"]["stderr"].as_str().unwrap()).unwrap()}));
    input["cli"]["stdout"] = json!(cli.iter().map(|row| format!("{row}\n")).collect::<String>());
    input["cli"]["exit"] = json!(1);
    for host in ["python", "node"] {
        let good = input[host]["result"]["records"][0].clone();
        let bad = json!({"query":query,"result":null,"failure":refused[host]["failure"]});
        let bad = if host == "node" {
            let mut failure = refused[host]["failure"].clone();
            failure["run_cautions"] = refused[host]["run_cautions"].clone();
            json!({"query":query,"ok":false,"failure":failure})
        } else {
            bad
        };
        input[host]["result"]["records"] = json!([good, bad]);
    }
    assert!(compare(&input).is_empty(), "{:?}", compare(&input));
    let mut wrong = input.clone();
    wrong["python"]["result"]["records"][1]["failure"]["message"] = json!("unrelated refusal");
    assert!(!compare(&wrong).is_empty());
    wrong = input.clone();
    wrong["python"]["result"]["records"][0]["result"]["run_cautions"] =
        json!([{"message":"wrong batch cautions"}]);
    assert!(!compare(&wrong).is_empty());
    wrong = input.clone();
    let serialized = &mut wrong["python"]["result"]["records"][0]["result"]["json"];
    let mut body: Value = serde_json::from_str(serialized.as_str().unwrap()).unwrap();
    body["path"] = json!("wrong.md");
    *serialized = json!(body.to_string());
    assert!(!compare(&wrong).is_empty());
    let mut report = json!({"failure":null,"result":{"selected_report":{"errors":[
        {"path":"docs/a.md","sites":[{"path":"docs/b.md"}]}]}}});
    report = sites::host("check", report, &context());
    assert_eq!(
        report["result"]["selected_report"]["errors"][0]["path"],
        "<repo>/docs/a.md"
    );
    assert_eq!(
        report["result"]["selected_report"]["errors"][0]["sites"][0]["path"],
        "<repo>/docs/b.md"
    );
    assert!(
        report["result"]["selected_report"]
            .get("warnings")
            .is_none()
    );
}

#[test]
fn every_returned_refs_count_and_summary_field_is_observable_including_partials() {
    let empty = record("refs")["python"].clone();
    let mut nonempty = empty.clone();
    // An authored nonempty consistency input extends the real empty envelope.
    // It does not claim execution of these synthetic hits by a binding.
    nonempty["result"]["hits"] = json!([
        {"project":null,"path":"a.md","line":1},
        {"project":null,"path":"a.md","line":1},
        {"project":"other","path":"b.md","line":2}]);
    nonempty["result"]["site_total"] = json!(3);
    nonempty["result"]["file_total"] = json!(2);
    nonempty["result"]["file_summaries"] = json!([
        {"project":null,"path":"a.md","sites":2},
        {"project":"other","path":"b.md","sites":1}]);
    for baseline in [empty, record("refs-nonempty")["python"].clone(), nonempty] {
        for partial in [false, true] {
            let wrap = |result: Value| {
                if partial {
                    let mut envelope = record("show")["python"].clone();
                    envelope["failure"]["partial_output"] = result;
                    envelope
                } else {
                    json!({"failure":null,"result":result,"run_cautions":[]})
                }
            };
            let result = baseline["result"].clone();
            let mut errors = Vec::new();
            projection::from_python(wrap(result.clone()), "refs", &mut errors);
            assert!(errors.is_empty(), "{errors:?}");
            if partial {
                let mut sparse = result.clone();
                for field in ["site_total", "file_total", "file_summaries"] {
                    sparse.as_object_mut().unwrap().remove(field);
                }
                projection::from_python(wrap(sparse.clone()), "refs", &mut errors);
                assert!(
                    errors.is_empty(),
                    "sparse generic partial JSON is valid: {errors:?}"
                );
                for field in ["site_total", "file_total"] {
                    let mut wrong = sparse.clone();
                    wrong[field] = json!(99999);
                    errors.clear();
                    projection::from_python(wrap(wrong), "refs", &mut errors);
                    assert!(!errors.is_empty(), "returned sparse partial count escaped");
                }
            }
            let mut corruptions = Vec::new();
            for field in ["site_total", "file_total"] {
                let mut wrong = result.clone();
                wrong[field] = json!(99999);
                corruptions.push(wrong);
            }
            if result["file_summaries"].as_array().unwrap().is_empty() {
                let mut wrong = result.clone();
                wrong["file_summaries"] = json!([{"project":null,"path":"wrong.md","sites":1}]);
                corruptions.push(wrong);
            } else {
                for field in ["project", "path", "sites"] {
                    let mut wrong = result.clone();
                    wrong["file_summaries"][0][field] = json!("corrupted");
                    corruptions.push(wrong);
                }
                let mut wrong = result.clone();
                wrong["file_summaries"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("project");
                corruptions.push(wrong);
                let mut wrong = result.clone();
                wrong["file_summaries"] = json!([]);
                corruptions.push(wrong);
            }
            for wrong in corruptions {
                errors.clear();
                projection::from_python(wrap(wrong), "refs", &mut errors);
                assert!(
                    !errors.is_empty(),
                    "corrupt returned refs fields escaped (partial={partial})"
                );
            }
        }
    }
}
