//! Test module: the `/**` and `/*!` openers are the configured `/*`, as `///`
//! and `//!` are the configured `//` (§FS-declarations.line.block-comment,
//! §AR-scanner.4.3). Read at the per-line seam every declaration reader shares,
//! [`declaration_id_on_line`], for a line read as comment-prefixed; the
//! cross-line block state of a bare line is pinned by the e2e cases
//! `bare-block-comment-line-*`.

use super::{declaration_id_on_line, render_id};
use crate::testing::{numbered_config, test_root};

/// The ID a comment-prefixed source line declares, or `None`.
fn declared(name: &str, line: &str) -> Option<String> {
    let mut config = numbered_config(test_root(name));
    config.rebuild_grammar().expect("rebuild the test grammar");
    declaration_id_on_line(&config.grammar, line, false, false)
        .map(|(id, _)| render_id(&config.grammar, &id))
}

/// Every opener declares on its own line, the one-line Javadoc too.
#[test]
fn every_block_comment_opener_declares_on_its_line() {
    for line in [
        "/* FS-042-user-login: the user logs in",
        "/** FS-042-user-login: the user logs in",
        "/*! FS-042-user-login: the user logs in",
        "/** FS-042-user-login: the user logs in */",
        "    /** FS-042-user-login: an indented member's Javadoc",
    ] {
        assert_eq!(
            declared("every_block_comment_opener_declares_on_its_line", line).as_deref(),
            Some("FS-042-user-login"),
            "opener line {line:?}"
        );
    }
}

/// Only `*` and `!` widen the opener: anything else glued to `/*` is no opener.
#[test]
fn an_opener_glued_to_another_character_declares_nothing() {
    for line in [
        "/*x FS-042-user-login: not an opener",
        "/**FS-042-user-login: no space after the opener",
    ] {
        assert_eq!(
            declared(
                "an_opener_glued_to_another_character_declares_nothing",
                line
            ),
            None,
            "line {line:?}"
        );
    }
}
