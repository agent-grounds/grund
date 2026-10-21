//! Deleting a declaration, nested section or alias and its calls must fail
//! against physical input (§REQ-one-answer.proof.evidence, §FS-workspace.3,
//! §FS-declarations.section.relative). No catalog supplies these control facts.

use super::{agreement_expectations as expectations, agreement_query_inventory as queries};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub fn inventory() -> Value {
    let citation = format!("See {}FS-missing.\n", '§');
    let source = format!("# FS-alpha: Alpha\n## 1: First\n### .1: Nested\n{citation}");
    let declaration = "docs/alpha.md:0";
    let parent = format!("docs/alpha.md:{}", "# FS-alpha: Alpha\n".len());
    let mut offset = 0;
    let mut segments = Vec::new();
    // Explicit authored classifications; the iterator only lays out their bytes.
    for (line, role) in [
        (
            "# FS-alpha: Alpha\n",
            json!({"kind":"declaration","id":"FS-alpha",
            "project":null,"sectionSeparator":".","point":"§FS-declarations.line"}),
        ),
        (
            "## 1: First\n",
            json!({"kind":"section","declaration":declaration,
            "component":"1","parent":null,"point":"§FS-declarations.section"}),
        ),
        (
            "### .1: Nested\n",
            json!({"kind":"section","declaration":declaration,
            "component":".1","parent":parent,"point":"§FS-declarations.section.relative"}),
        ),
        (
            citation.as_str(),
            json!({"kind":"citation","query":"FS-missing",
            "point":"§FS-refs.2"}),
        ),
    ] {
        let mut role = role;
        role["authoredFrom"] = json!("explicitly authored control input and defining point");
        segments.push(json!({"start":offset,"end":offset+line.len(),"source":line,"role":role}));
        offset += line.len();
    }
    let alias = "project_name = \"root\"\n";
    json!({"absentQuery":"FS-absent","files":{
        "docs/alpha.md":{"bytes":source.as_bytes(),"segments":segments},
        "grund.toml":{"bytes":alias.as_bytes(),"segments":[{
            "start":0,"end":alias.len(),"source":alias,"role":{
                "kind":"alias","alias":"root","project":null,"point":"§FS-workspace.3",
                "authoredFrom":"authored project_name entry"}}]}}})
}

pub fn variant() -> Value {
    let mut context = json!({});
    for name in super::agreement_truth::CONTEXT {
        context[*name] = Value::Null;
    }
    let mut variant = json!({"absent":null,"context":context,"queryInventory":inventory(),
        "coordinates":["FS-alpha","FS-alpha.1","FS-alpha.1.1","FS-missing","FS-absent",
            "root/FS-alpha","root/FS-alpha.1","root/FS-alpha.1.1","root/FS-absent"],
        "calls":{"editor:snapshot":{"editor":{}}},"writtenChanges":{}});
    for (key, producer) in expectations::obligations(&variant).unwrap() {
        variant["calls"][&key]["point"] = json!("§REQ-one-answer.proof.evidence");
        variant["calls"][&key]["authoredFrom"] =
            json!("query-shape control, never corpus answer facts");
        variant["calls"][&key][&producer] = json!({});
    }
    variant
}

fn remove_queries(variant: &mut Value, coordinates: &[&str]) {
    variant["coordinates"]
        .as_array_mut()
        .unwrap()
        .retain(|v| !coordinates.contains(&v.as_str().unwrap()));
    for coordinate in coordinates {
        for prefix in ["show:", "refs:", "editor:show:"] {
            variant["calls"]
                .as_object_mut()
                .unwrap()
                .remove(&format!("{prefix}{coordinate}"));
        }
    }
}

