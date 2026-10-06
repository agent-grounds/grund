//! Shared recognition contract (§AR-scanner.2.3, §FS-check.1.1.10).

use super::scan_tree;
use crate::config::Config;
use crate::testing::{test_root, write};

fn config(name: &str, marker: &str, strict: bool) -> Config {
    let root = test_root(name);
    let mut config = Config::default_for(root);
    config.id_format = "{kind}_{slug}".into();
    config.marker = marker.into();
    config.strict = strict;
    config.rebuild_grammar().expect("rebuild grammar");
    config
}

#[test]
fn word_character_marker_full_ids_keep_positions_and_sections() {
    for marker in ["\u{a7}", "_", "a", "tag_"] {
        for strict in [true, false] {
            let config = config("word_character_marker_positions", marker, strict);
            let source = config.root.join("src/lib.rs");
            let text = format!("//! 😀 {marker}FS_login.1\n");
            write(&source, &text);
            let (findings, _) = scan_tree(&config, Some(&config.root), true).unwrap();
            assert_eq!(
                findings.citations.len(),
                1,
                "marker {marker:?}, strict {strict}: {:?}",
                findings.citations
            );
            let citation = &findings.citations[0];
            assert_eq!(citation.file, source);
            assert_eq!(citation.line, 1);
            assert_eq!(citation.column, "//! 😀 ".len() + 1);
            assert_eq!(citation.id.kind, "FS");
            assert_eq!(citation.id.slug.as_deref(), Some("login"));
            assert_eq!(citation.id.num, None);
            assert_eq!(citation.section.as_deref(), Some("1"));
            assert!(citation.has_marker);
            assert!(!citation.shorthand);
            assert_eq!(citation.text, format!("{marker}FS_login.1"));
        }
    }
}

/// Passing guard: relaxing the marked path must preserve bare boundaries and exclusions.
#[test]
fn word_character_marker_preserves_bare_boundaries_and_exclusions() {
    for strict in [true, false] {
        let config = config("word_character_marker_exclusions", "_", strict);
        write(
            &config.root.join("docs/notes.md"),
            concat!(
                "xFS_login <_>FS_login <_>api/FS_login\n",
                "```rust\n_FS_login\n```\n",
                "~~~\n_FS_login\n~~~\n",
                "[link](FS_login)\n"
            ),
        );
        write(
            &config.root.join("src/lib.rs"),
            concat!(
                "// xFS_login <_>FS_login api/FS_login\n",
                "let bare = \"FS_login\";\n",
                "let qualified = \"_api/FS_login\";\n",
                "// `_api/FS_login`\n"
            ),
        );
        write(
            &config.root.join("src/data.py"),
            "DATA = \"\"\"_FS_login\"\"\"\n",
        );
        write(&config.root.join("docs/live.md"), "FS_login\n");
        let (findings, _) = scan_tree(&config, Some(&config.root), true).unwrap();
        assert_eq!(
            findings.citations.len(),
            usize::from(!strict),
            "strict {strict}: {:?}",
            findings.citations
        );
        if !strict {
            let citation = &findings.citations[0];
            assert_eq!(citation.file, config.root.join("docs/live.md"));
            assert_eq!(citation.line, 1);
            assert!(!citation.has_marker);
            assert_eq!(citation.text, "FS_login");
        }
    }
}

#[test]
fn word_character_marker_qualified_and_full_id_precedence_are_preserved() {
    let mut config = config("word_character_marker_precedence", "_", true);
    config.id_format = "{kind}_{number}_{slug}".into();
    config.rebuild_grammar().unwrap();
    write(
        &config.root.join("src/lib.rs"),
        "//! _FS_042_login.1 _api/FS_042_login.1\n",
    );
    let (findings, _) = scan_tree(&config, Some(&config.root), true).unwrap();
    assert_eq!(findings.citations.len(), 2, "{:?}", findings.citations);
    for namespace in [None, Some("api")] {
        let citation = findings
            .citations
            .iter()
            .find(|citation| citation.namespace.as_deref() == namespace)
            .expect("one citation in each namespace");
        assert_eq!(citation.id.kind, "FS");
        assert_eq!(citation.id.slug.as_deref(), Some("login"));
        assert_eq!(citation.id.num, Some(42));
        assert_eq!(citation.namespace.as_deref(), namespace);
        assert_eq!(citation.section.as_deref(), Some("1"));
        assert!(citation.has_marker);
        assert!(!citation.shorthand, "the full ID claims its token first");
    }
}
