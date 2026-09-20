//! The four synchronized documentation pins required by §FS-rules.10: runnable
//! example goldens, executable guide rows, skill bytes, and managed rendering.

use super::support::repo_root;
use std::fs;

fn marked<'a>(bytes: &'a [u8], begin: &[u8], end: &[u8]) -> &'a [u8] {
    let start = bytes
        .windows(begin.len())
        .position(|window| window == begin)
        .expect("begin marker")
        + begin.len();
    let finish = bytes[start..]
        .windows(end.len())
        .position(|window| window == end)
        .expect("end marker")
        + start;
    &bytes[start..finish]
}

#[test]
fn guide_example_readmes_and_goldens_ship_together() {
    let root = repo_root();
    for relative in [
        "docs/user-facing/rules.md",
        "examples/rules/README.md",
        "examples/rules/expected.exit",
        "examples/rules/expected.stdout",
        "examples/rules/expected.stderr",
    ] {
        assert!(root.join(relative).is_file(), "missing {relative}");
    }
    let readme = fs::read_to_string(root.join("README.md")).expect("README");
    let examples = fs::read_to_string(root.join("examples/README.md")).expect("examples index");
    assert!(readme.contains("docs/user-facing/rules.md"));
    assert!(readme.contains("examples/rules/"));
    assert!(examples.contains("rules/"));
}

#[test]
fn guide_writing_section_and_both_skill_copies_are_byte_identical() {
    let root = repo_root();
    let guide = fs::read(root.join("docs/user-facing/rules.md")).expect("rules guide");
    let repo_skill = fs::read(root.join("skills/grund-init/SKILL.md")).expect("repository skill");
    let embedded = fs::read(root.join("crates/grund-core/assets/skills/grund-init/SKILL.md"))
        .expect("embedded skill");
    assert_eq!(repo_skill, embedded, "whole skill copies drifted");
    let begin = b"<!-- BEGIN chapter-rules -->\n";
    let end = b"<!-- END chapter-rules -->";
    assert_eq!(marked(&guide, begin, end), marked(&repo_skill, begin, end));
}

#[test]
fn guide_marks_every_accepted_family_refusal_finding_and_channel_for_extraction() {
    let root = repo_root();
    let guide = fs::read_to_string(root.join("docs/user-facing/rules.md")).expect("rules guide");
    for marker in [
        "BEGIN chapter-rules-accepted",
        "BEGIN chapter-rules-refused",
        "chapter-cardinality",
        "citation-cardinality",
        "missing-citation",
        "uncited-unit",
        "forbidden-citation",
        "suggested-citation",
        "discouraged-citation",
        "--suggestions",
        "RULE-a, RULE-b",
    ] {
        assert!(guide.contains(marker), "rules guide is missing {marker}");
    }
}

#[test]
fn rules_example_inventory_covers_every_ruled_behavior() {
    let root = repo_root().join("examples/rules");
    let readme = fs::read_to_string(root.join("README.md")).expect("rules example README");
    for clause in [
        "must have",
        "must cite at least one",
        "must cite each",
        "be cited by",
        "must not cite any",
        "shared prefix",
        "config-to-rule",
        "rule-to-rule",
        "invalid-rule",
    ] {
        assert!(readme.contains(clause), "rules example is missing {clause}");
    }
}
