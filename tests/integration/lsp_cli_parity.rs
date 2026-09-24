//! §FS-lsp.4 — the parity sweep that used to be called future hardening: for
//! every e2e case that is a plain `check` of a fixture carrying its own config,
//! the diagnostics `grund-lsp` publishes on `initialized` are exactly the
//! located findings `grund check --format json` prints for the same tree —
//! path, line, code, severity and message — so the server is provably the same
//! engine behind a different transport (§AR-lsp) and not a second one that can
//! drift. Cases the CLI refuses (exit `2`) and fixtures with no config at their
//! root are skipped and counted, never silently, and a floor on the number of
//! compared cases keeps the sweep from shrinking unnoticed.
//!
//! The run-level `[workspace]` warnings of §FS-lsp.1.1.3 are held the same way,
//! against the same case's stderr: §FS-check.4.7.7, §FS-check.3.29.15, §FS-check.4.10.11
//! and §FS-workspace.6.1.7 travel in the run's warning channel and are rendered by
//! each frontend, so neither surface may carry one the other does not. They are
//! compared as their own set because the two shapes differ by design — the CLI
//! prints three of them as §FS-check.2.1.1 lines on stderr and §FS-check.3.29.13's
//! as one of `check`'s report objects with a null location, while the editor
//! publishes each on the `grund.toml` it anchors at. Every other
//! CLI-level `warning:` or `error:` line (§FS-errors.2.2) is stepped over: it is
//! settled before a report exists and neither surface carries it as a located
//! diagnostic.

#[path = "binaries.rs"]
mod binaries;
#[path = "corpus.rs"]
mod corpus;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use support::{send_message, start_server, wait_for_exit};

/// Fewer compared cases than this is a broken sweep, not a smaller corpus: the
/// corpus compares well over a hundred, and no rewrite of a handful of cases
/// into other commands should trip it.
const MIN_COMPARED_CASES: usize = 80;

/// Fewer cases carrying one of the three run-level `[workspace]` warnings than
/// this is a sweep that has stopped holding §FS-lsp.1.1.3: the corpus has several,
/// and a comparison that met none of them would pass on an LSP that publishes
/// nothing.
const MIN_RUN_WARNING_CASES: usize = 4;

/// The two CLI-level message prefixes of §FS-errors.2.2. A launch-time
/// diagnostic keeps its raw text on stderr under `--format json` as well
/// (§FS-errors.5.2), so the reduction below has to step over one — and over
/// nothing else: any other line on either stream is output this sweep does not
/// understand, and a harness that swallowed it would stop being the guard it is
/// here to be.
const CLI_LEVEL_PREFIXES: [&str; 2] = ["error: ", "warning: "];

/// The three run-level `[workspace]` warnings §FS-lsp.1.1.3 names, each by a phrase
/// of its own fixed text (§FS-errors.3): §FS-check.4.7's absorbed scan,
/// §FS-check.4.10's unread opted-out block and §FS-workspace.6.1.7's undecidable
/// ancestor claim.
///
/// Three, not four. §FS-check.3.29's unlisted block left this list when its ramp
/// ended: it is a located error in `check` and a located error in the editor now
/// (§FS-check.3.29.15), so it is compared below as an ordinary `Finding` — same
/// path, line, `error` severity, code and message on both surfaces. That is the
/// assertion that would catch the terminal and the editor drifting apart on the
/// one finding whose location has two places it could come from, and it is
/// stronger than holding a message against a stderr golden.
///
/// Matched on the message rather than on a code, because that is the one thing
/// both surfaces carry: the CLI prints these as text and never as a JSON object,
/// so there is no `code` on its side to compare. Naming the three here is also
/// what makes this sweep say which warnings it holds.
const RUN_LEVEL_WARNINGS: [&str; 3] = [
    "[workspace] members swallows this project's whole scan",
    "no project scans `",
    "cannot read [workspace] members (",
];

fn is_run_level_warning(message: &str) -> bool {
    RUN_LEVEL_WARNINGS
        .iter()
        .any(|marker| message.contains(marker))
}

