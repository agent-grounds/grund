//! Test module: a source body line loses its comment envelope — the whitespace
//! in front of the marker, the marker and one space — and keeps the rest
//! verbatim (§FS-show.2.3.2).

use super::comment_envelope::clean_body_line;

#[test]
fn block_continuation_drops_whitespace_before_star() {
    assert_eq!(
        clean_body_line(" * ## terms: Terms", false),
        "## terms: Terms"
    );
    assert_eq!(
        clean_body_line("     * ## terms: Terms", false),
        "## terms: Terms"
    );
    assert_eq!(clean_body_line("     *", false), "");
}

#[test]
fn indented_line_comment_drops_whitespace_before_marker() {
    assert_eq!(clean_body_line("    /// x", false), "x");
    assert_eq!(clean_body_line("\t//! x", false), "x");
    assert_eq!(clean_body_line("  # x", false), "x");
}

#[test]
fn indentation_after_the_marker_survives() {
    assert_eq!(clean_body_line("///   - nested", false), "  - nested");
    assert_eq!(clean_body_line("     *   - nested", false), "  - nested");
}

#[test]
fn closing_and_unmarked_lines() {
    assert_eq!(clean_body_line("     */", false), "");
    assert_eq!(clean_body_line("   plain text", false), "   plain text");
}

#[test]
fn markdown_lines_pass_through() {
    assert_eq!(clean_body_line("    /// x", true), "    /// x");
}
