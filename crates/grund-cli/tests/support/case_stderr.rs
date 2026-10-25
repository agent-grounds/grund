// The stderr-conciseness half of case_runner.rs judges ordinary diagnostics,
// JSON shapes and unwrapped migration entries (§FS-config-migrate.2.2).
// It shares case_name and the manifest readers with the runner.

/// Whether a case's command selects `--format json`. Decided from the
/// expanded argument vector rather than the raw `command.args` text, because
/// the corpus spells the flag both ways: `--format=json` in one argument, or
/// `--format` and `json` as two.
fn command_selects_json(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--format=json")
        || args
            .windows(2)
            .any(|pair| pair[0] == "--format" && pair[1] == "json")
}

/// A non-zero case's `expected.stderr` must stay a concise diagnostic
/// (`tests/e2e/README.md`, "Error output is part of the contract"). Reads the
/// case's `expected.exit` — a clean case has nothing to judge — then hands the
/// judgement itself to [`assert_stderr_is_concise`], which touches no file and
/// is what the self-tests at the bottom of this file drive directly.
fn assert_expected_errors_are_concise(
    case: &Path,
    name: &str,
    execution: &CaseExecution,
    stderr: &str,
) {
    if read_to_string(case.join("expected.exit")).trim() == "0" {
        return;
    }
    assert_stderr_is_concise(
        name,
        command_selects_json(&execution.args),
        command_is_config_migrate(&execution.args, execution.is_external),
        stderr,
    );
}

/// §FS-config-migrate.2.2: only Grund's migration report owns these records.
/// Match the dispatched command, never tokens in an external or unrelated argv.
/// `config` removes `--path-base` before dispatching its action (§FS-cli.3.4).
fn command_is_config_migrate(args: &[String], is_external: bool) -> bool {
    if is_external || args.first().map(String::as_str) != Some("config") {
        return false;
    }
    let mut words = args[1..].iter().map(String::as_str);
    while let Some(word) = words.next() {
        match word {
            "--path-base" => {
                if !matches!(words.next(), Some("project" | "invocation")) {
                    return false;
                }
            }
            "--path-base=project" | "--path-base=invocation" => {}
            action => return action == "migrate",
        }
    }
    false
}

/// §FS-config-migrate.2.2: eligibility starts at this exact block heading and
/// ends at the first line that is not a well-formed listed-comment record.
const MIGRATION_COMMENT_HEADING: &str =
    "comments with no single key to follow (--write refuses; --drop-comments drops exactly these):";

fn is_migration_comment_record(line: &str) -> bool {
    let Some((location, entry)) = line.strip_prefix("  ").and_then(|s| s.split_once("  ")) else {
        return false;
    };
    let Some((path, range)) = location.rsplit_once(':') else {
        return false;
    };
    let positive_line = |s: &str| {
        (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse::<usize>().ok())
            .flatten()
            .filter(|n| *n > 0)
    };
    let valid_range = if let Some((first, last)) = range.split_once('-') {
        matches!((positive_line(first), positive_line(last)), (Some(a), Some(b)) if a <= b)
    } else {
        positive_line(range).is_some()
    };
    let Some((attachment, guidance)) = entry.split_once(": ") else {
        return false;
    };
    let attached_key = attachment
        .strip_prefix("above ")
        .or_else(|| attachment.strip_prefix("inside "));
    !path.trim().is_empty()
        && path.trim() == path
        && valid_range
        && (attachment == "standing alone" || attached_key.is_some_and(is_migration_v1_key))
        && !guidance.trim().is_empty()
        && guidance.trim() == guidance
}

/// The v1 attachment spelling: a root key, a table, or a table and key.
fn is_migration_v1_key(key: &str) -> bool {
    let identifier = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    };
    if !key.starts_with('[') {
        return identifier(key);
    }
    let (header, key) = key
        .split_once(' ')
        .map_or((key, None), |(h, k)| (h, Some(k)));
    let table = header
        .strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
        .or_else(|| header.strip_prefix('[').and_then(|s| s.strip_suffix(']')));
    table.is_some_and(|s| s.split('.').all(identifier)) && key.is_none_or(identifier)
}

