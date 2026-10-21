//! End-to-end explanation of index entries excluded from use: §FS-check.4.1.2,
//! §FS-refs.3.1.1 and §FS-refs.3.3.1. Port of issue #909's triage reproducer.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};

const INDEX: &str = "docs/architecture/README.md";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
    [schema.kinds.AR]\nfolders = [\"docs/architecture\"]\n\n\
    [schema.kinds.NOTE]\nfiles = [\"docs/note.md\"]\n";

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-909-{name}-{}", std::process::id()));
        assert!(!root.exists(), "fixture must be fresh: {}", root.display());
        fs::create_dir_all(&root).unwrap();
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .arg(&root)
            .status()
            .unwrap();
        assert!(status.success());
        Self(root)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn run(&self, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
        assert!(output.stderr.is_empty(), "{args:?}: {output:?}");
        String::from_utf8(output.stdout).unwrap()
    }

    fn json(&self, args: &[&str]) -> Vec<Value> {
        let mut args = args.to_vec();
        args.extend(["--format", "json"]);
        self.run(&args)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn triage_repo(name: &str) -> Repo {
    let repo = Repo::new(name);
    repo.write("grund.toml", CONFIG);
    repo.write(
        INDEX,
        "# Architecture\n\n| ID | Subject |\n|---|---|\n\
        | \u{a7}AR-auth | The sign-in path |\n| \u{a7}AR-cache | The session cache |\n",
    );
    repo.write(
        "docs/architecture/AR-auth.md",
        "# AR-auth: Sign-in runs in one request handler\n\n\
        The handler stores the new session in the cache (\u{a7}AR-cache).\n",
    );
    repo.write(
        "docs/architecture/AR-cache.md",
        "# AR-cache: The session cache\n\n\
        A bounded map from session token to session.\n",
    );
    repo.write(
        "docs/note.md",
        "# NOTE-alone: A genuinely uncited declaration\n",
    );
    repo.run(&["fmt", "--write"]);
    repo
}

fn annotated_row<'a>(text: &'a str, prefix: &str) -> &'a str {
    let row = text
        .lines()
        .find(|row| row.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing preserved row prefix {prefix:?}:\n{text}"));
    assert!(
        row[prefix.len()..].contains("index entr"),
        "missing index-entry annotation: {row}"
    );
    &row[prefix.len()..]
}

fn accounting_controls(repo: &Repo) -> String {
    let check = repo.run(&["check"]);
    assert_eq!(
        check.lines().count(),
        2,
        "only index-only and actually uncited IDs warn: {check}"
    );
    assert!(
        check
            .contains("docs/architecture/AR-auth.md:1: warning: declared but never cited: AR-auth")
    );
    assert!(
        check
            .lines()
            .any(|row| row == "docs/note.md:1: warning: declared but never cited: NOTE-alone")
    );
    assert!(!check.contains("declared but never cited: AR-cache"));
    check
}

/// §FS-check.4.1.2: explain the same exclusion in text and JSON messages;
/// a true non-use keeps its old warning and an outside-index use still counts.
#[test]
#[ignore = "red until #909 explains excluded index entries; implement removes this"]
fn index_only_warning_locates_the_discounted_site() {
    let repo = triage_repo("warning");
    let check = accounting_controls(&repo);
    let prefix = "docs/architecture/AR-auth.md:1: warning: declared but never cited: AR-auth";
    let suffix = annotated_row(&check, prefix);
    assert!(
        suffix.contains(&format!("{INDEX}:5")),
        "missing excluded location: {check}"
    );
    assert!(
        suffix.contains("does not count as a use"),
        "missing explanation: {check}"
    );
    let records = repo.json(&["check"]);
    let auth = records
        .iter()
        .find(|row| row["path"] == "docs/architecture/AR-auth.md")
        .unwrap();
    assert_eq!(
        auth,
        &json!({
            "severity": "warning", "path": "docs/architecture/AR-auth.md", "line": 1,
            "code": "unused", "sites": null, "authority": null,
            "message": check.lines().find(|row| row.starts_with(prefix)).unwrap()
                .split_once("warning: ").unwrap().1
        })
    );
}

/// §FS-refs.3.1.1: both index-only and genuinely used IDs label index sites,
/// retaining every row and leaving ordinary rows byte-identical.
#[test]
#[ignore = "red until #909 labels excluded refs sites; implement removes this"]
fn refs_labels_index_entries_beside_real_uses() {
    let repo = triage_repo("sites");
    accounting_controls(&repo);
    let auth = repo.run(&["refs", "AR-auth"]);
    assert_eq!(auth.lines().count(), 1);
    annotated_row(&auth, &format!("{INDEX}:5: \u{a7}AR-auth"));
    let cache = repo.run(&["refs", "AR-cache"]);
    assert_eq!(cache.lines().count(), 2);
    assert_eq!(
        cache.lines().next().unwrap(),
        "docs/architecture/AR-auth.md:3: \u{a7}AR-cache"
    );
    annotated_row(&cache, &format!("{INDEX}:6: \u{a7}AR-cache"));
}

/// §FS-refs.3.3.1: keep summary rows/counts/line lists and append excluded lines.
#[test]
#[ignore = "red until #909 labels excluded refs summaries; implement removes this"]
fn summary_labels_index_entries_without_changing_the_fold() {
    let repo = triage_repo("summary");
    accounting_controls(&repo);
    let auth = repo.run(&["refs", "AR-auth", "--summary"]);
    assert_eq!(auth.lines().count(), 1);
    let suffix = annotated_row(&auth, &format!("{INDEX}: 1 (line 5)"));
    assert!(suffix.contains('5'), "missing excluded line: {auth}");
    let cache = repo.run(&["refs", "AR-cache", "--summary"]);
    assert_eq!(cache.lines().count(), 2);
    assert_eq!(
        cache.lines().next().unwrap(),
        "docs/architecture/AR-auth.md: 1 (line 3)"
    );
    let suffix = annotated_row(&cache, &format!("{INDEX}: 1 (line 6)"));
    assert!(suffix.contains('6'), "missing excluded line: {cache}");
}

/// §FS-check.4.1.2: multiple discounted sites are deterministic and informative.
#[test]
#[ignore = "red until #909 locates multiple excluded entries; implement removes this"]
fn warning_accounts_for_multiple_index_sites() {
    let repo = triage_repo("multiple");
    let path = repo.0.join(INDEX);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("| \u{a7}AR-auth | Another navigation entry |\n");
    repo.write(INDEX, &text);
    repo.run(&["fmt", "--write"]);
    let check = accounting_controls(&repo);
    assert_eq!(check, repo.run(&["check"]));
    let prefix = "docs/architecture/AR-auth.md:1: warning: declared but never cited: AR-auth";
    let suffix = annotated_row(&check, prefix);
    let first = suffix
        .find(&format!("{INDEX}:5"))
        .expect("first excluded site");
    if let Some(second) = suffix.find(&format!("{INDEX}:7")) {
        assert!(first < second, "excluded sites must be ordered: {check}");
    } else {
        assert!(
            suffix.contains("1 more"),
            "missing second site or remaining count: {check}"
        );
    }
    assert!(
        suffix.contains("count as a use"),
        "missing explanation: {check}"
    );
}

/// §FS-check.4.1.2 / §FS-errors.3.1: paths within the explanation use the
/// same output base as the declaration path, including invocation-relative output.
#[test]
#[ignore = "red until #909 renders excluded paths against the report base; implement removes this"]
fn warning_uses_the_invocation_report_base() {
    let repo = triage_repo("invocation");
    let check = repo.run(&[
        "check",
        "docs/architecture/AR-auth.md",
        "--path-base=invocation",
    ]);
    let suffix = annotated_row(
        &check,
        "AR-auth.md:1: warning: declared but never cited: AR-auth",
    );
    assert!(
        suffix.contains("README.md:5") && !suffix.contains("docs/architecture/README.md"),
        "embedded path uses the report base: {check}"
    );
}

/// §FS-refs.3.1.1 / §FS-refs.3.3.1 / §FS-check.4.1.2.1: bindings and
/// sections on an index page are uses, including a sectionless root binding.
#[test]
#[ignore = "red until #909 distinguishes index entries from bindings; implement removes this"]
fn mixed_index_summary_preserves_section_and_root_binding_uses() {
    let repo = Repo::new("bindings");
    repo.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"bindings\"\n\
        [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n",
    );
    repo.write(
        "values/quota.md",
        "# CONST-quota: Yearly quota\n## 1. 1200\n## 2. USD\n",
    );
    repo.write(
        "values/README.md",
        "# Values\n\n- \u{a7}CONST-quota\n\n\
        Quota: `1200 USD` (\u{a7}CONST-quota)\n\nSee \u{a7}CONST-quota.1.\n",
    );
    repo.run(&["fmt", "--write"]);
    assert_eq!(repo.run(&["check"]), "success\n");
    let sites = repo.run(&["refs", "CONST-quota"]);
    let rows: Vec<_> = sites.lines().collect();
    assert_eq!(rows.len(), 3);
    annotated_row(&sites, "values/README.md:3: \u{a7}CONST-quota");
    assert_eq!(rows[1], "values/README.md:5: \u{a7}CONST-quota");
    assert_eq!(rows[2], "values/README.md:7: \u{a7}CONST-quota.1");
    let summary = repo.run(&["refs", "CONST-quota", "--summary"]);
    let suffix = annotated_row(&summary, "values/README.md: 3 (lines 3, 5, 7)");
    assert!(
        suffix.contains('3') && !suffix.contains('5') && !suffix.contains('7'),
        "exclude only line 3: {summary}"
    );
}

