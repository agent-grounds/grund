//! The four synchronized documentation pins required by §FS-rules.10: runnable
//! example goldens, executable guide rows, skill bytes, and managed rendering.

use super::support::{fixture, repo_root, run, scratch, text};
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
fn guide_marked_rows_execute_against_the_released_parser() {
    let root = repo_root();
    let guide = fs::read_to_string(root.join("docs/user-facing/rules.md")).expect("rules guide");
    let accepted = marked(
        guide.as_bytes(),
        b"<!-- BEGIN chapter-rules-accepted -->\n",
        b"<!-- END chapter-rules-accepted -->",
    );
    let accepted = std::str::from_utf8(accepted).expect("accepted guide rows");
    let accepted_root = scratch("documented-accepted-rows");
    let accepted_config =
        fs::read_to_string(accepted_root.join("grund.toml")).expect("fixture config");
    fs::write(
        accepted_root.join("grund.toml"),
        accepted_config.replace("rules = true\n", ""),
    )
    .expect("disable configured rules");
    let mut accepted_count = 0;
    for line in accepted.lines().filter(|line| line.starts_with("- `")) {
        let sentence_end = line[3..].find('`').expect("accepted sentence end") + 3;
        let sentence = &line[3..sentence_end];
        let result = &line[sentence_end + 1..];
        let code_start = result.find('`').expect("accepted finding start") + 1;
        let code_end = result[code_start..]
            .find('`')
            .expect("accepted finding end")
            + code_start;
        let code = &result[code_start..code_end];
        let suggestion = result.contains("suggestion");
        let mut args = vec![
            "check", ".", "--rule", sentence, "--only", code, "--format", "json",
        ];
        if suggestion {
            args.push("--suggestions");
        }
        let output = run(&accepted_root, &args);
        assert_ne!(
            output.status.code(),
            Some(2),
            "documented accepted row was refused: {sentence}: {}",
            text(&output.stderr)
        );
        for row in text(&output.stdout).lines() {
            assert!(
                row.contains(&format!("\"code\":\"{code}\"")),
                "documented row produced the wrong finding: {sentence}: {row}"
            );
            assert_eq!(
                row.contains("\"channel\":\"suggestion\""),
                suggestion,
                "documented row produced the wrong channel: {sentence}: {row}"
            );
        }
        accepted_count += 1;
    }
    assert_eq!(accepted_count, 17, "accepted guide-row inventory drifted");

    let refused = marked(
        guide.as_bytes(),
        b"<!-- BEGIN chapter-rules-refused -->\n",
        b"<!-- END chapter-rules-refused -->",
    );
    let refused = std::str::from_utf8(refused).expect("refused guide rows");
    let named_off = scratch("documented-named-sections-refusal");
    let config = fs::read_to_string(named_off.join("grund.toml")).expect("fixture config");
    fs::write(
        named_off.join("grund.toml"),
        config.replace("named_sections = true", "named_sections = false"),
    )
    .expect("disable named sections");
    let fixture_root = fixture();
    let mut refused_count = 0;
    for line in refused.lines().filter(|line| line.starts_with("- ")) {
        let arrow = line.find(" → ").expect("refused row arrow");
        let left = &line[..arrow];
        let sentence_start = left.find('`').expect("refused sentence start") + 1;
        let sentence_end = left[sentence_start..]
            .find('`')
            .expect("refused sentence end")
            + sentence_start;
        let sentence = &left[sentence_start..sentence_end];
        let right = &line[arrow + " → ".len()..];
        let reason_start = right.find('`').expect("refusal reason start") + 1;
        let reason_end = right.rfind('`').expect("refusal reason end");
        let reason = &right[reason_start..reason_end];
        let repo = if left.starts_with("- With named sections off") {
            &named_off
        } else {
            &fixture_root
        };
        let output = run(repo, &["check", ".", "--rule", sentence]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "refusal was accepted: {sentence}"
        );
        assert_eq!(text(&output.stdout), "");
        assert_eq!(text(&output.stderr), format!("error: {reason}\n"));
        refused_count += 1;
    }
    assert_eq!(refused_count, 13, "refused guide-row inventory drifted");
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
