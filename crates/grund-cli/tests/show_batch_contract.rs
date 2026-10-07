//! Black-box contract for one-context batch declaration reads
//! (§FS-show.1, §FS-show.2.6, §FS-output-shapes.4.1, §AR-resolver.3.1).
//!
//! The tests that count workspace loads read an observer only the
//! `test-workspace-load-count` feature compiles in, so a build without it
//! ignores them, naming the feature, and every other assertion of the same
//! case sits in a test that runs in every build (§AR-ci.3.3).

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Self {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/show-batch-contract")
            .join(format!("{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create batch fixture");
        fs::create_dir_all(root.join("member/docs")).expect("create batch member fixture");

        write(
            &root.join("grund.toml"),
            "grund_config_version = 1\nproject_name = \"root\"\n\n\
             [reference]\nstrict = false\n\n\
             [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
             [scan]\ninclude = [\"docs\"]\n\n\
             [workspace]\nmembers = [\"member\"]\n",
        );
        write(
            &root.join("member/grund.toml"),
            "grund_config_version = 1\nproject_name = \"member\"\n\n\
             [reference]\nstrict = false\n\n\
             [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
             [scan]\ninclude = [\"docs\"]\n\n\
             [fmt.cross_refs]\nanchor_format = \"mkdocs\"\n",
        );
        write(
            &root.join("docs/FS-alpha.md"),
            "# FS-alpha: Alpha\n\nAlpha lead.\n\nAlpha second paragraph.\n\n\
             ## 1. Numeric\n\nNumeric lead.\n\n\
             ### 1.1 Child\n\nChild body.\n\n\
             ## goals: Named\n\nNamed body.\n",
        );
        write(
            &root.join("member/docs/FS-beta.md"),
            "# FS-beta: Beta\n\nBeta lead.\n\n## 2. Member — section\n\nMember body.\n",
        );
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn load_log(&self, name: &str) -> PathBuf {
        self.0.join(format!("{name}.loads"))
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, body: &str) {
    fs::write(path, body).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn run_batch(repo: &Repo, extra_args: &[&str], stdin: &str, load_log: Option<&Path>) -> Output {
    run_batch_at(repo.path(), extra_args, stdin, load_log)
}

fn run_batch_at(repo: &Path, extra_args: &[&str], stdin: &str, load_log: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
    command
        .args(["show", "--batch", "--format=json"])
        .args(extra_args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = load_log {
        command.env("GRUND_TEST_WORKSPACE_LOAD_LOG", path);
    }
    let mut child = command.spawn().expect("run grund show --batch");
    let write_result = child
        .stdin
        .take()
        .expect("batch stdin")
        .write_all(stdin.as_bytes());
    if let Err(error) = write_result
        && error.kind() != std::io::ErrorKind::BrokenPipe
    {
        panic!("write batch stdin: {error}");
    }
    child.wait_with_output().expect("read batch output")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn records(output: &Output) -> Vec<Value> {
    stdout(output)
        .lines()
        .map(|line| serde_json::from_str(line).expect("batch stdout record is JSON"))
        .collect()
}

/// Reads the workspace-load observer's log. Only a test ignored without
/// `test-workspace-load-count` may call this: in a build without the feature
/// nothing writes the log, and an unwritten log reads as zero loads (§AR-ci.3.3).
fn loads(path: &Path) -> usize {
    fs::read_to_string(path).unwrap_or_default().lines().count()
}

/// One NDJSON envelope per query in query order, the failure carried inside its
/// own envelope with stderr empty, and the aggregate exiting `1` after every
/// record has been emitted (§FS-show.3.3).
#[test]
fn show_batch_ambiguous_shorthand_is_a_query_failure_and_continues() {
    // §FS-show.2.6.3: an ambiguous shorthand is one failed coordinate, not a
    // run-level error that suppresses the records after it.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-shorthand-citation-ambiguous-and-unknown/repo");
    let output = run_batch_at(
        &repo,
        &[],
        concat!("{\"id\":\"FS-042\"}\n", "{\"id\":\"FS-042-user-login\"}\n",),
        None,
    );

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        concat!(
            "{\"query\":{\"id\":\"FS-042\",\"section\":null},\"ok\":false,\"result\":null,\"error\":{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"ambiguous\",\"message\":\"ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)\",\"sites\":null,\"authority\":null}}\n",
            "{\"query\":{\"id\":\"FS-042-user-login\",\"section\":null},\"ok\":true,\"result\":{\"id\":\"FS-042-user-login\",\"section\":null,\"body\":\"Cited as §FS-042-user-login.\\n\",\"kind_title\":\"What: behavior, requirements, and constraints\",\"anchor\":\"fs-042-user-login-user-login\",\"path\":\"docs/functional-spec/FS-042-user-login.md\",\"line\":1},\"error\":null}\n",
        )
    );
    assert_eq!(stderr(&output), "");
}

/// An explicit query is answered by the same default/`--brief`/`--toc`/`--full`
/// renderer as single-coordinate `show`, and the one invocation-level mode
/// applies to every record (§FS-show.2.6.1).
#[test]
fn show_batch_all_four_modes_use_single_show_rendering() {
    let repo = Repo::new("modes");
    let input = "{\"id\":\"FS-alpha\",\"section\":null}\n";
    let cases = [
        (None, "Alpha lead.\n\nAlpha second paragraph.\n", false),
        (Some("--brief"), "# FS-alpha: Alpha\n\nAlpha lead.\n", false),
        (
            Some("--toc"),
            "Alpha lead.\n\nAlpha second paragraph.\n\n## 1. Numeric\n### 1.1 Child\n## goals: Named\n",
            true,
        ),
        (
            Some("--full"),
            "Alpha lead.\n\nAlpha second paragraph.\n\n## 1. Numeric\n\nNumeric lead.\n\n### 1.1 Child\n\nChild body.\n\n## goals: Named\n\nNamed body.\n",
            false,
        ),
    ];

    for (flag, body, has_sections) in cases {
        let args = flag.into_iter().collect::<Vec<_>>();
        let output = run_batch(&repo, &args, input, None);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{flag:?} stderr:\n{}",
            stderr(&output)
        );
        let records = records(&output);
        assert_eq!(records.len(), 1, "{flag:?}");
        assert_eq!(records[0]["result"]["body"], body, "{flag:?}");
        assert_eq!(
            records[0]["result"].get("sections").is_some(),
            has_sections,
            "{flag:?}"
        );
    }
}

const MALFORMED_INPUT: &str = concat!(
    "{\"id\":\"FS-alpha\"}\n",
    "{\"id\":\"FS-alpha\",\"extra\":true}\n",
);

const MANY_QUERIES_INPUT: &str = concat!(
    "{\"id\":\"FS-alpha\"}\n",
    "{\"id\":\"FS-alpha\",\"section\":\"1\"}\n",
    "{\"id\":\"member/FS-beta\"}\n",
);

fn assert_malformed_rejected(output: &Output) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(output), "");
    assert_eq!(
        stderr(output),
        "error: batch input line 2: unknown field `extra`\n"
    );
}

fn assert_empty_succeeded(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(output), "");
    assert_eq!(stderr(output), "");
}

fn assert_many_queries_succeeded(explicit: &Output, all: &Output) {
    for (form, output) in [("explicit", explicit), ("--all", all)] {
        assert_eq!(
            output.status.code(),
            Some(0),
            "{form} stderr:\n{}",
            stderr(output)
        );
        assert_eq!(stderr(output), "", "{form}");
    }
}

/// Malformed batch input is a run-level failure: exit `2`, empty stdout, the
/// error on stderr, and no envelope at all (§FS-show.3.3, §AR-ci.3.3).
#[test]
fn show_batch_rejects_the_whole_malformed_stream() {
    let repo = Repo::new("malformed");
    assert_malformed_rejected(&run_batch(&repo, &[], MALFORMED_INPUT, None));
}

/// The same malformed stream is rejected before any workspace is loaded; the
/// load count is the test-only observer's, so a build without its feature
/// ignores this half (§FS-show.3.3, §AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the workspace-load observer; run with --features grund/test-workspace-load-count"
)]
fn show_batch_rejects_the_whole_malformed_stream_before_scanning() {
    let repo = Repo::new("malformed-loads");
    let log = repo.load_log("malformed");
    assert_malformed_rejected(&run_batch(&repo, &[], MALFORMED_INPUT, Some(&log)));
    assert_eq!(loads(&log), 0, "malformed input must be rejected pre-scan");
}