/// §FS-check.4.1.2 / §FS-refs.3.1.1 / §FS-refs.3.3.1: canonical external
/// enrollment is excluded, while a bare mention on the very same line is use.
#[test]
#[ignore = "red until #909 labels only the exact external enrollment; implement removes this"]
fn external_enrollment_summary_distinguishes_sites_on_the_same_line() {
    let repo = Repo::new("external");
    repo.write("grund.toml", CONFIG);
    repo.write("src/bus.rs", "/// AR-bus: The event bus\nfn bus() {}\n");
    repo.write(
        INDEX,
        "# Architecture\n\n- [\u{a7}AR-bus](../../src/bus.rs)\n",
    );
    let check = repo.run(&["check"]);
    assert!(check.starts_with("src/bus.rs:1: warning: declared but never cited: AR-bus"));
    // An ordinary unwrapped mention does not enroll an external declaration.
    // Keep the sites on one line so a line-only annotation cannot claim both.
    repo.write(
        INDEX,
        "# Architecture\n\n- [\u{a7}AR-bus](../../src/bus.rs); see \u{a7}AR-bus\n",
    );
    assert_eq!(repo.run(&["check"]), "success\n");
    let sites = repo.run(&["refs", "AR-bus"]);
    let rows: Vec<_> = sites.lines().collect();
    assert_eq!(rows.len(), 2);
    annotated_row(rows[0], &format!("{INDEX}:3: \u{a7}AR-bus"));
    assert_eq!(rows[1], format!("{INDEX}:3: \u{a7}AR-bus"));
    let summary = repo.run(&["refs", "AR-bus", "--summary"]);
    let suffix = annotated_row(&summary, &format!("{INDEX}: 2 (line 3)"));
    // The marker in `- [\u{a7}AR-bus]` is at column 4, not the later plain use.
    assert!(
        suffix.contains("3:4") || (suffix.contains('3') && suffix.contains("column 4")),
        "identify the excluded column: {summary}"
    );
}

