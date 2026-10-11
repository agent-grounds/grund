//! Every command gives the same answer about one tree (§REQ-one-answer), proved
//! by comparison as §REQ-one-answer.proof says: each question is asked of every
//! surface that answers it, each through its own entrypoint, and each answer is
//! checked against the point that defines it as well, because two surfaces wrong
//! alike agree.
//!
//! Seeded thin under §AR-ci.requirements.1. The trees are the reproduced rows of
//! #576 (r1, r2 and r3), one tree per row of §REQ-one-answer.differences, and a
//! fixed sample of the end-to-end corpus: the first case of each family in
//! `SAMPLED_FAMILIES` whose fixture is a plain project. The surfaces are the
//! command line's; the editor and the bindings are compared by
//! `lsp_cli_parity.rs` and `stub_resolution_agreement.rs` until this test drives
//! them too.
//!
//! `LEDGER` is the divergence ledger: one row per known divergence, naming the
//! tree and the invariant. A divergence it does not list fails, and so does a
//! row that no longer reproduces, so the ledger only shrinks. A divergence found
//! here that §REQ-one-answer.differences does not list is added there by an
//! amendment naming its baseline, or kept as a ledger-only row with its reason
//! (the list rule of §REQ-one-answer); it is never kept in this file alone.

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `(row of #576's ledger or of §REQ-one-answer.differences, tree, invariant)`.
const LEDGER: &[(&str, &str, &str)] = &[];

/// The families of `tests/e2e/cases/` whose first plain project is sampled.
const SAMPLED_FAMILIES: &[&str] = &[
    "check",
    "show",
    "refs",
    "list",
    "fmt",
    "stub",
    "workspace",
    "symlink",
    "doccomment",
    "scan",
    "glob",
    "cover",
];

const CONFIG_SLUG: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n";
const CONFIG_NUMBER: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{number}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n";

/// One fixture tree under the worktree's own `target/`.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/command-agreement")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (path, body) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
            fs::write(path, body).expect("write fixture");
        }
        Self(root.canonicalize().expect("canonical fixture root"))
    }

    fn grund(&self, args: &[&str]) -> Output {
        grund(&self.0, args)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn grund(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("run grund {args:?}: {error}"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}")))
        .collect()
}

/// One invariant asked of one tree: `Err` carries the two answers that differ.
type Verdict = Result<(), String>;

/// r1, row G01: a citation is its whole token, so `<§>FS-loginUser` is no citation
/// of `FS-login` — `check` reports it dangling and `refs FS-login` does not list
/// it (§FS-citations.recognized.marker-start, §REQ-one-answer.identity.declarations).
fn citation_is_its_whole_token() -> Verdict {
    let tree = Tree::new(
        "r1-citation-prefix",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                "# FS-login: Login\n\nLogs in.\n\n# FS-other: Other\n\nSee \u{a7}FS-loginUser.\n",
            ),
        ],
    );
    let refs = stdout(&tree.grund(&["refs", "FS-login"]));
    let check = json_lines(&stdout(&tree.grund(&["check", "--format", "json"])));
    let dangling = check
        .iter()
        .any(|f| f["code"] == "dangling-citation" && f["line"] == 7);
    let listed = refs.contains("docs/fs.md:7");
    if listed || !dangling {
        return Err(format!(
            "refs FS-login lists docs/fs.md:7: {listed}; check reports \u{a7}FS-loginUser \
             dangling: {dangling}"
        ));
    }
    Ok(())
}

/// r2, row R01: a path argument narrows what is reported, never what resolves —
/// the ID `check sub` resolves, `grund <ID> sub` shows
/// (§REQ-one-answer.scope.narrowed, §REQ-one-answer.identity.declarations).
fn a_narrowed_run_resolves_what_check_resolves() -> Verdict {
    let tree = Tree::new(
        "r2-path-argument",
        &[
            ("grund.toml", CONFIG_NUMBER),
            ("docs/fs.md", "# FS-001-a: Alpha\n\nAlpha body.\n"),
            ("sub/notes.md", "Uses \u{a7}FS-001-a.\n"),
        ],
    );
    let check = tree.grund(&["check", "sub"]).status.success();
    let show = tree.grund(&["FS-001-a", "sub"]);
    if check != show.status.success() {
        return Err(format!(
            "check sub succeeds: {check}; grund FS-001-a sub succeeds: {} ({})",
            show.status.success(),
            String::from_utf8_lossy(&show.stderr).trim()
        ));
    }
    Ok(())
}

/// r3, row P05: the anchor `show` reports is the one `fmt --cross-refs` writes
/// (§REQ-one-answer.writes.anchors), and both are the rendered heading's, a
/// closing `#` run taken off (§FS-show.3.1.3.1).
fn the_anchor_is_one_and_the_headings() -> Verdict {
    let tree = Tree::new(
        "r3-closing-hashes",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                concat!(
                    "# FS-a: Alpha ##\n\nAlpha body.\n\n## 1. Steps ##\n\nStep text.\n\n",
                    "# FS-b: Beta\n\nSee \u{a7}FS-a and \u{a7}FS-a.1.\n",
                ),
            ),
        ],
    );
    let shown = json_lines(&stdout(&tree.grund(&["FS-a", "--format", "json"])));
    let shown = shown
        .first()
        .and_then(|v| v["anchor"].as_str())
        .unwrap_or("")
        .to_string();
    tree.grund(&["fmt", "--cross-refs", "--write"]);
    let written = fs::read_to_string(tree.0.join("docs/fs.md")).expect("read fmt output");
    let wrote = written
        .split("fs.md#")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .unwrap_or("")
        .to_string();
    if shown != wrote || wrote != "fs-a-alpha" || !written.contains("fs.md#1-steps)") {
        return Err(format!(
            "show's anchor {shown:?}, fmt's {wrote:?}; the heading's is \"fs-a-alpha\" \
             (and \"1-steps\")"
        ));
    }
    Ok(())
}

