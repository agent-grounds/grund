//! Black-box regression for index-specific direction repairs (§FS-check.3.12.2).
//! The CLI's verdict, site and JSON shape stay fixed; the suggested edit must
//! preserve the marked index entry that §FS-check.3.18.5 requires.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const INDEX: &str = "docs/requirements/README.md";
const OLD_REPAIR: &str = " — re-point the citation or downgrade it to a plain Markdown link";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new(rules: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-842-index-repair-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repo = Self(root);
        repo.write(
            "grund.toml",
            &format!(
                r#"grund_config_version = 2
project_name = "issue-842"
[schema.sources]
languages = ["markdown"]
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.REQ]
folders = ["docs/requirements"]
{rules}
"#
            ),
        );
        repo.write("docs/goals.md", "# GOAL-fast: Fast\n");
        repo.write(
            "docs/requirements/REQ-safe.md",
            "# REQ-safe: Safe\n\nSupports \u{a7}GOAL-fast.\n",
        );
        repo.write(INDEX, "# Requirements\n\n- [\u{a7}REQ-safe](REQ-safe.md)\n");
        repo
    }

    fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn replace(&self, path: &str, from: &str, to: &str) {
        let body = fs::read_to_string(self.0.join(path)).unwrap();
        assert!(body.contains(from), "fixture replacement missed {from:?}");
        self.write(path, &body.replace(from, to));
    }

    fn check(&self, exit: i32, suggestions: bool) -> Vec<Value> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
        command
            .current_dir(&self.0)
            .args(["check", ".", "--format", "json"]);
        if suggestions {
            command.arg("--suggestions");
        }
        let output = command.output().expect("run grund check");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(exit), "{stdout}{stderr}");
        stdout
            .lines()
            .chain(stderr.lines())
            .map(|line| {
                serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}"))
            })
            .collect()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test repository");
    }
}

fn kind_default(level: &str) -> String {
    format!("[rules.citations.REQ]\ndefault = \"{level}\"\nmay = [\"GOAL\"]")
}

fn direction(rows: &[Value], severity: &str, authority: Value) -> String {
    let row = rows
        .iter()
        .find(|row| row["code"] == "forbidden-citation" && row["path"] == INDEX && row["line"] == 3)
        .unwrap_or_else(|| panic!("missing direction finding: {rows:#?}"));
    assert_eq!(row["severity"], severity, "{row}");
    assert_eq!(row["authority"], authority, "{row}");
    assert_eq!(row["sites"], Value::Null, "{row}");
    assert_eq!(
        row.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "authority",
            "code",
            "line",
            "message",
            "path",
            "severity",
            "sites"
        ]
    );
    let message = row["message"].as_str().unwrap();
    let label = authority
        .as_array()
        .and_then(|ids| ids.first())
        .and_then(Value::as_str)
        .unwrap_or("citation direction");
    assert!(
        message.starts_with(&format!("REQ must not cite REQ ({label}) — ")),
        "{message}"
    );
    message
        .split_once(" — ")
        .expect("repair suffix")
        .1
        .to_owned()
}

fn preserves_entry(repair: &str) {
    assert!(
        !repair.contains("plain Markdown link"),
        "index repair still proposes a refused edit: {repair}"
    );
    assert!(
        repair.contains("entry"),
        "repair must tell the reader to keep the entry: {repair}"
    );
}

fn default_repair(rules: &str, exit: i32, severity: &str) {
    let repo = Repo::new(rules);
    // Establish the accepted edit before judging advice, so an unrelated fixture
    // error cannot produce the intended red result (§FS-check.3.12.2).
    repo.replace(
        "grund.toml",
        "may = [\"GOAL\"]",
        "may = [\"GOAL\", \"REQ\"]",
    );
    let repaired = repo.check(0, false);
    assert!(
        repaired.iter().all(|row| row["severity"] != "error"),
        "{repaired:#?}"
    );
    assert!(
        !repaired
            .iter()
            .any(|row| row["code"] == "forbidden-citation")
    );
    repo.replace(
        "grund.toml",
        "may = [\"GOAL\", \"REQ\"]",
        "may = [\"GOAL\"]",
    );
    let rows = repo.check(exit, false);
    assert_eq!(
        rows.iter()
            .filter(|row| row["code"] == "forbidden-citation")
            .count(),
        1
    );
    let repair = direction(&rows, severity, Value::Null);
    preserves_entry(&repair);
    for word in ["REQ", "may", "[rules.citations.REQ]"] {
        assert!(
            repair.contains(word),
            "missing {word} in default repair: {repair}"
        );
    }
}