/// One located finding, in the shape both surfaces can be reduced to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    path: String,
    line: u64,
    severity: &'static str,
    code: String,
    message: String,
}

/// `grund check . --format json` from the fixture root: every located finding
/// on either stream (§FS-output-shapes.7.1), or `None` when the run is refused.
fn cli_findings(grund: &Path, root: &Path) -> Option<(BTreeSet<Finding>, BTreeSet<String>)> {
    let output = Command::new(grund)
        .args(["check", ".", "--format", "json"])
        .current_dir(root)
        .output()
        .expect("run grund check");
    if output.status.code() == Some(2) {
        return None;
    }
    let mut findings = BTreeSet::new();
    let mut run_warnings = BTreeSet::new();
    for stream in [&output.stdout, &output.stderr] {
        for line in String::from_utf8_lossy(stream).lines() {
            let Ok(value) = serde_json::from_str::<Value>(line) else {
                // §FS-errors.2.2: a CLI-level message is settled before a report
                // exists, so neither surface carries it as a *located* finding.
                // §FS-lsp.1.1.3's four arrive here too and are held on their own.
                if let Some(message) = line.strip_prefix("warning: ")
                    && is_run_level_warning(message)
                {
                    run_warnings.insert(message.to_string());
                    continue;
                }
                if CLI_LEVEL_PREFIXES
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
                {
                    continue;
                }
                panic!(
                    "non-JSON line from grund check --format json in {}: {line}",
                    root.display()
                );
            };
            // §FS-check.3.29.13: a run-level warning carried in the report rather
            // than the warning channel arrives as a nulls object — none of the three
            // does today, so this guard keeps the reduction honest if one moves back.
            if let Some(message) = value["message"].as_str()
                && is_run_level_warning(message)
            {
                run_warnings.insert(message.to_string());
                continue;
            }
            let Some(path) = value["path"].as_str() else {
                continue;
            };
            let severity = match value["severity"].as_str() {
                Some("error") => "error",
                Some("warning") => "warning",
                other => panic!("unexpected severity {other:?} in {line}"),
            };
            findings.insert(Finding {
                path: path.replace('\\', "/"),
                // The server anchors a line-less located finding on line 1.
                line: value["line"].as_u64().unwrap_or(1),
                severity,
                code: value["code"].as_str().unwrap_or_default().to_string(),
                message: value["message"].as_str().unwrap_or_default().to_string(),
            });
        }
    }
    Some((findings, run_warnings))
}

/// Everything the server publishes between `initialized` and the `shutdown`
/// response: messages are handled in order, so the diagnostics the handshake
/// pushed are all on the wire before the response to the request sent after it.
fn lsp_findings(root: &Path) -> (BTreeSet<Finding>, BTreeSet<String>) {
    let (mut child, mut stdin, receiver) = start_server(root);
    send_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null }),
    );
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut findings = BTreeSet::new();
    let mut run_warnings = BTreeSet::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = receiver.recv_timeout(remaining).unwrap_or_else(|_| {
            panic!("no shutdown response from grund-lsp for {}", root.display())
        });
        if message.get("id").and_then(Value::as_i64) == Some(2) {
            break;
        }
        if message.get("method").and_then(Value::as_str) != Some("textDocument/publishDiagnostics")
        {
            continue;
        }
        let uri = message["params"]["uri"].as_str().expect("diagnostic uri");
        let path = url::Url::parse(uri)
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .unwrap_or_else(|| panic!("diagnostic uri is not a file path: {uri}"));
        // Canonical on both sides: Windows spells a canonical path with a
        // `\\?\` prefix that a URI-derived one lacks.
        let path = fs::canonicalize(&path).unwrap_or(path);
        let relative = path
            .strip_prefix(&canonical_root)
            .or_else(|_| path.strip_prefix(root))
            .unwrap_or_else(|_| {
                panic!(
                    "diagnostic path {} is outside {}",
                    path.display(),
                    root.display()
                )
            });
        for diagnostic in message["params"]["diagnostics"]
            .as_array()
            .expect("diagnostics array")
        {
            let severity = match diagnostic["severity"].as_u64() {
                Some(1) => "error",
                Some(2) => "warning",
                other => panic!("unexpected diagnostic severity {other:?}"),
            };
            let message = diagnostic["message"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            // §FS-lsp.1.1.3: a run-level `[workspace]` warning is not a located
            // finding on either surface — it is held against the CLI's stderr.
            if is_run_level_warning(&message) {
                run_warnings.insert(message);
                continue;
            }
            findings.insert(Finding {
                path: relative.to_string_lossy().replace('\\', "/"),
                line: diagnostic["range"]["start"]["line"]
                    .as_u64()
                    .expect("start line")
                    + 1,
                severity,
                code: diagnostic["code"].as_str().unwrap_or_default().to_string(),
                message,
            });
        }
    }
    send_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "method": "exit", "params": null }),
    );
    wait_for_exit(&mut child);
    (findings, run_warnings)
}

