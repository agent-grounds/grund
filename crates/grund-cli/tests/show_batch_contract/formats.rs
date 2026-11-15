//! The batch contract in every format (§FS-show.1.8, §FS-show.3.3): the aggregate
//! cases the `json` tests pin, run again in `text` and `md`, where each answer is
//! headed `==> <coordinate> <==` above the single read's bytes (§FS-show.3.3.1) and
//! a failure keeps its message and hint in its place on stdout (§FS-show.3.3.2),
//! every hint read from the project the query names out of the one load
//! (§FS-show.3.5, §AR-resolver.3.1.1).
//!
//! Every test here is red until #1385 lets `--batch` answer in a format other
//! than `json`; the implementing change removes each `ignore`. The renderer they
//! pin has no function of its own yet, so the unit-level asks — the body under a
//! header is the single read, the header's spelling, one hint for a single read
//! and a batch — are pinned here at the process boundary, against single reads
//! of the same binary.

use super::{
    MALFORMED_INPUT, MANY_QUERIES_INPUT, NEXT_ROOT, Repo, assert_empty_succeeded,
    assert_malformed_rejected, assert_many_queries_succeeded, loads, observed, run_batch_in,
    stderr, stdout, write,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::Ordering;

/// Every format a batch answers in (§FS-show.1.8).
const FORMATS: [&str; 3] = ["json", "text", "md"];
/// The two formats that frame each answer under a header (§FS-show.3.3.1).
const FRAMED: [&str; 2] = ["text", "md"];

/// The stream §FS-show.3.3.1 specifies: each record's header and bytes, and one
/// empty line before every header but the first, the previous record's bytes
/// given a line break where they lack one.
fn framed(records: &[(String, String)]) -> String {
    let mut out = String::new();
    for (coordinate, bytes) in records {
        if !out.is_empty() {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(&format!("==> {coordinate} <==\n{bytes}"));
    }
    out
}

/// What the single read `grund show <read>` prints in `format` and `mode`: its
/// stdout when it succeeds, else its stderr, the message and hint a batch keeps
/// in place.
fn single(dir: &Path, read: &[&str], format: &str, mode: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("show")
        .args(read)
        .arg(format!("--format={format}"))
        .args(mode)
        .current_dir(dir)
        .output()
        .expect("run grund show");
    match output.status.code() {
        Some(0) => stdout(&output),
        Some(1) => stderr(&output),
        code => panic!("{read:?}: single read exited {code:?}: {}", stderr(&output)),
    }
}

fn assert_framed(output: &Output, code: i32, expected: &str, context: &str) {
    assert_eq!(
        (
            output.status.code(),
            stdout(output).as_str(),
            stderr(output).as_str()
        ),
        (Some(code), expected, ""),
        "{context}"
    );
}

/// The aggregate is the same in every format: a malformed stream is refused
/// whole, an empty one succeeds silently, and many queries or `--all` succeed
/// with stderr empty (§FS-show.3.3, §FS-show.2.6.3).
#[test]
#[ignore = "red until #1385 lets --batch answer in text and md; implement removes this"]
fn show_batch_aggregate_is_the_same_in_every_format() {
    let repo = Repo::new("formats-aggregate");
    for format in FORMATS {
        assert_malformed_rejected(&run_batch_in(repo.path(), format, &[], MALFORMED_INPUT));
        assert_empty_succeeded(&run_batch_in(repo.path(), format, &[], ""));
        let explicit = run_batch_in(repo.path(), format, &[], MANY_QUERIES_INPUT);
        let all = run_batch_in(repo.path(), format, &["--all"], "");
        assert_many_queries_succeeded(&explicit, &all);
    }
}

/// Every format answers from the one load an explicit stream or `--all` makes,
/// and loads nothing for a malformed or empty stream (§FS-show.1.9,
/// §FS-show.3.3). The count is the test-only observer's (§AR-ci.3.3).
#[test]
#[cfg_attr(
    not(feature = "test-workspace-load-count"),
    ignore = "reads the workspace-load observer; run with --features grund/test-workspace-load-count"
)]
#[cfg_attr(
    feature = "test-workspace-load-count",
    ignore = "red until #1385 lets --batch answer in text and md; implement removes this"
)]
fn show_batch_loads_one_workspace_in_every_format() {
    let repo = Repo::new("formats-loads");
    for format in FORMATS {
        let run = |name: &str, args: &[&str], stdin: &str| {
            let log = repo.load_log(&format!("{format}-{name}"));
            let output = observed::run_observed(repo.path(), format, args, stdin, &log);
            (loads(&log), output)
        };
        let (malformed, output) = run("malformed", &[], MALFORMED_INPUT);
        assert_malformed_rejected(&output);
        let (empty, output) = run("empty", &[], "");
        assert_empty_succeeded(&output);
        let (explicit, explicit_output) = run("explicit", &[], MANY_QUERIES_INPUT);
        let (all, all_output) = run("all", &["--all"], "");
        assert_many_queries_succeeded(&explicit_output, &all_output);
        assert_eq!((malformed, empty, explicit, all), (0, 0, 1, 1), "{format}");
    }
}