/// The absorbed-scan config error opens with this, after its `members`-line
/// breadcrumb. It was a carried warning for three releases before it became an
/// error, and its sentence is the one §FS-check.3.30.1 fixes, so it keeps those
/// bytes rather than the cap.
const ABSORBED_SCAN_SENTENCE: &str = ": [workspace] members swallows this project's whole scan — ";

/// A pinned `[citations]` target the run cannot accept was a load warning until
/// `0.19.0` made it an error (§FS-config.3.9.5.2), so its line ends in this
/// clause and keeps the bytes that point fixes rather than the cap.
const PROMOTED_LOAD_WARNING_SUFFIX: &str = "; this became an error in grund 0.19.0";

/// The judgement, isolated from the filesystem. A `--format json` case's
/// stderr is not uniformly JSON (§FS-errors.5.2.2): a launch-time `error: …`
/// line stays text and keeps the plain 180-byte cap, while a carried text
/// warning keeps the bytes its own specification already fixes
/// (§FS-check.4.10.5), and so does the absorbed-scan error (§FS-check.3.30.1). Only a line that opens a JSON object is parsed and
/// judged by its `message` field instead of its serialized length — the
/// scaffolding around it (§FS-distribution.3.0.1's `severity`, `path`,
/// `line`, `code`, `sites`, `authority`) is fixed cost the conciseness policy
/// was never about. Within Grund's exact migration comment-list block, only
/// well-formed entries keep their unwrapped guidance (§FS-config-migrate.2.2).
/// All other lines retain their limits; aggregate summaries remain forbidden.
fn assert_stderr_is_concise(name: &str, json_case: bool, migration_case: bool, stderr: &str) {
    assert!(
        !stderr.contains("error(s)") && !stderr.contains("warning(s)"),
        "{name}: stderr should not include aggregate summaries"
    );
    let mut in_comment_list = false;
    for line in stderr.lines() {
        let comment_record = migration_case && in_comment_list && is_migration_comment_record(line);
        in_comment_list = migration_case && (line == MIGRATION_COMMENT_HEADING || comment_record);
        if line.trim().is_empty() || comment_record {
            continue;
        }
        if line.starts_with("warning: ")
            || (line.starts_with("error: ") && line.contains(ABSORBED_SCAN_SENTENCE))
            || (line.starts_with("error: ") && line.ends_with(PROMOTED_LOAD_WARNING_SUFFIX))
        {
            continue;
        } else if json_case && line.starts_with('{') {
            assert_json_diagnostic_is_concise(name, line);
        } else {
            assert!(
                line.len() <= 180,
                "{name}: stderr line is too long for a concise diagnostic: {line}"
            );
        }
    }
}

/// Parse one `{`-line as the diagnostic object §FS-errors.5.2 / §FS-distribution.3.0.1
/// promise, assert its shape, and cap its `message`. Every rejection names the
/// case and the whole line, exactly as the plain-text cap does.
fn assert_json_diagnostic_is_concise(name: &str, line: &str) {
    let value: serde_json::Value = serde_json::from_str(line)
        .unwrap_or_else(|err| panic!("{name}: stderr line is not valid JSON ({err}): {line}"));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("{name}: stderr JSON line is not an object: {line}"));

    let mut actual_keys: Vec<&str> = object.keys().map(String::as_str).collect();
    actual_keys.sort_unstable();
    // §FS-errors.5.2: `authority` is present on every record of this shape and
    // always null here, so the key set is one set rather than a conditional.
    let mut expected_keys = [
        "severity",
        "path",
        "line",
        "code",
        "message",
        "sites",
        "authority",
    ];
    expected_keys.sort_unstable();
    assert_eq!(
        actual_keys, expected_keys,
        "{name}: stderr JSON line has the wrong key set: {line}"
    );

    let severity = object.get("severity").and_then(|v| v.as_str());
    assert!(
        matches!(severity, Some("error") | Some("warning")),
        "{name}: stderr JSON `severity` must be \"error\" or \"warning\": {line}"
    );
    assert!(
        object
            .get("code")
            .and_then(|v| v.as_str())
            .is_some_and(|code| !code.is_empty()),
        "{name}: stderr JSON `code` must be a non-empty string: {line}"
    );
    let message = object
        .get("message")
        .and_then(|v| v.as_str())
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| {
            panic!("{name}: stderr JSON `message` must be a non-empty string: {line}")
        });
    assert!(
        message.len() <= 180,
        "{name}: stderr JSON `message` is too long for a concise diagnostic ({} bytes): {line}",
        message.len()
    );
    assert!(
        matches!(
            object.get("path"),
            Some(serde_json::Value::Null) | Some(serde_json::Value::String(_))
        ),
        "{name}: stderr JSON `path` must be null or a string: {line}"
    );
    assert!(
        matches!(object.get("line"), Some(v) if v.is_null() || v.is_number()),
        "{name}: stderr JSON `line` must be null or a number: {line}"
    );
    assert_json_sites_are_well_formed(name, line, object.get("sites"));
    assert_json_authority_is_well_formed(name, line, object.get("authority"));
}

