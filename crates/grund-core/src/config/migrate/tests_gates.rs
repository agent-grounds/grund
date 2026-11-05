//! Test module: the records gate (§FS-config-migrate.5.1). A candidate that
//! means something else is caught, and each difference the gate ignores is a
//! real difference between the two projects that it passes. The report gate is
//! `api/tests_config_migrate_gates.rs`.

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use super::opted_out::Block;
use super::records_gate::compare;
use super::{GateResult, Located, Prepared, candidate, languages, translate};
use crate::config::discovery::pending_config;
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};
use crate::config::v1::default_project;
use crate::testing::{test_root, write};

const FS: &str = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n";
const NAMED: &str = "[id]\nnamed_sections = true\n";

/// A v1 tree of `toml` (after its version line, before `[id]`), with
/// `docs/FS-a.md` and the folders of `paths`, translated: the v1 project, the
/// loaded candidate, and the root. The candidate must not be refused.
fn translated(name: &str, toml: &str, paths: &[&str]) -> (Project, Project, PathBuf) {
    let root = test_root(&format!("migrate-gates-{name}"));
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    for path in paths {
        write(&root.join(path), "// x\n");
    }
    // A missing default include entry is refused (§FS-config-migrate.3.1.3).
    let include = if toml.contains("include =") {
        ""
    } else {
        "[scan]\ninclude = [\"docs\"]\n"
    };
    let text = format!("grund_config_version = 1\n{toml}{include}{NAMED}");
    write(&root.join("grund.toml"), &text);
    let located = Located {
        file: root.join("grund.toml"),
        root: root.clone(),
        config: "grund.toml".into(),
    };
    let Prepared::Translated { report, v1, .. } =
        translate(&located, false, false, &|| true, &|_| Ok(Vec::new()))
            .expect("the v1 file reads")
    else {
        panic!("read as version 2");
    };
    assert!(report.refusals.is_empty(), "refused: {:?}", report.refusals);
    assert_eq!(report.gates, Some(GateResult::Equal), "{name}");
    let candidate = report.candidate.expect("a candidate");
    let invocation = Invocation::new(&root, false, Selection::Project);
    let loaded = pending_config(&candidate, &root, &invocation)
        .expect("the candidate loads")
        .project()
        .clone();
    (*v1, loaded, root)
}

/// The two readings differ in `field`, so a pass below is the gate ignoring it.
fn differs(field: &str, v1: impl Debug, v2: impl Debug) {
    assert_ne!(format!("{v1:?}"), format!("{v2:?}"), "{field} is the same");
}

/// §FS-config-migrate.5.1: a candidate whose marker differs fails the gate,
/// and the difference names both readings.
#[test]
fn records_gate_catches_a_changed_marker() {
    let (v1, mut v2, root) = translated("marker", FS, &[]);
    v2.schema.citation.marker = "@".into();
    let lines = compare(&v1, &v2, &root, Block::Project);
    assert!(lines.iter().any(|line| line.starts_with("- ")), "{lines:?}");
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("+ ") && line.contains('@')),
        "{lines:?}"
    );
}

/// §FS-config-migrate.5.1: a candidate with another project name fails.
#[test]
fn records_gate_catches_a_changed_project_name() {
    let (v1, mut v2, root) = translated("name", &format!("project_name = \"demo\"\n{FS}"), &[]);
    v2.name = Some("other".into());
    assert!(!compare(&v1, &v2, &root, Block::Project).is_empty());
}

/// §FS-config-migrate.5.1: a candidate that lost a row's citation rules fails.
#[test]
fn records_gate_catches_a_lost_citation_rule() {
    let toml = format!("{FS}[citations.FS]\nmust = [\"FS\"]\n");
    let (v1, mut v2, root) = translated("citations", &toml, &[]);
    v2.rules.citations = crate::config::v1::default_project().rules.citations;
    assert!(!compare(&v1, &v2, &root, Block::Project).is_empty());
}

/// §FS-config-migrate.5.1: the version and where the config was found.
#[test]
fn records_gate_ignores_the_version_and_the_config_location() {
    let (v1, v2, root) = translated("version", &format!("project_name = \"demo\"\n{FS}"), &[]);
    differs("version", v1.version, v2.version);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: the dropped `[output]` preferences.
#[test]
fn records_gate_ignores_the_dropped_output_preferences() {
    let toml = format!("[output]\nformat = \"json\"\nrelative_paths = false\n{FS}");
    let (v1, v2, root) = translated("output", &toml, &[]);
    differs("[output]", &v1.presentation.output, &v2.presentation.output);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: the source model, compared suffix by suffix rather
/// than by the language names that carry it.
#[test]
fn records_gate_ignores_the_language_names_of_the_source_model() {
    let toml = format!("[scan]\ninclude = [\"docs\"]\nextensions = [\"md\", \"rs\", \"py\"]\n{FS}");
    let (v1, v2, root) = translated("sources", &toml, &[]);
    differs("sources", &v1.schema.sources, &v2.schema.sources);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: grounding, as the effective pair each row resolves to.
#[test]
fn records_gate_ignores_how_grounding_is_spelled() {
    let toml = format!(
        "[reference]\nrequire_grounding = true\ngrounding_level = 2\n{FS}\
         [[kinds]]\nkind = \"AR\"\nfolder = \"arch\"\nrequire_grounding = false\n"
    );
    let (v1, v2, root) = translated("grounding", &toml, &["arch/AR-a.md"]);
    differs("grounding", &v1.rules.grounding, &v2.rules.grounding);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: `include`, as the set of derived scan roots.
#[test]
fn records_gate_ignores_include_spelled_as_places() {
    let toml = format!("[scan]\ninclude = [\"docs\", \"src\", \"main.rs\"]\n{FS}");
    let (v1, v2, root) = translated("include", &toml, &["src/a.rs", "main.rs"]);
    differs("rows", &v1.schema.rows, &v2.schema.rows);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: the nesting rule, when no two v1 homes nest.
#[test]
fn records_gate_ignores_the_nesting_rule_when_nothing_nests() {
    let (v1, v2, root) = translated("nesting", FS, &[]);
    differs("nesting", v1.schema.nesting, v2.schema.nesting);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-config-migrate.5.1: a positive citation default, compared as `may`.
#[test]
fn records_gate_compares_a_positive_default_as_may() {
    let toml = format!("{FS}[citations]\ndefault = \"should\"\n");
    let (v1, v2, root) = translated("default", &toml, &[]);
    differs("citations", &v1.rules.citations, &v2.rules.citations);
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// §FS-scan.scope.extensions.empty: the candidate starts from a clone of a version 1 project that
/// holds the entry, and keeps neither the entry nor what it reads, as version 2 has no such entry.
#[test]
fn a_candidate_keeps_neither_the_empty_string_nor_what_it_reads() {
    let mut v1 = default_project();
    v1.schema
        .sources
        .set_extensions(vec!["md".into(), String::new()]);
    let model = languages::translate(&v1, None, None, false).model;
    let sources = candidate::records(&v1, model, Vec::new()).schema.sources;
    assert!(!sources.reads_no_extension);
    assert!(!sources.extensions.iter().any(String::is_empty));
    assert!(!sources.lists_extension(Path::new("FACTS")));
}
