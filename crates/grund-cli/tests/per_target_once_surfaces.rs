//! §FS-rules.9, §FS-init.2.3.5.10 and §REQ-one-answer.text.parity:
//! init and editor checks read the same per-target count grammar as check.

#[path = "support/per_target_once.rs"]
mod support;

use std::fs;
use support::{Repo, assert_clean, check, text};

const TITLE: &str = "SYS-overview.components must cite each AR at most once.";
const NUMERIC: &str = "SYS-overview.components must cite each AR at most 1 times.";
const REASON: &str = "numeric \"at most 1 times\" is not canonical; accepted form: SYS-overview.components must cite each AR at most once.";

/// §FS-init.2.3.5.10, §FS-rules.9.1: render the authored sentence exactly,
/// then check the same tree for agents-init drift and compare init's bytes.
#[test]
#[ignore = "red until #835 accepts once in init; implement removes this"]
fn per_target_once_init_renders_the_exact_title_without_drift() {
    let repo = Repo::new(1);
    repo.rule(TITLE);
    let output = repo.run(&["init"]);
    assert!(output.status.success(), "{output:?}");
    let agents = fs::read_to_string(repo.0.join("AGENTS.md")).unwrap();
    let start = agents
        .find("### Chapter rules\n")
        .expect("chapter rules section");
    let end = agents[start..].find("### Clickable citations").unwrap() + start;
    assert_eq!(
        &agents[start..end],
        concat!(
            "### Chapter rules\n\n",
            "`must`/`must not` are `grund check` errors; `should`/`should not` are ",
            "suggestions (`grund check --suggestions`).\n\n",
            "- SYS-overview.components must cite each AR at most once. ",
            "[\u{a7}RULE-once](docs/rules/once.md",
            "#rule-once-sys-overviewcomponents-must-cite-each-ar-at-most-once)\n\n"
        )
    );
    assert_clean(
        check(repo.run(&["check"]), 0, "success\n", "", "check after init")
            .into_iter()
            .collect(),
    );
    assert!(repo.run(&["init", "--check"]).status.success());
    assert_eq!(
        fs::read_to_string(repo.0.join("AGENTS.md")).unwrap(),
        agents
    );
}

/// §FS-rules.4, §FS-rules.7.1: invalid numeric one stops init before any
/// write, both with existing guidance/companions and on a fresh tree.
#[test]
#[ignore = "red until #835 refuses numeric upper-one before init writes; implement removes this"]
fn per_target_once_numeric_rule_refuses_init_before_writes() {
    let mut wrong = Vec::new();
    for existing in [false, true] {
        let repo = Repo::new(0);
        repo.rule(NUMERIC);
        if existing {
            for path in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"] {
                repo.write(path, &format!("sentinel {path}\n"));
            }
        }
        let paths = [
            "grund.toml",
            "docs/rules/once.md",
            "docs/sys.md",
            "AGENTS.md",
            "CLAUDE.md",
            "GEMINI.md",
        ];
        let before: Vec<_> = paths
            .iter()
            .map(|path| fs::read(repo.0.join(path)).ok())
            .collect();
        let expected =
            format!("docs/rules/once.md:1: error: RULE-once is not a valid rule: {REASON}\n");
        if let Some(error) = check(
            repo.run(&["init"]),
            1,
            &expected,
            "",
            &format!("init existing={existing}"),
        ) {
            wrong.push(error);
        }
        for (path, bytes) in paths.iter().zip(before) {
            if fs::read(repo.0.join(path)).ok() != bytes {
                wrong.push(format!("init existing={existing} changed {path}"));
            }
        }
        for path in ["docs/README.md", ".grund/sections.lock"] {
            if repo.0.join(path).exists() {
                wrong.push(format!("init existing={existing} created {path}"));
            }
        }
    }
    assert_clean(wrong);
}

/// §REQ-one-answer.text.parity, §FS-rules.7.1: an editor invalid-rule has
/// the exact CLI reason and rule-heading location, even for a should rule.
#[test]
#[ignore = "red until #835 exposes numeric upper-one as editor invalid-rule; implement removes this"]
fn per_target_once_editor_invalid_rule_agrees_with_cli() {
    let repo = Repo::new(0);
    let title = NUMERIC.replace(" must ", " should ");
    repo.rule(&title);
    let cli = repo.run(&["check", "--format", "json"]);
    let snapshot = grund_core::lsp_snapshot(grund_core::LspSnapshotOpts {
        path: repo.0.clone(),
        path_provided: true,
        open_documents: Default::default(),
    })
    .unwrap();
    assert_eq!(
        (cli.status.code(), snapshot.report.errors.len()),
        (Some(1), 1),
        "CLI {cli:?}; editor {:?}",
        snapshot.report
    );
    assert!(cli.stderr.is_empty(), "{cli:?}");
    let row: serde_json::Value = serde_json::from_str(&text(&cli.stdout)).unwrap();
    assert!(snapshot.report.warnings.is_empty());
    assert!(snapshot.report.suggestions.is_empty());
    let diagnostic = &snapshot.report.errors[0];
    let expected = format!(
        "RULE-once is not a valid rule: {}",
        REASON.replace(" must ", " should ")
    );
    assert_eq!(diagnostic.code, "invalid-rule");
    assert_eq!(diagnostic.severity, "error");
    assert_eq!(diagnostic.message, expected);
    assert_eq!(row["message"], diagnostic.message);
    assert_eq!(row["code"], diagnostic.code);
    assert_eq!(row["severity"], diagnostic.severity);
    assert_eq!(row["line"], 1);
    assert_eq!(diagnostic.line, Some(1));
    assert_eq!(row["path"], "docs/rules/once.md");
    assert!(
        diagnostic
            .path
            .as_ref()
            .unwrap()
            .ends_with("docs/rules/once.md")
    );
}