/// §FS-errors.5.2: `authority` names the rules that authored a finding, and no
/// query failure or run-level line is a rule's — so on stderr it is `null`, and
/// the shape it takes elsewhere (a list of origin strings) is asserted here too,
/// so one renderer cannot start emitting a shape the other never would.
fn assert_json_authority_is_well_formed(
    name: &str,
    line: &str,
    authority: Option<&serde_json::Value>,
) {
    match authority {
        Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Array(entries)) => {
            for origin in entries {
                assert!(
                    matches!(origin, serde_json::Value::String(origin) if !origin.is_empty()),
                    "{name}: stderr JSON `authority` entry must be a non-empty string: {line}"
                );
            }
        }
        _ => panic!("{name}: stderr JSON `authority` must be null or an array: {line}"),
    }
}

/// `sites` is `null`, or a list of `{ path, line }` locating every site of a
/// multi-site finding (§FS-distribution.3.0.1) — unlike the diagnostic's own
/// `path` / `line`, a site always has both: it exists to say where.
fn assert_json_sites_are_well_formed(name: &str, line: &str, sites: Option<&serde_json::Value>) {
    match sites {
        Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Array(entries)) => {
            for site in entries {
                let site = site.as_object().unwrap_or_else(|| {
                    panic!("{name}: stderr JSON `sites` entry is not an object: {line}")
                });
                let mut keys: Vec<&str> = site.keys().map(String::as_str).collect();
                keys.sort_unstable();
                assert_eq!(
                    keys,
                    ["line", "path"],
                    "{name}: stderr JSON `sites` entry has the wrong key set: {line}"
                );
                assert!(
                    matches!(site.get("path"), Some(serde_json::Value::String(_))),
                    "{name}: stderr JSON `sites[].path` must be a string: {line}"
                );
                assert!(
                    matches!(site.get("line"), Some(v) if v.is_number()),
                    "{name}: stderr JSON `sites[].line` must be a number: {line}"
                );
            }
        }
        _ => panic!("{name}: stderr JSON `sites` must be null or an array: {line}"),
    }
}

#[cfg(test)]
mod stderr_concise_tests {
    use super::assert_stderr_is_concise;
    use std::panic::{self, AssertUnwindSafe};

    /// Runs the judgement and returns the panic message, or panics itself if
    /// the stderr under test was wrongly accepted.
    fn rejects(json_case: bool, stderr: &str) -> String {
        match panic::catch_unwind(AssertUnwindSafe(|| {
            assert_stderr_is_concise("case", json_case, false, stderr)
        })) {
            Ok(()) => panic!("expected stderr to be rejected, but it passed: {stderr}"),
            Err(payload) => payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_default(),
        }
    }

