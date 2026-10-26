//! Binary regressions for agent-grounds/grund#1023: the no-kinds warning gives
//! usable offline help (§FS-check.4.5.2), and the parent kind table teaches the
//! child's complete key rows without another lookup (§FS-config-explain.1.2).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
const CHILD: &str = "[schema.kinds.<NAME>]";
const CHILD_KEYS: &[&str] = &[
    "files",
    "folders",
    "citable",
    "scan",
    "index",
    "id_format",
    "fetch",
    "form",
    "closed",
];

struct Project(PathBuf);

impl Project {
    fn new(version: u32) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1023-first-kind-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/adr")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let empty = if version == 1 { "kinds = []\n" } else { "" };
        fs::write(
            root.join("grund.toml"),
            format!("grund_config_version = {version}\nproject_name = \"orders\"\n{empty}"),
        )
        .unwrap();
        fs::write(
            root.join("docs/adr/use-postgres.md"),
            "# ADR-use-postgres: Use Postgres\n\nOrders live in Postgres.\n",
        )
        .unwrap();
        fs::write(
            root.join("src/db.py"),
            "# Orders are stored in Postgres (§ADR-use-postgres).\n",
        )
        .unwrap();
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run built grund")
    }

    fn explain(&self, target: &str) -> String {
        let out = self.run(&["config", "explain", target]);
        assert_eq!(out.status.code(), Some(0), "{target}: {out:?}");
        assert!(out.stderr.is_empty(), "{target}: {out:?}");
        String::from_utf8(out.stdout).unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

/// The existing human table rows include the name, values and complete meaning.
fn key_rows(answer: &str) -> Vec<&str> {
    answer
        .lines()
        .filter(|line| line.starts_with("  ") && line.contains(" ("))
        .map(str::trim)
        .collect()
}

/// §FS-check.4.5.2: both versions name a usable, quoted command, without moving
/// the diagnostic identity, stream, severity, location fields or exit.
fn warning_teaches_kind(version: u32, target: &str, place_keys: &[&str]) {
    let project = Project::new(version);
    let child = project.explain(target);
    let rows = key_rows(&child);
    for key in place_keys {
        assert!(
            rows.iter().any(|row| row.starts_with(&format!("{key} ("))),
            "precondition: {target} must already teach {key}:\n{child}"
        );
    }

    let out = project.run(&["check"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let stderr = String::from_utf8(out.stderr).unwrap();
    let warning = stderr
        .lines()
        .filter(|line| line.starts_with("warning: nothing recognized"))
        .collect::<Vec<_>>();
    assert_eq!(
        warning.len(),
        1,
        "precondition: no-kinds warning:\n{stderr}"
    );
    assert!(
        warning[0].contains("this project declares no kinds"),
        "{stderr}"
    );

    let json = project.run(&["check", "--format", "json"]);
    assert_eq!(json.status.code(), Some(0), "{json:?}");
    assert!(json.stdout.is_empty(), "{json:?}");
    let json_stderr = String::from_utf8(json.stderr).unwrap();
    let warnings: Vec<serde_json::Value> = json_stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON warning on stderr"))
        .collect();
    let diagnostic = warnings
        .iter()
        .find(|w| w["code"] == "nothing-recognized")
        .expect("same structured diagnostic");
    assert_eq!(warnings.len(), if version == 1 { 2 } else { 1 });
    assert_eq!(diagnostic["severity"], "warning");
    for field in ["path", "line", "column", "channel"] {
        assert!(diagnostic[field].is_null(), "{field}: {diagnostic}");
    }
    assert!(diagnostic["sites"].is_null(), "{diagnostic}");
    assert!(diagnostic["authority"].is_null(), "{diagnostic}");
    assert_eq!(
        diagnostic["message"].as_str().unwrap(),
        warning[0].strip_prefix("warning: ").unwrap()
    );

    let command = format!("grund config explain '{target}'");
    assert!(
        warning[0].contains(&command),
        "missing shell-ready `{command}` in no-kinds warning:\n{stderr}"
    );
    // Execute the command's exact target as one shell-quoted argument would.
    assert_eq!(project.explain(target), child);
}

#[test]
fn v2_no_kinds_warning_names_usable_kind_help() {
    warning_teaches_kind(2, CHILD, &["files", "folders"]);
}

#[test]
fn v1_no_kinds_warning_names_usable_kind_help() {
    warning_teaches_kind(1, "[[kinds]]", &["file", "folder"]);
}

/// §FS-config-explain.1.2: the parent shows the child's exact rows in reader
/// order, owned by that child. Index and --all keep each row economical.
fn parent_teaches_kind(target: &str) {
    let project = Project::new(2);
    let child = project.explain(CHILD);
    let rows = key_rows(&child);
    let keys: Vec<&str> = rows
        .iter()
        .map(|row| row.split_once(" (").unwrap().0)
        .collect();
    assert_eq!(
        keys, CHILD_KEYS,
        "precondition: complete direct child answer:\n{child}"
    );
    assert!(
        rows.iter()
            .all(|row| row.contains("): ") && !row.ends_with(": ")),
        "precondition: values and meanings in child rows:\n{child}"
    );

    let parent = project.explain(target);
    assert_eq!(
        key_rows(&parent),
        rows,
        "{target} must inline the complete per-kind rows (values, meanings, order):\n{parent}"
    );
    let first_row = parent.find(rows[0]).unwrap();
    assert!(
        parent[..first_row].lines().any(|line| (line.trim() == CHILD
            || (line.contains(CHILD) && line.to_ascii_lowercase().contains("keys")))
            && !line.starts_with("Tables under it:")
            && !line.starts_with("Next:")),
        "child keys need an owning-table label before their rows:\n{parent}"
    );
    let alias = if target == "schema.kinds" {
        "[schema.kinds]"
    } else {
        "schema.kinds"
    };
    assert_eq!(project.explain(alias), parent);

    let index = project.run(&["config", "explain"]);
    assert!(index.status.success(), "{index:?}");
    assert!(key_rows(std::str::from_utf8(&index.stdout).unwrap()).is_empty());
    let all = project.run(&["config", "explain", "--all"]);
    assert!(all.status.success(), "{all:?}");
    let all = String::from_utf8(all.stdout).unwrap();
    for row in rows {
        assert_eq!(
            all.lines().filter(|line| line.trim() == row).count(),
            1,
            "--all should print the child's row once: {row}"
        );
    }
}

#[test]
fn dotted_parent_lists_complete_kind_keys_inline() {
    parent_teaches_kind("schema.kinds");
}

#[test]
fn bracketed_parent_lists_complete_kind_keys_inline() {
    parent_teaches_kind("[schema.kinds]");
}