/// An ambiguous shorthand is one failed query, kept in its place under the query
/// as written, and the query after it is still answered, headed by the full ID;
/// the aggregate exits `1` with stderr empty (§FS-show.2.6.3, §FS-show.3.3.2).
#[test]
#[ignore = "red until #1385 lets --batch answer in text and md; implement removes this"]
fn show_batch_framed_ambiguous_shorthand_is_a_query_failure_and_continues() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-shorthand-citation-ambiguous-and-unknown/repo");
    let input = "{\"id\":\"FS-042\"}\n{\"id\":\"FS-042-user-login\"}\n";
    let failure = "ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)\n";
    for (format, body) in [
        ("text", "Cited as \u{a7}FS-042-user-login.\n"),
        (
            "md",
            "# FS-042-user-login: User login\n\nCited as \u{a7}FS-042-user-login.\n",
        ),
    ] {
        let expected = format!("==> FS-042 <==\n{failure}\n==> FS-042-user-login <==\n{body}");
        let output = run_batch_in(&repo, format, &[], input);
        assert_framed(&output, 1, &expected, format);
    }
}

/// Under each header is exactly what the single read prints for that coordinate
/// in the same format and mode, and the header spells the coordinate as `--all`
/// generates it: a member's with its alias, the current project's alias dropped
/// (§FS-show.3.3, §FS-show.3.3.1, §FS-show.2.6.2).
#[test]
#[ignore = "red until #1385 lets --batch answer in text and md; implement removes this"]
fn show_batch_framed_bodies_equal_single_reads() {
    let repo = Repo::new("formats-bodies");
    // (query id, query section, the header, the coordinate the single read takes)
    let queries = [
        ("FS-alpha", None, "FS-alpha", "FS-alpha"),
        ("FS-alpha", Some("1"), "FS-alpha.1", "FS-alpha.1"),
        ("FS-alpha", Some("1.1"), "FS-alpha.1.1", "FS-alpha.1.1"),
        (
            "FS-alpha",
            Some("goals"),
            "FS-alpha.goals",
            "FS-alpha.goals",
        ),
        ("root/FS-alpha", Some("1"), "FS-alpha.1", "root/FS-alpha.1"),
        ("member/FS-beta", None, "member/FS-beta", "member/FS-beta"),
        (
            "member/FS-beta",
            Some("2"),
            "member/FS-beta.2",
            "member/FS-beta.2",
        ),
    ];
    let input: String = queries
        .iter()
        .map(|(id, section, _, _)| match section {
            Some(section) => format!("{{\"id\":\"{id}\",\"section\":\"{section}\"}}\n"),
            None => format!("{{\"id\":\"{id}\"}}\n"),
        })
        .collect();
    // `--all` generates every coordinate once, in declaration order.
    let generated = [0, 1, 2, 3, 5, 6].map(|index| queries[index]);
    let modes: [&[&str]; 4] = [&[], &["--brief"], &["--toc"], &["--full"]];
    for format in FRAMED {
        for mode in modes {
            let answer = |(_, _, header, coordinate): &(&str, Option<&str>, &str, &str)| {
                (
                    header.to_string(),
                    single(repo.path(), &[coordinate], format, mode),
                )
            };
            let expected = framed(&queries.iter().map(answer).collect::<Vec<_>>());
            let output = run_batch_in(repo.path(), format, mode, &input);
            assert_framed(&output, 0, &expected, &format!("{format} {mode:?}"));

            let expected = framed(&generated.iter().map(answer).collect::<Vec<_>>());
            let mut all_args = vec!["--all"];
            all_args.extend_from_slice(mode);
            let output = run_batch_in(repo.path(), format, &all_args, "");
            assert_framed(&output, 0, &expected, &format!("{format} --all {mode:?}"));
        }
    }
}

