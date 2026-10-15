//! Test module: what a stub's target outside the walk lends the stubbed declaration
//! (§FS-declarations.stubs.target.5, §AR-scanner.4.6): its citations, read once per
//! target however many stubs name it, kept apart from the walk's own, and only
//! those that resolve.

use crate::config::load_config;
use crate::model::{Catalog, Citation};
use crate::testing::{scan_tree, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n\n\
[scan]\ninclude = [\"docs\"]\n";

/// `FS-x` in `src/x.rs`, outside the walk, citing `GOAL-g` and an ID nobody
/// declares, beside `FS-y`, which cites `GOAL-g` too.
const TARGET: &str = concat!(
    "/// FS-x: X\n///\n/// Lead, for \u{a7}GOAL-g and \u{a7}GOAL-zz.\npub fn x() {}\n\n",
    "/// FS-y: Y\n///\n/// Lead, for \u{a7}GOAL-g.\npub fn y() {}\n",
);

/// A tree whose two stubs of `FS-x` point at one target outside the walk.
fn two_stubs(name: &str) -> Catalog {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(&root.join("docs/fs/x.md"), "# FS-x: [x](../../src/x.rs)\n");
    write(
        &root.join("docs/fs/again.md"),
        "# FS-x: [x](../../src/x.rs)\n",
    );
    write(&root.join("docs/goals/g.md"), "# GOAL-g: G\n\nThe goal.\n");
    write(&root.join("src/x.rs"), TARGET);
    let config = load_config(&root).expect("load config");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    findings
}

fn sites(citations: &[Citation]) -> Vec<(String, usize)> {
    citations
        .iter()
        .map(|cite| {
            (
                format!("{}-{}", cite.id.kind, cite.id.slug.as_deref().unwrap_or("")),
                cite.line,
            )
        })
        .collect()
}

#[test]
fn a_target_read_for_two_stubs_lends_its_citations_once() {
    let findings = two_stubs("stub_homes_lends_once");
    assert_eq!(
        sites(&findings.lent_citations),
        [("GOAL-g".to_string(), 3)],
        "the stubbed declaration's resolved citation, once for both stubs"
    );
}

#[test]
fn a_lent_citation_is_an_edge_and_never_a_walked_citation() {
    let findings = two_stubs("stub_homes_lent_apart");
    assert!(
        findings
            .citations
            .iter()
            .all(|cite| !cite.file.ends_with("src/x.rs")),
        "no hygiene check reads the target's citations"
    );
    assert_eq!(
        findings
            .edges()
            .filter(|cite| cite.file.ends_with("src/x.rs"))
            .count(),
        1,
        "the edges carry the lent citation"
    );
}

/// §FS-declarations.stubs.target.5: rejection happens before records or edges
/// can be imported, even for a target with a real declaration and chapter.
#[test]
fn an_excluded_target_lends_neither_records_nor_edges() {
    let root = test_root("stub_homes_excluded_content");
    write(
        &root.join("grund.toml"),
        &format!("{CONFIG}exclude = [\"src\"]\n"),
    );
    write(&root.join("docs/fs/x.md"), "# FS-x: [x](../../src/x.rs)\n");
    write(&root.join("docs/goals/g.md"), "# GOAL-g: G\n\nThe goal.\n");
    write(
        &root.join("src/x.rs"),
        &TARGET.replace(
            "pub fn x() {}",
            "///\n/// ## 1. Detail\n///\n/// Chapter content.\npub fn x() {}",
        ),
    );
    let config = load_config(&root).expect("config");
    let (catalog, _) = scan_tree(&config, None, false).expect("scan");
    assert_eq!(
        catalog
            .edges()
            .filter(|cite| cite.file.ends_with("src/x.rs"))
            .count(),
        0,
        "excluded content cannot contribute citation edges"
    );
    assert!(catalog.lent_citations.is_empty());
    let declarations = catalog
        .declarations
        .values()
        .find(|decls| decls[0].id.slug.as_deref() == Some("x"))
        .unwrap();
    let paired = crate::model::paired_declarations(declarations);
    let sole = paired.sole().expect("stub stays a declaration");
    assert!(sole.is_broken_stub());
    assert!(
        sole.record.file.ends_with("docs/fs/x.md"),
        "no imported target record or chapters"
    );
    assert!(sole.record.sections.is_empty());
}

/// §FS-declarations.stubs.target.5: an excluded or ignored directory alias
/// cannot import records, sections, chapters or edges from its physical target.
#[cfg(unix)]
#[test]
fn a_refused_alias_lends_neither_records_nor_edges() {
    for ignore in [false, true] {
        let root = test_root(&format!("stub_homes_refused_alias_{ignore}"));
        let exclude = if ignore { "" } else { "exclude = [\"src\"]\n" };
        write(&root.join("grund.toml"), &format!("{CONFIG}{exclude}"));
        write(&root.join("docs/fs/x.md"), "# FS-x: [x](../../src/x.rs)\n");
        write(&root.join("docs/goals/g.md"), "# GOAL-g: G\n\nThe goal.\n");
        write(
            &root.join("physical/x.rs"),
            &TARGET.replace(
                "pub fn x() {}",
                "///\n/// ## 1. Detail\n///\n/// Chapter.\npub fn x() {}",
            ),
        );
        std::os::unix::fs::symlink("physical", root.join("src")).unwrap();
        if ignore {
            write(&root.join(".ignore"), "src/\n");
        }
        let config = load_config(&root).unwrap();
        let (catalog, errors) = scan_tree(&config, None, false).unwrap();
        assert!(errors.is_empty());
        assert!(catalog.lent_citations.is_empty());
        assert_eq!(
            catalog.edges().count(),
            0,
            "no outgoing edges from refused content"
        );
        assert!(
            catalog
                .files
                .keys()
                .all(|file| !file.ends_with("src/x.rs") && !file.ends_with("physical/x.rs"))
        );
        let declarations = catalog
            .declarations
            .values()
            .find(|decls| decls[0].id.slug.as_deref() == Some("x"))
            .unwrap();
        let paired = crate::model::paired_declarations(declarations);
        let sole = paired.sole().expect("only the refused stub remains");
        assert!(sole.is_broken_stub());
        assert!(sole.record.file.ends_with("docs/fs/x.md"));
        assert!(sole.record.sections.is_empty());
    }
}

/// §FS-declarations.stubs.target.3, §FS-declarations.stubs.target.5: a refused
/// spelling cannot borrow the readable spelling's records or lend its edges,
/// whether it is resolved before or after the readable one in the same run.
#[cfg(unix)]
#[test]
fn mixed_aliases_keep_their_verdicts_and_lent_content_separate() {
    for ignore in [false, true] {
        for refused in ["x", "y"] {
            let allowed = if refused == "x" { "y" } else { "x" };
            let root = test_root(&format!("stub_homes_mixed_aliases_{ignore}_{refused}"));
            let exclude = if ignore { "" } else { "exclude = [\"gen\"]\n" };
            write(&root.join("grund.toml"), &format!("{CONFIG}{exclude}"));
            write(&root.join("docs/goals/g.md"), "# GOAL-g: G\n\nThe goal.\n");
            write(
                &root.join("physical/x.rs"),
                &TARGET.replace("pub fn", "///\n/// ## 1. Detail\n///\n/// Chapter.\npub fn"),
            );
            std::fs::create_dir_all(root.join("notes")).unwrap();
            for alias in ["gen", "readable"] {
                std::os::unix::fs::symlink("../physical", root.join("notes").join(alias)).unwrap();
            }
            if ignore {
                write(&root.join(".ignore"), "notes/gen/\n");
            }
            for (id, alias) in [(refused, "gen"), (allowed, "readable")] {
                write(
                    &root.join(format!("docs/fs/{id}.md")),
                    &format!("# FS-{id}: [{id}](../../notes/{alias}/x.rs)\n"),
                );
            }
            let config = load_config(&root).unwrap();
            let (catalog, errors) = scan_tree(&config, None, false).unwrap();
            assert!(errors.is_empty());
            assert_eq!(
                catalog.lent_citations.len(),
                1,
                "only the readable ID lends an edge"
            );
            assert_eq!(
                catalog.lent_citations[0]
                    .enclosing_declaration
                    .as_ref()
                    .unwrap()
                    .slug
                    .as_deref(),
                Some(allowed)
            );
            for (id, broken) in [(refused, true), (allowed, false)] {
                let declarations = catalog
                    .declarations
                    .values()
                    .find(|decls| decls[0].id.slug.as_deref() == Some(id))
                    .unwrap();
                let paired = crate::model::paired_declarations(declarations);
                let sole = paired.sole().expect("each stub remains one declaration");
                assert_eq!(sole.is_broken_stub(), broken);
                assert_eq!(sole.record.sections.is_empty(), broken);
                assert_eq!(
                    sole.record.file.ends_with(format!("docs/fs/{id}.md")),
                    broken
                );
            }
        }
    }
}
