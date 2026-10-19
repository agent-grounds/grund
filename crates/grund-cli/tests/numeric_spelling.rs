//! End-to-end regression for #819: source spelling is presentation, numeric
//! identity and allocation remain numeric (§FS-config.3.2.8, §FS-id.1.2).

#[path = "support/numeric_spelling.rs"]
mod support;

use support::{FIRST, NOTES, Repo, SECOND, contains, equal};

/// §FS-config.3.2.8: text and JSON retain declaration/citation source digits.
fn reports(version: u8, per_kind: bool) {
    let repo = Repo::new(version, per_kind);
    let mut failures = Vec::new();
    let list = repo.run(&["list"], 0);
    for (id, path) in [("ADR-0001", "ADR-0001.md"), ("ADR-0002", "ADR-0002.md")] {
        contains(
            &mut failures,
            &list,
            &format!("{id}  docs/adr/{path}:1"),
            "list text",
        );
    }
    let records = repo.json(&["list", "--format", "json"], 0);
    assert_eq!(records.len(), 2, "fixture has exactly two declarations");
    for (record, expected) in records.iter().zip(["ADR-0001", "ADR-0002"]) {
        equal(&mut failures, &record["id"], expected, "list JSON id");
    }
    for query in ["ADR-0001", "ADR-1", "ADR-001"] {
        let shown = repo.json(&["show", query, "--format", "json"], 0);
        equal(&mut failures, &shown[0]["id"], "ADR-0001", "show JSON id");
        equal(
            &mut failures,
            &shown[0]["anchor"],
            "adr-0001-use-postgres",
            "show anchor",
        );
        equal(
            &mut failures,
            &shown[0]["path"],
            "docs/adr/ADR-0001.md",
            "show path",
        );
    }
    let refs = repo.json(&["refs", "ADR-1", "--format", "json"], 0);
    let original = refs
        .iter()
        .find(|record| record["text"] == "\u{a7}ADR-0001")
        .unwrap();
    equal(&mut failures, &original["id"], "ADR-0001", "refs JSON id");
    assert!(
        failures.is_empty(),
        "v{version}, per-kind={per_kind}:\n{}",
        failures.join("\n")
    );
}

/// §FS-config.3.2.8: the missing token's digits and the suggestion's digits have
/// different sources; neither may be reconstructed at a report width.
fn diagnostics(version: u8, per_kind: bool) {
    let repo = Repo::new(version, per_kind);
    let expected = "unknown reference ADR-0007; did you mean ADR-0001?";
    let mut failures = Vec::new();
    let text = repo.run(&["check", "--only", "dangling"], 1);
    contains(&mut failures, &text, expected, "check text");
    let records = repo.json(&["check", "--only", "dangling", "--format", "json"], 1);
    assert_eq!(records.len(), 1, "only the unknown citation is dangling");
    assert_eq!(records[0]["code"], "dangling");
    equal(
        &mut failures,
        &records[0]["message"],
        expected,
        "check JSON message",
    );
    assert!(
        failures.is_empty(),
        "v{version}, per-kind={per_kind}:\n{}",
        failures.join("\n")
    );
}

/// §FS-fmt.6.2.2, §FS-fmt.6.3: target heading spelling wins over equivalent
/// citation spellings, section fragments and visible text survive, and the
/// second pass is stable. Source headings must not be repadded to make it pass.
fn links(version: u8, per_kind: bool) {
    let repo = Repo::new(version, per_kind);
    repo.run(&["fmt", "--write"], 0);
    let once = repo.read("docs/notes.md");
    let mut failures = Vec::new();
    for (text, file, anchor) in [
        ("ADR-0001", "ADR-0001", "adr-0001-use-postgres"),
        ("ADR-0001.1", "ADR-0001", "1-why"),
        ("ADR-0002", "ADR-0002", "adr-0002-use-redis"),
        ("ADR-1", "ADR-0001", "adr-0001-use-postgres"),
        ("ADR-002", "ADR-0002", "adr-0002-use-redis"),
    ] {
        contains(
            &mut failures,
            &once,
            &format!("[\u{a7}{text}](adr/{file}.md#{anchor})"),
            "fmt link",
        );
    }
    contains(
        &mut failures,
        &once,
        "not \u{a7}ADR-0007.",
        "unresolved citation text",
    );
    assert_eq!(repo.read("docs/adr/ADR-0001.md"), FIRST);
    assert_eq!(repo.read("docs/adr/ADR-0002.md"), SECOND);
    repo.run(&["fmt", "--check"], 0);
    repo.run(&["fmt", "--write"], 0);
    assert_eq!(
        repo.read("docs/notes.md"),
        once,
        "second fmt pass is stable"
    );
    assert!(
        failures.is_empty(),
        "v{version}, per-kind={per_kind}:\n{}",
        failures.join("\n")
    );
}