/// A failed query is headed by the query as written, alias kept and a separate
/// section joined by the run's separator, and under it come the message and
/// `hint:` lines of the single read that fails the same way, a separate section
/// read with `--section`, on stdout; an unknown alias keeps its message and earns
/// no hint (§FS-show.3.3.1, §FS-show.3.3.2, §FS-show.3.5).
#[test]
#[ignore = "red until #1385 lets --batch answer in text and md; implement removes this"]
fn show_batch_framed_failure_keeps_the_single_read_hint_in_place() {
    let repo = Repo::new("formats-failures");
    // (query line, the header, the single read that fails the same way): a
    // separate `section` is the read's `--section`, an inline one its operand's.
    let failures: [(&str, &str, &[&str]); 7] = [
        ("{\"id\":\"FS-nope\"}", "FS-nope", &["FS-nope"]),
        (
            "{\"id\":\"FS-alpha\",\"section\":\"9.9\"}",
            "FS-alpha.9.9",
            &["FS-alpha", "--section", "9.9"],
        ),
        (
            "{\"id\":\"FS-alpha.9.9\"}",
            "FS-alpha.9.9",
            &["FS-alpha.9.9"],
        ),
        ("{\"id\":\"FS-alpha.9\"}", "FS-alpha.9", &["FS-alpha.9"]),
        (
            "{\"id\":\"root/FS-nope\"}",
            "root/FS-nope",
            &["root/FS-nope"],
        ),
        (
            "{\"id\":\"member/FS-beta\",\"section\":\"9\"}",
            "member/FS-beta.9",
            &["member/FS-beta", "--section", "9"],
        ),
        ("{\"id\":\"not-an-id\"}", "not-an-id", &["not-an-id"]),
    ];
    let mut input: String = failures
        .iter()
        .map(|(line, _, _)| format!("{line}\n"))
        .collect();
    input.push_str("{\"id\":\"missing/FS-alpha\"}\n");
    for format in FRAMED {
        let mut records: Vec<(String, String)> = failures
            .iter()
            .map(|(_, header, read)| (header.to_string(), single(repo.path(), read, format, &[])))
            .collect();
        records.push((
            "missing/FS-alpha".to_string(),
            "unknown project alias `missing`\nknown aliases: root, member\n".to_string(),
        ));
        let output = run_batch_in(repo.path(), format, &[], &input);
        assert_framed(&output, 1, &framed(&records), format);
    }
}