/// An empty explicit stream is one of the aggregate's success cases: exit `0`
/// with no envelopes on either stream (§FS-show.3.3, §AR-ci.3.3).
#[test]
fn show_batch_empty_input_is_a_successful_noop() {
    let repo = Repo::new("empty");
    assert_empty_succeeded(&run_batch(&repo, &[], "", None));
}

/// The same empty stream loads no workspace; the load count is the test-only
/// observer's, so a build without its feature ignores this half
/// (§FS-show.3.3, §AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the workspace-load observer; run with --features grund/test-workspace-load-count"
)]
fn show_batch_empty_input_is_a_successful_no_scan_noop() {
    let repo = Repo::new("empty-loads");
    let log = repo.load_log("empty");
    assert_empty_succeeded(&run_batch(&repo, &[], "", Some(&log)));
    assert_eq!(loads(&log), 0);
}

/// An explicit stream of several queries, one of them in a workspace member,
/// and `--batch --all` both succeed: exit `0` with stderr empty
/// (§FS-show.1.9, §FS-show.3.3, §AR-ci.3.3).
#[test]
fn show_batch_succeeds_for_many_queries_and_for_all() {
    let repo = Repo::new("many");
    let explicit = run_batch(&repo, &[], MANY_QUERIES_INPUT, None);
    let all = run_batch(&repo, &["--all"], "", None);
    assert_many_queries_succeeded(&explicit, &all);
}

