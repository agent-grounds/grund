//! `grund show --batch` answers in every `--format`, its `text` and `md` forms heading each
//! answer `==> <coordinate> <==` and keeping a failed query's message and hint in its place on
//! stdout (§FS-show.1.8, §FS-show.3.3, §FS-show.3.3.1, §FS-show.3.3.2). Each test mirrors one
//! golden case under `tests/e2e/cases/` where one exists, with the exact stdout, stderr and
//! exit code, so the bytes are pinned in Rust as well as by the golden.
//! The `json` goldens stay as they are and keep passing, byte for byte (§FS-output-shapes.4.1).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// What one run printed, compared whole so that no stream can pass as the change.
#[derive(Debug, PartialEq)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn expect(code: i32, stdout: &str, stderr: &str) -> Self {
        Self {
            code,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }
}

/// The directory a golden case without `command.cwd` runs in: the CLI crate's own.
fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A case's `{repo}` as the golden harness spells it, relative to [`crate_dir`].
fn case_repo(case: &str) -> String {
    format!("../../tests/e2e/cases/{case}/repo")
}

/// Runs `grund <args>` in `dir`, with `stdin` written to it when given.
fn grund(dir: &Path, args: &[&str], stdin: Option<&str>) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
    command
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("run grund");
    if let Some(input) = stdin {
        let written = child
            .stdin
            .take()
            .expect("grund stdin")
            .write_all(input.as_bytes());
        if let Err(error) = written
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            panic!("write grund stdin: {error}");
        }
    }
    let output = child.wait_with_output().expect("read grund output");
    Run {
        code: output.status.code().expect("exited with a status"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// The stdin `show-batch-mixed-json` pipes, shared by its `text` and `md` mirrors.
const MIXED_STDIN: &str = concat!(
    "{\"id\":\"FS-alpha\",\"section\":null}\n",
    "{\"id\":\"FS-alpha\",\"section\":\"1\"}\n",
    "{\"id\":\"FS-missing\"}\n",
    "{\"id\":\"missing/FS-alpha\",\"section\":null}\n",
    "{\"id\":\"FS-alpha\",\"section\":\"1\"}\n",
);

const NOT_FOUND_HINT: &str = "hint: run `grund list` to see every declared ID, or \
                              `grund id <KIND> \"<title>\"` to propose a new one\n";

/// Mirrors `show-batch-glob-default-text`, which replaces `show-batch-glob-requires-json`:
/// the same arguments with no `--format` answer in `text`, each match under its header
/// (§FS-show.1.8, §FS-show.1.9, §FS-show.3.3.1).
#[test]
fn show_batch_glob_default_text() {
    let repo = case_repo("show-batch-glob-json");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", "--glob", "FS-*.terms", &repo],
        None,
    );
    let stdout = concat!(
        "==> FS-convoy.terms <==\n",
        "## terms: Terms\n\nThe words config uses.\n",
        "\n",
        "==> FS-covey.terms <==\n",
        "## terms: Terms\n\nThe words cover uses.\n",
    );
    assert_eq!(run, Run::expect(0, stdout, ""));
}

/// Mirrors `show-batch-mixed-text`: the stdin of `show-batch-mixed-json` in `text`, every
/// outcome in query order on stdout, a failure's message and hint under its header, the
/// unknown alias with no hint, stderr empty and the aggregate `1` (§FS-show.3.3.2).
#[test]
fn show_batch_mixed_text() {
    let repo = case_repo("show-batch-mixed-json");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", &repo, "--format=text"],
        Some(MIXED_STDIN),
    );
    let stdout = [
        "==> FS-alpha <==\nAlpha lead.\n\nAlpha second paragraph.\n",
        "==> FS-alpha.1 <==\n## 1. Numeric\n\nNumeric lead.\n",
        &format!("==> FS-missing <==\nID not found: FS-missing\n{NOT_FOUND_HINT}"),
        "==> missing/FS-alpha <==\nunknown project alias `missing`\nknown aliases: root, member\n",
        "==> FS-alpha.1 <==\n## 1. Numeric\n\nNumeric lead.\n",
    ]
    .join("\n");
    assert_eq!(run, Run::expect(1, &stdout, ""));
}

/// Mirrors `show-batch-mixed-md`: `md` frames as `text` does, the header above each body's
/// own declaration heading, and a failure prints the same lines as in `text` (§FS-show.3.3.1).
#[test]
fn show_batch_mixed_md() {
    let repo = case_repo("show-batch-mixed-json");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", &repo, "--format=md"],
        Some(MIXED_STDIN),
    );
    let stdout = [
        "==> FS-alpha <==\n# FS-alpha: Alpha\n\nAlpha lead.\n\nAlpha second paragraph.\n",
        "==> FS-alpha.1 <==\n# FS-alpha: Alpha\n## 1. Numeric\n\nNumeric lead.\n",
        &format!("==> FS-missing <==\nID not found: FS-missing\n{NOT_FOUND_HINT}"),
        "==> missing/FS-alpha <==\nunknown project alias `missing`\nknown aliases: root, member\n",
        "==> FS-alpha.1 <==\n# FS-alpha: Alpha\n## 1. Numeric\n\nNumeric lead.\n",
    ]
    .join("\n");
    assert_eq!(run, Run::expect(1, &stdout, ""));
}

