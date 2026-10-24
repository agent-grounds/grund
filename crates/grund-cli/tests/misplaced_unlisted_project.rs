//! Black-box placement context and unchanged ownership controls for #1000.
//! §FS-declarations.checks.misplaced-declaration.unlisted-project; §FS-check.1.3.1;
//! §FS-check.3.29.3: plain nested configs gain no standalone finding.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const SINGLE_FILE: &str = "[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n";
const DECLARATION: &str = "# Requirements\n\n## FS-email: Receipts are sent by email\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);

#[path = "misplaced_unlisted_project/repairs.rs"]
mod repairs;

struct Fixture(PathBuf);

impl Fixture {
    fn new(home: &str, config: &str, declaration: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1000-placement-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let repo = Self(root);
        let init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo.0)
            .output()
            .unwrap();
        assert!(init.status.success(), "{init:?}");
        repo.write("grund.toml", &Self::root_config(home, false, false));
        repo.write("requirements.md", "# FS-shop: Shop\n");
        repo.write(
            "packages/billing/grund.toml",
            &format!("grund_config_version = 2\nproject_name = \"billing\"\n{SINGLE_FILE}"),
        );
        repo.write(
            "packages/billing/requirements.md",
            "# FS-billing: Billing\n",
        );
        repo.write(
            config,
            &format!("grund_config_version = 2\nproject_name = \"notify\"\n{SINGLE_FILE}"),
        );
        repo.write(declaration, DECLARATION);
        repo
    }

    fn issue() -> Self {
        Self::new(
            SINGLE_FILE,
            "packages/notify/grund.toml",
            "packages/notify/requirements.md",
        )
    }

    fn root_config(home: &str, listed: bool, excluded: bool) -> String {
        let exclusion = if excluded {
            "[schema.sources]\nexclude = [\"packages/notify/\"]\n"
        } else {
            ""
        };
        let members = if listed {
            "[\"packages/billing\", \"packages/notify\"]"
        } else {
            "[\"packages/billing\"]"
        };
        format!(
            "grund_config_version = 2\nproject_name = \"shop\"\n{exclusion}{home}\n\
             [workspace]\nmembers = {members}\n"
        )
    }

    fn write(&self, path: &str, body: &str) {
        let target = self.0.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, body).unwrap();
    }

    fn check(&self, json: bool, exit: i32) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
        command.arg("check").current_dir(&self.0);
        if json {
            command.args(["--format", "json"]);
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(exit), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        output
    }

    fn rows(&self, exit: i32) -> Vec<Value> {
        String::from_utf8(self.check(true, exit).stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn errors(rows: &[Value]) -> Vec<&Value> {
    rows.iter()
        .filter(|row| row["severity"] == "error")
        .collect()
}

fn assert_context(repo: &Fixture, config: &str, path: &str, original: &str) {
    let text = String::from_utf8(repo.check(false, 1).stdout).unwrap();
    let rows = repo.rows(1);
    assert_eq!(
        rows.len(),
        4,
        "only the existing error and unused warnings: {rows:?}"
    );
    let findings = errors(&rows);
    assert_eq!(findings.len(), 1, "{rows:?}");
    let finding = findings[0];
    let message = finding["message"].as_str().unwrap();
    assert_eq!(
        *finding,
        json!({"severity": "error", "path": path, "line": 3,
               "code": "misplaced-declaration", "message": message,
               "sites": null, "authority": null}),
        "enrichment changes only the existing message"
    );
    assert!(message.starts_with(original), "{message}");
    let text_errors: Vec<_> = text
        .lines()
        .filter(|line| line.contains(": error:"))
        .collect();
    assert_eq!(text_errors, [format!("{path}:3: error: {message}")]);
    // Check setup, identity and all three original messages before the red assertion.
    // Fragments pin the explanation's facts while leaving punctuation to the implementation.
    for required in [
        config,
        "notify",
        "not read",
        "no [workspace] members entry lists packages/notify",
        "grund.toml",
        "shop",
        "add",
        "\"packages/notify\"",
        "[schema.sources]",
        "exclude",
    ] {
        assert!(
            message.contains(required),
            "missing explanatory content {required:?} in existing placement error: {message}"
        );
    }
}

#[test]
fn single_file_unlisted_project_explains_text_and_json() {
    assert_context(
        &Fixture::issue(),
        "packages/notify/grund.toml",
        "packages/notify/requirements.md",
        "FS-email must be declared in requirements.md (single-file kind)",
    );
}

#[test]
fn other_kind_unlisted_project_explains_text_and_json() {
    let home = "[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\
                [schema.kinds.AR]\nfolders = [\"packages/notify\"]\nindex = false\n";
    assert_context(
        &Fixture::new(
            home,
            "packages/notify/grund.toml",
            "packages/notify/requirements.md",
        ),
        "packages/notify/grund.toml",
        "packages/notify/requirements.md",
        "FS-email declares kind FS inside AR home packages/notify",
    );
}

#[test]
fn non_citable_unlisted_project_explains_text_and_json() {
    let home = "[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\
                [schema.kinds.page]\ncitable = false\nfolders = [\"packages/notify\"]\n";
    assert_context(
        &Fixture::new(
            home,
            "packages/notify/grund.toml",
            "packages/notify/requirements.md",
        ),
        "packages/notify/grund.toml",
        "packages/notify/requirements.md",
        "FS-email must not be declared in packages/notify/ (not a citable home)",
    );
}

#[test]
fn deeper_declaration_gets_unlisted_project_context() {
    assert_context(
        &Fixture::new(
            SINGLE_FILE,
            "packages/notify/grund.toml",
            "packages/notify/docs/email.md",
        ),
        "packages/notify/grund.toml",
        "packages/notify/docs/email.md",
        "FS-email must be declared in requirements.md (single-file kind)",
    );
}

#[test]
fn agents_config_gets_unlisted_project_context() {
    assert_context(
        &Fixture::new(
            SINGLE_FILE,
            "packages/notify/.agents/grund.toml",
            "packages/notify/requirements.md",
        ),
        "packages/notify/.agents/grund.toml",
        "packages/notify/requirements.md",
        "FS-email must be declared in requirements.md (single-file kind)",
    );
}

#[test]
fn membership_alone_clears_placement_error() {
    let repo = Fixture::issue();
    assert_eq!(errors(&repo.rows(1)).len(), 1);
    repo.write(
        "grund.toml",
        &Fixture::root_config(SINGLE_FILE, true, false),
    );
    let rows = repo.rows(0);
    assert!(errors(&rows).is_empty(), "{rows:?}");
    assert_eq!(rows.len(), 3, "only unused warnings remain: {rows:?}");
}

#[test]
fn heading_depth_alone_does_not_clear_placement_error() {
    let repo = Fixture::issue();
    let before = repo.rows(1);
    repo.write(
        "packages/notify/requirements.md",
        &DECLARATION.replace("## FS-email", "# FS-email"),
    );
    let after = repo.rows(1);
    assert_eq!(errors(&before), errors(&after));
    assert_eq!(errors(&after).len(), 1);
}

#[test]
fn exclusion_prunes_unlisted_project_descendants() {
    let repo = Fixture::issue();
    assert_eq!(errors(&repo.rows(1)).len(), 1);
    // notify is below the complement root, not a scan root of its own.
    repo.write(
        "grund.toml",
        &Fixture::root_config(SINGLE_FILE, false, true),
    );
    let rows = repo.rows(0);
    assert!(errors(&rows).is_empty(), "{rows:?}");
    assert!(rows.iter().all(|row| {
        !row["path"]
            .as_str()
            .unwrap()
            .starts_with("packages/notify/")
    }));
}

#[test]
fn ordinary_misplacement_keeps_its_wording() {
    let repo = Fixture::issue();
    fs::remove_file(repo.0.join("packages/notify/grund.toml")).unwrap();
    let rows = repo.rows(1);
    assert_eq!(
        errors(&rows),
        [
            &json!({"severity": "error", "path": "packages/notify/requirements.md", "line": 3,
                "code": "misplaced-declaration",
                "message": "FS-email must be declared in requirements.md (single-file kind)",
                "sites": null, "authority": null})
        ]
    );
}

#[test]
fn plain_nested_config_without_placement_violation_is_quiet() {
    let repo = Fixture::issue();
    let home =
        "[schema.kinds.FS]\nfiles = [\"requirements.md\", \"packages/notify/requirements.md\"]\n";
    repo.write("grund.toml", &Fixture::root_config(home, false, false));
    let rows = repo.rows(0);
    assert!(errors(&rows).is_empty(), "{rows:?}");
    assert_eq!(rows.len(), 3, "plain config adds no finding: {rows:?}");
    assert!(rows.iter().all(|row| row["code"] == "unused"));
}

#[test]
fn standalone_unlisted_workspace_diagnostic_is_unchanged() {
    let repo = Fixture::issue();
    repo.write("packages/notify/grund.toml", &format!(
        "grund_config_version = 2\nproject_name = \"notify\"\n{SINGLE_FILE}\n[workspace]\nmembers = []\n"
    ));
    let rows = repo.rows(1);
    let findings = errors(&rows);
    assert_eq!(findings.len(), 2, "{rows:?}");
    let block = findings
        .iter()
        .find(|row| row["code"] == "unlisted-workspace-block")
        .unwrap();
    assert_eq!(block["path"], "packages/notify/grund.toml");
    assert_eq!(block["line"], 6);
    assert_eq!(
        block["message"],
        concat!(
            "this [workspace] is listed by no enclosing workspace — the projects under it are ",
            "absorbed into `shop` instead of named under their own alias path; add ",
            "\"packages/notify\" to [workspace] members in grund.toml, or keep it out of that ",
            "project's [scan] — an unlisted [workspace] became an error in grund 0.15.0"
        )
    );
}
