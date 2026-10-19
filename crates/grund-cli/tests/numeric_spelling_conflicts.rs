//! A conflict block's owner keeps its authored digits, in `check`'s line and in
//! `fmt`'s refusal alike, as a `duplicate` finding spells its ID
//! (§FS-declarations.checks.conflict-marker, §FS-config.3.2.8).

#[path = "support/numeric_spelling.rs"]
#[allow(dead_code)]
mod support;

use std::process::Command;

use support::{Repo, contains};

// The markers are escaped onto one line so that no line of this file starts with one.
const CONFLICTED: &str = concat!(
    "# ADR-0001: Use Postgres\n\nThe first decision.\n\n",
    "<<<<<<< HEAD\nTransactions.\n=======\nReplication.\n>>>>>>> other\n",
);
const MESSAGE: &str = "unresolved merge conflict in ADR-0001, lines 5\u{2013}9";

#[test]
fn a_conflict_block_names_its_owner_with_the_authored_digits() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            repo.write("docs/adr/ADR-0001.md", CONFLICTED);
            let mut failures = Vec::new();
            let check = repo.run(&["check"], 1);
            let reported = format!("docs/adr/ADR-0001.md:5: error: {MESSAGE}");
            contains(&mut failures, &check, &reported, "check");
            let fmt = Command::new(env!("CARGO_BIN_EXE_grund"))
                .args(["fmt", "--write"])
                .current_dir(&repo.root)
                .output()
                .expect("run grund fmt --write");
            assert_eq!(fmt.status.code(), Some(2), "fmt --write refuses");
            let refusal = String::from_utf8(fmt.stderr).unwrap();
            let refused =
                format!("error: nothing was rewritten: docs/adr/ADR-0001.md:5: {MESSAGE}");
            contains(&mut failures, &refusal, &refused, "fmt --write");
            assert_eq!(repo.read("docs/adr/ADR-0001.md"), CONFLICTED);
            assert!(
                failures.is_empty(),
                "v{version}, per-kind={per_kind}:\n{}",
                failures.join("\n")
            );
        }
    }
}