/// Every ID `list` prints is the ID `list --size` measures, at the same
/// `path:line` (§REQ-one-answer.identity.declarations).
fn one_declared_set(root: &Path) -> Verdict {
    let sites = |args: &[&str]| -> BTreeSet<(String, String, u64)> {
        json_lines(&stdout(&grund(root, args)))
            .iter()
            .filter(|row| row["section"].is_null())
            .map(|row| {
                (
                    row["id"].as_str().unwrap_or("").to_string(),
                    row["path"].as_str().unwrap_or("").to_string(),
                    row["line"].as_u64().unwrap_or(0),
                )
            })
            .collect()
    };
    let listed = sites(&["list", "--format", "json"]);
    let measured = sites(&["list", "--size", "--format", "json"]);
    if listed != measured {
        let only_list: Vec<_> = listed.difference(&measured).collect();
        let only_size: Vec<_> = measured.difference(&listed).collect();
        return Err(format!(
            "list only {only_list:?}; list --size only {only_size:?}"
        ));
    }
    Ok(())
}

/// The sampled end-to-end trees: the first plain project of each family.
fn sampled_trees() -> Vec<(String, PathBuf)> {
    let cases = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/e2e/cases");
    let mut names: Vec<String> = fs::read_dir(&cases)
        .expect("read e2e cases")
        .map(|entry| {
            entry
                .expect("case")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    SAMPLED_FAMILIES
        .iter()
        .filter_map(|family| {
            names
                .iter()
                .find(|name| {
                    name.starts_with(&format!("{family}-"))
                        && cases.join(name).join("repo/grund.toml").is_file()
                })
                .map(|name| (name.clone(), cases.join(name).join("repo")))
        })
        .collect()
}

/// Every divergence found, as `(tree, invariant, the answers that differ)`.
fn divergences() -> Vec<(String, &'static str, String)> {
    let mut found = Vec::new();
    let reproduced: [(&str, &str, fn() -> Verdict); 3] = [
        (
            "r1",
            "citation-is-its-whole-token",
            citation_is_its_whole_token,
        ),
        (
            "r2",
            "narrowed-resolves-as-check",
            a_narrowed_run_resolves_what_check_resolves,
        ),
        ("r3", "one-anchor", the_anchor_is_one_and_the_headings),
    ];
    for (tree, invariant, verdict) in reproduced {
        if let Err(answers) = verdict() {
            found.push((tree.to_string(), invariant, answers));
        }
    }
    for (name, root) in sampled_trees() {
        if let Err(answers) = one_declared_set(&root) {
            found.push((name, "one-declared-set", answers));
        }
    }
    found
}

/// What is wrong with `ledger` against what was `found`, both ways round.
fn problems(found: &[(String, &str, String)], ledger: &[(&str, &str, &str)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (tree, invariant, answers) in found {
        if !ledger.iter().any(|(_, t, i)| t == tree && i == invariant) {
            wrong.push(format!("{tree} / {invariant}: {answers}"));
        }
    }
    for (row, tree, invariant) in ledger {
        if !found.iter().any(|(t, i, _)| t == tree && i == invariant) {
            wrong.push(format!(
                "ledger row {row} ({tree} / {invariant}) no longer reproduces: take it out"
            ));
        }
    }
    wrong
}

#[test]
fn every_command_gives_one_answer_but_for_the_ledger() {
    let wrong = problems(&divergences(), LEDGER);
    assert!(
        wrong.is_empty(),
        "{} divergence(s) not as the ledger says (\u{a7}REQ-one-answer):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Each row of §REQ-one-answer.differences, as `differences.<group>` lists it.
fn difference_rows() -> Vec<String> {
    let requirement =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/requirements/REQ-one-answer.md");
    let text = fs::read_to_string(requirement).expect("read REQ-one-answer");
    let mut inside = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with("### differences.") {
            inside = true;
        } else if line.starts_with("## ") || line.starts_with("### ") {
            inside = false;
        } else if inside && let Some(rest) = line.strip_prefix("- **") {
            rows.push(rest.split(".**").next().unwrap_or(rest).to_string());
        }
    }
    rows
}

/// Every listed difference has its tree here, so its row closes only when this
/// test sees the two answers agree (§REQ-one-answer.proof).
#[test]
fn every_listed_difference_has_its_tree() {
    let built: BTreeSet<&str> = ["R01"].into();
    let missing: Vec<String> = difference_rows()
        .into_iter()
        .filter(|row| !built.contains(row.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "{} row(s) of \u{a7}REQ-one-answer.differences have no tree in this test: {}",
        missing.len(),
        missing.join(", ")
    );
}

#[test]
fn the_ledger_fails_on_a_new_divergence_and_on_a_stale_row() {
    let found = vec![("t".to_string(), "i", "a != b".to_string())];
    assert!(problems(&found, &[("X01", "t", "i")]).is_empty());
    assert_eq!(problems(&found, &[]), ["t / i: a != b"]);
    assert_eq!(
        problems(&[], &[("X01", "t", "i")]),
        ["ledger row X01 (t / i) no longer reproduces: take it out"]
    );
}

#[test]
fn the_sample_names_one_tree_per_family() {
    let sampled = sampled_trees();
    assert_eq!(
        sampled.len(),
        SAMPLED_FAMILIES.len(),
        "a family has no plain project to sample: {sampled:?}"
    );
}
