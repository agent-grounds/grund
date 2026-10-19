//! Unresolved query inputs retain their own digits; escaped illustrations keep
//! their source spelling, while remedies name the selected declaration
//! (§FS-config.3.2.8, §FS-check.2.3.1).

#[path = "support/numeric_spelling.rs"]
#[allow(dead_code)]
mod support;

use std::io::Write;
use std::process::{Command, Stdio};
use support::Repo;

#[test]
fn missing_show_and_refs_reports_preserve_input_digits_and_existing_exits() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            repo.write("docs/notes.md", "");
            for id in ["ADR-0007", "ADR-07", "ADR-7"] {
                for json in [false, true] {
                    let mut args = vec!["show", id];
                    if json {
                        args.push("--format=json");
                    }
                    // Show's text refusal is on stderr; JSON uses the public record.
                    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
                        .args(&args)
                        .current_dir(&repo.root)
                        .output()
                        .unwrap();
                    assert_eq!(output.status.code(), Some(1), "{output:?}");
                    let expected = format!("ID not found: {id}");
                    assert!(output.stdout.is_empty(), "{output:?}");
                    if json {
                        let row: serde_json::Value =
                            serde_json::from_slice(&output.stderr).unwrap();
                        assert_eq!(row["code"], "not-found");
                        assert!(
                            row["message"].as_str().unwrap().contains(&expected),
                            "{row}"
                        );
                    } else {
                        assert!(
                            String::from_utf8_lossy(&output.stderr).contains(&expected),
                            "{output:?}"
                        );
                    }
                    let mut args = vec!["refs", id];
                    if json {
                        args.push("--format=json");
                    }
                    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
                        .args(&args)
                        .current_dir(&repo.root)
                        .output()
                        .unwrap();
                    assert_eq!(output.status.code(), Some(0), "{output:?}");
                    assert!(output.stdout.is_empty(), "{output:?}");
                    assert!(
                        String::from_utf8_lossy(&output.stderr)
                            .contains(&format!("note: {id} is neither declared nor cited")),
                        "{output:?}"
                    );
                }
                let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
                    .args(["show", "--batch", "--format=json"])
                    .current_dir(&repo.root)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();
                let mut input = child.stdin.take().unwrap();
                writeln!(input, "{}", serde_json::json!({"id":id})).unwrap();
                drop(input);
                let output = child.wait_with_output().unwrap();
                assert_eq!(output.status.code(), Some(1), "{output:?}");
                let batch: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(batch["query"]["id"], id);
                assert_eq!(batch["ok"], false);
                assert_eq!(batch["error"]["code"], "not-found");
                assert!(
                    batch["error"]["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("ID not found: {id}")),
                    "{batch:?}"
                );
            }
        }
    }
}

#[test]
fn escaped_numeric_suggestions_select_declaration_spelling_without_making_an_edge() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            repo.write(
                "docs/notes.md",
                "Illustrations: <§>ADR-0001 and <§>ADR-01 and <§>ADR-0007.\n",
            );
            let args = [
                "check",
                "--suggestions",
                "--only",
                "escaped-citation-resolves",
                "--format=json",
            ];
            let rows = repo.json(&args, 0);
            let suggestions = rows
                .iter()
                .filter(|row| row["code"] == "escaped-citation-resolves")
                .collect::<Vec<_>>();
            assert_eq!(suggestions.len(), 2, "{rows:?}");
            for (row, source) in suggestions.iter().zip(["<§>ADR-0001", "<§>ADR-01"]) {
                let message = row["message"].as_str().unwrap();
                assert!(message.contains(source), "{message}");
                assert!(
                    message.contains("write §ADR-0001 for a live citation"),
                    "{message}"
                );
            }
            assert_eq!(repo.run(&["refs", "ADR-1", "--total"], 0), "not cited\n");
        }
    }
}
