//! §FS-config-explain.1.1: the built binary answers a path with exactly one
//! reading as its bracketed spelling, byte for byte, and §FS-config-explain.5:
//! a path with two readings and a bare name that tables hold fail naming every
//! spelling that answers. `grund config --help` teaches the bracketed spelling
//! with a table and a key example. Reported as agent-grounds/grund#904 and
//! agent-grounds/grund#1005.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A project outside any checkout holding only a `grund.toml` of `version`.
struct Project(PathBuf);

impl Project {
    fn new(version: u32) -> Self {
        let table = if version == 1 {
            "[reference]\nstrict = false\n"
        } else {
            "[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n"
        };
        let project = Self::without_config();
        let config =
            format!("grund_config_version = {version}\nproject_name = \"demo\"\n\n{table}");
        fs::write(project.0.join("grund.toml"), config).unwrap();
        project
    }

    /// A directory no `grund.toml` encloses, so both versions answer.
    fn without_config() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-explain-dotted-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
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

/// Stdout empty, exit 1, and stderr exactly `lines`, each ending in a newline.
fn assert_refused(project: &Project, target: &str, lines: &[&str]) {
    let out = project.explain(target);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let expected: String = lines.iter().map(|line| format!("{line}\n")).collect();
    assert_eq!(stderr, expected, "{target}");
    assert!(out.stdout.is_empty(), "{target}: stdout {:?}", out.stdout);
    assert_eq!(out.status.code(), Some(1), "{target}");
}

/// `target` prints exactly what `bracketed` prints, on both streams, and both
/// exit 0.
fn assert_answered_as(project: &Project, target: &str, bracketed: &str) {
    let expected = project.explain(bracketed);
    assert_eq!(
        expected.status.code(),
        Some(0),
        "{bracketed} does not answer"
    );
    let out = project.explain(target);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&expected.stdout),
        "{target} vs {bracketed}"
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&expected.stderr),
        "{target} vs {bracketed}"
    );
    assert_eq!(out.status.code(), Some(0), "{target}");
}

/// §FS-config-explain.1.1: every dotted example the spec gives prints the same
/// bytes as its bracketed spelling and exits 0, in version 2 and in version 1.
#[test]
fn a_path_with_one_reading_is_answered_as_its_bracketed_spelling() {
    let v2 = Project::new(2);
    for (target, bracketed) in [
        ("schema", "[schema]"),
        ("schema.sources", "[schema.sources]"),
        ("schema.sources.exclude", "[schema.sources] exclude"),
        ("schema.kinds.FS", "[schema.kinds.FS]"),
        ("schema.kinds.ADR", "[schema.kinds.ADR]"),
        ("schema.kinds.FS.id_format", "[schema.kinds.FS] id_format"),
        ("rules.citations.grounding", "[rules.citations.grounding]"),
    ] {
        assert_answered_as(&v2, target, bracketed);
    }
    let v1 = Project::new(1);
    for (target, bracketed) in [
        ("reference", "[reference]"),
        ("reference.strict", "[reference] strict"),
        (
            "reference.lead_size_warning.max",
            "[reference] lead_size_warning.max",
        ),
        ("fmt.cross_refs", "[fmt.cross_refs]"),
    ] {
        assert_answered_as(&v1, target, bracketed);
    }
}

/// §FS-config-explain.2: with no config both versions answer, so a path that
/// version 2 answers is exit 0, as its bracketed spelling is.
#[test]
fn a_path_one_version_answers_is_answered_with_no_config() {
    assert_answered_as(
        &Project::without_config(),
        "schema.sources",
        "[schema.sources]",
    );
}

/// §FS-config-explain.5: a path with two readings names the one with the name
/// left out on the line and the other on a `hint:` line, and is not answered.
#[test]
fn a_path_with_two_readings_names_both() {
    assert_refused(
        &Project::new(2),
        "schema.kinds.id_format",
        &[
            "version 2 has no top-level key or check code `schema.kinds.id_format`; for the key, ask grund config explain '[schema.kinds.<NAME>] id_format'",
            "hint: for a kind named id_format, ask grund config explain '[schema.kinds.id_format]'",
        ],
    );
}

/// §FS-config-explain.5: a bare name names every table that holds it, as a key
/// or as the end of its header, and is not answered even where one table does.
#[test]
fn a_bare_name_names_every_table_that_holds_it() {
    let v2 = Project::new(2);
    assert_refused(
        &v2,
        "exclude",
        &[
            "version 2 has no top-level key or check code `exclude`; for a key of a table, ask grund config explain '[schema.sources] exclude' or '[presentation.fmt] exclude'",
        ],
    );
    assert_refused(
        &v2,
        "sources",
        &[
            "version 2 has no top-level key or check code `sources`; for a table, ask grund config explain '[schema.sources]'",
        ],
    );
}

/// §FS-config-explain.5: a misspelt code and a path whose rest is no key of its
/// table keep the catch-all, byte for byte.
#[test]
fn a_misspelt_code_and_an_unknown_key_keep_the_catch_all() {
    let v2 = Project::new(2);
    for target in ["oversized-leed", "schema.sources.nope"] {
        assert_refused(
            &v2,
            target,
            &[&format!(
                "version 2 knows no table, key or check code `{target}`; grund config explain lists the tables, grund config explain --codes the codes"
            )],
        );
    }
}

/// §FS-config-explain.1.1: the help page's explain examples include a target
/// answered as a table and one answered as a key of a table.
#[test]
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