/// Mirrors `show-batch-all-text`: `--all --brief` with no `--format`, every generated
/// coordinate under its header, a member's qualified (§FS-show.2.6.2, §FS-show.3.3.1).
#[test]
fn show_batch_all_text() {
    let repo = case_repo("show-batch-all-json");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", "--all", &repo, "--brief"],
        None,
    );
    let stdout = [
        "==> FS-alpha <==\n# FS-alpha: Alpha\n\nAlpha lead.\n",
        "==> FS-alpha.1 <==\n## 1. Numeric\n\nNumeric lead.\n",
        "==> FS-alpha.1.1 <==\n### 1.1 Child\n\nChild body.\n",
        "==> FS-alpha.goals <==\n## goals: Named\n\nNamed body.\n",
        "==> member/FS-beta <==\n# FS-beta: Beta\n\nBeta lead.\n",
        "==> member/FS-beta.2 <==\n## 2. Member section\n\nMember body.\n",
    ]
    .join("\n");
    assert_eq!(run, Run::expect(0, &stdout, ""));
}

/// Mirrors `workspace-show-batch-glob-qualified-text`: two members' bodies that both open
/// `## 1. One` are told apart by their headers, the member's spelled with its alias and its
/// own separator `:`, the coordinate a reader pastes back (§FS-show.3.3.1).
#[test]
fn workspace_show_batch_glob_qualified_text() {
    let repo = case_repo("workspace-show-batch-glob-qualified");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", "--glob", "*/FS-*.1", &repo],
        None,
    );
    let stdout = concat!(
        "==> FS-root.1 <==\n## 1. One\n\nRoot one.\n",
        "\n",
        "==> api/FS-api:1 <==\n## 1. One\n\nApi one.\n",
    );
    assert_eq!(run, Run::expect(0, stdout, ""));
}

/// Mirrors `show-batch-format-unknown`: `--batch` refuses an unknown format as the single
/// read refuses it, exit `2` with empty stdout (§FS-show.1.8).
#[test]
fn show_batch_format_unknown() {
    let repo = case_repo("show-batch-mixed-json");
    let run = grund(
        &crate_dir(),
        &["show", "--batch", "--all", &repo, "--format", "yaml"],
        None,
    );
    assert_eq!(
        run,
        Run::expect(2, "", "error: unsupported show format `yaml`\n")
    );
}

/// The hint the refusal of a second coordinate appends, for the read's `--format` tail.
fn second_coordinate(first: &str, tail: &str) -> String {
    format!(
        "error: path does not exist: FS-beta\n\
         hint: `show` reads one coordinate, so `FS-beta` is the path; read both with \
         `printf '%s\\n' '{first}' '{{\"id\":\"FS-beta\"}}' | grund show --batch{tail}`\n"
    )
}

/// Mirrors the rewritten `show-second-coordinate-hint`: after a `text` read, the default,
/// the batch command it offers takes no `--format`, since the batch answers in `text`
/// (§FS-show.1.4.1).
#[test]
fn show_second_coordinate_hint() {
    let repo = PathBuf::from(case_repo("show-second-coordinate-hint"));
    let run = grund(&crate_dir().join(repo), &["FS-alpha", "FS-beta"], None);
    let stderr = second_coordinate("{\"id\":\"FS-alpha\"}", "");
    assert_eq!(run, Run::expect(2, "", &stderr));
}

/// Mirrors the rewritten `show-second-coordinate-hint-section-flag`: the `--section` object
/// keeps its place, and the command again takes no `--format` (§FS-show.1.4.1).
#[test]
fn show_second_coordinate_hint_section_flag() {
    let repo = PathBuf::from(case_repo("show-second-coordinate-hint-section-flag"));
    let run = grund(
        &crate_dir().join(repo),
        &["show", "FS-alpha", "FS-beta", "--section", "1"],
        None,
    );
    let stderr = second_coordinate("{\"id\":\"FS-alpha\",\"section\":\"1\"}", "");
    assert_eq!(run, Run::expect(2, "", &stderr));
}

/// The command follows the read's format however it was spelled: an explicit `--format text`
/// is the default's tail and `md` asks for `md`, while `json` keeps the tail the golden
/// `show-second-coordinate-hint-json` pins (§FS-show.1.4.1).
#[test]
fn show_second_coordinate_hint_follows_the_format() {
    let repo = crate_dir().join(case_repo("show-second-coordinate-hint"));
    let first = "{\"id\":\"FS-alpha\"}";
    for (format, tail) in [
        (&["--format", "text"][..], ""),
        (&["--format=text"][..], ""),
        (&["--format", "md"][..], " --format=md"),
        (&["--format=md"][..], " --format=md"),
        (&["--format", "json"][..], " --format=json"),
    ] {
        let mut args = vec!["show", "FS-alpha", "FS-beta"];
        args.extend_from_slice(format);
        let run = grund(&repo, &args, None);
        let expected = Run::expect(2, "", &second_coordinate(first, tail));
        assert_eq!(run, expected, "{format:?}");
    }
}

