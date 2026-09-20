//! Independent form, ownership, syntax, and target-existence findings for local
//! numeric section citations (§FS-check.3.2, §FS-check.3.24, §AR-checker).

use super::*;
use crate::config::load_config;
use crate::scanner::scan_tree;
use crate::testing::{numbered_config, test_root, write};

#[test]
fn local_section_findings_are_actionable_and_missing_is_independent() {
    let root = test_root("local_section_findings_are_actionable_and_missing_is_independent");
    write(
        &root.join("docs/functional-spec/FS-001-alpha.md"),
        concat!(
            "# FS-001-alpha: Alpha\n\n",
            "Valid \u{a7}2.\n",
            "Missing \u{a7}9.9.\n",
            "Unsupported \u{a7}2.goals and \u{a7}2abc.\n\n",
            "## 2. Target\n",
        ),
    );
    write(
        &root.join("docs/notes.md"),
        "# Notes\n\nOwnerless \u{a7}2.\n",
    );
    let config = numbered_config(root.clone());
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan fixture");
    assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
    let report = check_findings(&findings, &config);
    let mut actual = report
        .errors
        .iter()
        .map(|finding| (finding.code, finding.message.as_str()))
        .collect::<Vec<_>>();
    actual.sort_unstable_by_key(|(_, message)| *message);
    let mut expected = vec![
        (
            "local-section-citation",
            "local section citation \u{a7}2; write \u{a7}FS-001-alpha.2",
        ),
        (
            "local-section-citation",
            "local section citation \u{a7}9.9; write \u{a7}FS-001-alpha.9.9",
        ),
        ("missing-section", "missing section FS-001-alpha.9.9"),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2.goals; write a full citation or <§>2.goals to show the shape without citing it",
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2abc; write a full citation or <§>2abc to show the shape without citing it",
        ),
        (
            "local-section-citation",
            "local section citation \u{a7}2 has no enclosing declaration; write a full citation or <§>2 to show the shape without citing it",
        ),
    ];
    expected.sort_unstable_by_key(|(_, message)| *message);

    assert_eq!(actual, expected);
    assert!(
        !report
            .warnings
            .iter()
            .any(|finding| finding.code == "unused"),
        "owned local edges count as uses: {:?}",
        report
            .warnings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn owned_local_edges_feed_grounding_unused_and_citation_direction_checks() {
    let root = test_root("owned_local_edges_feed_grounding_unused_and_citation_direction_checks");
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "[reference]\nstrict = true\nrequire_grounding = true\n",
            "[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
            "[citations]\ndefault = \"may\"\n",
            "[citations.FS]\nmust-not = [\"FS\"]\n",
        ),
    );
    write(
        &root.join("docs/FS-alpha.md"),
        "# FS-alpha: Alpha\n\nLocal \u{a7}2.\n\n## 2. Target\n",
    );
    let config = load_config(&root).expect("load direction config");
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan fixture");
    assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
    let report = check_findings(&findings, &config);
    let messages = report
        .errors
        .iter()
        .chain(&report.warnings)
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("must not cite FS")),
        "the local edge reaches citation directions: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("ungrounded")
                && !message.contains("declared but never cited")),
        "the same edge grounds and counts as use: {messages:?}"
    );
}
