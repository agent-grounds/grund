//! Executable scope contract for §FS-rule-refusals.sentences.rewrite.recovered
//! and §FS-rule-refusals.selectors. The four-spec fixture distinguishes a
//! chapter citation from a citation elsewhere and from an absent chapter.

use super::support::{assert_run, case_repo, run, text, write};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

#[path = "offer_literal_prefix.rs"]
mod literal_prefix;

const CHAPTER: &str = "The requirements chapter of each FS";
const QUANTIFIED: &str = "chapter-quantified subjects are not accepted in phase 1";

fn repo() -> PathBuf {
    case_repo("check-rule-offer-scope-controls")
}

// A refused subject, its canonical meaning if recoverable, and its unchanged reason.
fn rows() -> Vec<(&'static str, Option<&'static str>, &'static str)> {
    vec![
        (
            "Each FS requirements chapter",
            Some(CHAPTER),
            "unknown kind \"FS requirements chapter\"",
        ),
        (
            "Each FS.requirements",
            Some(CHAPTER),
            "unknown kind \"FS.requirements\"",
        ),
        (
            "Each FS requirements",
            Some(CHAPTER),
            "unknown kind \"FS requirements\"",
        ),
        (
            "FS-store requirements chapter",
            Some("FS-store.requirements"),
            "literal subject \"FS-store requirements chapter\" does not match the configured ID grammar",
        ),
        (
            "Each FS-store.requirements",
            Some("FS-store.requirements"),
            "unknown kind \"FS-store.requirements\"",
        ),
        (
            "Each FS-store",
            Some("FS-store"),
            "unknown kind \"FS-store\"",
        ),
        ("Each chapter of each FS", None, QUANTIFIED),
    ]
}

fn check(root: &Path, sentence: &str) -> Output {
    run(root, &["check", ".", "--rule", sentence, "--only-rule"])
}

fn identities(output: &Output) -> Vec<String> {
    text(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once(": error: "))
        .map(|(_, message)| message.split_whitespace().next().unwrap().to_owned())
        .collect()
}

fn offer(message: &str, label: &str) -> Option<String> {
    message
        .split_once(label)
        .map(|(_, tail)| tail.lines().next().unwrap().to_owned())
}

/// Extract the actual offer, paste it back, and compare identities as well as
/// the predicate's complete canonical spelling. Safe suppression is permitted;
/// warning about an offer that changes scope is not.
fn check_offer(
    root: &Path,
    message: &str,
    canonical: Option<&str>,
    context: &str,
    wrong: &mut Vec<String>,
) {
    if let Some(offered) = offer(message, "; accepted form: ") {
        let pasted = check(root, &offered);
        if let Some(canonical) = canonical {
            let control = check(root, canonical);
            if offered != canonical
                || pasted.status.code() != control.status.code()
                || identities(&pasted) != identities(&control)
                || pasted.stdout != control.stdout
                || pasted.stderr != control.stderr
            {
                wrong.push(format!(
                    "{context}\noffer: {offered}\ncanonical: {canonical}\n\
                     pasted identities: {:?}\ncanonical identities: {:?}\n\
                     pasted stdout:\n{}canonical stdout:\n{}",
                    identities(&pasted),
                    identities(&control),
                    text(&pasted.stdout),
                    text(&control.stdout),
                ));
            }
        } else {
            wrong.push(format!(
                "{context}\nunsafe offer without a named chapter: {offered}"
            ));
        }
    } else if message.contains("accepted form") {
        wrong.push(format!("{context}\nunexpected offer label: {message}"));
    }
}

#[test]
fn canonical_chapter_and_literal_controls_keep_their_verdicts() {
    let root = repo();
    let chapter = check(&root, &format!("{CHAPTER} must cite at least one REQ."));
    assert_run(
        &chapter,
        1,
        include_str!("../../../../tests/e2e/cases/check-rule-offer-scope-controls/expected.stdout"),
        "",
    );
    assert_eq!(
        identities(&chapter),
        [
            "FS-rotate.requirements",
            "FS-search.requirements",
            "FS-audit"
        ]
    );
    for literal in ["FS-store", "FS-store.requirements"] {
        assert_run(
            &check(&root, &format!("{literal} must cite at least one REQ.")),
            0,
            "success\n",
            "",
        );
    }
    assert_eq!(
        identities(&check(&root, "Each FS must cite at least one REQ.")),
        ["FS-rotate"]
    );
    let inbound = check(
        &root,
        &format!("{CHAPTER} must be cited by at least one REQ."),
    );
    assert_eq!(inbound.status.code(), Some(1));
    assert_eq!(
        identities(&inbound),
        [
            "FS-store.requirements",
            "FS-rotate.requirements",
            "FS-search.requirements",
            "FS-audit"
        ]
    );
}

