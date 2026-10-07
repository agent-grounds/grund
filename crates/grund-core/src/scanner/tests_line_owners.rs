//! Test module: a `cover --lines` answer and a citation's
//! `enclosing_declaration` / `enclosing_section` come from one lookup
//! (§FS-cover.6.2, §AR-scanner.2.4.3, §AR-scanner.2.4.4). The case scans a tree
//! that cites on every kind of line ownership can fall on — the preamble, a lead,
//! a named chapter and subsection, numbered sections, the stretch a duplicate
//! heading closes, a nested declaration, the prose after a closing heading, a
//! doc-comment and the code below it — and asks the whole-file `--lines` answer
//! about each citation's line.

use std::path::Path;

use crate::model::{Catalog, FileLineOwnership, Id};
use crate::testing::{legacy_fs_folder_config, scan_findings, test_root, write};

/// The owner a whole-file `--lines` answer gives `line`, as the rendered
/// declaration and its section: `None` where the line lies in no owner run.
fn answered_owner(owned: &FileLineOwnership, line: usize) -> Option<(Id, Option<String>)> {
    let range = &owned.ranges[0];
    let run = range
        .owners
        .iter()
        .find(|run| run.start <= line && line <= run.end)?;
    let section = run
        .sections
        .iter()
        .find(|section| section.start <= line && line <= section.end)
        .expect("section runs partition their owner run");
    Some((run.declaration.clone(), section.section.clone()))
}

fn ownership_of<'a>(findings: &'a Catalog, file: &Path) -> &'a FileLineOwnership {
    findings
        .line_ownership
        .iter()
        .find(|owned| owned.file.ends_with(file))
        .expect("the scanned file records its requested ranges")
}

/// §FS-cover.6.2: for every citation the scan finds, the line-ownership answer
/// for its line names the same declaration and section the citation records —
/// owned and unowned alike — so the two callers of the lookup cannot drift.
#[test]
fn every_citation_is_owned_as_its_line_is() {
    let root = test_root("line_owners_agree_with_citations");
    let markdown = Path::new("docs/functional-spec/FS-001-alpha.md");
    // One line per element, so no line of this file opens with a heading the
    // repository's own scan would read.
    let markdown_lines = [
        "# Alpha",
        "",
        "A preamble \u{a7}FS-001-alpha.",
        "",
        "# FS-001-alpha: Alpha",
        "",
        "The lead \u{a7}FS-001-alpha.1.",
        "",
        "## terms: Terms",
        "",
        "Terms \u{a7}FS-001-alpha.",
        "",
        "### terms.words: Words",
        "",
        "Words \u{a7}FS-001-alpha.",
        "",
        "## 1. Inputs",
        "",
        "Inputs \u{a7}FS-001-alpha.",
        "",
        "### 1.1 First input",
        "",
        "```text",
        "# A fenced line \u{a7}FS-001-alpha",
        "```",
        "After the fence \u{a7}FS-001-alpha.",
        "",
        "## 1. A duplicate heading",
        "",
        "Under the duplicate \u{a7}FS-001-alpha.",
        "",
        "## 2. Outputs",
        "",
        "Outputs \u{a7}FS-001-alpha.",
        "",
        "# Beta",
        "",
        "Nothing owns this \u{a7}FS-001-alpha.",
        "",
        "## FS-002-beta: Beta",
        "",
        "Beta's lead \u{a7}FS-001-alpha.",
        "",
        "### 1. Only section",
        "",
        "Beta's section \u{a7}FS-001-alpha.",
        "",
        "# Appendix",
        "",
        "Nothing again \u{a7}FS-001-alpha.",
    ];
    write(&root.join(markdown), &(markdown_lines.join("\n") + "\n"));
    let source = Path::new("src/lib.rs");
    let source_lines = [
        "/// FS-003-gamma: Gamma",
        "///",
        "/// The lead \u{a7}FS-001-alpha.",
        "///",
        "/// ## 1. Behaviour",
        "///",
        "/// The section \u{a7}FS-001-alpha.",
        "pub fn gamma() {}",
        "",
        "// Below the block \u{a7}FS-001-alpha.",
        "pub fn other() {}",
    ];
    write(&root.join(source), &(source_lines.join("\n") + "\n"));
    let mut config = legacy_fs_folder_config(root.clone());
    config.named_sections = true;
    config.rebuild_grammar().expect("rebuild named grammar");
    config.owner_lines = vec![(1, usize::MAX)];
    let findings = scan_findings(&config, &root);

    let mut owned_sites = 0;
    let mut unowned_sites = 0;
    for file in [markdown, source] {
        let owned = ownership_of(&findings, file);
        let citations = findings
            .citations
            .iter()
            .filter(|citation| citation.file.ends_with(file));
        for citation in citations {
            let recorded = citation
                .enclosing_declaration
                .clone()
                .map(|id| (id, citation.enclosing_section.clone()));
            match &recorded {
                Some(_) => owned_sites += 1,
                None => unowned_sites += 1,
            }
            assert_eq!(
                answered_owner(owned, citation.line),
                recorded,
                "{}:{}",
                file.display(),
                citation.line
            );
        }
    }
    assert!(owned_sites >= 9, "the fixture cites inside every unit");
    assert!(unowned_sites >= 3, "and outside every body");
}
