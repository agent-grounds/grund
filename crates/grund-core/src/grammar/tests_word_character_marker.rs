//! Shared note classification regressions for §FS-check.1.1.10.

use super::comment_line::{comment_strip_prefixes, line_citation_ranges};
use super::inline_note_layout::{
    InlineNoteLayout, block_has_inline_note, content_conforms, line_layout_view,
};
use crate::config::Config;
use crate::testing::test_root;

#[test]
fn word_character_marker_note_classification_preserves_containing_full_ids() {
    for (marker, format, pattern, target) in [
        (
            "-",
            "{kind}-{slug}",
            "[A-Za-z0-9][A-Za-z0-9-]*",
            "FS-login-FS-logout",
        ),
        (
            "_",
            "{kind}_{slug}",
            "[A-Za-z0-9][A-Za-z0-9_]*",
            "FS_login_FS_logout",
        ),
    ] {
        let mut config = Config::default_for(test_root("word_character_marker_note_containing_id"));
        config.marker = marker.into();
        config.id_format = format.into();
        config.slug_pattern = pattern.into();
        config.rebuild_grammar().unwrap();
        for (strict, token) in [
            (false, target.to_string()),
            (false, format!("{marker}{target}")),
            (true, format!("{marker}{target}")),
            (false, format!("{marker}FS{marker}login")),
        ] {
            config.strict = strict;
            let line = format!("// {token}");
            let ranges = line_citation_ranges(&line, config.lexical(), &[]);
            assert_eq!(ranges, vec![(3, line.len())], "{line}");
            assert!(
                !block_has_inline_note(
                    &[&line],
                    config.lexical(),
                    &[],
                    &comment_strip_prefixes(config.lexical())
                ),
                "{line}"
            );

            let noted = format!("{line}: retain the whole target");
            let ranges = line_citation_ranges(&noted, config.lexical(), &[]);
            assert_eq!(ranges, vec![(3, line.len())], "{noted}");
            let (content, tokens) =
                line_layout_view(&noted, &ranges, &comment_strip_prefixes(config.lexical()))
                    .unwrap();
            assert!(
                content_conforms(
                    InlineNoteLayout::CitationFirst { delimiter: ':' },
                    content,
                    &tokens
                ),
                "{noted}"
            );
            assert!(
                block_has_inline_note(
                    &[&noted],
                    config.lexical(),
                    &[],
                    &comment_strip_prefixes(config.lexical())
                ),
                "{noted}"
            );
        }
    }
}

#[test]
fn word_character_marker_note_classification_keeps_overlapping_marker_ranges() {
    for (marker, run, skipped_bytes) in [
        ("__", "___", 1),
        ("__", "__", 0),
        ("::", ":::", 1),
        ("::", "::", 0),
        ("éé", "ééé", "é".len()),
    ] {
        let mut config = Config::default_for(test_root("word_character_marker_note_overlap"));
        config.marker = marker.into();
        config.id_format = "{kind}_{slug}".into();
        config.rebuild_grammar().unwrap();
        for strict in [true, false] {
            config.strict = strict;
            for prefix in ["// ", "// 😀 "] {
                let line = format!("{prefix}{run}FS_login");
                let ranges = line_citation_ranges(&line, config.lexical(), &[]);
                let start = prefix.len() + skipped_bytes;
                assert_eq!(ranges, vec![(start, line.len())], "{line}");
                assert_eq!(&line[start..], format!("{marker}FS_login"));
            }
        }
    }
}
