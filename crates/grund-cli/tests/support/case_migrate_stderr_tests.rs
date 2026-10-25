//! §FS-config-migrate.2.2: unwrapped listed-comment records are structured report
//! entries; ordinary diagnostics beside them keep their conciseness limits.

use super::{
    MIGRATION_COMMENT_HEADING, assert_expected_errors_are_concise, assert_stderr_is_concise,
    command_is_config_migrate, command_selects_json, prepare_case,
};
use std::panic::{self, AssertUnwindSafe};

const MIGRATE: &[&str] = &["config", "migrate", "--to", "2"];
const GUIDANCE: &str = "setting splits or merges into version-2 settings: \
    [schema.kinds.code] folders, docs and src; retain the prose, authorize the \
    listed drops with --write --drop-comments, then re-add it at one of these \
    named destinations after a successful write (subject to all other refusals and gates)";

fn record(location: &str, attachment: &str) -> String {
    let line = format!("  {location}  {attachment}: {GUIDANCE}");
    assert!(line.len() > 180, "fixture must exercise the exception");
    line
}

fn listing() -> String {
    format!(
        "{MIGRATION_COMMENT_HEADING}\n{}\n",
        record("grund.toml:14", "above [scan] include")
    )
}

fn judge(args: &[&str], external: bool, stderr: &str) {
    let args = args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    // Even a migration-shaped case name grants no exception on its own.
    assert_stderr_is_concise(
        "config-migrate-comments-refused",
        command_selects_json(&args),
        command_is_config_migrate(&args, external),
        stderr,
    );
}

fn rejects(check: impl FnOnce(), expected: &str) {
    let payload =
        panic::catch_unwind(AssertUnwindSafe(check)).expect_err("stderr must be rejected");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert!(message.contains(expected), "{message}");
}

#[test]
fn long_single_and_range_records_accept_every_attachment_form() {
    let mut stderr = format!("{MIGRATION_COMMENT_HEADING}\n");
    for location in [
        "grund.toml:14",
        "dir with spaces/grund.toml:14-16",
        "C:/repo/grund.toml:3-5",
    ] {
        for attachment in [
            "above [scan] include",
            "above [scan]",
            "inside [[kinds]] kind",
            "inside [scan] include",
            "standing alone",
            "above project_name",
            "above [fmt.cross_refs]",
            "above [[kinds]]",
        ] {
            stderr.push_str(&record(location, attachment));
            stderr.push('\n');
        }
    }
    judge(MIGRATE, false, &stderr);
}

#[test]
fn existing_long_migration_goldens_keep_their_bytes() {
    for stderr in [
        include_str!("../../../../tests/e2e/cases/config-migrate-comments-refused/expected.stderr"),
        include_str!("../../../../examples/migrate/migrate-1-refused/expected.stderr"),
        include_str!("../../../../examples/migrate/migrate-1-refused/expected.write.stderr"),
        include_str!("../../../../examples/migrate/migrate-2-repaired/expected.stderr"),
        include_str!("../../../../examples/migrate/migrate-4-full/expected.stderr"),
    ] {
        judge(MIGRATE, false, stderr);
    }
}

#[test]
fn only_grunds_dispatched_migration_command_owns_the_block() {
    let stderr = listing();
    for args in [
        MIGRATE,
        &["config", "--path-base", "invocation", "migrate", "--to=2"],
        &["config", "--path-base=project", "migrate", "--to", "2"],
    ] {
        judge(args, false, &stderr);
        rejects(|| judge(args, true, &stderr), "too long");
    }
    for args in [
        &[][..],
        &["check"],
        &["config", "show"],
        &["config"],
        &["check", "config", "migrate"],
        &["fetch", "config", "migrate"],
        &["other", "config", "migrate"],
        &["config", "explain", "config", "migrate"],
        &["config", "--path-base", "migrate"],
        &["config", "--path-base=invalid", "migrate"],
    ] {
        rejects(|| judge(args, false, &stderr), "too long");
        rejects(|| judge(args, true, &stderr), "too long");
    }
}

