//! The released text, JSON, CLI lifecycle, and compatibility surfaces of
//! §FS-rules.3, §FS-rules.4, §FS-rules.6, §FS-rules.7, §FS-rules.8 and
//! §FS-rules.9.

use super::support::{assert_run, fixture, run, scratch, text, write};
use std::fs;

const TEXT_FINDINGS: &str = concat!(
    "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one 2 times; ",
    "RULE-overview requires exactly once\n",
    "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one-more 0 times; ",
    "RULE-overview requires exactly once\n",
    "docs/fs/FS-demo.md:6: error: FS-demo.requirements must cite REQ (RULE-requirements)\n",
);

/// §FS-check.3.27: each off-count target of a `cite each` sentence is one
/// `citation-cardinality` error at the subject title.
#[test]
fn chapter_scope_and_exact_once_coverage_are_hard_findings() {
    let output = run(&fixture(), &["check", "."]);
    assert_run(&output, 1, TEXT_FINDINGS, "");
}

/// §FS-rules.7.6: rule-derived JSON adds no source-list field, and same-anchor
/// rows sort by target-ID bytes even where one target prefixes another.
#[test]
fn rule_findings_have_exact_ndjson_codes_and_bytes() {
    let output = run(&fixture(), &["check", ".", "--format", "json"]);
    let expected = concat!(
        "{\"severity\":\"error\",\"path\":\"docs/ar/AR-overview.md\",\"line\":3,",
        "\"code\":\"citation-cardinality\",\"message\":\"AR-overview.system-overview cites ",
        "AR-one 2 times; RULE-overview requires exactly once\",\"sites\":null}\n",
        "{\"severity\":\"error\",\"path\":\"docs/ar/AR-overview.md\",\"line\":3,",
        "\"code\":\"citation-cardinality\",\"message\":\"AR-overview.system-overview cites ",
        "AR-one-more 0 times; RULE-overview requires exactly once\",\"sites\":null}\n",
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,",
        "\"code\":\"missing-citation\",\"message\":\"FS-demo.requirements must cite REQ ",
        "(RULE-requirements)\",\"sites\":null}\n",
    );
    assert_run(&output, 1, expected, "");
}

/// §FS-rules.3.1, §FS-rules.7.2, §FS-check.3.26: a `have` sentence counts the named
/// direct chapters and reports a zero count at the declaration title; as a
/// `should` it is a suggestion, which never moves the exit (§FS-rules.7.6).
#[test]
fn ad_hoc_rule_is_additive_and_suggestions_do_not_change_the_exit() {
    let sentence = "Each FS should have exactly one security chapter.";
    let output = run(
        &fixture(),
        &["check", ".", "--rule", sentence, "--suggestions"],
    );
    let expected = format!(
        "{TEXT_FINDINGS}docs/fs/FS-demo.md:1: suggestion: FS-demo has 0 security chapters; --rule requires exactly one\n"
    );
    assert_run(&output, 1, &expected, "");
}

#[test]
fn selector_listing_has_exact_text_and_json_rows() {
    let output = run(&fixture(), &["list", ".", "--selector", "FS"]);
    assert_run(
        &output,
        0,
        "FS-demo  docs/fs/FS-demo.md:1  A chapter-scoped obligation\n",
        "",
    );

    let output = run(&fixture(), &["list", ".", "--selector", "FS.requirements"]);
    assert_run(
        &output,
        0,
        "FS-demo.requirements  docs/fs/FS-demo.md:6  Requirements\n",
        "",
    );

    let output = run(
        &fixture(),
        &[
            "list",
            ".",
            "--selector",
            "FS.requirements",
            "--format",
            "json",
        ],
    );
    assert_run(
        &output,
        0,
        "{\"id\":\"FS-demo\",\"section\":\"requirements\",\"kind\":\"FS\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,\"title\":\"Requirements\",\"stub\":false,\"defines\":null,\"refs\":0,\"duplicate\":false}\n",
        "",
    );
}

/// §FS-rules.7.1, §FS-check.3.25: a literal subject that resolves nowhere is
/// `invalid-rule`, named `--rule` for an ad-hoc sentence, in text and JSON.
#[test]
fn unresolved_literal_is_a_post_scan_finding() {
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            "FS-missing must cite at least one GOAL.",
            "--only",
            "invalid-rule",
        ],
    );
    assert_run(
        &output,
        1,
        "error: --rule is not a valid rule: literal subject FS-missing does not resolve\n",
        "",
    );

    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            "FS-missing must cite at least one GOAL.",
            "--only",
            "invalid-rule",
            "--format",
            "json",
        ],
    );
    assert_run(
        &output,
        1,
        "{\"severity\":\"error\",\"path\":null,\"line\":1,\"code\":\"invalid-rule\",\"message\":\"--rule is not a valid rule: literal subject FS-missing does not resolve\",\"sites\":null}\n",
        "",
    );
}

