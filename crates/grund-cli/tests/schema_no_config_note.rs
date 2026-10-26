//! Binary regressions for agent-grounds/grund#1041: with no grund.toml, the
//! `grund schema` table says that `grund init --scaffold` writes a different
//! layout, and which kinds it adds (§FS-schema.8.2).
//!
//! These cases live outside the `tests/e2e/cases` corpus because every case there
//! sits inside this repository, whose own grund.toml would be discovered: the
//! condition under test is a tree with no config at or above it. Each fixture is
//! built under the system temp directory, so run with a `TMPDIR` that no
//! grund.toml encloses.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A fresh repository: a `.git` marker, a README and a source folder, as the
/// ticket's reproducer set up before running `grund schema` and then `init`.
struct Fresh(PathBuf);

impl Fresh {
    fn new(what: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1041-{what}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("src/ledgerline")).unwrap();
        fs::write(root.join("README.md"), "# ledgerline\n").unwrap();
        fs::write(root.join("src/ledgerline/__init__.py"), "").unwrap();
        Self(root)
    }

    fn run(&self, args: &[&str]) -> String {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run built grund");
        assert_eq!(out.status.code(), Some(0), "grund {args:?}: {out:?}");
        // `init` reports what it wrote on stderr; `schema` prints nothing there.
        assert!(
            args[0] != "schema" || out.stderr.is_empty(),
            "grund {args:?}: {out:?}"
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn name(&self) -> String {
        let root = self.0.canonicalize().unwrap();
        root.file_name().unwrap().to_string_lossy().into_owned()
    }
}

impl Drop for Fresh {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

/// The `kind` rows of a table's `kinds and places` section, as (name, places).
fn kind_rows(table: &str) -> Vec<(String, String)> {
    table
        .lines()
        .skip_while(|line| *line != "kinds and places")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            (words.next()? == "kind").then(|| {
                let places = line.split_once(" kind ").unwrap().1.trim();
                (name.to_string(), places.to_string())
            })
        })
        .collect()
}

/// The note §FS-schema.8.2 asks for, derived from what `init --scaffold` really
/// writes: its version from the scaffold's header, its kinds from the rows the
/// scaffold has and the defaults lack.
fn expected_note(defaults: &str) -> String {
    let scaffold = Fresh::new("scaffold");
    scaffold.run(&["init", "--scaffold", "--name", "ledgerline"]);
    let table = scaffold.run(&["schema"]);
    let header = table.lines().next().unwrap();
    assert!(
        header.starts_with("ledgerline · grund.toml · v"),
        "the scaffold's own view: {table}"
    );
    let version = header.rsplit(" · v").next().unwrap();
    // §FS-schema.8.2: a tree whose config file was read prints no note.
    assert_eq!(table.lines().nth(1), Some(""), "a configured tree: {table}");
    let known: Vec<String> = kind_rows(defaults).into_iter().map(|(k, _)| k).collect();
    let added: Vec<String> = kind_rows(&table)
        .into_iter()
        .filter(|(kind, _)| !known.contains(kind))
        .map(|(kind, places)| format!("{kind} in {places}"))
        .collect();
    let mut note = format!(
        "note: these are the built-in defaults a tree with no grund.toml is held to; \
         grund init --scaffold writes a version-{version} grund.toml"
    );
    if !added.is_empty() {
        note.push_str(" that also declares ");
        note.push_str(&added.join(", "));
    }
    note
}

fn assert_no_config(fresh: &Fresh, header: &str) {
    assert!(
        !Path::new(&fresh.0).join("grund.toml").exists(),
        "the fixture must start with no grund.toml"
    );
    assert_eq!(
        header,
        format!("{} · defaults, no grund.toml · v1", fresh.name()),
        "a grund.toml encloses {}; rerun with a TMPDIR outside every grund tree",
        fresh.0.display()
    );
}

/// §FS-schema.8.2: the line after the no-config header names `grund init
/// --scaffold`, its version, and each kind it adds with its places.
#[test]
#[ignore = "red until #1041 prints the no-config note under the schema header; implement removes this"]
fn no_config_table_names_what_the_scaffold_adds() {
    let fresh = Fresh::new("schema");
    let table = fresh.run(&["schema"]);
    let mut lines = table.lines();
    assert_no_config(&fresh, lines.next().unwrap());
    let note = expected_note(&table);
    assert!(
        note.contains(" REQ in docs/requirements/"),
        "today's template: {note}"
    );
    assert_eq!(
        lines.next(),
        Some(note.as_str()),
        "the no-config table:\n{table}"
    );
    assert_eq!(
        lines.next(),
        Some(""),
        "the note is followed by the blank line"
    );
    assert_eq!(lines.next(), Some("kinds and places"));
}

/// §FS-schema.8.2: Mermaid and DOT carry the header comment and no note.
#[test]
fn no_config_sources_carry_no_note() {
    let fresh = Fresh::new("sources");
    let header = format!("{} · defaults, no grund.toml · v1", fresh.name());
    for (format, comment) in [("mermaid", "  %% "), ("dot", "  // ")] {
        let source = fresh.run(&["schema", "--format", format]);
        assert!(
            source.contains(&format!("{comment}{header}\n")),
            "{format} keeps the header comment:\n{source}"
        );
        assert!(
            !source.contains("grund init") && !source.contains("note:"),
            "{format} carries no note:\n{source}"
        );
    }
}
