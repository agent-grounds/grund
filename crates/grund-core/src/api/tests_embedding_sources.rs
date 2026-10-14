//! Test module: the source model a binding's effective config publishes
//! (§FS-config-v2.schema.sources.effective.bindings)

use serde_json::{Value, json};

use super::*;
use crate::testing::{test_root, write};

fn effective_config(root: &std::path::Path) -> Value {
    let data = embedding_call(EmbeddingRequest {
        operation: "effective_config".into(),
        root: root.to_path_buf(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    });
    assert_eq!(data["failure"], Value::Null, "{data}");
    data["result"]["config"].clone()
}

/// A v2 snapshot publishes the model under `sources`, every admitted language with
/// its own hosts, and leaves the v1-shaped `scan` keys v2 has no counterpart for
/// empty: no global prefix, no docstring switch, no `include` and no `exclude`.
#[test]
fn a_v2_snapshot_publishes_the_source_model_and_empties_the_v1_scan_keys() {
    let root = test_root("a_v2_snapshot_publishes_the_source_model");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 2
project_name = "fixture"
[schema.sources]
languages = ["markdown", "json", "python", "notes", "records"]
exclude = ["generated", "src/**/vendor/**"]
[schema.sources.extensions]
pyi = "python"
note = "notes"
rec = "records"
[schema.sources.definitions.notes]
class = "source"
comment_prefixes = ["%%"]
[schema.sources.definitions.records]
class = "data"
[schema.kinds.FS]
"#,
    );
    let config = effective_config(&root);
    let language = |name: &str, class: &str, suffixes: &[&str], prefixes: &[&str]| {
        json!({"name":name,"class":class,"suffixes":suffixes,"comment_prefixes":prefixes,
            "doc_comments":"none","docstrings":false,"builtin":!suffixes.is_empty()})
    };
    let mut python = language("python", "source", &["py"], &["#"]);
    python["doc_comments"] = json!("python");
    python["docstrings"] = json!(true);
    assert_eq!(
        config["sources"],
        json!({
            "languages": [
                language("markdown", "document", &["md"], &["//", "#", ";", "--", "*", "/*"]),
                language("json", "data", &["json"], &[]),
                python,
                language("notes", "source", &[], &["%%"]),
                language("records", "data", &[], &[]),
            ],
            "extensions": {"pyi": "python", "note": "notes", "rec": "records"},
            "declarations": "doc-comments",
            "exclude": ["generated", "src/**/vendor/**"],
        })
    );
    assert_eq!(
        config["scan"],
        json!({"include":null,"exclude":[],"extensions":["pyi", "note", "md", "py"],
            "comment_prefixes":[],"docstring_python":false,"respect_gitignore":true})
    );
}

/// A v1 snapshot has no `sources` key, and its `scan` keeps v1's prefixes and
/// docstring switch, so it keeps its bytes.
#[test]
fn a_v1_snapshot_publishes_no_source_model() {
    let root = test_root("a_v1_snapshot_publishes_no_source_model");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    let config = effective_config(&root);
    assert!(config.get("sources").is_none(), "{config}");
    assert_eq!(
        config["scan"]["comment_prefixes"],
        json!(["//", "#", ";", "--", "*", "/*"])
    );
    assert_eq!(config["scan"]["docstring_python"], json!(true));
}
