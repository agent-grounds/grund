//! One reader of what a field says (§FS-rules.2.2, §FS-fields.identity.name):
//! the field checks and a whose clause read a declaration's field alike, on a
//! relative heading and on a field written twice, where the first one wins.

use super::direct_sections;
use crate::config::load_config;
use crate::grammar::field_value;
use crate::model::Catalog;
use crate::testing::{scan_findings, test_root, write};

/// `SWR-brake`'s file is `body` under its H1 heading, in a version-2 project
/// whose `SWR` declares the scalar `verification`.
fn scan(name: &str, body: &str) -> Catalog {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"swr\"\n\n[schema.kinds.SWR]\n\
         files = [\"req/SWR-brake.md\"]\n[schema.kinds.SWR.fields.verification]\n\
         one_of = [\"Test\", \"Inspection\"]\n",
    );
    write(
        &root.join("req/SWR-brake.md"),
        &format!("# SWR-brake: Brake latency\n\n{body}"),
    );
    let config = load_config(&root).expect("load config");
    scan_findings(&config, &root)
}

/// What the field check reads `verification` as, and what a whose clause reads.
fn both(findings: &Catalog) -> (Option<String>, Option<String>) {
    let declaration = findings
        .declarations
        .values()
        .flatten()
        .next()
        .expect("SWR-brake");
    let checked = direct_sections(declaration)
        .into_iter()
        .find(|(name, ..)| *name == "verification")
        .map(|(_, _, display)| display.into_owned());
    let selected = field_value(declaration, "verification").map(|value| value.into_owned());
    (checked, selected)
}

#[test]
fn a_relative_heading_reads_as_its_complete_path_does() {
    let findings = scan(
        "field-reader-relative",
        "## .verification: Test\n\nOn the bench.\n",
    );
    let test = Some("Test".to_string());
    assert_eq!(both(&findings), (test.clone(), test));
}

#[test]
fn a_field_written_twice_reads_as_its_first() {
    let findings = scan(
        "field-reader-duplicate",
        "## verification: Test\n\nOn the bench.\n\n## verification: Inspection\n\nBy reading.\n",
    );
    let test = Some("Test".to_string());
    assert_eq!(both(&findings), (test.clone(), test));
}
