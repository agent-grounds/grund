//! A rule object kind pinned at a workspace member, across the scopes a command
//! can run in (§FS-rules.4.1). The sequence a single e2e case cannot express —
//! write the managed block from inside a member, then check it from the
//! workspace root — lives here, beside the classification table §FS-rules.4.1.1
//! fixes.

use super::support::{assert_run, case_repo, run, scratch_from, text, write};
use std::fs;
use std::path::Path;

const PINNED: &str = "The operations chapter of each SEG must cite at least one workshop/OP.";
const BULLET: &str = "- The operations chapter of each SEG must cite at least one workshop/OP. [\u{a7}RULE-seg-operations-workshop](docs/rules/RULE-seg-operations-workshop.md#rule-seg-operations-workshop-the-operations-chapter-of-each-seg-must-cite-at-least-one-workshopop)\n";

/// The fixture the aged-block e2e case carries: a two-member workspace whose
/// `member` holds a rule pinned at `workshop`, and whose managed block a grund
/// upgrade has aged from v15 to v1.
fn aged_workspace(name: &str) -> std::path::PathBuf {
    scratch_from(&case_repo("init-workspace-pinned-rule-member"), name)
}

/// The managed block's own version line, which is what says whether a run
/// rewrote the block — legible in a failure where two whole files are not.
fn version_marker(text: &str) -> String {
    text.lines()
        .find(|line| line.starts_with("## Grounding with grund"))
        .unwrap_or("<no managed block>")
        .to_string()
}

fn rule(root: &Path, project: &str, sentence: &str) {
    write(
        root,
        &format!("{project}/docs/rules/RULE-seg-operations-workshop.md"),
        &format!(
            "# RULE-seg-operations-workshop: {sentence}\n\nThe operations chapter carries the build evidence for \u{a7}GOAL-build.\n"
        ),
    );
}

