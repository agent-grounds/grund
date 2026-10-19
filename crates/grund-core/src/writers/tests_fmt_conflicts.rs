//! Test module: the two proof constructors of a complete declaration set refusing
//! on a conflict block (§FS-fmt.7.4), on the scan `fmt` takes, which classifies
//! nothing (§AR-scanner.2.9). Every marker line here is inside an indented string
//! literal, so no line of this file is itself one.

use std::path::PathBuf;

use super::fmt_complete_findings::CompleteScan;
use super::fmt_error::FmtConflictAbort;
use crate::resolver::load_workspace_context;
use crate::testing::{test_root, write};

fn project(name: &str, login: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema]\nmarker = \"\u{a7}\"\n\n[schema.kinds.FS]\n",
    );
    write(&root.join("docs/login.md"), login);
    root
}

/// A set holding a block is no proof: the reused one is withheld, and a fresh
/// scan refuses with the block's `conflict-marker` finding, owners and all,
/// though fmt's load never classified a citation.
#[test]
fn both_proof_constructors_refuse_a_set_holding_a_conflict_block() {
    let root = project(
        "both_proof_constructors_refuse_a_set_holding_a_conflict_block",
        concat!(
            "# FS-login: Login\n\n## format: Format\n\n### .1: Username\n\n",
            "<<<<<<< HEAD\nA username is 2 to 32 characters.\n=======\n",
            "A username is 3 to 64 characters.\n>>>>>>> other\n",
        ),
    );
    let context = load_workspace_context(&root, true).expect("load fmt's context");
    let project = &context.projects[0];
    assert!(
        !project.config.classify_citation_sources,
        "fmt's scan classifies nothing"
    );

    assert!(project.complete_findings().is_none());
    let error = CompleteScan::of_tree_or_abort(&project.config, &project.config)
        .err()
        .expect("a fresh scan refuses");
    let abort = error
        .downcast_ref::<FmtConflictAbort>()
        .expect("refused for the block");
    assert_eq!(abort.conflicts.len(), 1);
    let conflict = &abort.conflicts[0];
    assert_eq!(conflict.code, "conflict-marker");
    assert_eq!(
        (conflict.path.as_deref(), conflict.line),
        (Some("docs/login.md"), Some(7))
    );
    assert_eq!(
        abort.to_string(),
        "nothing was rewritten: docs/login.md:7: unresolved merge conflict in FS-login, \
         lines 7\u{2013}11: resolve it by hand"
    );
}

/// The refusal is the block's alone: the same file resolved is a proof again.
#[test]
fn a_set_with_no_block_is_a_proof() {
    let root = project(
        "a_set_with_no_block_is_a_proof",
        "# FS-login: Login\n\nA username is 3 to 64 characters.\n",
    );
    let context = load_workspace_context(&root, true).expect("load fmt's context");
    let project = &context.projects[0];

    assert!(project.complete_findings().is_some());
    assert!(CompleteScan::of_tree_or_abort(&project.config, &project.config).is_ok());
}
