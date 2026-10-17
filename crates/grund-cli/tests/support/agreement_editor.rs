//! Actual editor snapshots/hovers at the fixture cwd (§FS-lsp.2.2.1,
//! §REQ-one-answer.proof.matrix, §REQ-one-answer.text.targets).

use super::agreement_editor_worker::Worker;
use super::agreement_evidence as evidence;
use super::agreement_oracles::{physical, text, witnesses};
use super::agreement_paths::{logical_text, logical_value, saved_path};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn cwd(context: &Value, held: bool) -> PathBuf {
    if held {
        PathBuf::from(text(context, "base")).join(
            Path::new(context["cwd"].as_str().unwrap())
                .strip_prefix(text(context, "saved"))
                .unwrap(),
        )
    } else {
        PathBuf::from(text(context, "cwd"))
    }
}

pub fn snapshot(worker: &mut Worker, context: &Value, errors: &mut Vec<String>) -> Option<Value> {
    let path = PathBuf::from(text(context, "editorPath"));
    let response = worker.call(json!({"operation":"snapshot","cwd":cwd(context,true),
        "path":path,"explicit":context["explicit"],"overlays":context["overlays"]}));
    evidence::receipt(
        "editor",
        &path,
        json!({"path":path,"cwd":cwd(context,true),
        "explicit":context["explicit"],"overlays":context["overlays"],"question":"snapshot",
        "scope":context["scopes"]["editor"]}),
        response.clone(),
    );
    evidence::seconds(worker.last_seconds);
    if context.get("truth").is_some() {
        super::agreement_truth::editor(context, "editor:snapshot", &response, errors);
    }
    if response.get("failure").is_some() {
        if context.get("truth").is_none() {
            evidence::witness(json!({"point":"§FS-lsp.2.2.1","complete":false,
                "oracle":"fixture-config-bytes","configs":context["sourceInventory"]}));
        }
        return None;
    }
    assert_eq!(
        PathBuf::from(response["snapshot"]["root"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        PathBuf::from(context["scopes"]["editor"]["projectRoot"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        "editor loaded a project outside the independently discovered context"
    );
    if context.get("truth").is_none() {
        let rows: Vec<_> = ["declarations", "sections"]
            .into_iter()
            .flat_map(|key| response["snapshot"][key].as_array().unwrap())
            .map(|site| {
                json!({"path":saved_path(context,Path::new(site["path"].as_str().unwrap())),
            "line":site["line"]})
            })
            .collect();
        let mut scoped = context.clone();
        scoped["reportBase"] = context["scopes"]["cli"]["reportBase"].clone();
        match witnesses(&scoped, &rows, "§REQ-one-answer.identity.declarations") {
            Ok(witness) => evidence::witness(witness),
            Err(error) => errors.push(error),
        }
    }
    Some(response)
}

pub fn show(
    worker: &mut Worker,
    context: &Value,
    coordinate: &str,
    cli: &[Value],
    cli_refused: bool,
    errors: &mut Vec<String>,
) {
    let response =
        worker.call(json!({"operation":"show","cwd":cwd(context,true),"coordinate":coordinate}));
    evidence::receipt(
        "editor",
        Path::new(&text(context, "editorPath")),
        json!({"coordinate":coordinate,"question":"show","overlays":context["overlays"],
            "scope":context["scopes"]["editor"]}),
        response.clone(),
    );
    evidence::seconds(worker.last_seconds);
    if context.get("truth").is_some() {
        super::agreement_truth::editor(
            context,
            &format!("editor:show:{coordinate}"),
            &response,
            errors,
        );
    }
    if let Some(serialized) = response["json"].as_str() {
        let actual: Value = serde_json::from_str(serialized).unwrap();
        let mut editor_scope = context.clone();
        editor_scope["reportBase"] = context["scopes"]["editor"]["reportBase"].clone();
        if context.get("truth").is_none() {
            let mut saved_actual = actual.clone();
            if let Some(path) = actual["path"].as_str() {
                saved_actual["path"] = json!(saved_path(context, &physical(&editor_scope, path)));
            }
            match witnesses(context, &[saved_actual], "§REQ-one-answer.text.parity") {
                Ok(witness) => evidence::witness(witness),
                Err(error) => errors.push(error),
            }
        }
        // §REQ-one-answer.proof.evidence.closed: R01 closed with
        // §DF-path-narrows-the-report-in-0-19, so an editor answer the command
        // line refuses is a divergence wherever its home lies.
        if logical_value(json!(cli), context) != logical_value(json!([actual.clone()]), context) {
            errors.push(format!(
                "editor hover {coordinate}: {actual} against {cli:?}"
            ));
        }
    } else if !cli_refused || !cli.is_empty() {
        errors.push(format!("editor refuses {coordinate}, CLI answers {cli:?}"));
    } else if context.get("truth").is_none() {
        evidence::witness(
            json!({"oracle":"authored-coordinate-search","complete":false,
            "coordinate":coordinate,"point":"§REQ-one-answer.sections.map"}),
        );
    }
}

pub fn saved_control(
    worker: &mut Worker,
    context: &Value,
    loaded: &Value,
    errors: &mut Vec<String>,
) {
    if context["variant"] != "editor-overlay" {
        return;
    }
    let path = PathBuf::from(text(context, "path"));
    let saved = worker.call(json!({"operation":"saved","cwd":cwd(context,false),
        "path":path,"explicit":context["explicit"],"overlays":{}}));
    evidence::receipt(
        "editor-control",
        &path,
        json!({"path":path,"overlays":{},"heldBytes":context["heldBytes"],
            "relative":context["overlayRelative"]}),
        saved.clone(),
    );
    evidence::seconds(worker.last_seconds);
    if logical_value(loaded.clone(), context) != logical_value(saved.clone(), context) {
        errors.push(format!(
            "held/saved editor snapshot: {loaded} against {saved}"
        ));
    }
    evidence::witness(json!({"oracle":"saved-held-byte-equality",
        "heldBytes":context["heldBytes"],"point":"§REQ-one-answer.contract"}));
}

/// Check bounds reports while editor navigation retains its wider catalog.
pub fn check(context: &Value, loaded: &Value, host: &Value, errors: &mut Vec<String>) {
    if !host["failure"].is_null() {
        return;
    }
    let snapshot = &loaded["snapshot"];
    let mut editor_scope = context.clone();
    editor_scope["reportBase"] = context["scopes"]["editor"]["reportBase"].clone();
    // §FS-lsp.1.1: the editor names a site by its file's filesystem identity and
    // `check` by the spelling the walk kept (§FS-scan.identity.read-once), so both
    // are compared as the file they name.
    let render_path = |path: &str, scope: &Value| {
        let path = physical(scope, path);
        saved_path(context, &path.canonicalize().unwrap_or(path))
            .to_string_lossy()
            .replace('\\', "/")
    };
    let actual_warnings: BTreeSet<_> = snapshot["run_warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| logical_text(f["message"].as_str().unwrap(), context))
        .collect();
    let expected_warnings: BTreeSet<_> = host["run_cautions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|f| logical_text(f["message"].as_str().unwrap(), context))
        .collect();
    if actual_warnings != expected_warnings {
        errors.push(format!(
            "editor/check warning channel: {actual_warnings:?} against {expected_warnings:?}"
        ));
    }
    // §FS-lsp.1.1: a diagnostic is a located finding; a site-less CLI-level config
    // warning stays on `check` alone (§FS-check.checks.config-version-1).
    let facts = |report: &Value, scope: &Value| -> BTreeSet<_> {
        ["errors", "warnings"]
            .into_iter()
            .flat_map(|key| report[key].as_array().into_iter().flatten())
            .filter(|f| !f["path"].is_null())
            .map(|f| {
                (
                    f["severity"].as_str().unwrap().to_string(),
                    f["code"].as_str().unwrap().to_string(),
                    f["path"].as_str().map(|p| render_path(p, scope)),
                    f["line"].as_u64(),
                    logical_text(f["message"].as_str().unwrap(), context),
                )
            })
            .collect()
    };
    let actual = facts(&snapshot["report"], &editor_scope);
    let expected = facts(&host["result"]["selected_report"], context);
    // §REQ-one-answer.proof.evidence.closed: R02 closed with
    // §DF-path-narrows-the-report-in-0-19, so an alias the editor resolves and
    // `check` refuses is a divergence on every variant.
    if actual != expected {
        errors.push(format!(
            "editor/check findings: {actual:?} against {expected:?}"
        ));
    }
}
