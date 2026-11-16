//! §FS-workspace.8.1.1.4 / §FS-workspace.8.2.2.1: port of #1363's root
//! DA/member ACT fixture, in both config versions, without shared root kinds.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use grund_core::{BatchShowQuery, ShowOpts, show, show_batch_with_scope};
use serde_json::{Value, json};

const ID: &str = "ACT-001-founding-contract";
const QUALIFIED: &str = "company/ACT-001-founding-contract";
const FORMAT: &str = "{kind}-{number}-{slug}";
const BODY: &str =
    "# ACT-001-founding-contract: Founding contract\n\nThe founders sign the founding contract.\n";

struct Tree(PathBuf);

impl Tree {
    fn new(version: u8, test: &str) -> Self {
        let tree = Self(std::env::temp_dir().join(format!(
            "grund-1363-v{version}-{test}-{}",
            std::process::id()
        )));
        let _ = fs::remove_dir_all(&tree.0);
        for (member, name, kind, folder) in [
            ("", "platform", "DA", "docs/decisions"),
            ("company", "company", "ACT", "legal"),
        ] {
            let project = tree.0.join(member);
            fs::create_dir_all(project.join(folder)).unwrap();
            let mut config =
                format!("grund_config_version = {version}\nproject_name = \"{name}\"\n");
            if version == 1 {
                config.push_str(&format!(
                    "\n[id]\nformat = \"{FORMAT}\"\n\n[[kinds]]\nkind = \"{kind}\"\n\
                     folder = \"{folder}\"\nindex = false\n\n[scan]\ninclude = [\"{folder}\"]\n"
                ));
            } else {
                config.push_str(&format!(
                    "\n[schema]\nid_format = \"{FORMAT}\"\n\n[schema.kinds.{kind}]\n\
                     folders = [\"{folder}\"]\nindex = false\n"
                ));
            }
            if member.is_empty() {
                config.push_str("\n[workspace]\nmembers = [\"company\"]\n");
            }
            fs::write(project.join("grund.toml"), config).unwrap();
        }
        fs::write(
            tree.0.join("docs/decisions/DA-001-found-the-company.md"),
            "# DA-001-found-the-company: Found the company\n\n\
             The company is founded by the contract \u{a7}company/ACT-001-founding-contract.\n",
        )
        .unwrap();
        fs::write(tree.0.join("company/legal").join(format!("{ID}.md")), BODY).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(&tree.0)
                .status()
                .unwrap()
                .success()
        );
        tree
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }

    fn opts(&self) -> ShowOpts {
        let mut opts = ShowOpts::default();
        opts.path = self.0.clone();
        opts
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn out(run: &Output) -> &str {
    std::str::from_utf8(&run.stdout).unwrap()
}

fn err(run: &Output) -> &str {
    std::str::from_utf8(&run.stderr).unwrap()
}

fn refusal() -> String {
    format!("ID not found: {ID}; did you mean {QUALIFIED}?")
}

fn finding(code: &str, message: &str) -> Value {
    json!({"severity":"error", "path":null, "line":null, "code":code,
           "message":message, "sites":null, "authority":null})
}

// Collect mismatches so bare/explicit and JSON/batch cannot mask each other.
fn streams(run: &Output, status: i32, stdout: &str, stderr: &str, errors: &mut Vec<String>) {
    if run.status.code() != Some(status) || out(run) != stdout || err(run) != stderr {
        errors.push(format!(
            "expected status={status}, stdout={stdout:?}, stderr={stderr:?};\n\
             actual status={:?}, stdout={:?}, stderr={:?}",
            run.status.code(),
            out(run),
            err(run)
        ));
    }
}

fn show_text(version: u8) {
    let tree = Tree::new(version, "show-text");
    let mut errors = Vec::new();
    for args in [vec![ID, "--brief"], vec!["show", ID, "--brief"]] {
        streams(
            &tree.run(&args),
            1,
            "",
            &format!("{}\n", refusal()),
            &mut errors,
        );
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

fn show_json_batch(version: u8) {
    let tree = Tree::new(version, "show-json-batch");
    let mut errors = Vec::new();
    for args in [
        vec![ID, "--format", "json"],
        vec!["show", ID, "--format", "json"],
    ] {
        let run = tree.run(&args);
        assert_eq!(run.status.code(), Some(1));
        assert_eq!(out(&run), "");
        let actual: Value = serde_json::from_str(err(&run)).unwrap();
        let expected = finding("not-found", &refusal());
        if actual != expected {
            errors.push(format!("{args:?}: expected {expected}, actual {actual}"));
        }
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["show", "--batch", "--format", "json"])
        .current_dir(&tree.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        child.stdin.take().unwrap(),
        "{{\"id\":\"{ID}\"}}\n{{\"id\":\"{QUALIFIED}\"}}"
    )
    .unwrap();
    let run = child.wait_with_output().unwrap();
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(err(&run), "");
    let rows: Vec<Value> = out(&run)
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    let expected = json!({"query":{"id":ID,"section":null}, "ok":false, "result":null,
        "error":finding("not-found", &refusal())});
    if rows[0] != expected {
        errors.push(format!("batch: expected {expected}, actual {}", rows[0]));
    }
    assert_eq!(rows[1]["ok"], true, "{}", rows[1]);
    assert_eq!(rows[1]["result"]["id"], ID);
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

fn api_and_batch(version: u8) {
    let tree = Tree::new(version, "api");
    assert!(
        show(QUALIFIED, tree.opts()).is_ok(),
        "qualified API control"
    );
    let error = show(ID, tree.opts()).err().expect("show must refuse");
    let scalar_code = error
        .downcast_ref::<grund_core::OperationDiagnostic>()
        .map(|diagnostic| diagnostic.code)
        .or_else(|| {
            error
                .downcast_ref::<grund_core::ShowQueryError>()
                .map(|failure| failure.code)
        })
        .expect("typed show failure");
    let records = show_batch_with_scope(
        Some(vec![
            BatchShowQuery {
                id: ID.into(),
                section: None,
            },
            BatchShowQuery {
                id: QUALIFIED.into(),
                section: None,
            },
        ]),
        tree.opts(),
        true,
    )
    .1
    .unwrap();
    let failure = records[0].result.as_ref().err().expect("batch must refuse");
    assert!(records[1].result.is_ok(), "qualified batch API control");
    assert_eq!(
        (
            scalar_code,
            format!("{error:#}"),
            failure.code,
            failure.message.as_str()
        ),
        ("not-found", refusal(), "not-found", refusal().as_str()),
        "scalar API and batch must share the candidate refusal"
    );
}

fn refs_text(version: u8) {
    let tree = Tree::new(version, "refs");
    let mut errors = Vec::new();
    for args in [vec!["refs", ID], vec!["refs", ID, "--summary"]] {
        let run = tree.run(&args);
        assert_eq!(run.status.code(), Some(1));
        assert_eq!(out(&run), "");
        let lines: Vec<_> = err(&run).lines().collect();
        if lines.len() != 2
            || lines[0] != format!("invalid ID `{ID}`")
            || !lines[1].starts_with("hint:")
            || !lines[1].contains(&format!("grund refs {QUALIFIED}"))
            || lines[1].contains("config show")
            || lines[1].contains("id_format")
            || lines[1].contains("[id] format")
        {
            errors.push(format!(
                "{args:?}: expected invalid ID and qualified refs command hint; actual {:?}",
                err(&run)
            ));
        }
    }
    // §FS-workspace.8.2.2.1: the original finding shape and API data survive.
    let json_run = tree.run(&["refs", ID, "--format", "json"]);
    assert_eq!(json_run.status.code(), Some(1));
    assert_eq!(out(&json_run), "");
    assert_eq!(
        serde_json::from_str::<Value>(err(&json_run)).unwrap(),
        finding("invalid-id", &format!("invalid ID `{ID}`"))
    );
    let mut opts = grund_core::RefsOpts::default();
    opts.path = tree.0.clone();
    opts.id = ID.into();
    let outcome = grund_core::refs_outcome(opts).unwrap();
    assert!(outcome.output.hits.is_empty());
    let failure = outcome.query_failure.unwrap();
    assert_eq!(failure.kind, grund_core::RefsQueryFailureKind::InvalidId);
    assert_eq!(failure.message, format!("invalid ID `{ID}`"));
    assert_eq!(failure.format_hint.as_deref(), Some(FORMAT));
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

fn controls(version: u8) {
    let tree = Tree::new(version, "controls");
    let check = tree.run(&["check"]);
    assert_eq!(check.status.code(), Some(0), "{check:?}");
    let mut errors = Vec::new();
    streams(&tree.run(&[QUALIFIED, "--brief"]), 0, BODY, "", &mut errors);
    streams(
        &tree.run(&["refs", QUALIFIED, "--summary"]),
        0,
        "docs/decisions/DA-001-found-the-company.md: 1 (line 3)\n",
        "",
        &mut errors,
    );
    // Merely parseable by ACT's grammar is insufficient; malformed text too.
    for id in ["ACT-999-absent", "ACT-???"] {
        for prefix in [vec![], vec!["refs"]] {
            let mut args = prefix;
            args.push(id);
            let run = tree.run(&args);
            assert_eq!(run.status.code(), Some(1));
            assert_eq!(out(&run), "");
            assert!(err(&run).starts_with(&format!("invalid ID `{id}`\n")));
            assert!(err(&run).contains("config show"), "{run:?}");
            assert!(!err(&run).contains(QUALIFIED), "{run:?}");
        }
    }
    // §FS-scan.projects.narrowed: a root subdirectory outside members still loads them.
    streams(
        &tree.run(&[ID, "docs/decisions"]),
        1,
        "",
        &format!("{}\n", refusal()),
        &mut errors,
    );
    streams(
        &tree.run(&["refs", ID, "docs/decisions"]),
        1,
        "",
        &format!("invalid ID `{ID}`\nhint: run `grund refs {QUALIFIED}`\n"),
        &mut errors,
    );
    streams(
        &tree.run(&[ID, "company", "--brief"]),
        0,
        BODY,
        "",
        &mut errors,
    );
    // Root-valid absence stays an empty refs success, even with the same
    // declaration in a member. This uses DA, which the root already knows.
    let member_config = tree.0.join("company/grund.toml");
    let config = fs::read_to_string(&member_config)
        .unwrap()
        .replace("ACT", "DA");
    fs::write(member_config, config).unwrap();
    fs::write(
        tree.0.join("company/legal/DA-002-member-only.md"),
        "# DA-002-member-only: Member only\n\nAn uncited declaration.\n",
    )
    .unwrap();
    let run = tree.run(&["refs", "DA-002-member-only"]);
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(out(&run), "");
    assert!(err(&run).contains("neither declared nor cited"));
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

macro_rules! red {
    ($name:ident, $scenario:ident, $version:expr) => {
        #[test]
        fn $name() {
            $scenario($version);
        }
    };
}

red!(workspace_member_kind_v1_show_text, show_text, 1);
red!(workspace_member_kind_v2_show_text, show_text, 2);
red!(workspace_member_kind_v1_show_json_batch, show_json_batch, 1);
red!(workspace_member_kind_v2_show_json_batch, show_json_batch, 2);
red!(workspace_member_kind_v1_api_and_batch, api_and_batch, 1);
red!(workspace_member_kind_v2_api_and_batch, api_and_batch, 2);
red!(workspace_member_kind_v1_refs_text, refs_text, 1);
red!(workspace_member_kind_v2_refs_text, refs_text, 2);

#[test]
fn workspace_member_kind_v1_controls() {
    controls(1);
}

#[test]
fn workspace_member_kind_v2_controls() {
    controls(2);
}
