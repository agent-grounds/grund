//! §FS-config-explain.5: the built binary refuses a dotted path to a table or a
//! key the version admits with a line naming the bracketed spelling that
//! answers, and `grund config --help` teaches that spelling with a table and a
//! key example (§FS-config-explain.1.1). Reported as agent-grounds/grund#904.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A project outside any checkout holding only a `grund.toml` of `version`.
struct Project(PathBuf);

impl Project {
    fn new(version: u32) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-explain-dotted-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let table = if version == 1 {
            "[reference]\nstrict = false\n"
        } else {
            "[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n"
        };
        let config =
            format!("grund_config_version = {version}\nproject_name = \"demo\"\n\n{table}");
        fs::write(root.join("grund.toml"), config).unwrap();
        Self(root)
    }

    fn explain(&self, target: &str) -> Output {
        run(&self.0, &["config", "explain", target])
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run grund")
}

/// Stdout empty, exit 1, and stderr exactly `line`.
fn assert_refused(project: &Project, target: &str, line: &str) {
    let out = project.explain(target);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr, format!("{line}\n"), "{target}");
    assert!(out.stdout.is_empty(), "{target}: stdout {:?}", out.stdout);
    assert_eq!(out.status.code(), Some(1), "{target}");
}

/// §FS-config-explain.5: a dotted path to a table or a key names the bracketed
/// spelling, in version 2 and in version 1, and is still not answered.
#[test]
#[ignore = "red until #904 names the bracketed spelling for a dotted path; implement removes this"]
fn a_dotted_path_is_refused_with_its_bracketed_spelling() {
    let v2 = Project::new(2);
    assert_refused(
        &v2,
        "schema.sources",
        "version 2 has no top-level key or check code `schema.sources`; for the table, ask grund config explain '[schema.sources]'",
    );
    assert_refused(
        &v2,
        "schema.sources.exclude",
        "version 2 has no top-level key or check code `schema.sources.exclude`; for the key, ask grund config explain '[schema.sources] exclude'",
    );
    assert_refused(
        &v2,
        "schema.kinds.FS",
        "version 2 has no top-level key or check code `schema.kinds.FS`; for the table, ask grund config explain '[schema.kinds.FS]'",
    );
    let v1 = Project::new(1);
    assert_refused(
        &v1,
        "reference",
        "version 1 has no top-level key or check code `reference`; for the table, ask grund config explain '[reference]'",
    );
    assert_refused(
        &v1,
        "reference.strict",
        "version 1 has no top-level key or check code `reference.strict`; for the key, ask grund config explain '[reference] strict'",
    );
}

/// §FS-config-explain.5: a name that is no path from the top keeps the catch-all.
#[test]
fn a_bare_leaf_keeps_the_catch_all() {
    assert_refused(
        &Project::new(2),
        "sources",
        "version 2 knows no table, key or check code `sources`; grund config explain lists the tables, grund config explain --codes the codes",
    );
}

/// §FS-config-explain.1.1: the help page's explain examples include a target
/// answered as a table and one answered as a key of a table.
#[test]
#[ignore = "red until #904 adds a table and a key explain example to grund config --help; implement removes this"]
fn the_help_shows_a_table_and_a_key_example_that_answer() {
    let v2 = Project::new(2);
    let help = run(&v2.0, &["config", "--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    let targets: Vec<String> = help
        .lines()
        .filter_map(|line| {
            line.split_once("grund config explain ")
                .map(|(_, rest)| rest)
        })
        .filter_map(|rest| match rest.strip_prefix('\'') {
            Some(quoted) => quoted.split_once('\'').map(|(t, _)| t.to_string()),
            None if rest.starts_with('-') => None,
            None => rest.split_whitespace().next().map(str::to_string),
        })
        .collect();
    let headings: Vec<String> = targets
        .iter()
        .filter_map(|target| {
            let out = v2.explain(target);
            out.status.success().then(|| {
                let text = String::from_utf8_lossy(&out.stdout).into_owned();
                text.lines().nth(2).unwrap_or_default().to_string()
            })
        })
        .collect();
    let table = headings
        .iter()
        .any(|h| h.starts_with('[') && h.ends_with(']'));
    let key = headings.iter().any(|h| {
        h.starts_with('[') && h.contains("] ") && !h[h.find("] ").unwrap() + 2..].contains(' ')
    });
    assert!(
        table && key,
        "explain examples {targets:?} answer as {headings:?}"
    );
}
