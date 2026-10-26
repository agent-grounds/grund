//! The five synchronized documentation pins required by §FS-rules.10: runnable
//! example goldens, executable guide rows, skill bytes, managed rendering, and
//! the quoted `invalid-rule` message.

use super::support::{
    absent_chapter, assert_run, assert_v1_run, fixture, repo_root, run, scratch, text,
    unconfigured_rules,
};
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

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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

/// §FS-rules.10.1: the maintained example must teach the path-subject refusal
/// actually printed, including no form where none is offered.
#[test]
fn example_path_subject_entry_teaches_the_cli_refusal_without_a_rewrite() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("examples/rules/README.md"))
        .expect("maintained rules example README");
    let inventory = readme
        .split_once("## Refusal inventory")
        .expect("refusal inventory")
        .1
        .split("\n## ")
        .next()
        .expect("inventory body");
    let sentence = "Each file in vendor/ must cite at least one FS.";
    // Join continuation lines so wrapping the prose or an inline code span
    // cannot change what this entry teaches.
    let entry = inventory
        .split("\n- ")
        .skip(1)
        .map(collapsed)
        .find(|entry| entry.contains(sentence))
        .expect("maintained path-subject entry");
    let output = run(
        &root.join("examples/rules/repo"),
        &["check", ".", "--only-rule", "--rule", sentence],
    );
    let stderr = text(&output.stderr);
    assert_run(
        &output,
        2,
        "",
        "error: path subjects are not accepted in phase 1\n\
         known kinds: GOAL, REQ, FS, AR, RULE\n",
    );
    assert!(!stderr.contains("accepted form"), "CLI offered a rewrite");

    let documented: Vec<&str> = entry.split('`').skip(1).step_by(2).collect();
    let mut printed = vec![sentence];
    printed.extend(
        stderr
            .trim()
            .lines()
            .map(|line| line.strip_prefix("error: ").unwrap_or(line)),
    );
    assert_eq!(
        documented,
        printed,
        "examples/rules/README.md path-subject entry disagrees with the CLI\n\
         README entry: {entry}\nCLI exit: {:?}\nCLI stderr:\n{stderr}",
        output.status.code(),
    );
    assert!(
        ["no rewrite", "no accepted form", "no accepted-form"]
            .iter()
            .any(|phrase| entry.contains(phrase)),
        "path-subject entry must explain that no rewrite is offered: {entry}"
    );
    let introduction = collapsed(inventory.split("\n- ").next().expect("introduction"));
    assert!(
        !introduction.contains("the right-hand text is the exact accepted rewrite"),
        "inventory still promises an accepted rewrite for every entry: {introduction}"
    );
}

/// Every file of one skill directory, `SKILL.md` then `references/` in name order.
fn skill_files(dir: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let mut paths = vec![dir.join("SKILL.md")];
    let mut references: Vec<_> = fs::read_dir(dir.join("references"))
        .expect("the skill's references")
        .map(|entry| entry.unwrap().path())
        .collect();
    references.sort();
    paths.extend(references);
    paths
        .into_iter()
        .map(|path| {
            let name = path.strip_prefix(dir).unwrap().display().to_string();
            (name, fs::read(&path).expect("skill file"))
        })
        .collect()
}

/// The chapter-rules copy is the skill's `rules` section, and the two skill directories
/// are byte-identical file for file (§FS-rules.10, §FS-init.5.3).
#[test]
fn guide_writing_section_and_both_skill_copies_are_byte_identical() {
    let root = repo_root();
    let guide = fs::read(root.join("docs/user-facing/rules.md")).expect("rules guide");
    let repo_skill = root.join("skills/grund-init");
    let embedded = root.join("crates/grund-core/assets/skills/grund-init");
    assert_eq!(
        skill_files(&repo_skill),
        skill_files(&embedded),
        "whole skill copies drifted"
    );
    let rules = fs::read(repo_skill.join("references/rules.md")).expect("the rules section");
    let begin = b"<!-- BEGIN chapter-rules -->\n";
    let end = b"<!-- END chapter-rules -->";
    assert_eq!(marked(&guide, begin, end), marked(&rules, begin, end));
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
    // §FS-check.1.4: `--only` now carries `invalid-rule`, so a row whose literal
    // subject does not resolve would show; declare the one the guide names.
    fs::write(
        accepted_root.join("docs/fs/FS-login.md"),
        "# FS-login: A literal subject\n\n## goals: Goals\n\nGoals.\n\n\
         ## requirements: Requirements\n\nRequirements.\n",
    )
    .expect("declare the guide's literal subject");
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
    assert_eq!(accepted_count, 23, "accepted guide-row inventory drifted");

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
        let reason_end =
            right[reason_start..].find('`').expect("refusal reason end") + reason_start;
        let reason = &right[reason_start..reason_end];
        // §FS-rule-refusals.sentences.rewrite.no-form: a row that offers no form names the line
        // after it.
        let after = match right[reason_end + 1..].strip_prefix(", then `") {
            Some(rest) => format!("{}\n", rest.strip_suffix('`').expect("kinds line end")),
            None => String::new(),
        };
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
        assert_eq!(text(&output.stderr), format!("error: {reason}\n{after}"));
        refused_count += 1;
    }
    assert_eq!(refused_count, 20, "refused guide-row inventory drifted");
}

