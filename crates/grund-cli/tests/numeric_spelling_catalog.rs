//! Selected target spelling agrees across catalog reports, generated queries
//! and completion (§FS-config.3.2.8, §FS-show.2.6.2, §FS-completions.3).

#[path = "support/numeric_spelling.rs"]
#[allow(dead_code)]
mod support;

use support::{FIRST, Repo};

#[test]
fn paired_numeric_target_spelling_selects_and_completes_across_scan_boundaries() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            for scanned in [false, true] {
                let repo = Repo::new(version, per_kind);
                let (target, link) = if scanned {
                    ("docs/adr/target.md", "target.md")
                } else {
                    ("targets/decision.md", "../../targets/decision.md")
                };
                repo.write(target, FIRST);
                repo.write(
                    "docs/adr/ADR-0001.md",
                    &format!("# ADR-1: [{link}]({link})\n"),
                );
                let list = repo.json(&["list", "--format", "json"], 0);
                assert!(
                    list.iter()
                        .any(|row| row["id"] == "ADR-0001" && row["path"] == target)
                );
                let direct = repo.json(&["show", "ADR-001", "--format", "json"], 0);
                assert_eq!(direct[0]["id"], "ADR-0001");
                let exact = repo.json(
                    &["show", "--batch", "--glob", "ADR-0001", "--format", "json"],
                    0,
                );
                assert_eq!(exact.len(), 1);
                assert_eq!(exact[0]["query"]["id"], "ADR-0001");
                assert_eq!(exact[0]["result"], direct[0]);
                let sections = repo.json(
                    &[
                        "show",
                        "--batch",
                        "--glob",
                        "ADR-0001.1",
                        "--format",
                        "json",
                    ],
                    0,
                );
                assert_eq!(sections.len(), 1);
                assert_eq!(sections[0]["query"]["id"], "ADR-0001");
                assert_eq!(sections[0]["query"]["section"], "1");
                assert_eq!(sections[0]["result"]["id"], "ADR-0001");
                let all = repo.json(&["show", "--batch", "--all", "--format", "json"], 0);
                assert_eq!(all.len(), 3);
                for row in all {
                    assert_eq!(row["ok"], true);
                    assert_eq!(row["query"]["id"], row["result"]["id"]);
                }
                let complete = repo.run(&["complete", "ids", "--prefix", "ADR-000"], 0);
                assert_eq!(complete, "ADR-0001\nADR-0002\n");
                let section = repo.run(
                    &["complete", "ids", "--prefix", "ADR-0001.", "--sections"],
                    0,
                );
                assert_eq!(section, "ADR-0001.1\n");
            }
        }
    }
}
