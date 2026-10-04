// §FS-refs.4 / §FS-errors.5.2: the two resolver rejections share one exit and
// wire contract across refs, show, text, JSON, and rendering flags.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const FORMAT_HINT: &str = "hint: this repo's [id] format is `{kind}-{number}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist\n";
const INVALID: &str = "invalid ID `FS-bar`";
const AMBIGUOUS: &str = "ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)";

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(test: &str) -> Self {
        let unique = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "grund-refs-query-failure-{test}-{}-{unique}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale fixture");
        }
        fs::create_dir_all(root.join("docs")).expect("create fixture docs");
        fs::write(
            root.join("grund.toml"),
            "grund_config_version = 1\n\
             project_name = \"root\"\n\n\
             [reference]\n\
             strict = false\n\n\
             [id]\n\
             format = \"{kind}-{number}-{slug}\"\n\n\
             [scan]\n\
             include = [\"docs\"]\n",
        )
        .expect("write fixture config");
        fs::write(
            root.join("docs/FS-042-user-login.md"),
            "# FS-042-user-login: User login\n\nLogin.\n",
        )
        .expect("write login declaration");
        fs::write(
            root.join("docs/FS-042-user-logout.md"),
            "# FS-042-user-logout: User logout\n\nLogout.\n",
        )
        .expect("write logout declaration");
        fs::write(
            root.join("docs/FS-100-empty.md"),
            "# FS-100-empty: No inbound citations\n\nEmpty is a valid answer.\n",
        )
        .expect("write uncited declaration");
        Self { root }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run(root: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
    command.args(args).arg(root);
    command.output().expect("run grund")
}

fn status(output: &Output) -> i32 {
    output.status.code().expect("grund exited by status")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr is UTF-8")
}

/// §FS-output-shapes.6.1.2: the bare failed-query text, exit `1`; only an
/// invalid format keeps the configured-format hint. No release suffix and no
/// warning follow it: the landed half of the scalar clause is these ordinary
/// bytes (§FS-distribution.4.2.4).
fn expected_refs_text(message: &str, hint: bool) -> (i32, String) {
    let mut rendered = format!("{message}\n");
    if hint {
        rendered.push_str(FORMAT_HINT);
    }
    (1, rendered)
}

/// §FS-output-shapes.6.1.2: the one failed-query object, exit `1`, no hint.
fn expected_refs_json(message: &str, code: &str) -> (i32, String) {
    (
        1,
        format!(
            "{{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"{code}\",\"message\":\"{message}\",\"sites\":null,\"authority\":null}}\n"
        ),
    )
}

fn assert_run(output: &Output, expected_status: i32, expected_stderr: &str) {
    assert_eq!(stdout(output), "", "stdout must stay empty");
    assert_eq!(
        status(output),
        expected_status,
        "stderr:\n{}",
        stderr(output)
    );
    assert_eq!(stderr(output), expected_stderr);
}

#[test]
fn configured_format_rejection_is_a_failed_query_in_text_and_json() {
    let fixture = Fixture::new("invalid-format");
    let text = run(&fixture.root, &["refs", "FS-bar"]);
    let expected_text = expected_refs_text(INVALID, true);
    assert_run(&text, expected_text.0, &expected_text.1);

    // `--summary` is a renderer, not a different operand classification.
    let json = run(
        &fixture.root,
        &["refs", "FS-bar", "--summary", "--format", "json"],
    );
    let expected_json = expected_refs_json(INVALID, "invalid-id");
    assert_run(&json, expected_json.0, &expected_json.1);
}

#[test]
fn ambiguous_shorthand_is_a_failed_query_in_text_and_json() {
    let fixture = Fixture::new("ambiguous-shorthand");
    // `--section` is applied only after the operand resolves.
    let text = run(&fixture.root, &["refs", "FS-042", "--section", "1"]);
    let expected_text = expected_refs_text(AMBIGUOUS, false);
    assert_run(&text, expected_text.0, &expected_text.1);

    let json = run(&fixture.root, &["refs", "FS-042", "--format", "json"]);
    let expected_json = expected_refs_json(AMBIGUOUS, "ambiguous");
    assert_run(&json, expected_json.0, &expected_json.1);
}

/// §FS-errors.2.3.2: `show` is one of the ID queries the bare query-failure
/// shape is for — an invalid ID and an ambiguous ID both print the message with
/// no `error:` prefix on stderr, leave stdout empty, and exit `1` — the bytes
/// `refs` now matches.
#[test]
fn show_bytes_and_status_do_not_move_with_refs() {
    let fixture = Fixture::new("show-seam");
    let invalid = run(&fixture.root, &["show", "FS-bar"]);
    assert_run(&invalid, 1, &format!("{INVALID}\n{FORMAT_HINT}"));
    let ambiguous = run(&fixture.root, &["show", "FS-042"]);
    assert_run(&ambiguous, 1, &format!("{AMBIGUOUS}\n"));

    let invalid_json = run(&fixture.root, &["show", "FS-bar", "--format", "json"]);
    assert_run(
        &invalid_json,
        1,
        &format!(
            "{{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"invalid-id\",\"message\":\"{INVALID}\",\"sites\":null,\"authority\":null}}\n"
        ),
    );
}

/// §FS-errors.2.2.2: the exit code each prefix accompanies, read off the two
/// neighbours of a query failure — an unknown project alias and an unscannable
/// tree are launch/run failures, so both keep the `error:` prefix and exit `2`,
/// while a satisfiable query with an empty answer stays silent at exit `0`.
#[test]
fn empty_answer_and_context_failures_keep_their_neighboring_statuses() {
    let fixture = Fixture::new("negative-seams");
    let empty = run(&fixture.root, &["refs", "FS-100-empty"]);
    assert_eq!(status(&empty), 0, "stderr:\n{}", stderr(&empty));
    assert_eq!(stdout(&empty), "");
    assert_eq!(stderr(&empty), "");

    let unknown_alias = run(&fixture.root, &["refs", "other/FS-100-empty"]);
    assert_run(
        &unknown_alias,
        2,
        "error: unknown project alias `other`\n\
         note: workspace aliases are defined in the root grund.toml under [workspace]\n",
    );

    let missing = fixture.root.join("missing-tree");
    let scan_failure = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["refs", "FS-100-empty"])
        .arg(&missing)
        .output()
        .expect("run scan failure");
    assert_eq!(
        status(&scan_failure),
        2,
        "stderr:\n{}",
        stderr(&scan_failure)
    );
    assert_eq!(stdout(&scan_failure), "");
    assert!(
        stderr(&scan_failure).starts_with("error: "),
        "scan failure remains a run-level error: {}",
        stderr(&scan_failure)
    );
    assert!(
        !stderr(&scan_failure).contains("warning:"),
        "no migration warning follows a run-level error: {}",
        stderr(&scan_failure)
    );
}