#[test]
pub(super) fn declaration_section_alias_subsets_fail_even_with_all_their_calls_deleted() {
    let good = variant();
    assert!(expectations::validate_variant(&good).is_ok());
    for coordinates in [
        vec!["FS-alpha"],
        vec!["FS-alpha.1.1"],
        vec![
            "root/FS-alpha",
            "root/FS-alpha.1",
            "root/FS-alpha.1.1",
            "root/FS-absent",
        ],
        vec!["FS-missing"],
        vec!["FS-absent"],
    ] {
        let mut subset = good.clone();
        remove_queries(&mut subset, &coordinates);
        assert!(expectations::validate_variant(&subset).is_err());
        assert!(expectations::obligations(&subset).is_err());
    }
    for (file, index) in [
        ("docs/alpha.md", 0),
        ("docs/alpha.md", 2),
        ("grund.toml", 0),
    ] {
        let mut inventory = inventory();
        inventory["files"][file]["segments"]
            .as_array_mut()
            .unwrap()
            .remove(index);
        assert!(
            queries::closure(&inventory).is_err(),
            "deleted physical fact escaped partition"
        );
    }
    for (file, index, field) in [
        ("docs/alpha.md", 0, "id"),
        ("docs/alpha.md", 2, "component"),
        ("grund.toml", 0, "alias"),
    ] {
        let mut inventory = inventory();
        inventory["files"][file]["segments"][index]["role"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            queries::closure(&inventory).is_err(),
            "deleted semantic fact escaped closure"
        );
    }
}

#[test]
pub(super) fn physical_bytes_and_membership_reject_deleting_or_rewriting_the_inventory_input() {
    let good = variant();
    let mut original = json!({});
    for (path, file) in good["queryInventory"]["files"].as_object().unwrap() {
        original[format!("repo/{path}")] = json!({"bytes":file["bytes"]});
    }
    assert!(queries::validate_inputs(&good, &original).is_ok());
    let mut corrupt = good.clone();
    corrupt["queryInventory"]["files"]
        .as_object_mut()
        .unwrap()
        .remove("grund.toml");
    remove_queries(
        &mut corrupt,
        &[
            "root/FS-alpha",
            "root/FS-alpha.1",
            "root/FS-alpha.1.1",
            "root/FS-absent",
        ],
    );
    // The resulting self-contained closure is smaller but physical membership
    // still refuses it, even if every matching question was removed.
    assert!(queries::validate_inputs(&corrupt, &original).is_err());
    let mut corrupt = good.clone();
    corrupt["queryInventory"]["files"]["docs/alpha.md"]["segments"][0]["source"] =
        json!("# FS-wrong: Wrong\n");
    assert!(queries::closure(&corrupt["queryInventory"]).is_err());
    corrupt["queryInventory"]["files"]["docs/alpha.md"]["segments"][0]["role"]["id"] =
        json!("FS-wrong");
    assert!(queries::validate_inputs(&corrupt, &original).is_ok());
    assert!(queries::closure(&corrupt["queryInventory"]).is_err());
    let mut corrupt = good;
    corrupt["queryInventory"]["files"]["docs/alpha.md"]["bytes"] = json!(b"smaller input");
    assert!(queries::validate_inputs(&corrupt, &original).is_err());
}

#[test]
fn relative_nested_and_qualified_queries_are_pinned_by_authored_source_facts() {
    let inventory = inventory();
    let expected = BTreeSet::from(
        [
            "FS-alpha",
            "FS-alpha.1",
            "FS-alpha.1.1",
            "FS-missing",
            "FS-absent",
            "root/FS-alpha",
            "root/FS-alpha.1",
            "root/FS-alpha.1.1",
            "root/FS-absent",
        ]
        .map(String::from),
    );
    assert_eq!(queries::closure(&inventory).unwrap(), expected);
    let mut wrong = inventory.clone();
    wrong["files"]["docs/alpha.md"]["segments"][2]["role"]["parent"] = json!("docs/alpha.md:29");
    assert!(queries::closure(&wrong).is_err());
    let mut wrong = inventory.clone();
    wrong["files"]["grund.toml"]["segments"][0]["role"]
        .as_object_mut()
        .unwrap()
        .remove("project");
    assert!(queries::closure(&wrong).is_err());
    let mut wrong = inventory;
    wrong["absentQuery"] = json!("FS-alpha");
    assert!(queries::closure(&wrong).is_err());
}
