//! Negative controls for the reviewed failures (§REQ-one-answer.proof.evidence,
//! §FS-list.3, §FS-show.2.4, §FS-fmt.6.2, §FS-config.3.6.1).

use super::agreement_oracles::{golden, rows, witnesses};
use super::agreement_paths::{logical_text, logical_value, physical};
use super::agreement_tree::{CONFIG_SLUG, Tree};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[test]
fn portable_roots_preserve_wrong_files_and_fields() {
    let context = json!({"base":"C:\\Users\\runner\\case\\base\\repo",
        "saved":"C:\\Users\\runner\\case\\saved\\repo"});
    assert_eq!(
        logical_text("C:/Users/runner/case/base/repo/docs/a.md", &context),
        "<repo>/docs/a.md"
    );
    assert_eq!(
        logical_text(
            "\\\\?\\C:\\Users\\runner\\case\\base\\repo\\docs\\a.md",
            &context
        ),
        "<repo>\\docs\\a.md"
    );
    assert_eq!(
        logical_text("C:\\Users\\runner\\case\\saved\\repo\\docs\\a.md", &context),
        "<repo>\\docs\\a.md"
    );
    assert_ne!(
        logical_text("C:/Users/runner/case/base/repo-other/a.md", &context),
        "<repo>-other/a.md"
    );
    let escaped = json!({"base":"/scratch/a\\b/base/repo","saved":"/scratch/a\\b/saved/repo"});
    let held = json!({"root":"/scratch/a\\b/base/repo","range":[1,3],
        "json":json!({"path":"/scratch/a\\b/base/repo/docs/a.md","anchor":"a"}).to_string()});
    let saved = json!({"root":"/scratch/a\\b/saved/repo","range":[1,3],
        "json":json!({"path":"/scratch/a\\b/saved/repo/docs/a.md","anchor":"a"}).to_string()});
    assert_eq!(
        logical_value(held.clone(), &escaped),
        logical_value(saved.clone(), &escaped)
    );
    let mut corrupted = saved.clone();
    corrupted["range"] = json!([1, 4]);
    assert_ne!(
        logical_value(held.clone(), &escaped),
        logical_value(corrupted, &escaped)
    );
    corrupted = saved;
    corrupted["json"] = json!({"path":"/scratch/a\\b/saved/repo/docs/wrong.md","anchor":"a"})
        .to_string()
        .into();
    assert_ne!(
        logical_value(held, &escaped),
        logical_value(corrupted, &escaped)
    );
}

#[test]
fn nested_and_invocation_relative_sites_use_the_defined_base() {
    let root = super::agreement_host::repository()
        .join("tests/e2e/cases/workspace-member-absorbs-scan-default-include/repo");
    let output = super::agreement_corpus::invoke(
        &root.join("docs/src"),
        &[
            "list".into(),
            "--size".into(),
            "--format".into(),
            "json".into(),
        ],
        None,
    );
    assert!(output.status.success());
    let answer = rows(&output);
    assert!(
        answer
            .iter()
            .any(|r| r["id"] == "FS-doc" && r["path"] == "src/FS-doc.md")
    );
    let context = json!({"saved":root,"reportBase":root.join("docs")});
    assert!(witnesses(&context, &answer, "§FS-config.3.6.1").is_ok());
    let config = format!("{CONFIG_SLUG}[output]\nrelative_paths = false\n");
    let tree = Tree::new(
        "invocation-relative-witness",
        &[
            ("grund.toml", &config),
            ("docs/fs.md", "# FS-alpha: Alpha\n\nAuthored body.\n"),
        ],
    );
    let output = super::agreement_corpus::invoke(
        &tree.0.join("docs"),
        &[
            "list".into(),
            "--size".into(),
            "--format".into(),
            "json".into(),
        ],
        None,
    );
    assert!(output.status.success());
    let answer = rows(&output);
    assert!(answer.iter().any(|r| r["path"] == "fs.md"));
    let context = json!({"saved":tree.0,"reportBase":tree.0.join("docs")});
    assert_eq!(physical(&context, "fs.md"), tree.0.join("docs/fs.md"));
    assert!(witnesses(&context, &answer, "§FS-config.3.6.1").is_ok());
}