    #[test]
    fn json_case_accepts_a_representative_diagnostic_object() {
        let message = "ambiguous section: FS-001-login.1 (declared at \
                        docs/functional-spec/FS-001-login.md:5, \
                        docs/functional-spec/FS-001-login.md:9)";
        assert!(message.len() <= 180, "fixture message grew past the cap");
        let line = format!(
            "{{\"severity\":\"error\",\"path\":null,\"line\":null,\
              \"code\":\"ambiguous-section\",\"message\":\"{message}\",\"sites\":null,\
              \"authority\":null}}"
        );
        assert!(line.len() > 180, "fixture line no longer exceeds the cap");
        assert_stderr_is_concise("case", true, false, &line);
    }

    #[test]
    fn json_case_rejects_a_message_over_the_cap() {
        let message = "x".repeat(181);
        let line = format!(
            "{{\"severity\":\"error\",\"path\":null,\"line\":null,\
              \"code\":\"c\",\"message\":\"{message}\",\"sites\":null,\"authority\":null}}"
        );
        let panic_message = rejects(true, &line);
        assert!(panic_message.contains("message"), "{panic_message}");
    }

    #[test]
    fn json_case_rejects_a_line_with_an_extra_key() {
        let line = "{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"c\",\
                     \"message\":\"m\",\"sites\":null,\"authority\":null,\"extra\":1}";
        let panic_message = rejects(true, line);
        assert!(panic_message.contains("key set"), "{panic_message}");
    }

    #[test]
    fn json_case_rejects_a_line_with_a_missing_key() {
        let line = "{\"severity\":\"error\",\"path\":null,\"code\":\"c\",\
                     \"message\":\"m\",\"sites\":null,\"authority\":null}";
        let panic_message = rejects(true, line);
        assert!(panic_message.contains("key set"), "{panic_message}");
    }

    #[test]
    fn json_case_rejects_malformed_json() {
        let panic_message = rejects(true, "{\"severity\":\"error\"");
        assert!(panic_message.contains("not valid JSON"), "{panic_message}");
    }

    #[test]
    fn json_case_still_caps_a_text_line() {
        let line = format!("error: {}", "x".repeat(180));
        assert!(line.len() > 180, "fixture line no longer exceeds the cap");
        let panic_message = rejects(true, &line);
        assert!(panic_message.contains("too long"), "{panic_message}");
    }

    #[test]
    fn nonzero_case_preserves_a_carried_text_warning() {
        let warning = format!("warning: {}", "x".repeat(220));
        let stderr = format!("{warning}\nerror: concise refusal\n");
        assert_stderr_is_concise("case", false, false, &stderr);
        assert_stderr_is_concise("case", true, false, &stderr);
    }

    #[test]
    fn nonzero_case_preserves_the_absorbed_scan_error_and_nothing_beside_it() {
        let error = format!(
            "error: grund.toml:16: [workspace] members swallows this project's whole scan — {}",
            "x".repeat(220)
        );
        assert_stderr_is_concise("case", false, false, &error);
        assert_stderr_is_concise("case", true, false, &error);
        let other = format!(
            "error: grund.toml:16: [workspace] members {}",
            "x".repeat(220)
        );
        let panic_message = rejects(false, &other);
        assert!(panic_message.contains("too long"), "{panic_message}");
    }

    #[test]
    fn nonzero_case_preserves_a_promoted_load_warning_and_nothing_beside_it() {
        let error = format!(
            "error: grund.toml: [citations.API] should names `web/WEB`: {} — check from \
             the workspace root; this became an error in grund 0.19.0",
            "x".repeat(180)
        );
        assert_stderr_is_concise("case", false, false, &error);
        let other = format!(
            "error: grund.toml: [citations.API] should names {}",
            "x".repeat(180)
        );
        let panic_message = rejects(false, &other);
        assert!(panic_message.contains("too long"), "{panic_message}");
    }

    #[test]
    fn text_case_rejects_a_line_over_the_cap() {
        let line = "x".repeat(223);
        let panic_message = rejects(false, &line);
        assert!(panic_message.contains("too long"), "{panic_message}");
    }
}

#[cfg(test)]
#[path = "case_migrate_stderr_tests.rs"]
mod stderr_migration_tests;