/// §FS-rules.9.1, §FS-check.3.5.4: the bytes a member-scoped `init` writes are
/// the bytes both scopes then accept. This is the obligation a tidier fix — one
/// that omits the bullet it cannot verify — breaks: the member would write a
/// block its own workspace root reports as drifted, so `grund init` inside a
/// member and CI at the root could not both be satisfied.
#[test]
fn a_member_writes_the_block_the_workspace_root_then_accepts() {
    let root = aged_workspace("member-init-sequence");
    let member = root.join("member");
    let before = version_marker(&fs::read_to_string(member.join("AGENTS.md")).expect("aged block"));
    assert_eq!(
        before, "## Grounding with grund (v1)",
        "the fixture starts aged"
    );

    let member_arg = member.to_string_lossy().into_owned();
    let output = run(&root, &["init", &member_arg, "--no-vcs"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "the exit code does not move: {}",
        text(&output.stdout)
    );
    let agents = fs::read_to_string(member.join("AGENTS.md")).expect("managed block");
    assert_eq!(
        version_marker(&agents),
        "## Grounding with grund (v15)",
        "the write is what stops being withheld, so the aged block is rewritten"
    );
    assert!(
        agents.contains(BULLET),
        "the unverifiable rule renders as authored, and it did not:\n{agents}"
    );

    let root_arg = root.to_string_lossy().into_owned();
    let output = run(&root, &["check", &root_arg, "--only", "agents-init"]);
    assert_run(&output, 0, "success\n", "");
    let output = run(&root, &["check", &member_arg, "--only", "agents-init"]);
    assert_run(&output, 0, "success\n", "");

    let golden = case_repo("check-member-pinned-rule-block-current").join("member/AGENTS.md");
    assert_eq!(
        agents,
        fs::read_to_string(&golden).expect("root-scoped case fixture"),
        "the written block is the one the root-scoped case holds"
    );
}

/// What `init` did with one rule: `Some(true)` wrote the block, `Some(false)`
/// withheld it, and the bool is paired with the run's exit code.
fn init_verdict(root: &Path, project: &str) -> (i32, bool) {
    let target = if project.is_empty() {
        root.to_path_buf()
    } else {
        root.join(project)
    };
    let agents = target.join("AGENTS.md");
    let before = fs::read(&agents).ok();
    let target_arg = target.to_string_lossy().into_owned();
    let output = run(root, &["init", &target_arg, "--no-vcs"]);
    let after = fs::read(&agents).ok();
    (output.status.code().unwrap_or(-1), after != before)
}

/// §FS-rules.4.1.1: the seven rows of the classification table, read through the
/// only thing a user can see — whether the managed block was written. Only the
/// absence of every workspace namespace makes a sentence unverifiable, so an
/// alias the scope could judge and rejected still withholds the write.
#[test]
fn only_a_scope_with_no_workspace_at_all_makes_a_rule_unverifiable() {
    // A run that holds the workspace: the root project of the same fixture set,
    // which declares the rule kind itself and lists `workshop` as a member.
    for (sentence, exit, wrote) in [
        (PINNED, 0, true),
        (
            "The operations chapter of each SEG must cite at least one workshop/NOPE.",
            1,
            false,
        ),
        (
            "The operations chapter of each SEG must cite at least one typo/OP.",
            1,
            false,
        ),
        (
            "The operations chapter of each SEG must cite at least one */OP.",
            0,
            true,
        ),
        (
            "The operations chapter of each SEG must cite at least one */NOPE.",
            1,
            false,
        ),
    ] {
        let root = scratch_from(
            &case_repo("init-workspace-unknown-alias-rule"),
            "table-root",
        );
        rule(&root, ".", sentence);
        assert_eq!(
            init_verdict(&root, ""),
            (exit, wrote),
            "workspace in scope: {sentence}"
        );
    }

    // A run that holds none: the member of the two-member workspace, whose own
    // `grund.toml` declares no `[workspace]` at all.
    for (sentence, exit, wrote) in [
        (PINNED, 1, true),
        (
            "The operations chapter of each SEG must cite at least one */NOPE.",
            1,
            true,
        ),
        (
            "The operations chapter of each SEG must cite at least one GOAL.",
            0,
            true,
        ),
        (
            "The operations chapter of each SEG must cite at least one */GOAL.",
            0,
            true,
        ),
    ] {
        let root = aged_workspace("table-member");
        rule(&root, "member", sentence);
        assert_eq!(
            init_verdict(&root, "member"),
            (exit, wrote),
            "no workspace in scope: {sentence}"
        );
    }
}

/// §FS-rules.4.1.2: one genuinely invalid rule beside an unverifiable one puts
/// the whole run back under §FS-rules.4 — nothing written, exit nonzero. The
/// narrow exception is not a general relaxation.
#[test]
fn an_invalid_rule_beside_an_unverifiable_one_still_withholds_the_write() {
    let root = aged_workspace("narrowness");
    write(
        &root,
        "member/docs/rules/RULE-broken.md",
        "# RULE-broken: Segments ought to be nice.\n\nA sentence that is not a rule at all, for \u{a7}GOAL-build.\n",
    );
    assert_eq!(init_verdict(&root, "member"), (1, false));
}

/// §FS-rules.4.1, §FS-rules.4.1.2: one rule, invalid in its subject and
/// unverifiable in its object, reaches the same verdict as the same rule with a
/// local object — because unverifiability is about the object's namespace and
/// nothing else. The guard above pairs an invalid rule with a *separate*
/// unverifiable one; this is the case that hid inside a single sentence, where a
/// sibling-pinned object used to suppress the subject verdict the same scope
/// makes on its own and the block was written around a rule it called invalid.
#[test]
fn a_bogus_subject_is_judged_whether_or_not_the_object_resolves_here() {
    let mut verdicts = Vec::new();
    for object in ["workshop/OP", "GOAL"] {
        let root = aged_workspace("subject-under-each-object");
        rule(
            &root,
            "member",
            &format!("SEG-nonexistent must cite at least one {object}."),
        );
        let member = root.join("member");
        let member_arg = member.to_string_lossy().into_owned();
        let output = run(&root, &["init", &member_arg, "--no-vcs"]);
        let block = version_marker(&fs::read_to_string(member.join("AGENTS.md")).expect("block"));
        verdicts.push((output.status.code(), block, text(&output.stdout)));
    }

    let (pinned, local) = (&verdicts[0], &verdicts[1]);
    assert_eq!(
        pinned, local,
        "the object kind does not move the verdict on the subject"
    );
    assert_eq!(pinned.0, Some(1), "an invalid rule still exits nonzero");
    assert_eq!(
        pinned.1, "## Grounding with grund (v1)",
        "and still withholds the write, so the aged block survives"
    );
    assert!(
        pinned
            .2
            .contains("literal subject SEG-nonexistent does not resolve"),
        "the run names the fact it can act on: {}",
        pinned.2
    );
}
