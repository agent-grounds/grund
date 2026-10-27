//! A binding's scan snapshot says which format a value source member was read
//! in: `kind` stays `json` for both, and `format` names `json` or `yaml`
//! (§FS-distribution.3.2.2.3, §FS-values.2.2).

use std::fs;
use std::path::{Path, PathBuf};

use grund_core::{node_embedding_call, node_request};
use serde_json::{Value, json};

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create fixture directory");
    fs::write(path, text).expect("write fixture");
}

fn value_home() -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("grund-value-source-format-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n\
         [schema.kinds.CONST]\nfolders = [\"values\"]\nindex = false\nform = \"value\"\n",
    );
    write(&root.join("values/a.json"), "{\"CONST-ant\": [1]}\n");
    write(
        &root.join("values/b.yaml"),
        "# Counted on 2026-10-01.\nCONST-bee: [2]\n",
    );
    root
}

#[test]
fn the_snapshot_names_each_value_sources_format() {
    let root = value_home();
    let output = node_embedding_call(node_request("scan", vec![json!(root)], &root));
    assert!(output["failure"].is_null(), "{output}");
    let declarations = output["result"]["snapshot"]["declarations"]
        .as_array()
        .expect("declarations");
    let mut sources = declarations
        .iter()
        .map(|declaration| {
            let source = &declaration["source"];
            (
                declaration["id"].as_str().unwrap_or_default().to_string(),
                source["kind"].clone(),
                source["format"].clone(),
                source["member_slice"].clone(),
            )
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        sources,
        [
            (
                "CONST-ant".to_string(),
                json!("json"),
                json!("json"),
                json!("\"CONST-ant\": [1]")
            ),
            (
                "CONST-bee".to_string(),
                json!("json"),
                json!("yaml"),
                json!("CONST-bee: [2]")
            ),
        ],
        "{}",
        Value::Array(declarations.clone())
    );
    let _ = fs::remove_dir_all(&root);
}