/// `--batch --all` takes no stdin and discovers its query set from the selected
/// scope, answering it from the same single workspace load an explicit stream
/// gets (§FS-show.1.9). The load count is the test-only observer's, so a build
/// without its feature ignores this half (§AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the workspace-load observer; run with --features grund/test-workspace-load-count"
)]
fn show_batch_loads_one_workspace_for_many_queries_and_for_all() {
    let repo = Repo::new("load-count");
    let explicit_log = repo.load_log("explicit");
    let all_log = repo.load_log("all");
    let explicit = run_batch(&repo, &[], MANY_QUERIES_INPUT, Some(&explicit_log));
    let all = run_batch(&repo, &["--all"], "", Some(&all_log));

    assert_eq!(
        (loads(&explicit_log), loads(&all_log)),
        (1, 1),
        "expected one workspace-loader entry for explicit and exhaustive batches; \
         statuses were {:?} and {:?}; stderr was {:?} and {:?}",
        explicit.status.code(),
        all.status.code(),
        stderr(&explicit),
        stderr(&all)
    );
    assert_many_queries_succeeded(&explicit, &all);
}

fn run_single(repo: &Repo, coordinate: &str, extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["show", coordinate, "--format=json"])
        .args(extra_args)
        .current_dir(repo.path())
        .output()
        .expect("run grund show")
}

/// The raw bytes of a successful envelope's `result`: everything between its
/// `"result":` key and the closing `,"error":null}` (§FS-output-shapes.4.1).
fn raw_result(envelope: &str) -> &str {
    let start = envelope.find(",\"result\":").expect("envelope has result") + ",\"result\":".len();
    let end = envelope
        .strip_suffix(",\"error\":null}")
        .expect("successful envelope closes on a null error")
        .len();
    &envelope[start..end]
}

/// The keys of a JSON object line in wire order, read off the bytes rather
/// than through a map that might reorder them.
fn top_level_keys(object: &str) -> Vec<String> {
    let value: Value = serde_json::from_str(object).expect("result is JSON");
    let map = value.as_object().expect("result is an object");
    let mut keys: Vec<(usize, String)> = map
        .keys()
        .map(|key| {
            let needle = format!("\"{key}\":");
            let mut depth = 0usize;
            let bytes = object.as_bytes();
            let mut in_string = false;
            let mut escaped = false;
            let mut found = None;
            for (index, &byte) in bytes.iter().enumerate() {
                if in_string {
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        in_string = false;
                    }
                    continue;
                }
                match byte {
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => depth -= 1,
                    b'"' => {
                        if depth == 1 && object[index..].starts_with(&needle) {
                            found = Some(index);
                            break;
                        }
                        in_string = true;
                    }
                    _ => {}
                }
            }
            (found.expect("key found at top level"), key.clone())
        })
        .collect();
    keys.sort();
    keys.into_iter().map(|(_, key)| key).collect()
}

