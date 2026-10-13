//! Actual editor snapshots and hovers, held text against isolated saved bytes
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.text.targets).

use super::agreement_evidence;
use super::agreement_oracles::{text, witnesses};
use grund_core::{
    LspSnapshotOpts, LspSnapshotWithCompletion, ShowFormat, ShowMode, lsp_snapshot_with_completion,
    show_from_snapshot,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn snapshot(context: &Value, errors: &mut Vec<String>) -> Option<LspSnapshotWithCompletion> {
    let path = PathBuf::from(text(context, "editorPath"));
    let open_documents: BTreeMap<PathBuf, String> = context["overlays"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(path, text)| (PathBuf::from(path), text.as_str().unwrap().into()))
        .collect();
    let opts = LspSnapshotOpts {
        path: path.clone(),
        path_provided: context["explicit"].as_bool().unwrap(),
        open_documents,
    };
    let result = lsp_snapshot_with_completion(opts);
    let response = match &result {
        Ok(loaded) => json!({"snapshot":format!("{:#?}",loaded.metadata),
            "readFiles":loaded.show.read_files()}),
        Err(error) => json!({"failure":format!("{error:#}")}),
    };
    agreement_evidence::receipt(
        "editor",
        &path,
        json!({"path":path,
        "explicit":context["explicit"],"overlays":context["overlays"],"question":"snapshot"}),
        response,
    );
    if let Ok(loaded) = &result {
        let rows: Vec<Value> = loaded
            .metadata
            .snapshot
            .declarations
            .iter()
            .chain(loaded.metadata.snapshot.sections.iter())
            .map(|site| json!({"path":site.display_path,"line":site.line}))
            .collect();
        match witnesses(context, &rows, "§REQ-one-answer.identity.declarations") {
            Ok(witness) => agreement_evidence::witness(witness),
            Err(error) => errors.push(error),
        }
    } else {
        agreement_evidence::witness(json!({"point":"§FS-lsp.2.2.1",
            "oracle":"fixture-config-bytes","configs":context["sourceInventory"]}));
    }
    result.ok()
}

pub fn show(
    context: &Value,
    loaded: &LspSnapshotWithCompletion,
    coordinate: &str,
    cli: &[Value],
    cli_refused: bool,
    errors: &mut Vec<String>,
) {
    let result = show_from_snapshot(&loaded.show, coordinate, ShowMode::Toc, ShowFormat::Json);
    let response = match &result {
        Ok(out) => json!({"body":out.body,"path":out.path,"line":out.line,
            "json":out.json,"sections":out.sections.iter().map(|s|json!({"path":s.path,"title":s.title,"depth":s.depth})).collect::<Vec<_>>()}),
        Err(error) => json!({"failure":format!("{error:#}")}),
    };
    agreement_evidence::receipt(
        "editor",
        Path::new(&text(context, "editorPath")),
        json!({"coordinate":coordinate,"question":"show","overlays":context["overlays"]}),
        response,
    );
    if let Ok(out) = result {
        let actual: Value = serde_json::from_str(out.json.as_deref().expect("requested JSON"))
            .expect("snapshot hover JSON");
        match witnesses(context, &[actual.clone()], "§REQ-one-answer.text.parity") {
            Ok(witness) => agreement_evidence::witness(witness),
            Err(error) => errors.push(error),
        }
        // Paths belong to two independent copies; compare their logical site.
        if cli != [actual.clone()] {
            let selected = PathBuf::from(text(context, "path"));
            let outside_selection = actual["path"].as_str().is_some_and(|path| {
                !super::agreement_oracles::physical(context, path).starts_with(&selected)
            });
            if cli_refused
                && cli.is_empty()
                && context["variant"] != "whole-tree"
                && context["variant"] != "editor-overlay"
                && outside_selection
            {
                // R01 keeps the path-scoped command's missing outside home until
                // #621; do not correct that shipped answer in the proof harness.
                // §REQ-one-answer.differences.scope, §DF-one-answer-rulings-run.
                agreement_evidence::receipt(
                    "known-difference",
                    &selected,
                    json!({"coordinate":coordinate,"outsideHome":actual["path"]}),
                    json!({"row":"R01","editor":actual,"cli":cli}),
                );
            } else {
                errors.push(format!(
                    "editor hover {coordinate}: {actual} against {cli:?}"
                ));
            }
        }
    } else if !cli_refused || !cli.is_empty() {
        errors.push(format!("editor refuses {coordinate}, CLI answers {cli:?}"));
    } else {
        agreement_evidence::witness(json!({"oracle":"authored-coordinate-search",
            "coordinate":coordinate,"point":"§REQ-one-answer.sections.map"}));
    }
}

/// Snapshot invariance validates the actual held edit's semantic control, while
/// physical-source/golden expectations remain independent truth.
pub fn saved_control(
    context: &Value,
    loaded: &LspSnapshotWithCompletion,
    errors: &mut Vec<String>,
) {
    if context["variant"] != "editor-overlay" {
        return;
    }
    let path = PathBuf::from(text(context, "path"));
    let saved = lsp_snapshot_with_completion(LspSnapshotOpts {
        path: path.clone(),
        path_provided: true,
        ..Default::default()
    });
    agreement_evidence::receipt(
        "editor-control",
        &path,
        json!({"path":path,"overlays":{},
        "heldBytes":context["heldBytes"],"relative":context["overlayRelative"]}),
        match &saved {
            Ok(s) => json!({"snapshot":format!("{:#?}",s.metadata)}),
            Err(error) => json!({"failure":format!("{error:#}")}),
        },
    );
    let base = text(context, "base");
    let saved_root = text(context, "saved");
    let held = format!("{:#?}", loaded.metadata).replace(&base, &saved_root);
    let saved = saved.map(|s| format!("{:#?}", s.metadata));
    if saved.as_ref().ok() != Some(&held) {
        errors.push(format!(
            "held/saved editor snapshot: {held} against {saved:?}"
        ));
    }
    agreement_evidence::witness(json!({"oracle":"saved-held-byte-equality",
        "heldBytes":context["heldBytes"],"point":"§REQ-one-answer.contract"}));
}

/// Check bounds reports, while snapshot navigation retains its wider catalog
/// (§FS-scan.path.check). Compare diagnostics on the selected report domain.
pub fn check(
    context: &Value,
    loaded: &LspSnapshotWithCompletion,
    host: &Value,
    errors: &mut Vec<String>,
) {
    if !host["failure"].is_null() {
        return;
    }
    let snapshot = &loaded.metadata.snapshot;
    let base = text(context, "base");
    let saved = text(context, "saved");
    let render_path = |path: &str| {
        let path = path.replace(&base, &saved);
        let path = super::agreement_oracles::physical(context, &path);
        path.to_string_lossy().replace('\\', "/")
    };
    let actual_warnings: std::collections::BTreeSet<_> = snapshot
        .run_warnings
        .iter()
        .map(|finding| finding.message.replace(&base, &saved))
        .collect();
    let expected_warnings: std::collections::BTreeSet<_> = host["run_cautions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|finding| finding["message"].as_str().unwrap().to_string())
        .collect();
    if actual_warnings != expected_warnings {
        errors.push(format!(
            "editor/check warning channel: {actual_warnings:?} against {expected_warnings:?}"
        ));
    }
    let actual: std::collections::BTreeSet<_> = snapshot
        .report
        .errors
        .iter()
        .chain(&snapshot.report.warnings)
        .map(|finding| {
            (
                finding.severity.to_string(),
                finding.code.to_string(),
                finding.path.as_deref().map(&render_path),
                finding.line.map(|line| line as u64),
                finding.message.replace(&base, &saved),
            )
        })
        .collect();
    let expected: std::collections::BTreeSet<_> = ["errors", "warnings"]
        .into_iter()
        .flat_map(|key| {
            host["result"]["selected_report"][key]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .map(|finding| {
            (
                finding["severity"].as_str().unwrap().to_string(),
                finding["code"].as_str().unwrap().to_string(),
                finding["path"].as_str().map(&render_path),
                finding["line"].as_u64(),
                finding["message"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    if actual != expected {
        let only_cli: Vec<_> = expected.difference(&actual).collect();
        let only_editor: Vec<_> = actual.difference(&expected).collect();
        let known_alias_difference = !only_cli.is_empty()
            && only_editor.is_empty()
            && context["variant"] != "whole-tree"
            && context["variant"] != "editor-overlay"
            && snapshot.workspace
            && only_cli.iter().all(|finding| {
                finding.1 == "unknown-project"
                    && snapshot.citations.iter().any(|citation| {
                        let same_site = Some(render_path(citation.path.to_string_lossy().as_ref()))
                            == finding.2
                            && Some(citation.line as u64) == finding.3;
                        same_site
                            && citation.target_path.is_some()
                            && citation
                                .query_id
                                .split_once('/')
                                .is_some_and(|(alias, _)| finding.4.contains(&format!("`{alias}`")))
                    })
            });
        if known_alias_difference {
            // R02 holds precisely a narrowed alias refusal the wider editor can
            // resolve, not any other report difference (§REQ-one-answer.differences.scope).
            agreement_evidence::receipt(
                "known-difference",
                Path::new(&text(context, "path")),
                json!({"resolvedCitationSites":format!("{:#?}",snapshot.citations)}),
                json!({"row":"R02","onlyCli":only_cli}),
            );
        } else {
            errors.push(format!(
                "editor/check findings: {actual:?} against {expected:?}"
            ));
        }
    }
}
