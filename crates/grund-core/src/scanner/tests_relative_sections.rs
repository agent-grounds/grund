//! Test module: the parent pass (§FS-declarations.section.relative.parent,
//! §AR-scanner.2.2), one case per kind of parent a relative heading can meet.

use crate::config::load_config;
use crate::model::Catalog;
use crate::testing::{scan_findings, test_root, write};

/// `FS-login`'s file is `body` under its H1 heading, scanned with named sections
/// on or off.
fn scan(name: &str, named_sections: bool, body: &str) -> Catalog {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\nstrict = true\n\
             require_grounding = false\n\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
             named_sections = {named_sections}\n\n[[kinds]]\nkind = \"FS\"\n\
             folder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-login.md"),
        &format!("# FS-login: Login\n\n{body}"),
    );
    let config = load_config(&root).expect("load config");
    scan_findings(&config, &root)
}

/// Every section of `FS-login` as `path@line`, relative ones marked `~`.
fn sections(findings: &Catalog) -> Vec<String> {
    let decl = findings
        .declarations
        .values()
        .flatten()
        .next()
        .expect("FS-login");
    decl.sections
        .iter()
        .map(|(path, info)| {
            format!(
                "{}{path}@{}",
                if info.relative { "~" } else { "" },
                info.line
            )
        })
        .collect()
}

/// Every relative-section-parent finding as `line: written under <coordinate>`.
fn parents(findings: &Catalog) -> Vec<String> {
    findings
        .relative_section_parents
        .iter()
        .map(|failed| {
            format!(
                "{}: {} H{} under {:?}",
                failed.line, failed.written, failed.needed_level, failed.under
            )
        })
        .collect()
}

#[test]
fn the_declaration_heading_is_a_parent() {
    let findings = scan(
        "relative_parent_declaration",
        true,
        "## .format: Formatting\n\nText.\n",
    );
    assert_eq!(sections(&findings), ["~format@3"]);
    assert!(parents(&findings).is_empty());
}

#[test]
fn a_complete_path_section_is_a_parent() {
    let findings = scan(
        "relative_parent_complete",
        true,
        "## format: Formatting\n\n### .1: Username\n\n### .2. Wrong punctuation\n",
    );
    assert_eq!(sections(&findings), ["format@3", "~format.1@5"]);
    assert_eq!(
        findings.unmarked_headings.len(),
        1,
        "a numeric step under a name takes `:`"
    );
}

#[test]
fn relative_headings_chain() {
    let findings = scan(
        "relative_parent_chain",
        true,
        "## format: Formatting\n\n### .user: User\n\n#### .1: Username\n",
    );
    assert_eq!(
        sections(&findings),
        ["format@3", "~format.user@5", "~format.user.1@7"]
    );
}

#[test]
fn a_skipped_level_has_no_parent() {
    let findings = scan(
        "relative_parent_skipped",
        true,
        "## format: Formatting\n\n#### .1: Deep\n",
    );
    assert_eq!(sections(&findings), ["format@3"]);
    assert_eq!(parents(&findings), ["5: #### .1 H3 under \"format\""]);
}

#[test]
fn a_plain_heading_is_not_a_parent() {
    let findings = scan(
        "relative_parent_plain",
        true,
        "## Notes\n\n### .1: Username\n",
    );
    assert!(sections(&findings).is_empty());
    assert_eq!(parents(&findings), ["5: ### .1 H2 under \"\""]);
}

#[test]
fn a_duplicated_path_is_not_a_parent() {
    let findings = scan(
        "relative_parent_duplicate",
        true,
        "## format: One\n\n## format: Two\n\n### .1: Username\n",
    );
    assert_eq!(sections(&findings), ["format@3"]);
    assert_eq!(parents(&findings), ["7: ### .1 H2 under \"\""]);
}

#[test]
fn an_orphan_is_not_a_parent() {
    let findings = scan(
        "relative_parent_orphan",
        true,
        "## format.user: User\n\n### .1: Username\n",
    );
    assert_eq!(parents(&findings), ["5: ### .1 H2 under \"\""]);
}

#[test]
fn a_failed_relative_heading_is_not_a_parent() {
    let findings = scan(
        "relative_parent_failed",
        true,
        "## format: Formatting\n\n#### .user: Deep\n\n##### .1: Deeper\n",
    );
    assert_eq!(
        parents(&findings),
        [
            "5: #### .user H3 under \"format\"",
            "7: ##### .1 H4 under \"format\""
        ]
    );
}

#[test]
fn a_fenced_heading_is_not_a_parent() {
    let findings = scan(
        "relative_parent_fenced",
        true,
        "```markdown\n## format: Formatting\n```\n\n### .1: Username\n",
    );
    assert_eq!(parents(&findings), ["7: ### .1 H2 under \"\""]);
}

#[test]
fn a_relative_and_a_complete_heading_on_one_path_are_duplicates() {
    let findings = scan(
        "relative_parent_same_path",
        true,
        "## format: Formatting\n\n### format.1: Complete\n\n### .1: Relative\n",
    );
    assert_eq!(sections(&findings), ["format@3", "format.1@5"]);
    let decl = findings
        .declarations
        .values()
        .flatten()
        .next()
        .expect("FS-login");
    assert_eq!(decl.duplicate_sections.len(), 1);
    assert!(decl.duplicate_sections[0].1.relative);
}

#[test]
fn a_name_with_named_sections_off_stays_unmarked() {
    let findings = scan(
        "relative_parent_gate_off",
        false,
        "## .format: Formatting\n",
    );
    assert!(sections(&findings).is_empty());
    assert!(parents(&findings).is_empty());
    assert_eq!(findings.unmarked_headings.len(), 1);
}

#[test]
fn a_numeric_relative_heading_takes_an_optional_period() {
    let findings = scan(
        "relative_parent_numeric",
        false,
        "## 1. Overview\n\n### .1. Scope\n\n### .2 Terms\n",
    );
    assert_eq!(sections(&findings), ["1@3", "~1.1@5", "~1.2@7"]);
}

#[test]
fn outside_every_body_it_is_only_text() {
    let findings = scan(
        "relative_parent_outside",
        true,
        "Text.\n\n# Notes\n\n## .1: Loose\n",
    );
    assert!(parents(&findings).is_empty());
    assert_eq!(sections(&findings), Vec::<String>::new());
}

/// §FS-declarations.checks.orphan-section: the one answer the orphan check and
/// the parent pass both read, a named path's first missing prefix, and none for a
/// numeric path, whose parents need not be recorded.
#[test]
fn an_orphan_is_a_named_path_missing_a_prefix() {
    let findings = scan(
        "relative_parent_missing_prefix",
        true,
        "#### format.user.name: Name\n\n### 2.1 Numeric\n\n\
         ## parse: Parsing\n\n### parse.1: Tokens\n",
    );
    let decl = findings
        .declarations
        .values()
        .flatten()
        .next()
        .expect("FS-login");
    let missing = |path: &str| decl.missing_section_prefix(path);
    assert_eq!(missing("format.user.name").as_deref(), Some("format"));
    assert_eq!(missing("2.1"), None);
    assert_eq!(missing("parse.1"), None);
}
