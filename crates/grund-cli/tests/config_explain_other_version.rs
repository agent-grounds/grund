//! §FS-config-explain.5: the built binary answers a bare or dotted target whose
//! leading segments name a table only the other version admits with the failure
//! that table's bracketed spelling gets, and reads the version's own tables
//! first. Reported as agent-grounds/grund#935.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A project outside any checkout holding only a `grund.toml` of `version`.
struct Project(PathBuf);

impl Project {
    fn new(version: u32) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-explain-other-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let config = format!("grund_config_version = {version}\nproject_name = \"demo\"\n");
        fs::write(root.join("grund.toml"), config).unwrap();
        Self(root)
    }

    fn explain(&self, target: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["config", "explain", target])
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `target` prints exactly what `bracketed` prints, which is the other-version
/// answer holding `marker`: nothing on stdout, exit 1.
fn assert_answered_as(project: &Project, target: &str, bracketed: &str, marker: &str) {
    let expected = project.explain(bracketed);
    let expected_err = String::from_utf8_lossy(&expected.stderr).into_owned();
    assert!(
        expected_err.contains(marker),
        "{bracketed} is not the other-version answer: {expected_err:?}"
    );
    let out = project.explain(target);
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        expected_err,
        "{target} vs {bracketed}"
    );
    assert!(out.stdout.is_empty(), "{target}: stdout {:?}", out.stdout);
    assert_eq!(out.status.code(), Some(1), "{target}");
}

/// Stdout empty, exit 1, and stderr exactly `line`.
fn assert_refused(project: &Project, target: &str, line: &str) {
    let out = project.explain(target);
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("{line}\n"),
        "{target}"
    );
    assert!(out.stdout.is_empty(), "{target}: stdout {:?}", out.stdout);
    assert_eq!(out.status.code(), Some(1), "{target}");
}

/// §FS-config-explain.5: under version 2, a version-1 table asked bare or dotted
/// gets the answer its bracketed spelling gets, by the longest leading run.
#[test]
fn a_version_one_table_asked_bare_or_dotted_under_version_two() {
    let v2 = Project::new(2);
    let v1_table = "it is a version 1 table";
    assert_answered_as(&v2, "reference", "[reference]", v1_table);
    assert_answered_as(&v2, "reference.strict", "[reference]", v1_table);
    assert_answered_as(
        &v2,
        "reference.lead_size_warning.max",
        "[reference]",
        v1_table,
    );
    assert_answered_as(&v2, "scan.include", "[scan]", v1_table);
    assert_answered_as(&v2, "id.format", "[id]", v1_table);
    assert_answered_as(&v2, "kinds", "[[kinds]]", v1_table);
    assert_answered_as(&v2, "citations.FS", "[citations.FS]", v1_table);
    assert_answered_as(&v2, "fmt.cross_refs", "[fmt.cross_refs]", v1_table);
}

/// §FS-config-explain.5: under version 1, a version-2 table asked dotted gets
/// the answer its bracketed spelling gets.
#[test]
fn a_version_two_table_asked_dotted_under_version_one() {
    let v1 = Project::new(1);
    let v2_table = "it is a version 2 table";
    assert_answered_as(&v1, "schema.sources", "[schema.sources]", v2_table);
    assert_answered_as(&v1, "schema.sources.exclude", "[schema.sources]", v2_table);
    assert_answered_as(&v1, "schema.kinds.FS", "[schema.kinds.FS]", v2_table);
}

/// §FS-config-explain.5: the version's own tables are read first, a table both
/// versions admit is the version's own, and a name neither reading knows keeps
/// the catch-all.
#[test]
fn own_tables_come_first_and_the_rest_keeps_the_catch_all() {
    let v1 = Project::new(1);
    assert_refused(
        &v1,
        "workspace.members",
        "version 1 has no top-level key or check code `workspace.members`; for the key, ask grund config explain '[workspace] members'",
    );
    assert_refused(
        &v1,
        "workspace.nope",
        "version 1 knows no table, key or check code `workspace.nope`; grund config explain lists the tables, grund config explain --codes the codes",
    );
    let v2 = Project::new(2);
    assert_refused(
        &v2,
        "workspace",
        "version 2 has no top-level key or check code `workspace`; for the table, ask grund config explain '[workspace]'",
    );
    assert_refused(
        &v2,
        "sources",
        "version 2 knows no table, key or check code `sources`; grund config explain lists the tables, grund config explain --codes the codes",
    );
}
