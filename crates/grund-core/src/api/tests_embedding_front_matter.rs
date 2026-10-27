//! Test module: a front-matter coordinate in the binding snapshot's `Section`
//! record (§FS-fields.front-matter.surfaces, §FS-distribution.3.2.2.3)

use serde_json::{Value, json};

use super::*;
use crate::testing::{test_root, write};

/// The `sections` of the one declaration the Node binding's `scan` snapshot holds.
fn snapshot_sections(root: &std::path::Path) -> Vec<Value> {
    let request = node_request("scan", vec![json!(root.to_string_lossy())], root);
    let data = node_embedding_call(request);
    assert_eq!(data["failure"], Value::Null, "{data}");
    let declarations = data["result"]["snapshot"]["declarations"]
        .as_array()
        .unwrap();
    assert_eq!(declarations.len(), 1, "{data}");
    declarations[0]["sections"].as_array().unwrap().clone()
}

/// An entry of `sections` like any other: `headingLevel` `0` is the front-matter
/// mark, `title` the key's one-line value or the empty string, `value` and
/// `valueRoot` `null`, and the record gains no field, so a heading's entry keeps
/// its bytes.
#[test]
fn a_front_matter_coordinate_is_a_section_record_at_heading_level_zero() {
    let root = test_root("a_front_matter_coordinate_is_a_section_record");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 2
project_name = "madr"
[schema.kinds.ADR]
folders = ["docs/adr"]
index = false
[schema.kinds.ADR.fields.front-matter]
"#,
    );
    write(
        &root.join("docs/adr/cache.md"),
        "---\nstatus: accepted\nsources:\n  - a\n---\n\n# ADR-cache: Cache\n\nText.\n\n## context: Context\n\nWhy.\n",
    );
    let section = |path: &str, title: &str, line: u64, level: u64| {
        json!({"path":path,"title":title,"line":line,"heading_level":level,
            "value":null,"value_root":null})
    };
    assert_eq!(
        snapshot_sections(&root),
        vec![
            section("context", "context: Context", 11, 2),
            section("front-matter", "", 1, 0),
            section("front-matter.sources", "", 3, 0),
            section("front-matter.status", "accepted", 2, 0),
        ]
    );
}