#[test]
fn lsp_diagnostics_are_the_cli_findings_for_every_plain_check_case() {
    let repo = binaries::repo_root();
    let grund = binaries::grund();
    let _ = support::SERVER_BINARY.set(binaries::grund_lsp());
    let corpus::Selection {
        cases,
        other_commands,
        no_config,
    } = corpus::plain_check_cases(&repo);
    let mut compared = 0;
    let mut refused = Vec::new();
    let mut mismatches = Vec::new();
    let mut run_warning_cases = 0;
    for case in &cases {
        let Some((cli, cli_run_warnings)) = cli_findings(&grund, &case.root) else {
            refused.push(case.name.clone());
            continue;
        };
        let (lsp, lsp_run_warnings) = lsp_findings(&case.root);
        compared += 1;
        if cli != lsp {
            let only_cli = cli.difference(&lsp).collect::<Vec<_>>();
            let only_lsp = lsp.difference(&cli).collect::<Vec<_>>();
            mismatches.push(format!(
                "{}:\n  only the CLI reports: {only_cli:#?}\n  only the LSP reports: {only_lsp:#?}",
                case.name
            ));
        }
        // §FS-lsp.1.1.3, §FS-lsp.4: the four run-level `[workspace]` warnings, held
        // against the same case's stderr — neither surface may carry one the other
        // does not, and the message is the CLI's byte for byte.
        if !cli_run_warnings.is_empty() {
            run_warning_cases += 1;
        }
        if cli_run_warnings != lsp_run_warnings {
            let only_cli = cli_run_warnings
                .difference(&lsp_run_warnings)
                .collect::<Vec<_>>();
            let only_lsp = lsp_run_warnings
                .difference(&cli_run_warnings)
                .collect::<Vec<_>>();
            mismatches.push(format!(
                "{}: run-level [workspace] warnings\n  only the CLI prints:                  {only_cli:#?}\n  only the LSP publishes: {only_lsp:#?}",
                case.name
            ));
        }
    }
    eprintln!(
        "lsp/cli parity: {compared} case(s) compared, {run_warning_cases} of them carrying a \
         run-level [workspace] warning, {} refused by the CLI (exit 2), \
         {} with no config at the fixture root, {other_commands} not a plain check",
        refused.len(),
        no_config.len()
    );
    assert!(
        mismatches.is_empty(),
        "{} case(s) where grund-lsp and grund check disagree:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    // §FS-lsp.4: a sweep that compared no run-level warning at all would hold
    // §FS-lsp.1.1.3's promise vacuously, which is the way this guard fails quietly.
    assert!(
        run_warning_cases >= MIN_RUN_WARNING_CASES,
        "only {run_warning_cases} case(s) carried a run-level [workspace] warning; \
         the sweep expects at least {MIN_RUN_WARNING_CASES}"
    );
    assert!(
        compared >= MIN_COMPARED_CASES,
        "only {compared} case(s) compared; the sweep expects at least {MIN_COMPARED_CASES} \
         (refused: {refused:?}; no config: {no_config:?})"
    );
}