#[test]
fn display_name_matches_presence_rules_and_selector_rows() {
    let sentence = "Each FS must have exactly one Requirements chapter.";
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            sentence,
            "--only",
            "chapter-cardinality",
        ],
    );
    assert_run(&output, 0, "success\n", "");
}

#[test]
fn every_markdown_sibling_heading_closes_the_accepted_chapter() {
    let root = scratch("chapter-sibling-scope");
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Sibling scope\n\n## requirements: Requirements\n\nNo citation here.\n\n## Plain unmarked heading\n\nLater \u{a7}REQ-demo must not leak backward.\n",
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(
        &output,
        1,
        "docs/fs/FS-demo.md:3: error: FS-demo.requirements must cite REQ (RULE-requirements)\n",
        "",
    );
}

/// §FS-rules.1, §FS-check.3.25: the rationale must be non-empty; without one the
/// declaration is `invalid-rule` at its heading and its sentence never runs.
#[test]
fn whitespace_only_rule_rationale_is_invalid_and_never_executes() {
    let root = scratch("empty-rule-rationale");
    write(
        &root,
        "docs/rules/RULE-empty.md",
        "# RULE-empty: Each FS must cite at least one GOAL.\n\n   \n\t\n",
    );
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--only",
            "invalid-rule",
            "--only",
            "missing-citation",
        ],
    );
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains(
        "docs/rules/RULE-empty.md:1: error: RULE-empty is not a valid rule: rule rationale is empty\n"
    ));
    assert!(!stdout.contains("(RULE-empty)"));
}

#[test]
fn semantic_duplicates_collapse_in_both_directions() {
    let root = scratch("deduplication");
    write(
        &root,
        "docs/rules/RULE-a.md",
        "# RULE-a: Each FS must cite at least one GOAL.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/rules/RULE-b.md",
        "# RULE-b: Each FS must cite at least one GOAL.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(
        &root,
        "grund.toml",
        &format!("{config}\n[citations.FS]\nmust = [\"GOAL\"]\n"),
    );
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Missing goal\n\n## requirements: Requirements\n\nThe inherited chapter rule is satisfied by \u{a7}REQ-demo.\n",
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(
        &output,
        1,
        "docs/fs/FS-demo.md:1: error: FS-demo must cite GOAL (citation direction)\n",
        "",
    );

    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(
        &root,
        "grund.toml",
        &config.replace("\n[citations.FS]\nmust = [\"GOAL\"]\n", "\n"),
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(
        &output,
        1,
        "docs/fs/FS-demo.md:1: error: FS-demo must cite GOAL (RULE-a, RULE-b)\n",
        "",
    );
}

/// §FS-rules.3.4, §FS-rules.7.4, §FS-check.3.28: an inbound count off its
/// constraint is `uncited-unit` at the chapter title; a hard prohibition
/// reuses `forbidden-citation` with the rule ID in its message (§FS-rules.7.5).
#[test]
fn inbound_and_prohibition_families_reach_their_released_findings() {
    let root = scratch("remaining-families");
    write(
        &root,
        "docs/rules/RULE-inbound.md",
        "# RULE-inbound: The requirements chapter of each FS must be cited by at least one AR.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/rules/RULE-prohibition.md",
        "# RULE-prohibition: Each FS must not cite any AR.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: All five families\n\n## goals: Goals\n\nThe wrong chapter cites \u{a7}REQ-demo.\n## requirements: Requirements\n\nThis chapter cites prohibited \u{a7}AR-one.\n",
    );
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--only",
            "uncited-unit",
            "--only",
            "forbidden-citation",
        ],
    );
    assert_run(
        &output,
        1,
        concat!(
            "docs/fs/FS-demo.md:6: error: FS-demo.requirements is cited by AR 0 times; ",
            "RULE-inbound requires at least one\n",
            "docs/fs/FS-demo.md:8: error: FS must not cite AR (RULE-prohibition)\n",
        ),
        "",
    );
}

/// §FS-rules.7.5: `should cite` and `should not cite` reuse `suggested-citation`
/// and `discouraged-citation`, each naming its rule.
#[test]
fn positive_and_negative_citation_recommendations_reuse_suggestion_codes() {
    let root = scratch("recommendations");
    write(
        &root,
        "docs/rules/RULE-positive.md",
        "# RULE-positive: Each FS should cite at least one GOAL.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/rules/RULE-negative.md",
        "# RULE-negative: Each FS should not cite any AR.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Suggested directions\n\n## requirements: Requirements\n\nCites \u{a7}REQ-demo and discouraged \u{a7}AR-one.\n",
    );
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--suggestions",
            "--only",
            "suggested-citation",
            "--only",
            "discouraged-citation",
        ],
    );
    assert_run(
        &output,
        0,
        concat!(
            "docs/fs/FS-demo.md:1: suggestion: FS-demo should cite GOAL (RULE-positive)\n",
            "docs/fs/FS-demo.md:5: suggestion: FS should not cite AR (RULE-negative)\n",
        ),
        "",
    );
}

