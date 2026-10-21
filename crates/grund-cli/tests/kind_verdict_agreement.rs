//! One kind verdict per scope, read by every command (§REQ-one-answer.identity.kinds).
//!
//! One workspace tree: the root configures `FS`, the non-citable `e2e` and named
//! sections off; its member `api` configures `API`, the non-citable `SKILL` and
//! named sections on. Each row asks one kind name in one scope
//! (§FS-scan.projects.kinds), and each column is a surface that asks in that
//! scope: `list --kind` and `list --selector` in every loaded project, a rule's
//! subject, a rule's object and a `[citations]` target in the root, a pinned
//! object and target in `api`, and a `*/` object and target in any loaded
//! project. Every column of a row must give the row's one verdict class — accepted, refused as not citable
//! (§FS-errors.2.5.1), or refused otherwise — so no two surfaces disagree, and
//! none disagrees with the point that defines the answer. A second test holds a
//! member's named chapter to one answer (§REQ-one-answer.sections.named), and a
//! third pins the wording `code` is refused with since its window closed: its one
//! verdict's reason alone (§FS-errors.3.8.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// What a surface made of a kind name, read from its exit and its stderr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    /// Accepted: the run did not refuse the name.
    Accepted,
    /// Refused because the kind is configured but declares no IDs.
    NonCitable,
    /// Refused for any other reason: unknown in the scope.
    Refused,
}
use Verdict::{Accepted, NonCitable, Refused};

/// The scope a surface asks in (§FS-scan.projects.kinds).
#[derive(Clone, Copy, Debug)]
enum Scope {
    /// Every loaded project: `list`.
    Workspace,
    /// The root project: a rule subject, a bare object, a bare target.
    Root,
    /// The project `api`: a pinned object and a pinned target.
    Api,
    /// Any loaded project: a `*/` object and a `*/` target.
    Any,
}

/// One kind name, and its verdict in each scope.
struct Row {
    name: &'static str,
    workspace: Verdict,
    root: Verdict,
    api: Verdict,
    any: Verdict,
}

const ROWS: &[Row] = &[
    // A local citable kind: `api` does not configure it.
    Row {
        name: "FS",
        workspace: Accepted,
        root: Accepted,
        api: Refused,
        any: Accepted,
    },
    // A member's citable kind: unknown to the root alone.
    Row {
        name: "API",
        workspace: Accepted,
        root: Refused,
        api: Accepted,
        any: Accepted,
    },
    // A configured kind that declares no IDs.
    Row {
        name: "e2e",
        workspace: NonCitable,
        root: NonCitable,
        api: Refused,
        any: NonCitable,
    },
    // A member's kind that declares no IDs: unknown to the root alone.
    Row {
        name: "SKILL",
        workspace: NonCitable,
        root: Refused,
        api: NonCitable,
        any: NonCitable,
    },
    // The homeless kind's default name, which neither project configures and
    // `[[kinds]]` leaves without IDs (§FS-config.3.9.2).
    Row {
        name: "code",
        workspace: NonCitable,
        root: NonCitable,
        api: NonCitable,
        any: NonCitable,
    },
    // A kind no project configures.
    Row {
        name: "POLICY",
        workspace: Refused,
        root: Refused,
        api: Refused,
        any: Refused,
    },
];

const ROOT_CONFIG: &str = "grund_config_version = 1
project_name = \"root\"

[reference]
marker = \"\u{a7}\"
strict = true

[id]
format = \"{kind}-{slug}\"
slug_pattern = \"[a-z][a-z0-9-]*\"

[[kinds]]
kind = \"FS\"
folder = \"docs/fs\"
index = false

[[kinds]]
kind = \"e2e\"
folder = \"tests/e2e\"
citable = false

