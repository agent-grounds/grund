//! A stub's home outside the walk lends the stubbed declaration its chapters
//! (§FS-declarations.stubs.target.5): a citation of one of them counts for that
//! chapter, as on the scanned tree (§FS-rules.5.1), each is a node with a chapter
//! row, cited or not, judged as a scanned one since §FS-rules.7.11 closed (§AR-rules.3), and rules read
//! the home out of the scan's one reading of the stub's target (§AR-scanner.4.6),
//! never a reading of their own.

use super::facts::{NodeKey, RuleFacts};
use super::markdown::adapt_markdown;
use crate::checker::{check_chapter_rules, check_findings, markdown_project};
use crate::config::{Config, load_config};
use crate::model::Catalog;
use crate::resolver::WorkspaceCheckTarget;
use crate::testing::{scan_tree, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
[[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\nindex = false\n\n\
[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n\n\
[scan]\ninclude = [\"docs\"]\n";

/// `AR-second`, whose `goals` chapter holds a numbered section.
const SOURCE: &str = concat!(
    "/// AR-second: Second\n///\n/// Second lead.\n///\n",
    "/// ## goals: Goals\n///\n/// Second goals.\n///\n",
    "/// ### goals.1: Detail\n///\n/// Second detail.\npub fn second() {}\n",
);

/// What the rule reports of `AR-second` when no citation counts for the declaration.
const UNCITED: &str = "AR-second is cited by FS 0 times; RULE-ar-cited requires at least one; add or remove the citations of it until the count fits";

/// A stub of `AR-second` in `docs/` pointing at `source.rs`, which `[scan] include`
/// leaves out, `FS-uses` citing `uses`, and a rule that every AR is cited.
fn stub_repo(name: &str, uses: &str) -> (Config, Catalog) {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/ar/second.md"),
        "# AR-second: [../../source.rs](../../source.rs)\n",
    );
    write(
        &root.join("docs/fs/uses.md"),
        &format!("# FS-uses: Uses\n\nUses {uses}.\n"),
    );
    write(
        &root.join("docs/rules/RULE-ar-cited.md"),
        "# RULE-ar-cited: Each AR must be cited by at least one FS.\n\n\
         An architecture point no functional point cites, as \u{a7}FS-uses does, is unread.\n",
    );
    write(&root.join("source.rs"), SOURCE);
    let config = load_config(&root).expect("load config");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    (config, findings)
}

fn label(facts: &RuleFacts, node: &NodeKey) -> String {
    facts.nodes[node].label.clone()
}

/// The targets of `FS-uses`'s `cites` facts, by label, in fact order.
fn cited_by_uses(facts: &RuleFacts) -> Vec<String> {
    facts
        .cites
        .iter()
        .filter(|(_, from, _)| label(facts, from) == "FS-uses")
        .map(|(_, _, target)| label(facts, target))
        .collect()
}

#[test]
fn an_uncited_chapter_of_an_unscanned_home_is_still_lent() {
    let (config, findings) = stub_repo("rules_unscanned_home_bare", "\u{a7}AR-second");
    let facts = adapt_markdown(
        &findings,
        markdown_project(
            config.frame().name,
            &WorkspaceCheckTarget::of(&findings, config.schema(), config.frame()),
        ),
        true,
    );
    assert_eq!(cited_by_uses(&facts), ["AR-second"]);
    assert!(
        facts
            .nodes
            .values()
            .any(|meta| meta.label == "AR-second.goals"),
        "§FS-declarations.stubs.target.5: a chapter is lent, cited or not"
    );
}

#[test]
fn a_chapter_of_an_unscanned_home_is_a_lent_chapter() {
    let uses = "\u{a7}AR-second.goals and \u{a7}AR-second.goals.1";
    let (config, findings) = stub_repo("rules_unscanned_home_chapter", uses);
    let facts = adapt_markdown(
        &findings,
        markdown_project(
            config.frame().name,
            &WorkspaceCheckTarget::of(&findings, config.schema(), config.frame()),
        ),
        true,
    );
    assert_eq!(
        cited_by_uses(&facts),
        ["AR-second.goals", "AR-second.goals"],
        "§FS-rules.5.1: the chapter, and the numbered section's nearest chapter"
    );
    let goals = facts
        .nodes
        .iter()
        .find(|(_, meta)| meta.label == "AR-second.goals")
        .map(|(node, meta)| (node.clone(), meta.anchor.clone()))
        .expect("the chapter's node");
    assert!(
        goals.1.path.ends_with("source.rs") && goals.1.line == 5,
        "anchored at the target's heading, not the stub: {:?}",
        goals.1
    );
    assert!(
        facts
            .contains
            .iter()
            .any(|(parent, child)| *child == goals.0 && label(&facts, parent) == "AR-second"),
        "contained by the stub's declaration, so its kind and owner resolve"
    );
    assert!(
        facts.chapter.iter().any(|(node, _, _)| *node == goals.0),
        "§FS-declarations.stubs.target.5: a chapter row, so subjects and counts reach it"
    );
}

#[test]
fn a_section_citation_answers_from_the_scans_reading() {
    let uses = "\u{a7}AR-second.goals and \u{a7}AR-second.goals.1";
    let (config, findings) = stub_repo("rules_unscanned_home_read_once", uses);
    let mut report = check_findings(&findings, &config);

    // A second read would find no `goals` chapter now, so what the rules count
    // comes from the scan's reading of the target (§AR-scanner.4.6).
    write(
        &config.root.join("source.rs"),
        &SOURCE.replace("goals", "aims"),
    );
    check_chapter_rules(
        &findings,
        config.rules(),
        config.schema(),
        config.frame(),
        true,
        None,
        None,
        &mut report,
    );
    let errors: Vec<&str> = report
        .errors
        .iter()
        .map(|error| error.message.as_str())
        .collect();
    assert!(
        errors.contains(&UNCITED),
        "§FS-rules.5.1: both citations count for the chapter, as on the scanned tree: {errors:?}"
    );
}
