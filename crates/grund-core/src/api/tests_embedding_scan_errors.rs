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
        crate::scanner::scan_error_message("broken symlink: the target does not exist")
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
        io.contains(
            &json!(crate::scanner::scan_error_message(
                "broken symlink: the target does not exist"
            ))
            .to_string()
        ),
        "{check}"
    );
}

/// The library `scan` refuses an incomplete walk with the line `check` prints,
/// `io`'s Fix included, because a strict scan is one more place the line is
/// printed (§FS-check.2.4, §FS-workspace.8.7.3).
#[test]
#[ignore = "red until #1189 has scan_tree_strict render through scan_error_message; implement removes this"]
fn the_library_scan_refuses_with_the_fix_check_prints() {
    let root = test_root("the_library_scan_refuses_with_the_fix_check_prints");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs\"]\n",
    );
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nLead.\n");
    symlink("missing.md", &root.join("docs/FS-b.md"));
    let error = match scan(&root) {
        Ok(_) => panic!("a broken link left the library scan complete"),
        Err(error) => error.to_string(),
    };
    let fix = crate::scanner::scan_error_message("broken symlink: the target does not exist");
    assert!(error.ends_with(&format!(": {fix}")), "{error}");
}