#[test]
fn not_found_does_not_agree_with_an_unrelated_configuration_refusal() {
    let root = super::agreement_host::repository().join("tests/e2e/cases/show-json/repo");
    let context = json!({"path":root,"explicit":true});
    let question =
        super::agreement_questions::coordinate_questions(&context, "FS-999-missing").remove(0);
    let output = super::agreement_corpus::invoke(&root, &question.cli_args(), None);
    let good = json!({"failure":{"kind":"query","code":"not-found","message":"ID not found: FS-999-missing",
        "path":null,"line":null,"column":null,"sites":[],"authority":[]},"result":null});
    let mut errors = Vec::new();
    super::agreement_compare::compare_cli(&question, &output, &good, &mut errors);
    assert!(
        errors.is_empty(),
        "matching independently authored refusal: {errors:?}"
    );
    let bad = json!({"failure":{"kind":"config","code":"wrong-refusal","message":"unrelated configuration refusal"},"result":null});
    super::agreement_compare::compare_cli(&question, &output, &bad, &mut errors);
    assert!(!errors.is_empty());
    for field in ["kind", "code", "message", "path"] {
        let mut changed = good.clone();
        changed["failure"][field] = json!("corrupted");
        errors.clear();
        super::agreement_compare::compare_cli(&question, &output, &changed, &mut errors);
        assert!(!errors.is_empty(), "refusal corruption escaped: {field}");
    }
}

#[test]
fn scanned_targets_cannot_use_the_placement_exception() {
    let config = "[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n[scan]\ninclude = [\"docs\"]\n";
    let tree = Tree::new(
        "scanned-placement-mask",
        &[
            ("grund.toml", config),
            ("docs/home.md", "# FS-alpha: [target.rs](target.rs)\n"),
            (
                "docs/target.rs",
                "/// FS-alpha: Alpha\n///\n/// Target body.\npub fn alpha() {}\n",
            ),
        ],
    );
    let loaded = grund_core::lsp_snapshot_with_completion(grund_core::LspSnapshotOpts {
        path: tree.0.clone(),
        path_provided: true,
        ..Default::default()
    })
    .unwrap();
    assert!(
        loaded
            .metadata
            .snapshot
            .scanned_files
            .contains(&tree.0.join("docs/target.rs"))
    );
    let loaded = super::agreement_snapshot::facts(&loaded);
    let context = json!({"saved":tree.0,"base":tree.0,"reportBase":tree.0});
    let mut answers = BTreeMap::new();
    answers.insert("list",json!({"result":{"entries":[{"id":"FS-alpha","project":null,
        "path":"docs/home.md","line":1,"stub":true,"defines":"docs/target.rs","duplicate":false}]}}));
    answers.insert(
        "list-size",
        json!({"result":{"entries":[{"id":"FS-alpha","project":null,
        "path":"docs/target.rs","line":1,"stub":false,"duplicate":false}]}}),
    );
    let mut errors = Vec::new();
    let calls = super::agreement_evidence::capture(|| {
        super::agreement_facts::declarations(&context, &answers, Some(&loaded), &mut errors)
    });
    assert!(!errors.is_empty());
    assert!(calls.iter().all(|call| call.result["row"] != "Placement"));
}

#[test]
fn common_omissions_and_field_corruptions_fail_reviewed_truth() {
    let case = super::agreement_host::repository().join("tests/e2e/cases/show-json");
    let root = case.join("repo");
    let mut manifests = json!({});
    for name in [
        "expected.exit",
        "expected.stdout",
        "expected.stderr",
        "spec.refs",
        "command.args",
    ] {
        manifests[name] = json!(fs::read_to_string(case.join(name)).unwrap());
    }
    let context = json!({"case":"show-json","base":root,"saved":root,"manifests":manifests});
    let mut actual = super::agreement_corpus::invoke(
        &root,
        &[
            "show".into(),
            "--full".into(),
            "--format=json".into(),
            "FS-001-alpha.1".into(),
            root.to_string_lossy().into(),
        ],
        None,
    );
    assert!(golden(&context, &actual).is_ok());
    actual.stdout.clear();
    assert!(
        golden(&context, &actual).is_err(),
        "shared omission must fail independent truth"
    );
    let body: Value = serde_json::from_str(manifests["expected.stdout"].as_str().unwrap()).unwrap();
    for field in ["body", "anchor", "path", "line"] {
        let mut wrong = body.clone();
        wrong[field] = json!("shared corruption");
        actual.stdout = format!("{wrong}\n").into_bytes();
        assert!(
            golden(&context, &actual).is_err(),
            "shared field corruption: {field}"
        );
    }
}