#[test]
fn rule_offers_keep_typed_chapter_literal_and_predicate_scope() {
    let root = repo();
    let mut wrong = Vec::new();
    for (subject, canonical, reason) in rows() {
        for predicate in [
            "must cite at least one REQ.",
            "must be cited by at least one REQ.",
            "should cite at most 2 REQ.",
        ] {
            let sentence = format!("{subject} {predicate}");
            let output = check(&root, &sentence);
            let message = text(&output.stderr);
            assert_eq!(output.status.code(), Some(2), "{sentence}: {message}");
            assert!(output.stdout.is_empty(), "{sentence}");
            assert!(
                message.starts_with(&format!("error: {reason}")),
                "{sentence}: {message}"
            );
            let canonical = canonical.map(|subject| format!("{subject} {predicate}"));
            check_offer(&root, &message, canonical.as_deref(), &sentence, &mut wrong);
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

struct Configured(PathBuf);

impl Configured {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1001-configured-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for file in [
            "grund.toml",
            "requirements.md",
            "docs/requirements/REQ-no-plaintext.md",
            "docs/requirements/REQ-audit-trail.md",
        ] {
            write(&root, file, &fs::read_to_string(repo().join(file)).unwrap());
        }
        let config = fs::read_to_string(root.join("grund.toml")).unwrap();
        write(
            &root,
            "grund.toml",
            &format!(
                "{config}\n[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nform = \"rule\"\n"
            ),
        );
        Self(root)
    }
}

impl Drop for Configured {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove isolated configured fixture");
    }
}

#[test]
fn configured_invalid_rules_offer_the_same_scope_as_check_rule() {
    let root = Configured::new();
    for (index, (subject, _, _)) in rows().iter().enumerate() {
        write(
            &root.0,
            &format!("docs/rules/RULE-scope-{index}.md"),
            &format!("# RULE-scope-{index}: {subject} must cite at least one REQ.\n"),
        );
    }
    let output = run(&root.0, &["check", ".", "--only", "invalid-rule"]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    let mut wrong = Vec::new();
    for (index, (subject, canonical, reason)) in rows().iter().enumerate() {
        let prefix = format!("RULE-scope-{index} is not a valid rule: ");
        let finding = stdout
            .lines()
            .find_map(|line| line.split_once(&prefix).map(|(_, message)| message))
            .expect("configured invalid-rule finding");
        assert!(finding.starts_with(reason), "{finding}");
        let sentence = format!("{subject} must cite at least one REQ.");
        let ad_hoc = check(&root.0, &sentence);
        assert_eq!(ad_hoc.status.code(), Some(2));
        assert_eq!(
            text(&ad_hoc.stderr).lines().next().unwrap(),
            format!("error: {finding}"),
            "both refusal surfaces share a reason and offer"
        );
        let canonical = canonical.map(|subject| format!("{subject} must cite at least one REQ."));
        check_offer(
            &root.0,
            finding,
            canonical.as_deref(),
            &sentence,
            &mut wrong,
        );
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

#[test]
fn selector_offers_keep_typed_chapter_and_literal_scope() {
    let root = repo();
    let mut wrong = Vec::new();
    for (subject, canonical, reason) in rows() {
        let output = run(&root, &["list", ".", "--selector", subject]);
        let message = text(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{subject}: {message}");
        assert!(output.stdout.is_empty());
        assert!(
            message.starts_with(&format!("error: {reason}")),
            "{message}"
        );
        if let Some(offered) = offer(&message, "; accepted selector: ") {
            let pasted = run(&root, &["list", ".", "--selector", &offered]);
            if let Some(canonical) = canonical {
                let control = run(&root, &["list", ".", "--selector", canonical]);
                assert_eq!(control.status.code(), Some(0));
                assert!(!control.stdout.is_empty());
                if pasted.status.code() != Some(0) || pasted.stdout != control.stdout {
                    wrong.push(format!(
                        "{subject}\noffer: {offered}\ncanonical: {canonical}\n\
                         pasted selection:\n{}canonical selection:\n{}",
                        text(&pasted.stdout),
                        text(&control.stdout)
                    ));
                }
            } else {
                wrong.push(format!("{subject}\nunsafe selector offer: {offered}"));
            }
        } else if message.contains("accepted selector") {
            wrong.push(format!("{subject}\nunexpected offer label: {message}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

/// §FS-rule-refusals.sentences.rewrite.same-units: a rule that would lose the typed
/// component is offered nothing, while the selector keeps its longest valid prefix.
#[test]
fn refused_components_keep_their_valid_prefix_only_as_a_selector() {
    assert_run(
        &check(
            &repo(),
            "FS-store.requirements.1 must cite at least one REQ.",
        ),
        2,
        "",
        "error: numbered chapter subjects can detach when headings move\n",
    );
    let output = run(&repo(), &["list", ".", "--selector", "FS.requirements.1"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("; accepted selector: FS.requirements\n"));
}

/// §FS-rule-refusals.sentences.rewrite.same-units: a refused path component in an
/// `Each` head is offered nothing as a rule, and no known kinds, since the kind was
/// recovered; §FS-rule-refusals.selectors keeps the prefix repair for a selector.
#[test]
fn each_refused_components_keep_their_valid_prefix_only_as_a_selector() {
    let root = repo();
    for (subject, canonical) in [
        ("Each FS.*", "Each FS"),
        ("Each FS.requirements.*", CHAPTER),
        ("Each FS.requirements.1", CHAPTER),
    ] {
        let sentence = format!("{subject} must cite at least one REQ.");
        assert_run(
            &check(&root, &sentence),
            2,
            "",
            &format!(
                "error: unknown kind \"{}\"\n",
                subject.strip_prefix("Each ").unwrap()
            ),
        );

        let refused = run(&root, &["list", ".", "--selector", subject]);
        assert_eq!(refused.status.code(), Some(2));
        let message = text(&refused.stderr);
        let offered = offer(&message, "; accepted selector: ").expect("safe prefix offer");
        assert_eq!(offered, canonical);
        let pasted = run(&root, &["list", ".", "--selector", &offered]);
        let control = run(&root, &["list", ".", "--selector", canonical]);
        assert_eq!(control.status.code(), Some(0));
        assert!(!control.stdout.is_empty());
        assert_eq!(pasted.status.code(), control.status.code());
        assert_eq!(pasted.stdout, control.stdout, "{subject}");
        assert_eq!(pasted.stderr, control.stderr, "{subject}");
    }
}