macro_rules! regressions {
    ($reports:ident, $diagnostics:ident, $links:ident, $version:literal, $per_kind:literal) => {
        #[test]
        #[ignore = "red until #819 retains numeric source spelling; implement removes this"]
        fn $reports() {
            reports($version, $per_kind);
        }
        #[test]
        #[ignore = "red until #819 retains diagnostic source spelling; implement removes this"]
        fn $diagnostics() {
            diagnostics($version, $per_kind);
        }
        #[test]
        #[ignore = "red until #819 derives anchors from target headings; implement removes this"]
        fn $links() {
            links($version, $per_kind);
        }
    };
}

regressions!(
    v2_default_reports,
    v2_default_diagnostics,
    v2_default_links,
    2,
    false
);
regressions!(
    v2_per_kind_reports,
    v2_per_kind_diagnostics,
    v2_per_kind_links,
    2,
    true
);
regressions!(
    v1_default_reports,
    v1_default_diagnostics,
    v1_default_links,
    1,
    false
);
regressions!(
    v1_per_kind_reports,
    v1_per_kind_diagnostics,
    v1_per_kind_links,
    1,
    true
);

/// §FS-config.3.2.8: no single report width can recover a mixed-width tree.
#[test]
#[ignore = "red until #819 retains each declaration's source digits; implement removes this"]
fn mixed_width_declarations_are_not_reconstructed_at_any_fixed_width() {
    let mut failures = Vec::new();
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            repo.write("docs/adr/ADR-0001.md", "# ADR-1: Use Postgres\n");
            repo.write("docs/adr/ADR-01000.md", "# ADR-01000: At rollover\n");
            let records = repo.json(&["list", "--format", "json"], 0);
            assert_eq!(records.len(), 3);
            for (title, expected) in [
                ("Use Postgres", "ADR-1"),
                ("Use Redis", "ADR-0002"),
                ("At rollover", "ADR-01000"),
            ] {
                let record = records.iter().find(|row| row["title"] == title).unwrap();
                equal(
                    &mut failures,
                    &record["id"],
                    expected,
                    &format!("v{version}, per-kind={per_kind}: mixed-width list JSON id"),
                );
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-config.3.2.8, §FS-declarations.checks.duplicate, §FS-id.1.2: these controls already pass;
/// padding must never become identity or an inferred allocation setting.
#[test]
fn equivalent_numbers_collide_and_allocation_width_is_independent() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            for query in ["ADR-1", "ADR-001", "ADR-0001"] {
                let body = repo.run(&["show", query], 0);
                assert!(body.contains("The first decision."));
            }
            repo.write(
                "docs/notes.md",
                &NOTES.replace("\u{a7}ADR-0007", "ADR-0007"),
            );
            repo.run(&["check", "--only", "dangling"], 0);
            for (width, expected) in [("3", "ADR-003"), ("4", "ADR-0003"), ("1", "ADR-3")] {
                let records = repo.json(
                    &["id", "ADR", "Next", "--width", width, "--format", "json"],
                    0,
                );
                assert_eq!(records[0]["id"], expected);
                assert_eq!(records[0]["number"], 3, "numeric JSON field stays a number");
            }
            assert_eq!(repo.run(&["id", "ADR", "Next"], 0), "ADR-003\n");
            repo.write(
                "docs/adr/ADR-1.md",
                "# ADR-1: Same numeric identity\n\nDuplicate.\n",
            );
            let duplicate = repo.run(&["check", "--only", "duplicate"], 1);
            assert!(
                duplicate.contains("ADR-0001.md") && duplicate.contains("ADR-1.md"),
                "{duplicate}"
            );
        }
    }
}
