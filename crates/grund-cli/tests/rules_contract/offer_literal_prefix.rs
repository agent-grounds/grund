//! Each-prefixed literals keep the longest valid chapter prefix as a selector
//! (§FS-rule-refusals.selectors), and as a rule only where it drops no typed
//! component (§FS-rule-refusals.sentences.rewrite.recovered,
//! §FS-rule-refusals.sentences.rewrite.same-units), using the four-FS scope fixture.

use super::super::support::case_repo;
use super::{Configured, check, check_offer, identities, offer, run, text, write};
use std::fs;
use std::path::Path;

// A refused path, the selector's prefix, and whether a rule is offered it too.
const PATHS: [(&str, &str, bool); 5] = [
    ("requirements.1", "requirements", false),
    ("requirements.*", "requirements", false),
    ("requirements.", "requirements", true),
    (".requirements", "", false),
    ("*", "", false),
];

fn prefixes(root: &Path, separator: &str) {
    for (index, (path, _, _)) in PATHS.iter().enumerate() {
        write(
            root,
            &format!("docs/rules/RULE-prefix-{index}.md"),
            &format!(
                "# RULE-prefix-{index}: Each FS-store{separator}{path} must cite at least one REQ.\n"
            ),
        );
    }
    let configured = run(root, &["check", ".", "--only", "invalid-rule"]);
    assert_eq!(configured.status.code(), Some(1));
    assert!(configured.stderr.is_empty(), "{}", text(&configured.stderr));
    let findings = text(&configured.stdout);
    let mut wrong = Vec::new();
    for (index, (path, prefix, kept)) in PATHS.iter().enumerate() {
        let subject = format!("Each FS-store{separator}{path}");
        let canonical = if prefix.is_empty() {
            "FS-store".to_string()
        } else {
            format!("FS-store{separator}{prefix}")
        };
        let reason = format!(
            "unknown kind \"{}\"",
            subject.strip_prefix("Each ").unwrap()
        );
        for predicate in [
            "must cite at least one REQ.",
            "must be cited by at least one REQ.",
        ] {
            let sentence = format!("{subject} {predicate}");
            let expected = format!("{canonical} {predicate}");
            let offered = kept.then_some(expected.as_str());
            let refused = check(root, &sentence);
            assert_eq!(refused.status.code(), Some(2), "{sentence}");
            assert!(refused.stdout.is_empty());
            let message = text(&refused.stderr);
            if *kept {
                assert!(message.starts_with(&format!("error: {reason};")));
            } else {
                assert_eq!(message, format!("error: {reason}\n"), "{sentence}");
            }
            assert_eq!(
                offer(&message, "; accepted form: ").as_deref(),
                offered,
                "{sentence}: {message}"
            );
            check_offer(root, &message, offered, &sentence, &mut wrong);
            let control = check(root, &expected);
            if predicate == "must cite at least one REQ." {
                assert_eq!(control.status.code(), Some(0));
                assert_eq!(text(&control.stdout), "success\n");
                let finding_prefix = format!("RULE-prefix-{index} is not a valid rule: ");
                let finding = findings
                    .lines()
                    .find_map(|line| line.split_once(&finding_prefix).map(|(_, tail)| tail))
                    .expect("configured invalid-rule finding");
                assert_eq!(message, format!("error: {finding}\n"));
                check_offer(root, finding, offered, &sentence, &mut wrong);
            } else {
                assert_eq!(control.status.code(), Some(1));
                assert_eq!(identities(&control), [canonical.as_str()]);
            }
        }
        let refused = run(root, &["list", ".", "--selector", &subject]);
        assert_eq!(refused.status.code(), Some(2));
        assert!(refused.stdout.is_empty());
        let message = text(&refused.stderr);
        assert!(message.starts_with(&format!("error: {reason};")));
        let offered = offer(&message, "; accepted selector: ").expect("safe literal prefix");
        assert_eq!(offered, canonical, "{subject}");
        let pasted = run(root, &["list", ".", "--selector", &offered]);
        let control = run(root, &["list", ".", "--selector", &canonical]);
        assert_eq!(control.status.code(), Some(0));
        let selected: Vec<_> = text(&control.stdout)
            .lines()
            .map(|line| line.split_whitespace().next().unwrap().to_string())
            .collect();
        assert_eq!(selected, [canonical.as_str()]);
        assert_eq!(pasted.status.code(), control.status.code(), "{subject}");
        assert_eq!(pasted.stdout, control.stdout, "{subject}");
        assert_eq!(pasted.stderr, control.stderr, "{subject}");
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

#[test]
fn each_literal_refused_paths_preserve_scope_on_all_three_surfaces() {
    let root = Configured::new();
    prefixes(&root.0, ".");
}

#[test]
fn each_literal_refused_paths_preserve_scope_with_configured_separator() {
    let root = Configured::new();
    let config = fs::read_to_string(root.0.join("grund.toml")).unwrap();
    write(
        &root.0,
        "grund.toml",
        &format!("{config}\n[schema]\nsection_separator = \"#\"\n"),
    );
    prefixes(&root.0, "#");
}

/// §FS-rule-refusals.sentences.named-sections: suppress the alternative `Each`
/// chapter offers while preserving existing canonical after-enabling retries.
#[test]
fn each_literal_refused_paths_suppress_unsafe_named_off_offers() {
    let root = case_repo("check-rule-named-off-rebuilt-subject");
    for path in PATHS
        .iter()
        .map(|(path, _, _)| *path)
        .chain(["requirements"])
    {
        let subject = format!("Each FS-login.{path}");
        let reason = format!("error: unknown kind \"FS-login.{path}\"");
        for (output, label) in [
            (
                check(&root, &format!("{subject} must cite at least one REQ.")),
                "accepted form",
            ),
            (
                run(&root, &["list", "--selector", &subject]),
                "accepted selector",
            ),
        ] {
            assert_eq!(output.status.code(), Some(2));
            let message = text(&output.stderr);
            assert!(message.contains(&reason), "{message}");
            assert!(!message.contains(label), "{message}");
        }
    }
    for (subject, rule, selector) in [
        (
            "FS-login.requirements",
            "FS-login.requirements",
            "FS-login.requirements",
        ),
        (
            "FS-*.requirements",
            "The requirements chapter of each FS",
            "FS.requirements",
        ),
    ] {
        let refused = check(&root, &format!("{subject} must cite at least one REQ."));
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(
            offer(
                &text(&refused.stderr),
                "; accepted form after enabling it: "
            ),
            Some(format!("{rule} must cite at least one REQ."))
        );
        let refused = run(&root, &["list", "--selector", subject]);
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(
            offer(
                &text(&refused.stderr),
                "; accepted selector after enabling it: "
            )
            .as_deref(),
            Some(selector)
        );
    }
}
