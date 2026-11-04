//! Test module: what a binding publishes for a path that names the report base reads `.`
//! (§FS-cli.3.4), a scan failure's `path` and a declaration record's alike (§AR-bindings.2).

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::{Value, json};

use super::*;
use crate::testing::{test_root, write};

fn binding(root: &Path, operation: &str, args: Vec<Value>) -> Value {
    embedding_call(EmbeddingRequest {
        operation: operation.into(),
        root: root.to_path_buf(),
        explicit: true,
        args,
        options: json!({}),
    })
}

fn node(root: &Path, operation: &str, args: Vec<Value>) -> Value {
    node_embedding_call(node_request(operation, args, root))
}

/// Puts a directory's mode back before the temp dir is cleaned up.
struct ModeGuard<'a>(&'a Path);

impl Drop for ModeGuard<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
    }
}

/// A project whose config root this user may not list, or `None` where mode 311 does not bar the
/// user (root), which would prove nothing, so the test passes without asserting.
fn unlistable(root: &Path) -> Option<ModeGuard<'_>> {
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs\"]\n",
    );
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nLead.\n");
    fs::set_permissions(root, fs::Permissions::from_mode(0o311)).expect("set the mode");
    let guard = ModeGuard(root);
    fs::read_dir(root).is_err().then_some(guard)
}

#[test]
fn show_batch_names_an_unlistable_config_root_dot() {
    let root = test_root("show_batch_names_an_unlistable_config_root_dot");
    let Some(_guard) = unlistable(&root) else {
        return;
    };
    let reply = binding(&root, "show_batch", Vec::new());
    assert_eq!(reply["failure"]["code"], "io", "{reply}");
    assert_eq!(reply["failure"]["path"], ".", "{reply}");
}

#[test]
fn node_scan_names_an_unlistable_config_root_dot() {
    let root = test_root("node_scan_names_an_unlistable_config_root_dot");
    let Some(_guard) = unlistable(&root) else {
        return;
    };
    let reply = node(&root, "scan", vec![json!(root.to_str().unwrap())]);
    assert_eq!(reply["failure"]["kind"], "io", "{reply}");
    assert_eq!(reply["failure"]["path"], ".", "{reply}");
}

/// Every `path` on a record naming `E2E-login`, wherever in the reply it sits: a `show` result
/// names none, being the answer about the one ID asked for, so its `body` marks it instead.
fn record_paths(data: &Value) -> Vec<String> {
    match data {
        Value::Object(map) => {
            let own = (map.get("id") == Some(&json!("E2E-login")) || map.contains_key("body"))
                .then(|| map.get("path").and_then(Value::as_str).map(str::to_owned))
                .flatten();
            own.into_iter()
                .chain(map.values().flat_map(record_paths))
                .collect()
        }
        Value::Array(items) => items.iter().flat_map(record_paths).collect(),
        _ => Vec::new(),
    }
}

/// `relative_paths = false` makes the root a binding is called on the report base, so an E2E case
/// called on its own directory is a record at the base.
#[test]
fn a_case_at_the_base_reads_dot_from_every_binding() {
    let root = test_root("a_case_at_the_base_reads_dot_from_every_binding");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[output]\nrelative_paths = false\n[id]\n\
         format = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/cases\"\n",
    );
    let case = root.join("e2e/cases/login");
    write(&case.join("command.args"), "check repo\n");
    write(&case.join("expected.exit"), "0\n");
    let id = || vec![json!("E2E-login")];
    let batch = || vec![json!(["E2E-login"])];
    let replies = [
        ("list_ids", binding(&case, "list_ids", Vec::new())),
        ("list_sizes", binding(&case, "list_sizes", Vec::new())),
        ("scan", binding(&case, "scan", Vec::new())),
        ("show", binding(&case, "show", id())),
        ("show_batch", binding(&case, "show_batch", batch())),
        (
            "node scan",
            node(&case, "scan", vec![json!(case.to_str().unwrap())]),
        ),
        ("node show", node(&case, "show", id())),
        ("node showBatch", node(&case, "showBatch", batch())),
    ];
    for (operation, reply) in replies {
        let paths = record_paths(&reply);
        assert!(!paths.is_empty(), "{operation}: {reply}");
        assert!(paths.iter().all(|path| path == "."), "{operation}: {reply}");
    }
}
