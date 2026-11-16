//! The bare form's filesystem-path breadcrumb in a workspace (§FS-show.3.5.2): its format hint
//! is the one an invalid ID earns, naming the run's own template (§FS-show.3.5.1), whether the
//! run is at the workspace root, inside a member, or rooted in a member by its path argument.
//! The same unknown alias with no path behind it, and a batch query that names one, keep their
//! own bytes and print no breadcrumb (§FS-workspace.8.1, §FS-show.3.3.2). Each test pins the
//! exact stdout, stderr and exit code, mirroring the golden case `workspace-show-path-breadcrumb`
//! where it is the same run, so a change to how the hint is built changes none of them.

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

/// The fixture of `workspace-show-path-breadcrumb`: a root project whose `[id] format` is
/// `{kind}-{slug}` and a member `api` whose format is `{kind}-{number}-{slug}`, so the
/// template a hint names says which project's config it was read from.
fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/workspace-show-path-breadcrumb/repo")
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

/// The invalid-ID hint naming the root's template (§FS-show.3.5.1).
const ROOT_HINT: &str = "hint: this repo's [id] format is `{kind}-{slug}` \
                         (run `grund config show`); `grund list` shows the IDs that exist\n";

/// The invalid-ID hint naming the member's template (§FS-show.3.5.1).
const API_HINT: &str = "hint: this repo's [id] format is `{kind}-{number}-{slug}` \
                        (run `grund config show`); `grund list` shows the IDs that exist\n";

/// The breadcrumb's three lines for `operand`, its format hint `hint` (§FS-show.3.5.2).
fn breadcrumb(operand: &str, hint: &str) -> String {
    format!("invalid ID `{operand}`\n{hint}hint: run `grund check {operand}` to validate a path\n")
}

/// Mirrors `workspace-show-path-breadcrumb`: at the workspace root `docs` names no project, so
/// the hint names the root's template, the very line `grund FS-` prints there (§FS-show.3.5.1).
#[test]
fn breadcrumb_at_the_workspace_root_names_the_root_template() {
    let root = workspace();
    assert_eq!(
        grund(&root, &["docs/FS-root.md"], None),
        Run::expect(1, "", &breadcrumb("docs/FS-root.md", ROOT_HINT))
    );
    assert_eq!(
        grund(&root, &["FS-"], None),
        Run::expect(1, "", &format!("invalid ID `FS-`\n{ROOT_HINT}"))
    );
}

/// Inside the member the run is the member's (§FS-workspace.5.1), so its breadcrumb names the
/// member's template, the line `grund FS-` prints there (§FS-show.3.5.1).
#[test]
fn breadcrumb_inside_a_member_names_the_member_template() {
    let api = workspace().join("api");
    assert_eq!(
        grund(&api, &["docs/FS-001-api.md"], None),
        Run::expect(1, "", &breadcrumb("docs/FS-001-api.md", API_HINT))
    );
    assert_eq!(
        grund(&api, &["FS-"], None),
        Run::expect(1, "", &format!("invalid ID `FS-`\n{API_HINT}"))
    );
}

/// A path argument naming the member roots the run there (§FS-show.1.4): the operand is still
/// looked up from the working directory, and the hint names the run's template, the member's
/// (§FS-show.3.5.1, §FS-show.3.5.2).
#[test]
fn breadcrumb_under_a_member_path_names_the_member_template() {
    assert_eq!(
        grund(&workspace(), &["docs/FS-root.md", "api"], None),
        Run::expect(1, "", &breadcrumb("docs/FS-root.md", API_HINT))
    );
}

/// The same unknown alias with no path behind it keeps its own refusal, exit `2` with neither
/// breadcrumb nor format hint, at the root and inside the member (§FS-workspace.8.1,
/// §FS-show.3.5.2).
#[test]
fn unknown_alias_with_no_path_behind_it_keeps_its_refusal() {
    assert_eq!(
        grund(&workspace(), &["docs/FS-gone.md"], None),
        Run::expect(
            2,
            "",
            "error: unknown project alias `docs`\nknown aliases: root, api\n"
        )
    );
    assert_eq!(
        grund(&workspace().join("api"), &["docs/FS-gone.md"], None),
        Run::expect(
            2,
            "",
            "error: unknown project alias `docs`\n\
             note: workspace aliases are defined in the root grund.toml under [workspace]\n"
        )
    );
}

/// A batch query whose alias names no project fails in its place with no hint line, even where
/// its `id` is a path that exists: the breadcrumb is the bare form's alone, and a batch prints
/// the hint lines its failure earns and nothing more, in every format (§FS-show.3.3.2,
/// §FS-show.3.5.2).
#[test]
fn batch_unknown_alias_over_an_existing_path_prints_no_breadcrumb() {
    let stdin = "{\"id\":\"docs/FS-root.md\"}\n{\"id\":\"FS-root\"}\n";
    let failure =
        "==> docs/FS-root.md <==\nunknown project alias `docs`\nknown aliases: root, api\n";
    let batch = |format: &str| {
        let format = format!("--format={format}");
        grund(&workspace(), &["show", "--batch", &format], Some(stdin))
    };
    assert_eq!(
        batch("text"),
        Run::expect(1, &format!("{failure}\n==> FS-root <==\nRoot lead.\n"), "")
    );
    assert_eq!(
        batch("md"),
        Run::expect(
            1,
            &format!("{failure}\n==> FS-root <==\n# FS-root: Root\n\nRoot lead.\n"),
            ""
        )
    );
    let json = concat!(
        "{\"query\":{\"id\":\"docs/FS-root.md\",\"section\":null},\"ok\":false,\"result\":null,",
        "\"error\":{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"unknown-project\",",
        "\"message\":\"unknown project alias `docs`\\nknown aliases: root, api\",\"sites\":null,",
        "\"authority\":null}}\n",
        "{\"query\":{\"id\":\"FS-root\",\"section\":null},\"ok\":true,\"result\":{\"id\":\"FS-root\",",
        "\"section\":null,\"body\":\"Root lead.\\n\",\"anchor\":\"fs-root-root\",",
        "\"path\":\"docs/FS-root.md\",\"line\":1},\"error\":null}\n",
    );
    assert_eq!(batch("json"), Run::expect(1, json, ""));
}
