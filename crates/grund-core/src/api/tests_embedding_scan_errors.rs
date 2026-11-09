//! Test module: one scan failure, one message, whichever binding operation
//! publishes it (§FS-check.2.4, §AR-bindings.2)

use serde_json::{Value, json};

use super::*;
use crate::testing::{symlink, test_root, write};

fn call(root: &std::path::Path, operation: &str) -> Value {
    embedding_call(EmbeddingRequest {
        operation: operation.into(),
        root: root.to_path_buf(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    })
}

/// What a reason renders to at a path `exclude` reaches (§FS-check.2.4.3).
fn entry_message(reason: &str) -> String {
    crate::scanner::scan_error_message(&crate::scanner::ScanError::unread_entry(
        std::path::PathBuf::new(),
        reason,
    ))
}

/// Every `{"path", "message"}` pair a reply carries under a `scan_errors` key.
fn scan_error_messages(data: &Value) -> Vec<String> {
    match data {
        Value::Object(map) => map
            .iter()
            .flat_map(|(key, value)| match (key.as_str(), value) {
                ("scan_errors", Value::Array(errors)) => errors
                    .iter()
                    .map(|e| {
                        format!(
                            "{}: {}",
                            e["path"].as_str().unwrap(),
                            e["message"].as_str().unwrap()
                        )
                    })
                    .collect(),
                _ => scan_error_messages(value),
            })
            .collect(),
        Value::Array(items) => items.iter().flat_map(scan_error_messages).collect(),
        _ => Vec::new(),
    }
}

/// `scan` publishes the reason with io's Fix appended, the bytes `list_ids` and
/// `check` publish and the CLI prints after `error: <path>: `.
#[test]
fn scan_publishes_the_message_every_other_operation_publishes() {
    let root = test_root("scan_publishes_the_message_every_other_operation_publishes");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs\"]\n",
    );
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nLead.\n");
    symlink("missing.md", &root.join("docs/FS-b.md"));
    let expected = vec![format!(
        "docs/FS-b.md: {}",
        entry_message("broken symlink: the target does not exist")
    )];
    for operation in ["scan", "list_ids"] {
        assert_eq!(
            scan_error_messages(&call(&root, operation)),
            expected,
            "{operation}"
        );
    }
    let check = call(&root, "check");
    let io = check.to_string();
    assert!(
        io.contains(&json!(entry_message("broken symlink: the target does not exist")).to_string()),
        "{check}"
    );
}

/// Node's strict `scan` failure and an operation's scan-failure record both carry the record's own
/// code rather than a hard-coded `io` (§FS-check.2.4.3, §AR-bindings.2).
#[test]
fn strict_and_listed_scan_failures_carry_the_records_code() {
    let root = test_root("strict_and_listed_scan_failures_carry_the_records_code");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n",
    );
    write(&root.join("values/a.json"), "{ not json\n");
    let strict = node_embedding_call(node_request(
        "scan",
        vec![json!(root.to_string_lossy())],
        &root,
    ));
    assert_eq!(
        strict["failure"]["kind"],
        json!("unparsable-file"),
        "{strict}"
    );
    let listed = call(&root, "scan");
    assert_eq!(
        listed["result"]["scan_errors"][0]["code"],
        json!("unparsable-file"),
        "{listed}"
    );
    let options = json!({"root": root.to_string_lossy()});
    for (operation, id) in [
        ("show", json!("CONST-a")),
        ("showBatch", json!(["CONST-a"])),
    ] {
        let args = vec![id, options.clone()];
        let failed = node_embedding_call(node_request(operation, args, &root));
        assert_eq!(
            failed["failure"]["kind"],
            json!("unparsable-file"),
            "{operation}: {failed}"
        );
        assert_eq!(failed["failure"]["code"], json!("filesystem"), "{failed}");
    }
}

/// Node's strict `scan` rejects with the record an operation lists for the same failure: its path
/// and its message, the reason followed by the fix (§FS-distribution.3.2.2.3, §FS-check.2.4.3).
#[test]
#[ignore = "red until the strict scan says its fix; the fix removes this"]
fn the_strict_scan_failure_carries_the_fix_its_record_carries() {
    let root = test_root("the_strict_scan_failure_carries_the_fix_its_record_carries");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n",
    );
    write(&root.join("values/a.json"), "{ not json\n");
    let strict = node_embedding_call(node_request(
        "scan",
        vec![json!(root.to_string_lossy())],
        &root,
    ));
    let listed = call(&root, "scan");
    let record = &listed["result"]["scan_errors"][0];
    assert_eq!(strict["failure"]["message"], record["message"], "{strict}");
    assert_eq!(strict["failure"]["path"], record["path"], "{strict}");
}
