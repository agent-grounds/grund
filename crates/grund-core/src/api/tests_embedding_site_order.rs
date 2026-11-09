//! Test module: a host's query failure carries a duplicate's sites in the one order
//! (§FS-errors.4.1.1, §FS-distribution.3.3.2), through Python's entrypoint and Node's.

use std::path::Path;

use serde_json::{Value, json};

use super::*;
use crate::testing::{test_root, write};

fn duplicate_on_lines_nine_and_ten(root: &Path) {
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema]\nid_format = \"{kind}-{slug}\"\n\
         [schema.kinds.SPEC]\nfolders = [\"docs\"]\nindex = false\n",
    );
    write(
        &root.join("docs/one.md"),
        "# SPEC-other: O\n\nBody.\n\nFive.\n\nSeven.\n\n\
         # SPEC-login: First\n# SPEC-login: Second\n\nBody.\n",
    );
}

fn lines(failure: &Value) -> Vec<u64> {
    failure["sites"]
        .as_array()
        .unwrap_or_else(|| panic!("a query failure with sites: {failure}"))
        .iter()
        .map(|site| site["line"].as_u64().expect("a line"))
        .collect()
}

#[test]
fn a_hosts_query_failure_lists_a_duplicates_sites_in_line_order() {
    let root = test_root("a_hosts_query_failure_lists_a_duplicates_sites_in_line_order");
    duplicate_on_lines_nine_and_ten(&root);
    let python = embedding_call(EmbeddingRequest {
        operation: "show".into(),
        root: root.clone(),
        explicit: true,
        args: vec![json!("SPEC-login")],
        options: json!({}),
    });
    assert_eq!(lines(&python["failure"]), [9, 10], "Python: {python}");
    let node = node_embedding_call(node_request(
        "show",
        vec![json!("SPEC-login"), json!({"root": root.to_string_lossy()})],
        &root,
    ));
    assert_eq!(lines(&node["failure"]), [9, 10], "Node: {node}");
}