#[test]
fn incomplete_scan_suppresses_absence_and_count_conclusions() {
    let root = scratch("incomplete");
    fs::write(root.join("docs/unreadable.md"), [0xff, 0xfe]).expect("invalid UTF-8 fixture");
    let output = run(&root, &["check", "."]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = text(&output.stdout);
    assert!(!stdout.contains("chapter-cardinality"));
    assert!(!stdout.contains("citation-cardinality"));
    assert!(!stdout.contains("must cite REQ"));
    assert!(text(&output.stderr).contains("docs/unreadable.md"));
}

#[test]
fn invalid_rule_makes_init_a_no_write_operation() {
    let root = scratch("init-no-write");
    write(&root, "AGENTS.md", "sentinel\n");
    write(
        &root,
        "docs/rules/RULE-requirements.md",
        "# RULE-requirements: Each FS may not cite any AR.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    let before = fs::read(root.join("AGENTS.md")).expect("sentinel");
    let root_arg = root.to_string_lossy();
    let output = run(&root, &["init", &root_arg]);
    assert_eq!(output.status.code(), Some(1));
    assert_run(
        &output,
        1,
        "docs/rules/RULE-requirements.md:1: error: RULE-requirements is not a valid rule: modality \"may not\" is not accepted; accepted form: Each FS must not cite any AR.\n",
        "",
    );
    assert_eq!(fs::read(root.join("AGENTS.md")).expect("sentinel"), before);
}

/// §FS-init.2.3.5.10: a rule kind renders the exact ordered `### Chapter rules`
/// section after the directions, and `check` rejects any byte drift in it.
#[test]
fn valid_rules_render_exact_sentences_in_a_v11_managed_section() {
    let root = scratch("init-rendering");
    let root_arg = root.to_string_lossy();
    let output = run(&root, &["init", &root_arg]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("rendered AGENTS.md");
    assert!(agents.contains("Grounding with grund (v11)"));
    let directions = agents.find("### Citation directions").expect("directions");
    let rules = agents.find("### Chapter rules").expect("rules section");
    let clickable = agents[rules..]
        .find("### Clickable citations")
        .map(|offset| rules + offset)
        .expect("section after chapter rules");
    assert!(directions < rules);
    let expected = concat!(
        "### Chapter rules\n\n",
        "`must`/`must not` are `grund check` errors; `should`/`should not` are ",
        "suggestions (`grund check --suggestions`).\n\n",
        "- AR-overview.system-overview must cite each AR exactly once. ",
        "\u{a7}RULE-overview\n",
        "- The requirements chapter of each FS must cite at least one REQ. ",
        "\u{a7}RULE-requirements\n\n",
    );
    assert_eq!(&agents[rules..clickable], expected);

    let drifted = agents.replacen(
        "- AR-overview.system-overview must cite each AR exactly once.",
        "- AR-overview.system-overview must cite each AR exactly once. (edited)",
        1,
    );
    assert_ne!(drifted, agents, "the fixture must drift the rule section");
    write(&root, "AGENTS.md", &drifted);
    let output = run(&root, &["check", ".", "--only", "agents-init"]);
    assert_run(
        &output,
        1,
        concat!(
            "AGENTS.md:3: error: stale grund init block: chapter rules differ from ",
            "grund.toml (run `grund init` to refresh) ",
            "\u{2014} repo maintenance; citation checks still ran; wording changes in grund 0.15.0\n",
        ),
        "",
    );
}

/// §FS-rules.1: the rule title is a grammar island no formatter pass rewrites.
#[test]
fn formatter_treats_the_rule_sentence_as_a_grammar_island() {
    let root = scratch("grammar-island");
    let rule = root.join("docs/rules/RULE-overview.md");
    let first = fs::read_to_string(&rule)
        .expect("rule file")
        .lines()
        .next()
        .expect("rule heading")
        .to_string();
    let rule_arg = rule.to_string_lossy();
    let output = run(&root, &["fmt", &rule_arg, "--marker", "--write"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let after = fs::read_to_string(rule).expect("formatted rule file");
    assert_eq!(after.lines().next(), Some(first.as_str()));
}

/// §FS-rules.1: without a `rules = true` row, check output and the managed
/// block are what they were before the feature.
#[test]
fn a_project_without_rule_opt_in_keeps_the_old_check_bytes() {
    let root = scratch("no-opt-in");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(&root, "grund.toml", &config.replace("rules = true\n", ""));
    let output = run(&root, &["check", "."]);
    assert_run(&output, 0, "success\n", "");

    let root_arg = root.to_string_lossy();
    let output = run(&root, &["init", &root_arg]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("rendered AGENTS.md");
    assert!(agents.contains("Grounding with grund (v10)"));
    assert!(!agents.contains("### Chapter rules"));

    let output = run(&root, &["check", ".", "--only", "agents-init"]);
    assert_run(&output, 0, "success\n", "");
}
