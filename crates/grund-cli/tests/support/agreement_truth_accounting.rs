//! Exact authored question/producer accounting over completed receipts
//! (§REQ-one-answer.proof.evidence, §FS-distribution.3.3.1).

use super::{agreement_evidence::Call, agreement_expectations};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub type Obligation = (String, String, String, String);

pub fn observe(
    call: &Call,
    manifests: &BTreeMap<String, Value>,
    case: &str,
    variant: &str,
) -> Option<Obligation> {
    if call.expected["oracle"] != "input-authored-facts" {
        return None;
    }
    let key = call.expected["key"]
        .as_str()
        .expect("exact authored question");
    let producer = call.expected["producer"]
        .as_str()
        .expect("actual answering producer");
    assert!(if matches!(producer, "python" | "node") {
        producer == call.surface
    } else if producer == "editor" {
        call.surface == "editor"
    } else {
        producer == "cli" && !matches!(call.surface.as_str(), "python" | "node" | "editor")
    });
    let mut facts = manifests[case]["variants"][variant].clone();
    // §FS-refs.2.1: validate literal note paths with the execution truth's binding.
    super::agreement_render_show::bind_refs_path(
        &mut facts,
        call.context
            .as_ref()
            .expect("verified invocation attribution"),
    );
    let definition = &facts["calls"][key];
    assert_eq!(
        call.expected["facts"], definition[producer],
        "receipt must use loaded authored facts"
    );
    Some((case.into(), variant.into(), key.into(), producer.into()))
}

pub fn required(
    manifests: &BTreeMap<String, Value>,
    materializations: &BTreeMap<(String, String), &Call>,
) -> BTreeSet<Obligation> {
    let mut required = BTreeSet::new();
    for (case, document) in manifests {
        for variant in super::agreement_corpus::VARIANTS {
            if materializations
                .get(&(case.clone(), variant.to_string()))
                .is_some_and(|call| call.result["symlinkError"]["winerror"] == 1314)
            {
                continue; // Physical Windows privilege refusal was asserted by acceptance.
            }
            for (key, producer) in
                agreement_expectations::obligations(&document["variants"][*variant]).unwrap()
            {
                required.insert((case.clone(), variant.to_string(), key, producer));
            }
        }
    }
    required
}
