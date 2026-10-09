//! A chapter subject refused for its chapter path names the component that
//! failed, whichever spelling reached it, and keeps every byte after that
//! reason (§FS-rules.3.5.3). The fixture is the `check-rules-chapter-path-reasons`
//! e2e case's: kinds `FS`, `REQ` and `RULE`, named sections on, `FS-login`
//! holding a named `requirements` chapter that cites `REQ-password`, and one
//! `rules = true` declaration per sentence below.

use super::support::{assert_run, case_repo, fixture, run, scratch_from, text};
use std::fs;
use std::path::{Path, PathBuf};

/// The accepted form every row of §FS-rules.3.5.3's table ends in.
const TAIL: &str = "; accepted form: FS-login.requirements must cite at least one REQ.";

fn repo() -> PathBuf {
    case_repo("check-rules-chapter-path-reasons")
}

/// The fixture with its configured rules off, so a `--rule` run reports that
/// sentence and nothing else.
fn unconfigured() -> PathBuf {
    let root = scratch_from(&repo(), "chapter-path-reasons-unconfigured");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("rules = true\n", ""),
    )
    .expect("disable configured rules");
    root
}

/// §FS-rules.3.5.3: every row of the exact table, as the fixture's rule
/// declaration that writes it, its subject, and the reason and accepted form
/// both rule surfaces print.
fn rows() -> Vec<(&'static str, &'static str, String)> {
    let grammar = |subject: &str| {
        format!(
            "named chapter subject \"{subject}\" does not match the configured section grammar{TAIL}"
        )
    };
    vec![
        (
            "RULE-numbered",
            "The requirements.1 chapter of each FS",
            format!("numbered chapter subjects can detach when headings move{TAIL}"),
        ),
        (
            "RULE-wildcard",
            "The requirements.* chapter of each FS",
            format!("section-component wildcards are not accepted in phase 1{TAIL}"),
        ),
        (
            "RULE-trailing",
            "FS-login.requirements.",
            grammar("FS-login.requirements."),
        ),
        (
            "RULE-doubled",
            "FS-login..requirements",
            grammar("FS-login..requirements"),
        ),
    ]
}

/// Every row whose `check --rule` refusal in `root` is not exactly its own, all
/// run before the caller decides, so one wrong row never hides another.
fn wrong_check_rule_rows(root: &Path, rows: &[(&str, &str, String)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (_, subject, refusal) in rows {
        let sentence = format!("{subject} must cite at least one REQ.");
        let output = run(root, &["check", ".", "--rule", &sentence]);
        let expected = format!("error: {refusal}\n");
        let (exit, stdout, stderr) = (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        );
        if exit != Some(2) || !stdout.is_empty() || stderr != expected {
            wrong.push(format!(
                "--rule {sentence:?}: exit {exit:?}, stdout {stdout:?}\n  stderr   {stderr:?}\n  expected {expected:?}"
            ));
        }
    }
    wrong
}

/// §FS-rules.3.5.3: `check --rule` refuses each sentence for the component of
/// its chapter path that failed: `requirements.1` as numbered and
/// `requirements.*` for its wildcard in `The PATH chapter of each KIND`, as the
/// literal spelling already is, and an empty component off the section grammar
/// rather than as numbered.
#[test]
fn check_rule_refuses_a_chapter_path_for_the_component_that_failed() {
    let rows = rows();
    let wrong = wrong_check_rule_rows(&unconfigured(), &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} chapter paths were not refused for the component that failed:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.3, §FS-rules.7.1: a configured rule declaration's
/// `invalid-rule` finding carries the same reason and accepted form after
/// `is not a valid rule: `.
#[test]
fn the_invalid_rule_finding_names_the_component_that_failed() {
    let output = run(&repo(), &["check", "."]);
    assert_eq!(output.status.code(), Some(1), "unexpected exit");
    let stdout = text(&output.stdout);
    let wrong = rows()
        .into_iter()
        .filter_map(|(id, _, refusal)| {
            let expected =
                format!("docs/rules/{id}.md:1: error: {id} is not a valid rule: {refusal}");
            let actual = stdout
                .lines()
                .find(|line| line.contains(&format!(" {id} is not a valid rule: ")))
                .unwrap_or("no invalid-rule finding");
            (actual != expected)
                .then(|| format!("{id}\n  finding  {actual:?}\n  expected {expected:?}"))
        })
        .collect::<Vec<_>>();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Guard, green before and after §FS-rules.3.5.3: the unnumbered spellings of
/// the same chapter are accepted on both rule surfaces, so the rows above are
/// refused for their paths and not for the fixture.
#[test]
fn the_unnumbered_spellings_of_the_same_chapter_are_accepted() {
    for sentence in [
        "The requirements chapter of each FS must cite at least one REQ.",
        "FS-login.requirements must cite at least one REQ.",
    ] {
        let output = run(&unconfigured(), &["check", ".", "--rule", sentence]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{sentence:?} was not accepted: {}",
            text(&output.stderr)
        );
    }
    let stdout = text(&run(&repo(), &["check", "."]).stdout);
    for id in ["RULE-valid", "RULE-exact"] {
        assert!(
            !stdout.contains(&format!(" {id} is not a valid rule")),
            "{id} was refused:\n{stdout}"
        );
    }
}

/// §FS-rules.3.5.3: only the reason follows the component. A literal subject
/// refused for an empty component is offered the accepted form its typed
/// declaration gives a numbered literal, here `FS-demo`, not `FS-login`.
#[test]
fn an_empty_component_keeps_the_accepted_form_its_declaration_gives() {
    for (subject, numbered) in [
        ("FS-demo.requirements.", false),
        ("FS-demo..requirements", false),
        ("FS-demo.2", true),
    ] {
        let sentence = format!("{subject} must cite at least one REQ.");
        let reason = if numbered {
            "numbered chapter subjects can detach when headings move".to_string()
        } else {
            format!(
                "named chapter subject \"{subject}\" does not match the configured section grammar"
            )
        };
        let output = run(&fixture(), &["check", ".", "--rule", &sentence]);
        assert_run(
            &output,
            2,
            "",
            &format!(
                "error: {reason}; accepted form: FS-demo.requirements must cite at least one REQ.\n"
            ),
        );
    }
}