/// Every successful read, single or batch, carries the selected heading's
/// anchor before the closing `path`, `line` pair, each `--toc` entry carries
/// its own, and a member read takes the member's `anchor_format` profile
/// (§FS-show.3.1.3.1, §FS-output-shapes.4, §FS-output-shapes.4.1).
#[test]
fn show_batch_results_equal_single_reads_and_carry_the_heading_anchor() {
    let repo = Repo::new("anchor");
    let coordinates = [
        ("FS-alpha", None, "fs-alpha-alpha"),
        ("FS-alpha", Some("1"), "1-numeric"),
        ("FS-alpha", Some("1.1"), "11-child"),
        ("FS-alpha", Some("goals"), "goals-named"),
        ("member/FS-beta", None, "fs-beta-beta"),
        ("member/FS-beta", Some("2"), "2-member-section"),
    ];
    let input: String = coordinates
        .iter()
        .map(|(id, section, _)| match section {
            Some(section) => format!("{{\"id\":\"{id}\",\"section\":\"{section}\"}}\n"),
            None => format!("{{\"id\":\"{id}\"}}\n"),
        })
        .collect();
    for flag in [None, Some("--brief"), Some("--toc"), Some("--full")] {
        let args = flag.into_iter().collect::<Vec<_>>();
        let batch = run_batch(&repo, &args, &input, None);
        assert_eq!(batch.status.code(), Some(0), "{flag:?}: {}", stderr(&batch));
        let batch_out = stdout(&batch);
        let envelopes: Vec<&str> = batch_out.lines().collect();
        assert_eq!(envelopes.len(), coordinates.len(), "{flag:?}");
        for ((id, section, anchor), envelope) in coordinates.iter().zip(&envelopes) {
            let coordinate = match section {
                Some(section) => format!("{id}.{section}"),
                None => id.to_string(),
            };
            let single = run_single(&repo, &coordinate, &args);
            assert_eq!(single.status.code(), Some(0), "{coordinate} {flag:?}");
            let single_out = stdout(&single);
            let single_line = single_out.trim_end_matches('\n');
            assert_eq!(raw_result(envelope), single_line, "{coordinate} {flag:?}");

            let mut expected_keys = vec!["id", "section", "body"];
            if flag == Some("--toc") {
                expected_keys.push("sections");
            }
            expected_keys.extend(["kind_title", "anchor", "path", "line"]);
            assert_eq!(
                top_level_keys(single_line),
                expected_keys,
                "{coordinate} {flag:?}: {single_line}"
            );
            let value: Value = serde_json::from_str(single_line).unwrap();
            assert_eq!(value["anchor"], *anchor, "{coordinate} {flag:?}");
        }
    }

    let toc = run_single(&repo, "FS-alpha", &["--toc"]);
    let toc: Value = serde_json::from_str(stdout(&toc).trim_end()).unwrap();
    assert_eq!(
        toc["sections"],
        serde_json::json!([
            {"path": "1", "title": "Numeric", "depth": 1, "anchor": "1-numeric"},
            {"path": "1.1", "title": "Child", "depth": 2, "anchor": "11-child"},
            {"path": "goals", "title": "Named", "depth": 1, "anchor": "goals-named"},
        ])
    );

    let all = run_batch(&repo, &["--all"], "", None);
    assert_eq!(all.status.code(), Some(0), "--all: {}", stderr(&all));
    let all_out = stdout(&all);
    for envelope in all_out.lines() {
        let record: Value = serde_json::from_str(envelope).unwrap();
        let id = record["query"]["id"].as_str().unwrap();
        let coordinate = match record["query"]["section"].as_str() {
            Some(section) => format!("{id}.{section}"),
            None => id.to_string(),
        };
        let single = run_single(&repo, &coordinate, &[]);
        assert_eq!(
            raw_result(envelope),
            stdout(&single).trim_end_matches('\n'),
            "--all {coordinate}"
        );
        assert!(
            record["result"]["anchor"].is_string(),
            "--all {coordinate}: {envelope}"
        );
    }
}