#[test]
fn kind_default_index_repair_names_the_may_edit() {
    default_repair(&kind_default("must-not"), 1, "error");
}

#[test]
fn warning_default_index_repair_keeps_its_channel() {
    default_repair(&kind_default("warn-not"), 0, "warning");
}

#[test]
fn global_default_index_repair_names_the_kind_override() {
    default_repair(
        "[rules.citations]\ndefault = \"must-not\"\n[rules.citations.REQ]\nmay = [\"GOAL\"]",
        1,
        "error",
    );
}

#[test]
fn explicit_prohibition_index_repair_names_the_actual_list() {
    let repo = Repo::new("[rules.citations.REQ]\nmust-not = [\"REQ\"]");
    let rows = repo.check(1, false);
    let repair = direction(&rows, "error", Value::Null);
    repo.replace("grund.toml", "must-not = [\"REQ\"]", "may = [\"REQ\"]");
    repo.check(0, false);
    preserves_entry(&repair);
    for word in ["must-not", "[rules.citations.REQ]"] {
        assert!(
            repair.contains(word),
            "missing {word} in explicit repair: {repair}"
        );
    }
}

#[test]
fn rule_sentence_index_repair_names_its_authority() {
    let repo = Repo::new(concat!(
        "[rules.citations.REQ]\nmay = [\"GOAL\", \"REQ\"]\n",
        "[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\""
    ));
    repo.write(
        INDEX,
        "# REQ-index: Requirements\n\n- [\u{a7}REQ-safe](REQ-safe.md)\n- [\u{a7}REQ-index](README.md)\n",
    );
    repo.write(
        "docs/rules/r.md",
        "# RULE-index: Each REQ must not cite any REQ.\n\nIndependent prohibition.\n",
    );
    let rows = repo.check(1, false);
    assert_eq!(
        rows.iter().filter(|row| row["severity"] == "error").count(),
        2,
        "{rows:#?}"
    );
    let repair = direction(&rows, "error", serde_json::json!(["RULE-index"]));
    fs::remove_file(repo.0.join("docs/rules/r.md")).unwrap();
    repo.check(0, false);
    preserves_entry(&repair);
    assert!(
        repair.contains("RULE-index"),
        "name the rule to change: {repair}"
    );
}

