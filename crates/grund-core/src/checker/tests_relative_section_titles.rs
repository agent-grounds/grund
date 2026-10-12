//! Test module: a relative heading's display name
//! (§FS-declarations.section.relative.shown), one case per surface that reads
//! it — a chapter rule, a version-2 field and a `list --selector` row.

use crate::api::{ListOpts, list};
use crate::config::load_config;
use crate::model::CheckReport;
use crate::testing::{check_findings, check_run, scan_findings, test_root, write};
use std::path::PathBuf;

/// A project whose `FS-login` writes its chapters in the relative form, held to
/// `RULE-terms`.
fn chaptered(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\
         require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\
         named_sections = true\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n\
         index = false\n\n[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\n\
         index = false\nrules = true\n\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/rules/RULE-terms.md"),
        "# RULE-terms: Each FS must have exactly one Terms chapter.\n\n\
         A reader needs the words first.\n",
    );
    write(
        &root.join("docs/fs/FS-login.md"),
        "# FS-login: Login\n\n## .terms: Terms\n\nWords.\n\n\
         ## .format: Formatting\n\n### .1: Username\n\nLower case.\n",
    );
    root
}

fn messages(report: &CheckReport) -> Vec<&str> {
    report
        .errors
        .iter()
        .map(|error| error.message.as_str())
        .collect()
}

/// §FS-rules.5.1.1: `## .terms: Terms` is the one Terms chapter `RULE-terms`
/// asks for, not a chapter named `.terms: Terms`.
#[test]
fn a_relative_chapter_satisfies_a_chapter_rule() {
    let run = check_run(&chaptered("relative_chapter_rule"), false);
    let chapters = run
        .report
        .errors
        .iter()
        .filter(|error| error.code == "chapter-cardinality")
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>();
    assert!(chapters.is_empty(), "{chapters:?}");
}

/// §FS-fields.content.one-of: `## .status: Accepted` holds `Accepted`.
#[test]
fn a_relative_field_reads_its_value() {
    let root = test_root("relative_field_value");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"fields\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.kinds.DF]\nfolders = [\"docs/df\"]\nindex = false\n\
         [schema.kinds.DF.fields.status]\n\
         one_of = [\"Proposed\", \"Accepted\", \"Superseded\"]\n\
         presence = \"must\"\nposition = \"first\"\n",
    );
    write(
        &root.join("docs/df/DF-cache.md"),
        "# DF-cache: Sessions live in memory\n\n## .status: Accepted\n\nDecided.\n",
    );
    let config = load_config(&root).expect("load config");
    let report = check_findings(&scan_findings(&config, &root), &config);
    assert!(report.errors.is_empty(), "{:?}", messages(&report));
}

/// §FS-rules.8: a selected relative chapter is titled as its explicit form is.
#[test]
fn list_titles_a_relative_chapter_like_an_explicit_one() {
    let root = chaptered("relative_chapter_list");
    let titles = ["FS.terms", "FS.format"].map(|selector| {
        let output = list(ListOpts {
            path: root.clone(),
            path_provided: true,
            selector: Some(selector.to_string()),
            ..ListOpts::default()
        })
        .expect("list the selector");
        output
            .entries
            .iter()
            .map(|entry| (entry.section.clone(), entry.title.clone()))
            .collect::<Vec<_>>()
    });
    assert_eq!(
        titles,
        [
            vec![(Some("terms".to_string()), Some("Terms".to_string()))],
            vec![(Some("format".to_string()), Some("Formatting".to_string()))],
        ]
    );
}
