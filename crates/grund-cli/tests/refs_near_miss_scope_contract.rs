//! End-to-end regression for §FS-scan.path.refs, from issue #1126's reproducer.
//! A path selects hits; the project resolves its declared compatible operand.

#[path = "../../grund-core/tests/support/refs_near_miss_scope.rs"]
mod support;

use std::process::Command;
use support::Repo;

#[test]
#[ignore = "red until #1126 resolves compatible IDs against the project; implement removes this"]
fn refs_near_miss_scope_cli_text_and_json() {
    let repo = Repo::new();
    let selected = "hits/citer.md:1: \u{a7}FS-BadHaskell\nhits/citer.md:1: \u{a7}FS-BadHaskell.1\n";
    let mut failures = Vec::new();
    for path in [
        None,
        Some("src"),
        Some("hits/citer.md"),
        Some("hits"),
        Some("docs/FS-001-alpha.md"),
        Some("docs"),
        Some("empty"),
    ] {
        for json in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
            command.current_dir(&repo.0).args(["refs", "FS-BadHaskell"]);
            if let Some(path) = path {
                command.arg(path);
            }
            if json {
                command.args(["--format", "json"]);
            }
            let output = command.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            if !output.status.success() {
                failures.push(format!(
                    "path={path:?} json={json}: exit {:?}, stdout={stdout:?}, stderr={stderr:?}",
                    output.status.code()
                ));
                continue;
            }
            assert!(stderr.is_empty(), "{stderr}");
            let has_hits = matches!(path, None | Some("hits" | "hits/citer.md"));
            if json {
                let hits = stdout
                    .lines()
                    .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(hits.len(), if has_hits { 2 } else { 0 }, "{stdout}");
                if has_hits {
                    assert_eq!(hits[0]["path"], "hits/citer.md");
                    assert_eq!(hits[1]["path"], "hits/citer.md");
                    assert_eq!(hits[0]["line"], 1);
                    assert_eq!(hits[1]["line"], 1);
                    assert!(hits[0]["section"].is_null(), "{stdout}");
                    assert_eq!(hits[1]["section"], "1");
                }
            } else {
                assert_eq!(stdout, if has_hits { selected } else { "" }, "{path:?}");
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