#[test]
fn common_written_corruption_fails_the_reviewed_output_bytes() {
    let expected = super::agreement_host::repository()
        .join("tests/e2e/cases/fmt-cross-refs-heading-local-section-anchor-write/expected.repo");
    fn files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files(root, &path, out);
            } else {
                out.push((
                    path.strip_prefix(root).unwrap().to_string_lossy().into(),
                    fs::read_to_string(path).unwrap(),
                ));
            }
        }
    }
    let mut bytes = Vec::new();
    files(&expected, &expected, &mut bytes);
    let (_, citation) = bytes
        .iter_mut()
        .find(|(_, body)| body.contains("]("))
        .expect("reviewed written anchor");
    *citation = citation.replacen("](", "](wrong-anchor-", 1);
    let borrowed: Vec<_> = bytes
        .iter()
        .map(|(path, body)| (path.as_str(), body.as_str()))
        .collect();
    let writers: Vec<_> = ["common-cli", "common-python", "common-node"]
        .into_iter()
        .map(|name| Tree::new(name, &borrowed))
        .collect();
    for writer in &writers {
        assert!(super::agreement_corpus::compare_tree(&writers[0].0, &writer.0).is_ok());
        assert!(super::agreement_corpus::compare_tree(&expected, &writer.0).is_err());
    }
}

#[test]
fn real_held_saved_snapshots_preserve_escaped_roots() {
    let text = "# FS-alpha: Alpha\n\nBody.\n";
    let held_text = format!("{text}\n");
    let held = Tree::new(
        "escaped\\held",
        &[("grund.toml", CONFIG_SLUG), ("docs/fs.md", text)],
    );
    let saved = Tree::new(
        "escaped\\saved",
        &[("grund.toml", CONFIG_SLUG), ("docs/fs.md", &held_text)],
    );
    let overlay = held.0.join("docs/fs.md");
    let mut overlays = json!({});
    overlays[overlay.to_string_lossy().as_ref()] = json!(held_text);
    let context = json!({"base":held.0,"saved":saved.0,"path":saved.0,"cwd":saved.0,
        "editorPath":held.0,"explicit":true,"variant":"editor-overlay","overlays":overlays,
        "heldBytes":held_text,"overlayRelative":"docs/fs.md",
        "scopes":{"editor":{"projectRoot":held.0,"reportBase":held.0},
            "cli":{"projectRoot":saved.0,"reportBase":saved.0}}});
    let mut worker = super::agreement_editor_worker::Worker::new();
    let mut errors = Vec::new();
    let loaded = super::agreement_editor::snapshot(&mut worker, &context, &mut errors).unwrap();
    super::agreement_editor::saved_control(&mut worker, &context, &loaded, &mut errors);
    worker.release();
    assert!(
        errors.is_empty(),
        "equivalent snapshots on escaped roots: {errors:?}"
    );
}

#[path = "agreement_control_ownership.rs"]
mod ownership;

/// Allocation extracted unchanged for ownership probes (§AR-ci.10.2, §AR-ci.10.3).
fn no_config_tree() -> Tree {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    for ancestor in parent.ancestors() {
        assert!(
            !ancestor.join("grund.toml").is_file()
                && !ancestor.join(".agents/grund.toml").is_file(),
            "configured fixture ancestor: {}",
            ancestor.display()
        );
    }
    let root = parent.join(format!("grund-no-config-control-{}", std::process::id()));
    fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
    Tree(root.canonicalize().unwrap())
}

#[test]
fn no_config_cli_and_editor_stay_in_the_disposable_input() {
    let tree = no_config_tree();
    fs::write(
        tree.0.join("docs/functional-spec/FS-001-local.md"),
        "# FS-001-local: Local\n\nLocal input only.\n",
    )
    .unwrap();
    let output = super::agreement_corpus::invoke(
        &tree.0,
        &[
            "list".into(),
            "--size".into(),
            "--format".into(),
            "json".into(),
        ],
        None,
    );
    assert!(output.status.success());
    let entries = rows(&output);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], "FS-001-local");
    let context = json!({"base":tree.0,"saved":tree.0,"path":tree.0,"cwd":tree.0,
        "editorPath":tree.0,"explicit":false,"variant":"narrowed","overlays":{},
        "scopes":{"editor":{"projectRoot":tree.0,"reportBase":tree.0},
            "cli":{"projectRoot":tree.0,"reportBase":tree.0}}});
    let mut worker = super::agreement_editor_worker::Worker::new();
    let mut errors = Vec::new();
    let loaded = super::agreement_editor::snapshot(&mut worker, &context, &mut errors).unwrap();
    worker.release();
    assert_eq!(
        loaded["snapshot"]["root"],
        tree.0.to_string_lossy().as_ref()
    );
    assert!(
        errors.is_empty(),
        "isolated no-config entrypoints: {errors:?}"
    );
}
