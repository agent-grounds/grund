//! A stub's home outside the walk answers a citation into it and nothing else
//! (§FS-check.3.2.1): a citation of one of its chapters counts for that chapter,
//! as on the scanned tree (§FS-rules.5.1), through a node no subject or count of
//! chapters reaches (§AR-rules.3), and rules read the home only where a section
//! citation misses, out of the read the section lookup makes (§AR-resolver.5).

use super::facts::{NodeKey, RuleFacts};
use super::markdown::adapt_markdown;
use crate::checker::{check_chapter_rules, check_findings};
use crate::config::{Config, load_config};
use crate::model::Findings;
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

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
const UNCITED: &str = "AR-second is cited by FS 0 times; RULE-ar-cited requires at least one";

/// A stub of `AR-second` in `docs/` pointing at `source.rs`, which `[scan] include`
/// leaves out, `FS-uses` citing `uses`, and a rule that every AR is cited.
fn stub_repo(name: &str, uses: &str) -> (Config, Findings) {
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
fn rules_read_no_target_without_a_section_citation() {
    let (config, findings) = stub_repo("rules_unscanned_home_bare", "\u{a7}AR-second");
    let facts = adapt_markdown(&findings, &config, true);
    assert_eq!(
        findings.stub_targets.read_count(),
        0,
        "§AR-resolver.5: no section asks, so no target is read"
    );
    assert_eq!(cited_by_uses(&facts), ["AR-second"]);
    assert!(
        facts
            .nodes
            .values()
            .all(|meta| meta.label != "AR-second.goals"),
        "a home no citation asks into mints no node"
    );
}

#[test]
fn a_chapter_of_an_unscanned_home_is_a_node_without_a_chapter_row() {
    let uses = "\u{a7}AR-second.goals and \u{a7}AR-second.goals.1";
    let (config, findings) = stub_repo("rules_unscanned_home_chapter", uses);
    let facts = adapt_markdown(&findings, &config, true);
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
        facts.chapter.iter().all(|(node, _, _)| *node != goals.0),
        "§FS-check.3.2.1: no chapter row, so no subject or count of chapters reaches it"
    );
}

#[test]
fn a_section_citation_reads_its_target_once_for_the_whole_check() {
    let uses = "\u{a7}AR-second.goals and \u{a7}AR-second.goals.1";
    let (config, findings) = stub_repo("rules_unscanned_home_read_once", uses);
    let mut report = check_findings(&findings, &config);
    assert_eq!(
        findings.stub_targets.read_count(),
        1,
        "the section lookup reads the target on its miss"
    );

    // A second read would find no `goals` chapter now, so what the rules count
    // comes from the lookup's read.
    write(
        &config.root.join("source.rs"),
        &SOURCE.replace("goals", "aims"),
    );
    check_chapter_rules(&findings, &config, true, None, None, &mut report);
    assert_eq!(
        findings.stub_targets.read_count(),
        1,
        "§AR-resolver.5: one read for the lookup and the rules together"
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