/// §FS-check.3.18.5: removing the marker is not an accepted repair.
#[test]
fn markerless_index_link_still_fails_after_the_may_edit() {
    let repo = Repo::new("[rules.citations.REQ]\nmay = [\"GOAL\", \"REQ\"]");
    repo.check(0, false);
    repo.replace(INDEX, "\u{a7}REQ-safe", "REQ-safe");
    let rows = repo.check(1, false);
    let errors = rows
        .iter()
        .filter(|row| row["severity"] == "error")
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{rows:#?}");
    assert_eq!(errors[0]["code"], "missing-index-entry");
    assert_eq!(errors[0]["path"], "docs/requirements/REQ-safe.md");
    assert!(
        errors[0]["message"]
            .as_str()
            .unwrap()
            .contains("not as an entry")
    );
}

/// §FS-check.3.12.2: a different target in the same index is an ordinary site.
#[test]
fn ordinary_index_citation_retains_plain_link_advice() {
    let repo = Repo::new("[rules.citations.REQ]\nmay = [\"REQ\"]\nmust-not = [\"GOAL\"]");
    repo.write("docs/requirements/REQ-safe.md", "# REQ-safe: Safe\n");
    repo.write(
        INDEX,
        "# Requirements\n\n- [\u{a7}REQ-safe](REQ-safe.md)\n\nSee \u{a7}GOAL-fast.\n",
    );
    let rows = repo.check(1, false);
    let forbidden = rows
        .iter()
        .find(|row| row["code"] == "forbidden-citation")
        .unwrap();
    assert_eq!(forbidden["path"], INDEX);
    assert_eq!(forbidden["line"], 5);
    assert_eq!(
        forbidden["message"],
        format!("REQ must not cite GOAL (citation direction){OLD_REPAIR}")
    );
    repo.replace(INDEX, "\u{a7}GOAL-fast", "[GOAL-fast](../goals.md)");
    repo.check(0, false);
}

/// §FS-check.3.12.2: suggestion defaults keep the existing suffix-free wording.
#[test]
fn suggestion_default_keeps_its_existing_index_message() {
    let repo = Repo::new(&kind_default("should-not"));
    let rows = repo.check(0, true);
    let suggestion = rows
        .iter()
        .find(|row| row["code"] == "discouraged-citation")
        .unwrap();
    assert_eq!(suggestion["path"], INDEX);
    assert_eq!(suggestion["line"], 3);
    assert_eq!(
        suggestion["message"],
        "REQ should not cite REQ (citation direction) — re-point the citation or downgrade it to a plain Markdown link"
    );
}

/// §FS-check.3.12.2: rule advice preserves entries, on a target pinned at the rule's own nested
/// alias path too, which is the ordinary error since `0.19.0` (§FS-rules.7.9).
#[test]
fn nested_alias_rule_index_repair_preserves_entries_as_an_error() {
    let repo = Repo::new(concat!(
        "[rules.citations.REQ]\nmay = [\"GOAL\", \"REQ\"]\n",
        "[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\""
    ));
    fs::create_dir_all(repo.0.join("group/api")).unwrap();
    for path in ["grund.toml", "docs"] {
        fs::rename(repo.0.join(path), repo.0.join("group/api").join(path)).unwrap();
    }
    repo.replace(
        "group/api/grund.toml",
        "project_name = \"issue-842\"",
        "project_name = \"api\"",
    );
    let base = "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\"]\n";
    repo.write(
        "grund.toml",
        &format!("{base}[workspace]\nmembers = [\"group\"]\n"),
    );
    repo.write(
        "group/grund.toml",
        &format!("{base}[workspace]\nmembers = [\"api\"]\n"),
    );
    let index_path = format!("group/api/{INDEX}");
    let marked = "# REQ-index: Requirements\n\n- [\u{a7}REQ-safe](REQ-safe.md)\n- [\u{a7}REQ-index](README.md)\n";
    let rule_path = "group/api/docs/rules/r.md";
    repo.write(&index_path, marked);
    repo.write(
        rule_path,
        "# RULE-index: Each REQ must not cite any group/api/REQ.\n\nIndependent prohibition.\n",
    );
    let rows = repo.check(1, false);
    let errors = rows
        .iter()
        .filter(|row| row["severity"] == "error")
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 2, "{rows:#?}");
    // §FS-rules.7.9: the nested-alias window closed in 0.19.0, so no clause follows the advice.
    for line in [3, 4] {
        let row = errors
            .iter()
            .find(|row| row["line"] == line)
            .unwrap_or_else(|| panic!("missing entry at line {line}: {rows:#?}"));
        assert_eq!(row["code"], "forbidden-citation");
        assert_eq!(row["path"], index_path);
        assert_eq!(row["authority"], serde_json::json!(["RULE-index"]));
        assert_eq!(row["sites"], Value::Null);
        assert_eq!(
            row["message"],
            "REQ must not cite group/api/REQ (RULE-index) — retain the marked index entry and change the prohibition in rule RULE-index"
        );
    }
    repo.write(&index_path, &marked.replace('\u{a7}', ""));
    let markerless = repo.check(1, false);
    let errors = markerless
        .iter()
        .filter(|row| row["severity"] == "error")
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 2, "{markerless:#?}");
    for path in [
        index_path.as_str(),
        "group/api/docs/requirements/REQ-safe.md",
    ] {
        assert!(
            errors
                .iter()
                .any(|row| row["code"] == "missing-index-entry" && row["path"] == path),
            "{markerless:#?}"
        );
    }
    repo.write(&index_path, marked);
    fs::remove_file(repo.0.join(rule_path)).unwrap();
    let repaired = repo.check(0, false);
    assert!(repaired.iter().all(|row| row["severity"] != "error"));
    assert!(
        !repaired
            .iter()
            .any(|row| row["code"] == "forbidden-citation")
    );
}