/// The guide may not teach the emptiness rule in words the specification does
/// not use: the pairing sentence is one string in both places, so neither can
/// be reworded, and neither dropped, without the other (§FS-rules.10).
#[test]
fn the_documented_emptiness_rule_is_the_specifications() {
    let root = repo_root();
    let guide = fs::read(root.join("docs/user-facing/rules.md")).expect("rules guide");
    let begin = b"<!-- BEGIN chapter-rules-emptiness -->\n";
    let end = b"<!-- END chapter-rules-emptiness -->";
    assert!(
        guide.windows(begin.len()).any(|window| window == begin),
        "the rules guide has no chapter-rules-emptiness region"
    );
    let region = std::str::from_utf8(marked(&guide, begin, end)).expect("emptiness region");
    let lead = collapsed(
        region
            .split("\n\n")
            .find(|paragraph| !paragraph.trim().is_empty())
            .expect("emptiness region lead paragraph"),
    );
    let spec =
        fs::read_to_string(root.join("docs/functional-spec/FS-rules.md")).expect("rules spec");
    assert!(
        collapsed(&spec).contains(&lead),
        "the guide's emptiness rule is not the specification's: {lead}"
    );
}

/// The behaviour the documentation asserts, either way round: the rule reports
/// the chapter that cites nothing, and it reports the declaration that has no
/// such chapter rather than passing over it (§FS-rules.2,
/// §FS-rules.checks.unreached-declaration). Without the second half the accepted
/// guide row above passes vacuously.
#[test]
fn a_chapter_rule_reports_the_declaration_without_the_chapter() {
    let sentence = "The requirements chapter of each FS must cite at least one REQ.";
    let filtered = [
        "check",
        ".",
        "--rule",
        sentence,
        "--only",
        "missing-citation",
        "--format",
        "json",
    ];

    let present = unconfigured_rules("documented-present-chapter");
    assert_run(
        &run(&present, &filtered),
        1,
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,\
         \"code\":\"missing-citation\",\
         \"message\":\"FS-demo.requirements must cite REQ (--rule); add such a citation within it\",\"sites\":null,\
         \"authority\":[\"--rule\"]}\n",
        "",
    );

    // `--only` selects after the scan and drops the exit code with the findings
    // it filters, so the reporting half runs unfiltered: the absence is the
    // whole of what the run says.

    // It is an error at the declaration and fails the run, with the landed
    // clause in its own bytes (§FS-rules.7, §FS-errors.5.5).
    let unfiltered = ["check", ".", "--rule", sentence, "--format", "json"];
    let absent = absent_chapter("documented-absent-chapter");
    assert_v1_run(
        &run(&absent, &unfiltered),
        1,
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":1,\
         \"code\":\"unreached-declaration\",\
         \"message\":\"FS-demo has no requirements chapter, so --rule cannot reach it; \
         add the chapter, or narrow the rule to the declarations that have one; \
         this became an error in grund 0.16.0\",\"sites\":null,\
         \"authority\":[\"--rule\"]}\n",
        "",
    );
}

/// The section quotes the `invalid-rule` message an exact chapter subject
/// produces once its chapter is gone, and ships that quote into every
/// consuming repository, so the binary's own wording pins it (§FS-rules.10).
/// The guide names the illustration `FS-login` where a run names the fixture's
/// `FS-demo`, so the quote is compared with the ID substituted.
#[test]
fn the_documented_invalid_rule_quote_is_the_binarys() {
    let guide = fs::read(repo_root().join("docs/user-facing/rules.md")).expect("rules guide");
    let region = std::str::from_utf8(marked(
        &guide,
        b"<!-- BEGIN chapter-rules-emptiness -->\n",
        b"<!-- END chapter-rules-emptiness -->",
    ))
    .expect("emptiness region");
    let quoted = "literal subject FS-login.requirements does not resolve";
    assert!(
        collapsed(region).contains(quoted),
        "the emptiness region no longer quotes the invalid-rule message"
    );

    let absent = absent_chapter("documented-quoted-invalid-rule");
    let output = run(
        &absent,
        &[
            "check",
            ".",
            "--rule",
            "FS-demo.requirements must cite at least one REQ.",
            "--format",
            "json",
        ],
    );
    assert_v1_run(
        &output,
        1,
        "{\"severity\":\"error\",\"path\":null,\"line\":1,\
         \"code\":\"invalid-rule\",\"message\":\"--rule is not a valid rule: \
         literal subject FS-demo.requirements does not resolve\",\"sites\":null,\
         \"authority\":[\"--rule\"]}\n",
        "",
    );
    assert!(
        text(&output.stdout).contains(&quoted.replace("FS-login", "FS-demo")),
        "the binary no longer produces the message the guide quotes"
    );
}

#[test]
fn rules_example_inventory_covers_every_ruled_behavior() {
    let root = repo_root().join("examples/rules");
    let readme = fs::read_to_string(root.join("README.md")).expect("rules example README");
    for clause in [
        "must have",
        "must cite at least one",
        "must cite at least 2",
        "must cite each",
        "must cite each AR at most once.",
        "numeric \"at most 1 times\" is not canonical; accepted form:",
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
