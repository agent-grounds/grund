//! Invocation controls use real copies and portable path spellings, without
//! obtaining expected answers from an engine (§REQ-one-answer.proof.matrix,
//! §FS-scan.text.buffer, §FS-scan.identity.buffers).

use super::{agreement_context as context, agreement_truth as truth};
use serde_json::{Value, json};
use std::process::Command;

#[test]
fn repeated_physical_overlay_materializations_have_the_same_authored_identity() {
    // Only the fixture materializer runs: no CLI, native package or editor load.
    let output = Command::new(std::env::var_os("PYTHON").unwrap_or_else(|| "python".into()))
        .current_dir(super::agreement_host::repository().join("tests/bindings"))
        .args([
            "-c",
            r#"
import json
from pathlib import Path
from agreement_fixtures import materialize
from agreement_scope import isolated_fixtures
with isolated_fixtures() as first, isolated_fixtures() as second:
    print(json.dumps([materialize(Path(first), 'list-basic', 'editor-overlay'),
                      materialize(Path(second), 'list-basic', 'editor-overlay')]))
"#,
        ])
        .output()
        .expect("two independent physical materializations");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let contexts: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_ne!(contexts[0]["base"], contexts[1]["base"]);
    let expected = context::identity(&contexts[0]).unwrap();
    assert_eq!(expected, context::identity(&contexts[1]).unwrap());
    assert_eq!(expected["path"], "<saved>");
    assert_eq!(expected["editorPath"], "<held>");
    assert_eq!(
        expected["scopes"]["editor"]["configFile"],
        "<held>/grund.toml"
    );
    assert_eq!(
        expected["scopes"]["cli"]["configFile"],
        "<saved>/grund.toml"
    );
    let file = "<held>/docs/architecture/AR-001-auth.md";
    assert_eq!(
        expected["overlays"][file],
        format!(
            "# AR-001-auth: The auth module\n\nImplements {}FS-001-login.\n\n",
            '§'
        )
    );
    let facts = json!({"absent":null,"context":expected});
    let mut second = contexts[1].clone();
    truth::bind(&mut second, &facts);
    assert_eq!(second["truth"], facts);
    for field in [
        "cwd",
        "path",
        "editorPath",
        "relative",
        "overlayRelative",
        "heldBytes",
    ] {
        let mut wrong = contexts[1].clone();
        wrong[field] = json!("changed input");
        rejected(wrong, &facts);
    }
    for field in ["explicit", "bufferOnly"] {
        let mut wrong = contexts[1].clone();
        wrong[field] = json!(!wrong[field].as_bool().unwrap());
        rejected(wrong, &facts);
    }
    for field in [
        "projectRoot",
        "reportBase",
        "invocationBase",
        "configFile",
        "parseError",
    ] {
        let mut wrong = contexts[1].clone();
        wrong["scopes"]["editor"][field] = json!("changed scope");
        rejected(wrong, &facts);
    }
    let mut wrong = contexts[1].clone();
    let overlays = wrong["overlays"].as_object_mut().unwrap();
    let key = overlays.keys().next().unwrap().clone();
    let text = overlays.remove(&key).unwrap();
    overlays.insert(key.replace("AR-001-auth.md", "wrong.md"), text);
    rejected(wrong, &facts);
    let mut wrong = contexts[1].clone();
    if let Some(value) = wrong["overlays"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
    {
        *value = json!("wrong text");
    }
    rejected(wrong, &facts);
}

fn rejected(mut actual: Value, facts: &Value) {
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            truth::bind(&mut actual, facts);
        }))
        .is_err(),
        "changed invocation escaped authored context"
    );
}

#[test]
fn invocation_paths_preserve_roles_text_map_keys_ranges_and_wrong_files() {
    let value = json!({"base":r"C:\scratch\base\repo", "saved":r"C:\scratch\saved\repo",
        "absent":null,"relative":r"docs\a.md", "path":r"C:\scratch\saved\repo\docs\a.md",
        "cwd":r"C:\scratch\saved\repo", "editorPath":r"C:\scratch\base\repo\docs\a.md",
        "explicit":true,"overlayRelative":r"docs\a.md", "bufferOnly":false,
        "heldBytes":r"literal C:\scratch\base\repo\docs\a.md",
        "overlays":{r"C:\scratch\base\repo\docs\a.md":r"literal C:\scratch\base\repo\docs\a.md"},
        "scopes":{"editor":{"configFile":r"C:\scratch\base\repo\grund.toml",
            "sectionMap":{r"1.path\name":{"range":[1,3]}}}}});
    let expected = context::identity(&value).unwrap();
    assert_eq!(expected["relative"], "docs/a.md");
    assert_eq!(expected["editorPath"], "<held>/docs/a.md");
    assert_eq!(expected["path"], "<saved>/docs/a.md");
    assert_eq!(expected["heldBytes"], value["heldBytes"]);
    assert_eq!(expected["overlays"]["<held>/docs/a.md"], value["heldBytes"]);
    assert_eq!(
        expected["scopes"]["editor"]["sectionMap"],
        value["scopes"]["editor"]["sectionMap"]
    );
    let facts = json!({"absent":null,"context":expected});
    let mut native = value.clone();
    native["path"] = json!(r"\\?\C:\scratch\saved\repo\docs\a.md");
    native["editorPath"] = json!("C:/scratch/base/repo/docs/a.md");
    native["overlays"] = json!({r"\\?\C:\scratch\base\repo\docs\a.md":value["heldBytes"]});
    truth::bind(&mut native, &facts);
    assert_eq!(
        context::path(r"C:\scratch\base\repo-other\a.md", &value),
        "C:/scratch/base/repo-other/a.md"
    );
    let unc = json!({"base":r"\\server\share\base\repo", "saved":r"\\server\share\saved\repo"});
    assert_eq!(
        context::path(r"\\?\UNC\server\share\base\repo\docs\a.md", &unc),
        "<held>/docs/a.md"
    );
    let unix = json!({"base":r"/scratch/a\b/base/repo", "saved":r"/scratch/a\b/saved/repo"});
    assert_eq!(
        context::path(r"/scratch/a\b/base/repo/docs/a\b.md", &unix),
        r"<held>/docs/a\b.md"
    );
    let mut wrong = value.clone();
    wrong["path"] = value["editorPath"].clone();
    rejected(wrong, &facts);
    let mut wrong = value.clone();
    wrong["scopes"]["editor"]["sectionMap"][r"1.path\name"]["range"] = json!([1, 4]);
    rejected(wrong, &facts);
    let mut duplicate = value;
    duplicate["overlays"][r"\\?\C:\scratch\base\repo\docs\a.md"] = json!("conflicting text");
    assert!(context::identity(&duplicate).is_err());
}
