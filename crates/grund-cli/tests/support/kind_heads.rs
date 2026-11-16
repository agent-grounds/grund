//! Reciprocal citations, attribution, deletion and glob controls for both config
//! readers (§FS-config.3.4.5.1, §FS-config.3.2,
//! §FS-coordinate-globs.errors.note, §FS-check.3.1).

use super::case_runner::{CaseKind::E2e, assert_every_case_passed, copy_dir, run_case};
use super::repo_root;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "fixture_cleanup.rs"]
mod fixture_cleanup;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "grund-1357-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("exclusively claim fixture");
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.0, "remove kind-head fixture");
    }
}

#[test]
#[ignore = "red until #1357 GR/GRUND config admission; implement removes this"]
fn kind_heads_reciprocal_contract() {
    let root = repo_root();
    let outcomes = ["kind-heads-v1", "kind-heads-v2"]
        .iter()
        .map(|name| run_case(&root, &root.join("tests/e2e/cases").join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("kind heads reciprocal", &outcomes);
}

fn run(root: &Path, args: &[&str], exit: i32) -> (String, String) {
    let result = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund");
    let out = String::from_utf8(result.stdout).unwrap();
    let err = String::from_utf8(result.stderr).unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "{args:?}\nstdout: {out}\nstderr: {err}"
    );
    (out, err)
}

fn json(root: &Path, args: &[&str]) -> Vec<Value> {
    let (out, _) = run(root, args, 0);
    out.lines()
        .map(|line| serde_json::from_str(line).expect("JSON record"))
        .collect()
}

#[test]
#[ignore = "red until #1357 GR/GRUND config admission; implement removes this"]
fn kind_heads_queries_dangling_and_glob_contract() {
    for version in [1, 2] {
        let fixture = repo_root().join(format!("tests/e2e/cases/kind-heads-v{version}/repo"));
        let temp = Fixture::new();
        copy_dir(&fixture, temp.path());
        let root = temp.path();
        let listed = json(root, &["list", "--format", "json"]);
        assert_eq!(listed.len(), 2);
        for (id, kind, path, source, enclosing) in [
            (
                "GR-80923",
                "GR",
                "docs/tickets.md",
                "docs/grund.md",
                "GRUND-why",
            ),
            (
                "GRUND-why",
                "GRUND",
                "docs/grund.md",
                "docs/tickets.md",
                "GR-80923",
            ),
        ] {
            let row = listed
                .iter()
                .find(|row| row["id"] == id)
                .expect("listed target");
            assert_eq!(row["kind"], kind);
            assert_eq!(row["path"], path);
            assert_eq!(row["refs"], 1);
            let scoped = json(root, &["list", "--kind", kind, "--format", "json"]);
            assert_eq!(scoped.len(), 1);
            assert_eq!(scoped[0]["id"], id);
            let shown = json(root, &[id, "--format", "json"]);
            assert_eq!(shown.len(), 1);
            assert_eq!(shown[0]["id"], id);
            assert_eq!(shown[0]["path"], path);
            assert!(shown[0]["body"].as_str().unwrap().contains(enclosing));
            let refs = json(root, &["refs", id, "--format", "json"]);
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0]["id"], id);
            assert_eq!(refs[0]["path"], source);
            assert_eq!(refs[0]["enclosing_declaration"], enclosing);
        }
        for pattern in ["GR-*", "GRUND-*"] {
            let (out, err) = run(root, &["list", "--glob", pattern], 0);
            assert_eq!(out.lines().count(), 1);
            assert!(!err.contains("no configured kind"));
        }
        let (out, err) = run(root, &["list", "--glob", "GRUND_*"], 0);
        assert!(out.is_empty());
        assert!(err.contains("note: no configured kind in scope starts with \"GRUND_\""));
        for (file, id, remaining) in [
            ("docs/grund.md", "GRUND-why", "GR-80923"),
            ("docs/tickets.md", "GR-80923", "GRUND-why"),
        ] {
            let path = root.join(file);
            let saved = std::fs::read_to_string(&path).unwrap();
            std::fs::write(
                &path,
                format!("# No declaration\n\nSee \u{a7}{remaining}.\n"),
            )
            .unwrap();
            let (out, _) = run(root, &["check"], 1);
            assert_eq!(
                out.matches(&format!("unknown reference {id}")).count(),
                1,
                "{out}"
            );
            std::fs::write(path, saved).unwrap();
        }
    }
}

/// §FS-fmt.2.1: a reachable alias-qualified trigger in an already-loadable
/// config must not consume the default's boundary in place of its kind's.
#[test]
#[ignore = "red until #1357 alias-qualified trigger literals; implement removes this"]
fn kind_heads_alias_fmt_keeps_off_format_trigger() {
    let temp = Fixture::new();
    let root = temp.path();
    std::fs::write(
        root.join("grund.toml"),
        "grund_config_version = 2\n[schema]\nid_format = \"{kind}-{number}-{slug}\"\n\
         [schema.kinds.ADR]\nfiles = [\"adr.md\"]\nid_format = \"{kind}:{number}:{slug}\"\n",
    )
    .unwrap();
    std::fs::write(root.join("adr.md"), "# ADR:7:why: Why\n").unwrap();
    let source = "See $$api/ADR:7:Upper.\n";
    std::fs::write(root.join("notes.md"), source).unwrap();
    run(root, &["fmt", "--write"], 0);
    assert_eq!(
        std::fs::read_to_string(root.join("notes.md")).unwrap(),
        source
    );
}

/// §FS-config.3.2 / §FS-fmt.6.3.1: scanner, shorthand resolution and accepted
/// shorthand links must use the longer matched kind's own literal separators.
#[test]
#[ignore = "red until #1357 GR/GRUND admission and parsed-kind literals; implement removes this"]
fn kind_heads_per_kind_literals_survive_scan_and_fmt() {
    let temp = Fixture::new();
    let root = temp.path();
    std::fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n[reference]\nshorthand = \"accepted\"\n[id]\nformat = \"{kind}-{slug}\"\nsection_separator = \"::\"\n\
         [[kinds]]\nkind = \"GR\"\nfile = \"tickets.md\"\nformat = \"{kind}-{number}\"\n\
         [[kinds]]\nkind = \"GRUND\"\nfile = \"grund.md\"\nformat = \"{kind}.{number}.{slug}\"\n\
         [fmt.cross_refs]\nenabled = true\n",
    )
    .unwrap();
    std::fs::write(
        root.join("tickets.md"),
        "# GR-80923: Ticket\n\nSee \u{a7}GRUND.7 and \u{a7}GRUND.7.Upper.\n",
    )
    .unwrap();
    std::fs::write(
        root.join("grund.md"),
        "# GRUND.7.why: Why\n\nSee \u{a7}GR-80923.\n",
    )
    .unwrap();
    let (out, _) = run(root, &["check"], 1);
    assert_eq!(
        out.matches("off-format citation \u{a7}GRUND.7.Upper")
            .count(),
        1,
        "{out}"
    );
    let refs = json(root, &["refs", "GRUND.7.why", "--format", "json"]);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["id"], "GRUND.7.why");
    run(root, &["fmt", "--write"], 0);
    let text = std::fs::read_to_string(root.join("tickets.md")).unwrap();
    assert!(text.contains("[\u{a7}GRUND.7]("), "{text}");
    assert!(text.contains(" and \u{a7}GRUND.7.Upper."), "{text}");
}
