//! Authored-fact omission, coordinate and writer-output controls
//! (§REQ-one-answer.proof.evidence, §REQ-one-answer.writes.anchors,
//! §FS-show.2.4, §FS-fmt.6.2).

use super::{
    agreement_expectations as expectations, agreement_expected_bytes as bytes,
    agreement_tree::{CONFIG_SLUG, Tree},
    agreement_truth as truth,
};
use serde_json::{Value, json};

fn facts() -> Value {
    // These are test-authored facts, not output obtained from a reader.
    json!({"id":"FS-alpha","section":"1","path":"<repo>/docs/fs.md","line":5,
        "end_line":9,"body":"## 1: First\n\nAuthored body.\n",
        "anchor":"1-first","sections":[{"path":"1.1","title":"Nested","depth":2}],
        "measurements":[{"unit":"words","lead":4,"full":6}]})
}

#[test]
fn exact_authored_generic_facts_reject_omissions_and_every_complex_field_corruption() {
    let expected = facts();
    let context = json!({"truth":{"calls":{"show:FS-alpha.1":{
        "point":"§REQ-one-answer.sections.map","authoredFrom":"docs/fs.md:5-9 and defining points",
        "cli":expected,"python":expected,"node":expected}}}});
    for producer in ["cli", "python", "node"] {
        let mut errors = Vec::new();
        truth::attach(
            &context,
            "show:FS-alpha.1",
            producer,
            expected.clone(),
            &mut errors,
        );
        assert!(errors.is_empty());
        let mut corruptions = vec![Value::Null, json!([]), json!({})];
        for field in [
            "body",
            "end_line",
            "sections",
            "anchor",
            "measurements",
            "path",
            "line",
            "id",
        ] {
            let mut wrong = expected.clone();
            wrong[field] = json!("common corruption");
            corruptions.push(wrong);
            let mut wrong = expected.clone();
            wrong.as_object_mut().unwrap().remove(field);
            corruptions.push(wrong);
        }
        corruptions.push(json!({"failure":{"code":"invalid-config","message":"unrelated refusal"},"result":null}));
        for wrong in corruptions {
            errors.clear();
            truth::attach(&context, "show:FS-alpha.1", producer, wrong, &mut errors);
            assert!(
                !errors.is_empty(),
                "shared corruption escaped authored facts"
            );
        }
    }
    let context = json!({"truth":{"calls":{"list":{"point":"§FS-list.3",
        "authoredFrom":"authored absence of declarations in an empty input","cli":[]}}}});
    let mut errors = Vec::new();
    truth::attach(&context, "list", "cli", json!([]), &mut errors);
    assert!(errors.is_empty());
    truth::attach(&context, "list", "cli", json!([facts()]), &mut errors);
    assert!(
        !errors.is_empty(),
        "wrong nonempty answer escaped authored emptiness"
    );
}

#[test]
fn exact_question_inventory_is_independent_of_a_returned_empty_catalog() {
    let variant = json!({"coordinates":["FS-alpha","FS-alpha.1","FS-absent"],
        "calls":{"editor:snapshot":{"editor":{"snapshot":{}}}}});
    let coordinates = expectations::coordinates(&variant).unwrap();
    assert_eq!(coordinates.len(), 3);
    let obligations = expectations::obligations(&variant).unwrap();
    for coordinate in coordinates {
        for producer in ["cli", "python", "node"] {
            assert!(obligations.contains(&(format!("show:{coordinate}"), producer.into())));
            assert!(obligations.contains(&(format!("refs:{coordinate}"), producer.into())));
        }
        assert!(obligations.contains(&(format!("editor:show:{coordinate}"), "editor".into())));
    }
    let mut incomplete = variant;
    incomplete["coordinates"] = json!([]);
    assert!(expectations::coordinates(&incomplete).is_err());
    incomplete["coordinates"] = json!(["FS-alpha", "FS-alpha"]);
    assert!(expectations::coordinates(&incomplete).is_err());
    assert!(expectations::validate_variant(&incomplete).is_err());
}

#[test]
fn generic_writer_oracle_rejects_common_wrong_anchors_and_pins_unchanged_bytes() {
    let text = format!("See [{}FS-alpha](fs.md#fs-alpha-alpha).\n", '§');
    let expected_tree = Tree::new(
        "authored-writer-output",
        &[("grund.toml", CONFIG_SLUG), ("docs/citations.md", &text)],
    );
    let before = bytes::inventory(&expected_tree.0);
    assert_eq!(bytes::resulting(&before, &json!({})).unwrap(), before);
    assert!(
        bytes::resulting(&before, &Value::Null).is_err(),
        "no output oracle is not unchanged bytes"
    );
    let changes = json!({"docs/citations.md":{"bytes":text.as_bytes()}});
    let expected = bytes::resulting(&before, &changes).unwrap();
    for producer in ["cli", "python", "node"] {
        let wrong = text.replace("#fs-alpha-alpha", "#wrong-anchor");
        let tree = Tree::new(
            producer,
            &[("grund.toml", CONFIG_SLUG), ("docs/citations.md", &wrong)],
        );
        let mut definition = json!({
            "point":"§REQ-one-answer.writes.anchors","authoredFrom":"authored resulting anchor bytes",
        });
        definition[producer] = json!({"failure":null,"result":{}});
        let context = json!({"saved":tree.0,"truth":{"calls":{"fmt-write":definition}}});
        let mut errors = Vec::new();
        truth::writer(
            &context,
            producer,
            json!({"failure":null,"result":{}}),
            &expected,
            &mut errors,
        );
        assert!(
            !errors.is_empty(),
            "generic writer must reject equal-but-wrong output bytes"
        );
    }
}
