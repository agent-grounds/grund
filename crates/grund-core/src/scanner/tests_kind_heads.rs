//! Prefix-related kinds must retain every canonical and off-format citation
//! exactly once (§FS-config.3.4.5, §FS-citations.recognized.off-format).

use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

#[test]
fn kind_heads_scan_canonical_and_off_format_citations_once() {
    let root = test_root("kind_heads_scan");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n\
         [schema.kinds.GR]\nfiles = [\"tickets.md\"]\nid_format = \"{kind}-{number}\"\n\
         [schema.kinds.GRUND]\nfiles = [\"grund.md\"]\n",
    );
    write(
        &root.join("grund.md"),
        "# GRUND-why: Why\n\nSee \u{a7}GR-80923.\n",
    );
    write(
        &root.join("tickets.md"),
        "## GR-80923: Ticket\n\nSee \u{a7}GRUND-why.\n",
    );
    write(
        &root.join("source.md"),
        "Bad \u{a7}GR_80923 and \u{a7}GRUND_why.\n",
    );
    let config = load_config(&root).unwrap_or_else(|e| panic!("{e:#}"));
    let findings = scan_findings(&config, &root);
    let mut canonical = findings
        .citations
        .iter()
        .map(|c| c.id.kind.as_str())
        .collect::<Vec<_>>();
    canonical.sort();
    assert_eq!(canonical, ["GR", "GRUND"]);
    let mut off = findings
        .off_format_citations
        .iter()
        .map(|c| (c.token.as_str(), c.kind.as_str()))
        .collect::<Vec<_>>();
    off.sort();
    assert_eq!(off, [("GRUND_why", "GRUND"), ("GR_80923", "GR")]);
    // Add headings only after the citation assertions: a catalog-backed legacy
    // heading would otherwise claim its own off-format citation first.
    write(
        &root.join("bad.md"),
        "# GR_80923: Bad ticket\n\n# GRUND_why: Bad ground\n",
    );
    let findings = scan_findings(&config, &root);
    assert_eq!(findings.near_miss_headings.len(), 2);
    assert!(
        findings
            .near_miss_headings
            .iter()
            .any(|h| h.text == "GR_80923")
    );
    assert!(
        findings
            .near_miss_headings
            .iter()
            .any(|h| h.text == "GRUND_why")
    );
}
