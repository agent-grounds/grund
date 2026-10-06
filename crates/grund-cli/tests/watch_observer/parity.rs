//! Completed stream/status parity, including empty JSON (§FS-check.6.2).
use crate::support::*;
use grund::{WatchObservation, watch_test};
use grund_core::CheckFindingSelection;

#[test]
fn watch_json_clean_dangling_repaired_completion() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, Some("json"), None);
    let clean = h.completed();
    assert_eq!(clean.status, 0);
    assert!(clean.stdout.is_empty() && clean.stderr.is_empty());
    f.write("src/main.rs", BAD);
    let bad = watch_test::one_shot(f.opts(), &CheckFindingSelection::default(), Some("json"));
    assert_ne!(bad.stdout, clean.stdout);
    h.matches(&bad);
    f.write("src/main.rs", CLEAN);
    h.matches(&clean);
    f.write("src/main.rs", BAD);
    h.matches(&bad);
    h.stop(1);
}

#[test]
fn watch_completed_run_option_matrix() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    // §FS-check.6.2: multisite duplicate ordering and run-level stderr routing.
    f.write("docs/functional-spec/FS-dupe.md", "# FS-live: Duplicate\n");
    f.write(".agents/grund.toml", CONFIG);
    f.write("src/ungrounded.rs", "fn ungrounded() {}\n");
    f.write("outside/note.md", "\u{a7}FS-outside-missing\n");
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"src\", \"docs/functional-spec\"]\n"),
    );
    for format in ["text", "json"] {
        for option in [
            "default",
            "only",
            "ignore",
            "suggestions",
            "grounding",
            "full",
            "trial",
            "narrowed",
        ] {
            let mut opts = f.opts();
            let mut selection = CheckFindingSelection::default();
            match option {
                "only" => {
                    selection.add_only("dangling").unwrap();
                }
                "ignore" => {
                    selection.add_ignore("dangling").unwrap();
                }
                "suggestions" => {
                    opts.include_suggestions = true;
                    opts.rule = Some("Each FS should cite at least one GOAL.".into());
                }
                "grounding" => opts.require_grounding = true,
                "full" => opts.full = true,
                "trial" => {
                    opts.rule = Some("Each FS must cite at least one FS.".into());
                    selection.scope_to_trial_rule();
                }
                "narrowed" => {
                    opts.path = f.0.join("src/main.rs");
                    opts.path_provided = true;
                }
                _ => {}
            }
            let expected = watch_test::one_shot(opts.clone(), &selection, Some(format));
            if matches!(option, "trial" | "suggestions") {
                let baseline =
                    watch_test::one_shot(f.opts(), &CheckFindingSelection::default(), Some(format));
                assert_ne!(expected.status, 2, "{option} must exercise a valid rule");
                assert_ne!(expected.stdout, baseline.stdout);
                let output = String::from_utf8_lossy(&expected.stdout);
                let (channel, message) = if option == "trial" {
                    assert_eq!(expected.status, 1);
                    ("error", "FS-live must cite FS (--rule)")
                } else {
                    ("suggestion", "FS-live should cite GOAL (--rule)")
                };
                assert!(output.contains(message), "{option}: {output}");
                if format == "text" {
                    assert!(output.contains(&format!("{channel}: {message}")));
                } else {
                    assert!(output.lines().any(|line| {
                        let row: serde_json::Value = serde_json::from_str(line).unwrap();
                        row["severity"] == channel && row["message"] == message
                    }));
                }
            }
            let mut h = Harness::options(opts, selection, Some(format), None, None);
            h.matches(&expected);
            h.stop(expected.status);
        }
    }
}

#[test]
fn watch_changed_defaults_and_explicit_format() {
    let _serial = crate::support::serial();
    for explicit in [None, Some("text")] {
        let f = Fixture::new();
        f.write("src/main.rs", BAD);
        let mut h = Harness::start(&f, explicit, None);
        let before = h.completed();
        f.write(
            "grund.toml",
            &format!("{CONFIG}\n[output]\nformat = \"json\"\n"),
        );
        let after = watch_test::one_shot(f.opts(), &CheckFindingSelection::default(), explicit);
        if explicit.is_none() {
            assert_ne!(before.stdout, after.stdout);
        }
        h.matches(&after);
        h.stop(after.status);
    }
}

#[test]
fn watch_config_dependent_trial_refusal_repairs() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut opts = f.opts();
    opts.rule = Some("Each FS must cite at least one FS.".into());
    let mut h = Harness::options(
        opts.clone(),
        CheckFindingSelection::default(),
        None,
        None,
        None,
    );
    let initial = h.completed();
    assert_eq!(initial.status, 1);
    let ordinary = watch_test::one_shot(opts.clone(), &CheckFindingSelection::default(), None);
    assert_eq!(initial.stdout, ordinary.stdout);
    assert_eq!(initial.stderr, ordinary.stderr);
    assert_eq!(initial.status, ordinary.status);
    assert!(String::from_utf8_lossy(&initial.stdout).contains("FS-live must cite FS (--rule)"));
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[[kinds]]\nkind = \"CUSTOM\"\nfolder = \"docs\"\n"),
    );
    let refused = watch_test::one_shot(opts.clone(), &CheckFindingSelection::default(), None);
    assert_eq!(refused.status, 2);
    assert!(!refused.stderr.is_empty());
    h.matches(&refused);
    f.write("grund.toml", CONFIG);
    h.matches(&initial);
    h.stop(1);
    assert!(
        h.history
            .iter()
            .any(|event| matches!(event, WatchObservation::Completed(2, _, _)))
    );
}
