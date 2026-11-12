//! Test module: one scan failure has one rendering, its location, message and code, whichever
//! binding operation publishes it, and the message names a file it could not parse from the
//! report base rather than by an absolute path (§FS-errors.2.2.5, §FS-workspace.8.7.3,
//! §FS-errors.4, §AR-scanner.1.failure).

use serde_json::{Value, json};

use super::*;
use crate::testing::{test_root, write};

const E2E_CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
                          slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.E2E]\n\
                          folders = [\"e2e/cases\"]\nindex = false\n";

/// A project whose one E2E case, `a`, holds an `expected.exit` that is not an integer.
fn unparsable_exit(name: &str) -> std::path::PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), E2E_CONFIG);
    write(&root.join("e2e/cases/a/expected.exit"), "not-an-exit\n");
    root
}

/// The record the listing `scan` operation publishes for the tree's one failure.
fn listed_record(root: &std::path::Path) -> Value {
    let listed = embedding_call(EmbeddingRequest {
        operation: "scan".into(),
        root: root.to_path_buf(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    });
    listed["result"]["scan_errors"][0].clone()
}

/// The record names the file it could not parse as the report spells it, never absolute.
#[test]
#[ignore = "red until #1207 renders a scan failure's parsed path at report time; implement removes this"]
fn an_unparsable_expected_exit_is_named_from_the_report_base() {
    let root = unparsable_exit("an_unparsable_expected_exit_is_named_from_the_report_base");
    let record = listed_record(&root);
    assert_eq!(
        record["path"],
        json!("e2e/cases/a/expected.exit"),
        "{record}"
    );
    assert_eq!(
        record["message"],
        json!(
            "parse e2e/cases/a/expected.exit: invalid digit found in string; \
             write the case's expected exit status in the file as one integer"
        ),
        "{record}"
    );
}

/// Every Node operation that fails on the scan failure fails with the record's own path, message
/// and kind: `show` does not prefix its message with the path its `path` field already carries.
/// `refs` is not here: it completes, its result marked incomplete.
#[test]
#[ignore = "red until #1207 builds every operation's scan failure from one function; implement removes this"]
fn every_operation_fails_with_the_record_the_scan_lists() {
    let root = unparsable_exit("every_operation_fails_with_the_record_the_scan_lists");
    let record = listed_record(&root);
    let options = json!({"root": root.to_string_lossy()});
    let calls = [
        ("scan", vec![json!(root.to_string_lossy())]),
        ("show", vec![json!("E2E-a"), options.clone()]),
        ("showBatch", vec![json!(["E2E-a"]), options.clone()]),
    ];
    let mut differs = Vec::new();
    for (operation, args) in calls {
        let failed = node_embedding_call(node_request(operation, args, &root));
        let failure = &failed["failure"];
        let got = (&failure["path"], &failure["message"], &failure["kind"]);
        if got != (&record["path"], &record["message"], &record["code"]) {
            differs.push(format!("{operation}: {failure}"));
        }
    }
    assert!(
        differs.is_empty(),
        "record {record}\ndiffers:\n{}",
        differs.join("\n")
    );
}

/// `proposeId`'s failure carries the record's message, fix included (§FS-errors.2.2.5). Its
/// `kind`, `code` and `path` are not held here: they differ from the record's today, and moving
/// a field is a contract this ticket does not make.
#[test]
#[ignore = "red until #1207 builds every operation's scan failure from one function; implement removes this"]
fn propose_id_fails_with_the_records_message_and_fix() {
    let root = unparsable_exit("propose_id_fails_with_the_records_message_and_fix");
    let record = listed_record(&root);
    let options = json!({"root": root.to_string_lossy()});
    let args = vec![json!("E2E"), json!("x"), options];
    let failed = node_embedding_call(node_request("proposeId", args, &root));
    let message = failed["failure"]["message"].as_str().unwrap_or_default();
    let expected = record["message"].as_str().expect("the record's message");
    assert!(
        message.ends_with(expected),
        "{failed}\nexpected to end with {expected}"
    );
}
