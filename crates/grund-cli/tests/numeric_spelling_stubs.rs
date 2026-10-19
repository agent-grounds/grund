//! Target spelling survives an equivalent stub, scanned or not
//! (§FS-config.3.2.8, §FS-fmt.6.2.2).

#[path = "support/numeric_spelling.rs"]
mod support;

use support::{FIRST, Repo, contains, equal};

/// §FS-fmt.6.2.2: show, list and fmt read the real home rather than borrowing
/// the digits of a stub or citation. The two scan boundaries give the same anchor.
#[test]
fn equivalent_numeric_stub_uses_target_spelling_inside_and_outside_the_walk() {
    let mut failures = Vec::new();
    for version in [1, 2] {
        for per_kind in [false, true] {
            for scanned in [false, true] {
                let repo = Repo::new(version, per_kind);
                let (target, stub_link, citation_link) = if scanned {
                    ("docs/adr/target.md", "target.md", "adr/target.md")
                } else {
                    (
                        "targets/decision.md",
                        "../../targets/decision.md",
                        "../targets/decision.md",
                    )
                };
                repo.write(target, FIRST);
                repo.write(
                    "docs/adr/ADR-0001.md",
                    &format!("# ADR-1: [{stub_link}]({stub_link})\n"),
                );
                let context = format!("v{version}, per-kind={per_kind}, scanned={scanned}");
                let shown = repo.json(&["show", "ADR-001", "--format", "json"], 0);
                equal(&mut failures, &shown[0]["id"], "ADR-0001", &context);
                equal(
                    &mut failures,
                    &shown[0]["anchor"],
                    "adr-0001-use-postgres",
                    &context,
                );
                equal(&mut failures, &shown[0]["path"], target, &context);
                let listed = repo.json(&["list", "--format", "json"], 0);
                let home = listed.iter().find(|row| row["path"] == target).unwrap();
                equal(&mut failures, &home["id"], "ADR-0001", &context);
                repo.run(&["fmt", "--write"], 0);
                let notes = repo.read("docs/notes.md");
                for spelling in ["ADR-0001", "ADR-1"] {
                    contains(
                        &mut failures,
                        &notes,
                        &format!("[\u{a7}{spelling}]({citation_link}#adr-0001-use-postgres)"),
                        &context,
                    );
                }
                contains(
                    &mut failures,
                    &notes,
                    &format!("[\u{a7}ADR-0001.1]({citation_link}#1-why)"),
                    &context,
                );
                assert_eq!(repo.read(target), FIRST);
                repo.run(&["fmt", "--check"], 0);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
