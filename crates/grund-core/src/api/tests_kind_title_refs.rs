//! Detailed refs metadata comes from the queried target (§FS-config.3.4.3,
//! §FS-refs.3.2); public carrier construction remains compatible (§AR-bindings.2).

use super::*;
use crate::queries::{LspSnapshot, LspSnapshotOpts, LspUsage, lsp_title_hover_body};
use crate::testing::{test_root, write};

#[test]
fn kind_title_refs_preserve_existing_public_records_and_undeclared_queries() {
    let root = test_root("kind_title_refs_public_records");
    write(&root.join("src/user.rs"), "//! §FS-authored\n");
    // No declaration is needed to query a valid ID's citation sites.
    for title in [None, Some("Target title"), Some("")] {
        let metadata = title
            .map(|title| format!("title = \"{title}\"\n"))
            .unwrap_or_default();
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n{metadata}"
            ),
        );
        let opts = RefsOpts {
            path: root.clone(),
            path_provided: true,
            id: "FS-authored".into(),
            section: None,
        };
        // Exhaustive literals deliberately have no metadata fields.
        let expected = RefsOutcome {
            output: RefsOutput {
                output_format: "text".into(),
                workspace: false,
                hits: vec![RefHit {
                    project: None,
                    path: "src/user.rs".into(),
                    line: 1,
                    column: 5,
                    id: "FS-authored".into(),
                    section: None,
                    marker: true,
                    text: "§FS-authored".into(),
                }],
                note: None,
                scan_errors: vec![],
            },
            query_failure: None,
        };
        let result = refs_with_metadata(opts.clone()).unwrap();
        assert_eq!(result.kind_title.as_deref(), title);
        assert_eq!(result.outcome, expected);
        assert_eq!(refs_outcome(opts.clone()).unwrap(), expected);
        assert_eq!(refs(opts).unwrap(), expected.output);
    }
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n",
    );
    let result = refs_with_metadata(RefsOpts {
        path: root,
        path_provided: true,
        id: "FS-authored".into(),
        section: None,
    })
    .unwrap();
    assert_eq!(
        result.kind_title.as_deref(),
        Some("What: behavior, requirements, and constraints")
    );
}

#[test]
fn kind_title_refs_target_differs_from_caller_and_citers() {
    let root = test_root("kind_title_refs_workspace");
    let config = |title: &str| {
        format!(
            "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\ntitle = \"{title}\"\n"
        )
    };
    write(
        &root.join("grund.toml"),
        &(config("Caller title") + "[workspace]\nmembers = [\"target\", \"citer\"]\n"),
    );
    write(&root.join("target/grund.toml"), &config("Target title"));
    write(&root.join("citer/grund.toml"), &config("Citer title"));
    write(&root.join("citer/src/user.rs"), "//! §target/FS-authored\n");
    write(&root.join("target/src/user.rs"), "//! §FS-authored\n");
    write(&root.join("src/user.rs"), "//! §target/FS-authored\n");
    let result = refs_with_metadata(RefsOpts {
        path: root,
        path_provided: true,
        id: "target/FS-authored".into(),
        section: None,
    })
    .unwrap();
    assert_eq!(result.kind_title.as_deref(), Some("Target title"));
    assert_eq!(result.outcome.query_failure, None);
    assert_eq!(
        result.outcome.output.hits,
        vec![
            RefHit {
                project: Some("citer".into()),
                path: "citer/src/user.rs".into(),
                line: 1,
                column: 5,
                id: "FS-authored".into(),
                section: None,
                marker: true,
                text: "§target/FS-authored".into()
            },
            RefHit {
                project: Some("root".into()),
                path: "src/user.rs".into(),
                line: 1,
                column: 5,
                id: "FS-authored".into(),
                section: None,
                marker: true,
                text: "§target/FS-authored".into()
            },
            RefHit {
                project: Some("target".into()),
                path: "target/src/user.rs".into(),
                line: 1,
                column: 5,
                id: "FS-authored".into(),
                section: None,
                marker: true,
                text: "§FS-authored".into()
            },
        ]
    );
}

#[test]
fn kind_title_snapshot_addition_preserves_existing_carrier_and_title_helper() {
    let root = test_root("kind_title_snapshot_api_compatibility");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\ntitle = \"Product contracts\"\n",
    );
    write(
        &root.join("docs/FS-authored.md"),
        "# FS-authored: Authored title\n\n## 1. Detail\nBody.\n",
    );
    write(&root.join("src/user.rs"), "//! §FS-authored.1\n");
    let opts = LspSnapshotOpts {
        path: root,
        path_provided: true,
        open_documents: Default::default(),
    };
    let added = lsp_snapshot_with_metadata(opts.clone()).unwrap();
    let old = lsp_snapshot(opts).unwrap();
    assert_eq!(added.snapshot, old);
    assert_eq!(
        added.kind_titles,
        [
            ("FS-authored".to_string(), "Product contracts".to_string()),
            ("FS-authored.1".to_string(), "Product contracts".to_string()),
        ]
        .into_iter()
        .collect()
    );
    let rebuilt = LspSnapshot {
        root: old.root.clone(),
        marker: old.marker.clone(),
        trigger: old.trigger.clone(),
        workspace: old.workspace,
        report: old.report.clone(),
        declarations: old.declarations.clone(),
        sections: old.sections.clone(),
        finding_ranges: old.finding_ranges.clone(),
        stubs: old.stubs.clone(),
        citations: old.citations.clone(),
        scanned_files: old.scanned_files.clone(),
        scan_errors: old.scan_errors.clone(),
    };
    assert_eq!(rebuilt, old);
    assert_eq!(
        lsp_title_hover_body(
            "FS-authored: Authored title",
            LspUsage { sites: 1, files: 1 }
        ),
        "`FS-authored: Authored title` — cited at 1 site across 1 file"
    );
}
