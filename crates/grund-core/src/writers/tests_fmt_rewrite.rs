//! Test module: `fmt --marker` and the escape position (§FS-fmt.2.3,
//! §FS-check.1.1.9) — the bare-to-marker pass may not splice a marker inside the
//! escape brackets, because that turns an illustration into a live citation.

use std::path::PathBuf;

use super::fmt_rewrite::add_markers;
use crate::config::Config;
use crate::grammar::DocstringContent;

fn marked(line: &str, is_md: bool) -> String {
    let config = Config::default_for(PathBuf::from("."));
    add_markers(line, DocstringContent::default(), &config, is_md, &mut [])
}

/// §FS-fmt.2.3: in Markdown prose the pass promotes the bare citation and leaves
/// the escaped illustration on the same line byte-identical. Without the
/// carve-out the escape becomes `<§>§FS-001-login`, a live dangling citation the
/// writer invented (§FS-check.1.1.9).
#[test]
fn the_marker_pass_leaves_a_markdown_escape_and_still_promotes_its_neighbour() {
    assert_eq!(
        marked(
            "Login is FS-001-login, whose shape is written <§>FS-001-login.",
            true
        ),
        "Login is §FS-001-login, whose shape is written <§>FS-001-login."
    );
}

/// §FS-fmt.2.3: the same in a source comment, where the pass has no Markdown
/// branch and the escape is the only thing standing between the illustration and
/// a marker spliced inside its brackets.
#[test]
fn the_marker_pass_leaves_a_source_escape_and_still_promotes_its_neighbour() {
    assert_eq!(
        marked(
            "// A comment illustrating <§>FS-001-login beside a live FS-001-login.",
            false
        ),
        "// A comment illustrating <§>FS-001-login beside a live §FS-001-login."
    );
    assert_eq!(
        marked(
            "/// Opens a session. The shape of a citation is written <§>FS-001-login here.",
            false
        ),
        "/// Opens a session. The shape of a citation is written <§>FS-001-login here.",
        "a doc-comment escape is left byte-identical, so nothing on the line changes"
    );
}