/// Mirrors the rewritten `cli-help-show`: the batch usage line takes `--format` as the
/// single read's does, and a text batch joins the examples (§FS-cli.2.3, §FS-show.1.8).
#[test]
fn cli_help_show() {
    let repo = crate_dir().join(case_repo("cli-help-show"));
    let run = grund(&repo, &["help", "show"], None);
    assert_eq!(run, Run::expect(0, HELP_SHOW, ""));
}

/// The command the refusal of a numbered selector names, run exactly as printed on the unit
/// its accepted selector selects, succeeds: the batch it names takes no `--format`
/// (§FS-rule-refusals.selectors, §FS-show.1.8, §FS-show.3.3.1).
#[test]
fn list_selector_refusal_hint_runs_as_printed() {
    let repo = crate_dir().join(case_repo("list-selector-refused-numbered-section"));
    let refusal = grund(&repo, &["list", "--selector", "FS.requirements.1"], None);
    assert_eq!(refusal.code, 2, "{refusal:?}");
    let hint = refusal
        .stderr
        .lines()
        .find_map(|line| line.strip_prefix("hint: "))
        .expect("the refusal prints a hint");
    let command = hint
        .strip_suffix(" expands each selected unit into its sections")
        .expect("the hint names a command");
    let args: Vec<&str> = command.split(' ').collect();
    assert_eq!(args, ["grund", "show", "--batch", "--toc"], "{hint}");
    let unit = "{\"id\":\"FS-login\",\"section\":\"requirements\"}\n";
    let run = grund(&repo, &args[1..], Some(unit));
    let stdout = concat!(
        "==> FS-login.requirements <==\n",
        "## requirements: What login holds to\n",
        "\n",
        "### requirements.1: A password is required\n",
        "### requirements.2: A lockout follows five failures\n",
    );
    assert_eq!(run, Run::expect(0, stdout, ""));
}

const HELP_SHOW: &str = r#"grund show — print a coordinate's lead, so an agent pulls a single fact into
context without loading the whole document. `show` is the default command.

Usage:  grund [show] <ID>[.<section>] [PATH] [--section S] [--brief|--toc|--full] [--format text|md|json] [--path PATH]
        grund show --batch [--all | --glob PATTERN...] [PATH] [--brief|--toc|--full] [--format text|md|json] [--path PATH]

Modes form an ordered ladder (each adds to the previous):
  --brief                heading + first paragraph    e.g. grund --brief FS-login
  (default)              + the rest of the lead, cut at the first child section
  --toc                  + the nested section map     e.g. grund --toc FS-login
  --full                 + every subsection body      e.g. grund --full FS-login

Other options:
  --section S            show only that section path  e.g. grund FS-login --section 3.1
  --format text|md|json  text (default) is the body, opening with a selected section's
                         own heading line; md adds the declaration heading above it;
                         json wraps the text body
  --path PATH            repo or subtree to resolve the ID in (no PATH answers as `.`);
                         narrows --batch --all and --glob, whose queries with no PATH
                         cover the whole project or workspace discovered from the
                         current directory
                         e.g. grund FS-login --path docs/
  --batch                 read ordered {"id":…,"section":…} NDJSON queries from stdin
  --all                   with --batch, query every declaration and section
  --glob PATTERN          with --batch, query the coordinates a pattern matches; repeatable
  --path-base project|invocation
                       project (default) spells report paths from the config root;
                       invocation: relative to the path you passed, or the current directory if you passed none.
                       e.g. grund FS-login docs/ --path-base=invocation
                       --path-base=invocation replaces a version-1 `[output] relative_paths = false`.

Exit:  0 printed · 1 ID not found / ambiguous / broken stub / unknown section · 2 unknown project alias, or CLI error.
Batch: 0 all queries succeeded · 1 any query failed · 2 invocation, input, or scan error.

Examples:
  grund FS-login                   # the lead — the cheap default
  grund FS-login --toc             # lead + section map
  grund FS-login.3.1               # the lead of that nested section
  grund FS-login --full            # the whole declaration body
  grund show --batch --glob 'FS-login.*' # each top-level section, headed ==> <coordinate> <==
  printf '%s\n' '{"id":"FS-login"}' | grund show --batch --format=json
  grund show --batch --all --format=json # every coordinate, one scan
  grund show --batch --glob 'FS-login{,.**}' --toc --format json # one ID and its sections

ID not found? `grund list` shows every declared ID; `grund id <KIND> "…"` proposes a new one.

Guide:    https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md
Example:  https://github.com/agent-grounds/grund/tree/main/examples/review-checklist
"#;
