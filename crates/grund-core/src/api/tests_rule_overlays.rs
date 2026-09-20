//! Unsaved rule rationales use the shared overlay scan (§FS-lsp.1.1,
//! §FS-lsp.4, §FS-rules.9).

use super::lsp_snapshot;
use crate::queries::LspSnapshotOpts;
use crate::testing::{test_root, write};
use std::collections::BTreeMap;

fn rule_repo(name: &str) -> std::path::PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/goals/GOAL-rules.md"),
        "# GOAL-rules: Rules\n\nGoal.\n",
    );
    write(
        &root.join("docs/fs/FS-demo.md"),
        "# FS-demo: Demo\n\nNo citation.\n",
    );
    root
}

#[test]
fn lsp_rule_rationale_follows_unsaved_addition_and_deletion() {
    let root = rule_repo("lsp_rule_rationale_overlays");
    let rule = root.join("docs/rules/RULE-goal.md");
    let title = "# RULE-goal: Each FS must cite at least one GOAL.\n";
    write(&rule, &format!("{title}\nSaved rationale.\n"));

    let deleted = lsp_snapshot(LspSnapshotOpts {
        path: root.clone(),
        path_provided: true,
        open_documents: BTreeMap::from([(rule.clone(), format!("{title}\n   \n"))]),
    })
    .expect("snapshot with deleted rationale");
    assert!(deleted.report.errors.iter().any(|finding| {
        finding.code == "invalid-rule" && finding.message.contains("rule rationale is empty")
    }));

    write(&rule, &format!("{title}\n   \n"));
    let added = lsp_snapshot(LspSnapshotOpts {
        path: root,
        path_provided: true,
        open_documents: BTreeMap::from([(
            rule,
            format!("{title}\nUnsaved rationale for §GOAL-rules.\n"),
        )]),
    })
    .expect("snapshot with added rationale");
    assert!(added.report.errors.iter().all(|finding| {
        !(finding.code == "invalid-rule" && finding.message.contains("rule rationale is empty"))
    }));
    assert!(added.report.errors.iter().any(|finding| {
        finding.code == "missing-citation" && finding.message.contains("RULE-goal")
    }));
}