[scan]
include = [\"docs\", \"tests\"]

[workspace]
members = [\"api\"]
include_root = true
";

const API_CONFIG: &str = "grund_config_version = 1
project_name = \"api\"

[reference]
marker = \"\u{a7}\"
strict = true

[id]
format = \"{kind}-{slug}\"
slug_pattern = \"[a-z][a-z0-9-]*\"
named_sections = true

[[kinds]]
kind = \"API\"
folder = \"docs/api\"
index = false

[[kinds]]
kind = \"SKILL\"
folder = \"skills\"
citable = false

[scan]
include = [\"docs\"]
";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
    fs::write(path, contents).expect("write fixture file");
}

/// The workspace tree, with `citations` appended to the root's config.
fn tree(name: &str, citations: &str) -> PathBuf {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = repo_root()
        .join("target/kind-verdict-agreement-work")
        .join(format!("{name}-{}-{serial}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale tree");
    }
    write(&root, "grund.toml", &format!("{ROOT_CONFIG}{citations}"));
    write(
        &root,
        "docs/fs/FS-x.md",
        "# FS-x: A feature\n\nServes \u{a7}api/API-a.terms.\n",
    );
    write(&root, "tests/e2e/case.sh", "# proves \u{a7}FS-x\n");
    write(&root, "api/grund.toml", API_CONFIG);
    write(
        &root,
        "api/docs/api/API-a.md",
        "# API-a: An endpoint\n\nServes \u{a7}root/FS-x.\n\n## terms: Terms\n\nIts words.\n",
    );
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund")
}

/// The verdict a run gave, and what it printed, for the report.
fn verdict(output: &Output) -> (Verdict, String) {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let refused = output.status.code() == Some(2);
    let verdict = if stderr.contains("[citations.") && !refused {
        // A pinned target the run cannot accept fails the load since `0.19.0`
        // (§FS-config.3.9.5.2), so a `[citations]` line on a run that went on
        // refused nothing.
        Accepted
    } else if stderr.contains("declares no IDs") || stderr.contains("non-citable target kind") {
        NonCitable
    } else if refused {
        Refused
    } else {
        Accepted
    };
    (verdict, stderr)
}

/// Every surface that asks about `name` in `scope`, as a label and a run.
fn surfaces(scope: Scope, name: &str) -> Vec<(String, Output)> {
    let plain = tree("plain", "");
    match scope {
        Scope::Workspace => vec![
            (
                format!("list --kind {name}"),
                run(&plain, &["list", "--kind", name]),
            ),
            (
                format!("list --selector {name}"),
                run(&plain, &["list", "--selector", name]),
            ),
        ],
        Scope::Root | Scope::Api | Scope::Any => {
            let target = match scope {
                Scope::Api => format!("api/{name}"),
                Scope::Any => format!("*/{name}"),
                _ => name.to_owned(),
            };
            let mut asked = Vec::new();
            if let Scope::Root = scope {
                let subject = format!("Each {name} must cite at least one FS.");
                asked.push((
                    format!("rule subject: {subject}"),
                    run(&plain, &["check", ".", "--rule", &subject]),
                ));
            }
            let object = format!("Each FS must cite at least one {target}.");
            asked.push((
                format!("rule object: {object}"),
                run(&plain, &["check", ".", "--rule", &object]),
            ));
            let configured = tree(
                "citations",
                &format!("\n[citations.FS]\nshould = [\"{target}\"]\n"),
            );
            asked.push((
                format!("[citations.FS] should = [\"{target}\"]"),
                run(&configured, &["check", "."]),
            ));
            asked
        }
    }
}

/// §REQ-one-answer.identity.kinds: in each scope, every surface gives a name the
/// one verdict §FS-scan.projects.kinds defines for it.
#[test]
fn every_surface_gives_a_kind_name_one_verdict_in_its_scope() {
    let mut wrong = Vec::new();
    for row in ROWS {
        for (scope, expected) in [
            (Scope::Workspace, row.workspace),
            (Scope::Root, row.root),
            (Scope::Api, row.api),
            (Scope::Any, row.any),
        ] {
            for (surface, output) in surfaces(scope, row.name) {
                let (got, stderr) = verdict(&output);
                if got != expected {
                    wrong.push(format!(
                        "{} in {scope:?}: {surface}\n  expected {expected:?}, got {got:?} \
                         (exit {:?})\n  stderr {stderr:?}",
                        row.name,
                        output.status.code()
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "surfaces disagree on a kind verdict:\n{}",
        wrong.join("\n")
    );
}

/// §FS-errors.3.8.1: `code`, the homeless kind's default name, has no `[[kinds]]`
/// row in either project (§FS-config.3.9.2). Since its window closed (#680, row G09
/// of §REQ-one-answer.differences.identity) every surface other than a `[citations]`
/// target refuses it with §FS-errors.2.5.1's form with no home alone: no legacy
/// `unknown kind` or `literal subject` in front, and no notice that the wording
/// changes.
#[test]
fn the_homeless_kinds_default_name_is_refused_as_declaring_no_ids_alone() {
    let mut wrong = Vec::new();
    for scope in [Scope::Workspace, Scope::Root, Scope::Api, Scope::Any] {
        for (surface, output) in surfaces(scope, "code") {
            if surface.starts_with("[citations.") {
                continue;
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            let headline = stderr.lines().next().unwrap_or_default();
            let alone = [
                "error: kind \"code\" declares no IDs",
                "error: kind `code` declares no IDs",
            ]
            .contains(&headline);
            if !alone || output.status.code() != Some(2) {
                wrong.push(format!(
                    "code in {scope:?}: {surface}\n  (exit {:?})\n  stderr {stderr:?}",
                    output.status.code()
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "`code` is not refused with the no-home reason alone:\n{}",
        wrong.join("\n")
    );
}

/// §REQ-one-answer.sections.named: a member's named chapter answers the same on
/// `show` and on an unfiltered `list --selector`, whatever the root sets.
#[test]
fn a_members_named_chapter_answers_on_show_and_on_a_selector() {
    let root = tree("named", "");
    let show = run(&root, &["api/API-a.terms"]);
    let list = run(&root, &["list", "--selector", "API.terms"]);
    let listed = String::from_utf8_lossy(&list.stdout);
    assert_eq!(
        show.status.code(),
        Some(0),
        "show refused api/API-a.terms: {}",
        String::from_utf8_lossy(&show.stderr)
    );
    assert!(
        list.status.code() == Some(0)
            && listed
                .lines()
                .any(|line| line.starts_with("api/API-a.terms ")),
        "show answers api/API-a.terms and list --selector API.terms does not list it:\n  \
         exit {:?}\n  stdout {listed:?}\n  stderr {:?}",
        list.status.code(),
        String::from_utf8_lossy(&list.stderr)
    );
}