/// §FS-refs.3.1.1 / §FS-refs.3.3.1: site-mode has a separate text loop;
/// both glob and outgoing queries must obey the same annotation contract.
#[test]
#[ignore = "red until #909 labels site-mode refs; implement removes this"]
fn glob_and_from_render_the_same_index_annotations() {
    let repo = triage_repo("modes");
    let mut index = fs::read_to_string(repo.0.join(INDEX)).unwrap();
    index = index.replacen("# Architecture", "# AR-directory: Architecture", 1);
    repo.write(INDEX, &index);
    for args in [
        ["refs", "--glob", "AR-*"],
        ["refs", "--from", "AR-directory"],
    ] {
        let text = repo.run(&args);
        annotated_row(&text, &format!("{INDEX}:5: \u{a7}AR-auth"));
        annotated_row(&text, &format!("{INDEX}:6: \u{a7}AR-cache"));
        let mut args = args.to_vec();
        args.push("--summary");
        let summary = repo.run(&args);
        let suffix = annotated_row(&summary, &format!("{INDEX}: 2 (lines 5, 6)"));
        assert!(
            suffix.contains('5') && suffix.contains('6'),
            "identify both excluded lines: {summary}"
        );
    }
}

/// §FS-refs.3.2 / §FS-refs.3.3.1 / §FS-refs.3.4: passing compatibility
/// controls, deliberately active while the explanation tests above are red.
#[test]
fn json_records_and_totals_keep_index_citations_unchanged() {
    let repo = triage_repo("machine-controls");
    let check = accounting_controls(&repo);
    let records = repo.json(&["check"]);
    assert_eq!(records.len(), 2);
    for (path, id) in [
        ("docs/architecture/AR-auth.md", "AR-auth"),
        ("docs/note.md", "NOTE-alone"),
    ] {
        let record = records.iter().find(|row| row["path"] == path).unwrap();
        let message = check
            .lines()
            .find(|row| row.starts_with(&format!("{path}:1:")))
            .unwrap()
            .split_once("warning: ")
            .unwrap()
            .1;
        assert!(message.starts_with(&format!("declared but never cited: {id}")));
        assert_eq!(
            record,
            &json!({"severity": "warning", "path": path, "line": 1,
            "code": "unused", "message": message, "sites": null, "authority": null})
        );
    }
    assert_eq!(
        repo.json(&["refs", "AR-auth"]),
        vec![json!({
            "path": INDEX, "line": 5, "column": 4, "id": "AR-auth", "section": null,
            "marker": true, "text": "\u{a7}AR-auth", "enclosing_declaration": null, "enclosing_section": null
        })]
    );
    assert_eq!(
        repo.json(&["refs", "AR-auth", "--summary"]),
        vec![json!({"path": INDEX, "count": 1, "lines": [5]})]
    );
    assert_eq!(
        repo.json(&["refs", "AR-cache", "--summary"]),
        vec![
            json!({"path": "docs/architecture/AR-auth.md", "count": 1, "lines": [3]}),
            json!({"path": INDEX, "count": 1, "lines": [6]})
        ]
    );
    assert_eq!(
        repo.run(&["refs", "AR-auth", "--total"]),
        "cited at 1 site across 1 file\n"
    );
    assert_eq!(
        repo.json(&["refs", "AR-auth", "--total"]),
        vec![json!({"sites": 1, "files": 1})]
    );
}
