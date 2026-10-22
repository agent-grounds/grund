//! An E2E case's `args` is its parsed `command.args` on every structured surface,
//! `[]` when it has none, and only the text form's first line says `grund check`
//! (§FS-show.2.4, §FS-show.2.4.2, §FS-output-shapes.4.2, §FS-distribution.3.2.2.3).

use super::*;
use crate::queries::{ShowFormat, ShowMode, ShowOpts};
use crate::testing::{test_root, write};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Three cases: `login` has no `command.args`, `with-args` names `list`, and
/// `bare` has a `command.args` that holds no words.
fn cases(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
         [[kinds]]\nkind = \"E2E\"\nfolder = \"cases\"\nindex = false\n",
    );
    for case in ["login", "with-args", "bare"] {
        write(&root.join(format!("cases/{case}/expected.exit")), "0\n");
    }
    write(&root.join("cases/with-args/command.args"), "list\n");
    write(&root.join("cases/bare/command.args"), "\n");
    root
}

fn shown(root: &Path, id: &str, format: ShowFormat) -> crate::model::ShowOutput {
    show(
        id,
        ShowOpts {
            path: root.to_path_buf(),
            section: None,
            mode: ShowMode::Full,
            format,
        },
    )
    .unwrap()
}

fn json_args(root: &Path, id: &str) -> Value {
    let json = shown(root, id, ShowFormat::Json).json.unwrap();
    serde_json::from_str::<Value>(&json).unwrap()["args"].clone()
}

#[test]
#[ignore = "red until #947 keeps the manifest in the E2E record; implement removes this"]
fn show_json_args_are_empty_for_a_case_without_command_args() {
    let root = cases("show_json_args_are_empty_without_command_args");
    assert_eq!(json_args(&root, "E2E-with-args"), json!(["list"]));
    assert_eq!(json_args(&root, "E2E-bare"), json!([]));
    assert_eq!(json_args(&root, "E2E-login"), json!([]));
}

#[test]
#[ignore = "red until #947 keeps the manifest in the E2E record; implement removes this"]
fn the_text_form_alone_says_grund_check_for_a_case_without_command_args() {
    let root = cases("the_text_form_alone_says_grund_check");
    let first = |id: &str| {
        let body = shown(&root, id, ShowFormat::Text).body;
        body.lines().next().unwrap().to_string()
    };
    assert_eq!(first("E2E-login"), "grund check");
    assert_eq!(first("E2E-with-args"), "grund list");
    assert_eq!(first("E2E-bare"), "grund");
}

#[test]
#[ignore = "red until #947 keeps the manifest in the E2E record; implement removes this"]
fn the_node_show_manifest_args_are_empty_for_a_case_without_command_args() {
    let root = cases("node_show_manifest_args_are_empty");
    let args = |id: &str| {
        let request = node_request(
            "show",
            vec![json!(id), json!({"root": root.to_string_lossy()})],
            &root,
        );
        let data = node_embedding_call(request);
        assert_eq!(data["failure"], Value::Null, "{data}");
        data["result"]["manifest"]["args"].clone()
    };
    assert_eq!(args("E2E-with-args"), json!(["list"]));
    assert_eq!(args("E2E-login"), json!([]));
}

#[test]
#[ignore = "red until #947 keeps the manifest in the E2E record; implement removes this"]
fn the_scan_snapshot_e2e_case_args_are_empty_for_a_case_without_command_args() {
    let root = cases("scan_snapshot_e2e_case_args_are_empty");
    let request = node_request("scan", vec![json!(root.to_string_lossy())], &root);
    let data = node_embedding_call(request);
    assert_eq!(data["failure"], Value::Null, "{data}");
    let args = |id: &str| {
        data["result"]["snapshot"]["declarations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == id)
            .unwrap_or_else(|| panic!("{id} in {data}"))["e2e_case"]["args"]
            .clone()
    };
    assert_eq!(args("E2E-with-args"), json!(["list"]));
    assert_eq!(args("E2E-login"), json!([]));
}