#[test]
fn prepared_case_carries_external_identity_into_the_judgement() {
    // §FS-examples.5.4: exercise the real manifest reader, without spawning.
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!("grund-1004-stderr-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let scratch = Scratch(path);
    let case = &scratch.0;
    std::fs::write(case.join("expected.exit"), "1\n").unwrap();
    std::fs::write(case.join("command.args"), "config migrate --to 2").unwrap();
    let execution = prepare_case(case, case, "migration");
    assert_expected_errors_are_concise(case, "migration", &execution, &listing());
    std::fs::remove_file(case.join("command.args")).unwrap();
    for words in [
        vec!["config", "migrate", "--to", "2"],
        vec!["{grund}", "config", "migrate", "--to", "2"],
        vec!["other", "config", "migrate", "--to", "2"],
    ] {
        std::fs::write(
            case.join("command.external"),
            serde_json::to_string(&words).unwrap(),
        )
        .unwrap();
        let execution = prepare_case(case, case, "external");
        rejects(
            || assert_expected_errors_are_concise(case, "external", &execution, &listing()),
            "too long",
        );
    }
}

#[test]
fn records_require_the_exact_heading_and_end_at_the_block_boundary() {
    let entry = record("grund.toml:14", "above [scan] include");
    for heading in [
        "",
        "comments with no single key to follow:",
        &format!(" {MIGRATION_COMMENT_HEADING}"),
    ] {
        rejects(
            || judge(MIGRATE, false, &format!("{heading}\n{entry}")),
            "too long",
        );
    }
    for boundary in [
        "",
        "  ",
        "mapped: grund.toml:15: [scan] include → [schema.kinds.code] folders",
        "error: concise refusal",
        "hint: concise hint",
        "warning: carried warning",
        "  repair: choose a setting",
        "  grund.toml:20  below [scan]: malformed attachment",
    ] {
        let stderr = format!("{}{boundary}\n{entry}", listing());
        rejects(|| judge(MIGRATE, false, &stderr), "too long");
    }
    rejects(
        || judge(MIGRATE, false, &format!("{entry}\n{}", listing())),
        "too long",
    );
}

#[test]
fn malformed_long_records_keep_the_plain_cap() {
    for location in [
        "grund.toml",
        ":14",
        "grund.toml:",
        "grund.toml:0",
        "grund.toml:-1",
        "grund.toml:+1",
        "grund.toml:one",
        "grund.toml:1-",
        "grund.toml:3-2",
        "grund.toml:0-2",
        "grund.toml:1-2-3",
        "grund.toml:1- 3",
    ] {
        let stderr = format!(
            "{MIGRATION_COMMENT_HEADING}\n{}",
            record(location, "above [scan] include")
        );
        rejects(|| judge(MIGRATE, false, &stderr), "too long");
    }
    for attachment in [
        "below [scan] include",
        "above ",
        "inside ",
        "standing alone extra",
        "above [scan",
        "above [scan]] include",
        "inside [[kinds] kind",
        "above [scan] include extra",
        "above [scan] ",
        "above [scan..include]",
    ] {
        let stderr = format!(
            "{MIGRATION_COMMENT_HEADING}\n{}",
            record("grund.toml:14", attachment)
        );
        rejects(|| judge(MIGRATE, false, &stderr), "too long");
    }
    let entry = record("grund.toml:14", "above [scan] include");
    for malformed in [
        entry.trim_start().to_string(),
        format!(" {entry}"),
        format!("  {entry}"),
        entry.replacen("14  above", "14 above", 1),
        entry.replacen("include: ", "include ", 1),
        format!("  grund.toml:14  above {}: ", "x".repeat(181)),
        format!("  grund.toml:14  above [scan] include: {}", " ".repeat(181)),
    ] {
        rejects(
            || {
                judge(
                    MIGRATE,
                    false,
                    &format!("{MIGRATION_COMMENT_HEADING}\n{malformed}"),
                )
            },
            "too long",
        );
    }
}

#[test]
fn ordinary_errors_hints_and_prose_keep_the_cap_around_a_listing() {
    for prefix in ["error: ", "hint: ", "  repair: ", "  ", ""] {
        let long = format!("{prefix}{}", "x".repeat(181));
        for stderr in [
            format!("{long}\n{}", listing()),
            format!(
                "{MIGRATION_COMMENT_HEADING}\n{long}\n{}",
                record("grund.toml:14", "above [scan]")
            ),
            format!("{}{long}\n", listing()),
            format!("{}\n{long}\n", listing()),
        ] {
            rejects(|| judge(MIGRATE, false, &stderr), "too long");
        }
    }
    judge(
        MIGRATE,
        false,
        &format!(
            "error: concise\nhint: concise\n{}error: concise\n",
            listing()
        ),
    );
}

#[test]
fn aggregate_summaries_are_rejected_even_inside_valid_records() {
    for summary in ["error(s)", "warning(s)"] {
        let stderr = listing().replace(
            "retain the prose",
            &format!("retain the prose about {summary}"),
        );
        rejects(|| judge(MIGRATE, false, &stderr), "aggregate summaries");
    }
}

#[test]
fn listing_context_preserves_json_structure_and_message_limits() {
    let args = &["config", "migrate", "--to", "2", "--format", "json"];
    let diagnostic = serde_json::json!({
        "severity": "error", "path": null, "line": null, "code": "c",
        "message": "concise", "sites": null, "authority": null,
    });
    judge(args, false, &format!("{}{}", listing(), diagnostic));
    let mut long = diagnostic.clone();
    long["message"] = "x".repeat(181).into();
    let mut missing = diagnostic.clone();
    missing.as_object_mut().unwrap().remove("code");
    let mut extra = diagnostic.clone();
    extra["extra"] = true.into();
    let mut sites = diagnostic.clone();
    sites["sites"] = serde_json::json!([{"path": "file"}]);
    let mut authority = diagnostic;
    authority["authority"] = serde_json::json!([42]);
    for (line, expected) in [
        (long.to_string(), "message` is too long"),
        (missing.to_string(), "key set"),
        (extra.to_string(), "key set"),
        (sites.to_string(), "sites` entry"),
        (authority.to_string(), "authority` entry"),
        ("{invalid".into(), "not valid JSON"),
    ] {
        rejects(
            || judge(args, false, &format!("{}{line}", listing())),
            expected,
        );
    }
    rejects(
        || {
            judge(
                args,
                false,
                &format!("{}hint: {}", listing(), "x".repeat(181)),
            )
        },
        "too long",
    );
}