/// A workspace whose member `api` takes another ID format and the separator `:`,
/// built under the batch fixtures' target directory (§FS-config.3.2).
fn member_format_repo() -> Repo {
    let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/show-batch-contract")
        .join(format!("member-format-{}-{serial}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).expect("create the root fixture");
    fs::create_dir_all(root.join("api/docs")).expect("create the member fixture");
    let kinds = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
                 [scan]\ninclude = [\"docs\"]\n";
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\n\n{kinds}\n\
             [workspace]\nmembers = [\"api\"]\n"
        ),
    );
    write(
        &root.join("api/grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"api\"\n\n\
             [id]\nformat = \"{{kind}}-{{number}}-{{slug}}\"\nsection_separator = \":\"\n\n\
             {kinds}"
        ),
    );
    write(
        &root.join("docs/FS-root.md"),
        "# FS-root: Root\n\nRoot lead.\n\n## 1. One\n\nRoot one.\n",
    );
    write(
        &root.join("api/docs/FS-001-api.md"),
        "# FS-001-api: Api\n\nApi lead.\n\n## 1. One\n\nApi one.\n",
    );
    Repo(root)
}

const SECTION_HINT_ROOT: &str =
    "hint: run `grund FS-root --toc` to print the lead with the section map\n";
const SECTION_HINT_API: &str =
    "hint: run `grund api/FS-001-api --toc` to print the lead with the section map\n";

fn format_hint(format: &str) -> String {
    format!(
        "hint: this repo's [id] format is `{format}` (run `grund config show`); \
         `grund list` shows the IDs that exist\n"
    )
}

/// Each hint is read from the project the query names: an invalid ID in the
/// member names the member's ID format, a missing section there is cut at the
/// member's `:`, a shorthand's header is the full ID, and a failed query with a
/// separate section is headed with the run's separator (§FS-show.3.3.1,
/// §FS-show.3.3.2, §FS-show.3.5, §FS-show.3.5.1).
#[test]
#[ignore = "red until #1385 lets --batch answer in text with each hint from the owning project; implement removes this"]
fn show_batch_text_reads_each_hint_from_the_project_the_query_names() {
    let repo = member_format_repo();
    let input = concat!(
        "{\"id\":\"FS-root\",\"section\":\"1\"}\n",
        "{\"id\":\"FS-root\",\"section\":\"9\"}\n",
        "{\"id\":\"root-bad\"}\n",
        "{\"id\":\"api/FS-001\",\"section\":\"1\"}\n",
        "{\"id\":\"api/FS-api\"}\n",
        "{\"id\":\"api/FS-001-api:9\"}\n",
        "{\"id\":\"api/FS-001-api\",\"section\":\"9\"}\n",
    );
    let expected = [
        "==> FS-root.1 <==\n## 1. One\n\nRoot one.\n".to_string(),
        format!("==> FS-root.9 <==\nsection not found: FS-root.9\n{SECTION_HINT_ROOT}"),
        format!(
            "==> root-bad <==\ninvalid ID `root-bad`\n{}",
            format_hint("{kind}-{slug}")
        ),
        "==> api/FS-001-api:1 <==\n## 1. One\n\nApi one.\n".to_string(),
        format!(
            "==> api/FS-api <==\ninvalid ID `FS-api`\n{}",
            format_hint("{kind}-{number}-{slug}")
        ),
        format!("==> api/FS-001-api:9 <==\nsection not found: FS-001-api:9\n{SECTION_HINT_API}"),
        format!("==> api/FS-001-api.9 <==\nsection not found: FS-001-api:9\n{SECTION_HINT_API}"),
    ]
    .join("\n");
    let output = run_batch_in(repo.path(), "text", &[], input);
    assert_framed(&output, 1, &expected, "text");
}

/// The single read cuts a missing section's hint at the separator of the project
/// the query names, so `api/FS-001-api:9` and its shorthand `api/FS-001:9` each
/// get a `--toc` command that answers (§FS-show.3.5).
#[test]
#[ignore = "red until #1385 cuts the missing-section hint at the owning project's separator; implement removes this"]
fn show_single_read_cuts_a_missing_section_hint_at_the_member_separator() {
    let repo = member_format_repo();
    for (coordinate, hinted) in [
        ("api/FS-001-api:9", "api/FS-001-api"),
        ("api/FS-001:9", "api/FS-001"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["show", coordinate])
            .current_dir(repo.path())
            .output()
            .expect("run grund show");
        let expected = format!(
            "section not found: FS-001-api:9\n\
             hint: run `grund {hinted} --toc` to print the lead with the section map\n"
        );
        assert_eq!(
            (output.status.code(), stdout(&output), stderr(&output)),
            (Some(1), String::new(), expected),
            "{coordinate}"
        );
    }
}

/// A batch prints for each failure the lines the single read prints for it, in
/// either framed format, so there is one hint for a single read and a batch
/// (§FS-show.3.3.2, §FS-show.3.5).
#[test]
#[ignore = "red until #1385 lets --batch answer in text and md; implement removes this"]
fn show_batch_framed_hint_equals_the_single_read_hint() {
    let repo = member_format_repo();
    let failures = [
        "FS-root.9",
        "root-bad",
        "FS-nope",
        "api/FS-api",
        "api/FS-001-api:9",
        "api/FS-001:9",
        "api/FS-002-nope",
    ];
    let input: String = failures
        .iter()
        .map(|coordinate| format!("{{\"id\":\"{coordinate}\"}}\n"))
        .collect();
    for format in FRAMED {
        let records: Vec<(String, String)> = failures
            .iter()
            .map(|coordinate| {
                let failure = single(repo.path(), &[coordinate], format, &[]);
                (coordinate.to_string(), failure)
            })
            .collect();
        let output = run_batch_in(repo.path(), format, &[], &input);
        assert_framed(&output, 1, &framed(&records), format);
    }
}
