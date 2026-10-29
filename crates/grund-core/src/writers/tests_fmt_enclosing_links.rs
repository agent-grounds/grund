//! Test module: the cross-reference pass's three-way reading of a citation
//! (§FS-citations.wrap) — a wrap is re-derived, a citation inside a link its author
//! wrote is left with its link (§FS-citations.wrap.enclosing), and an unwrapped
//! citation is wrapped fresh. The reading has one owner in the grammar; these
//! cases pin it through the pass that writes, line by line.

use super::wrap_markdown_links;
use crate::config::{Config, load_config};
use crate::model::Catalog;
use crate::testing::{scan_findings, test_root, write};

/// The canonical destination of `FS-001-login` from `docs/notes.md`.
const CANON: &str = "specs/FS-001-login.md#fs-001-login-a-user-logs-in";

/// A project whose `FS` kind keeps `FS-001-login` under `docs/specs`, loaded and
/// scanned: what the pass resolves a citation against.
fn project(name: &str) -> (Config, Catalog) {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/specs/FS-001-login.md"),
        "# FS-001-login: A user logs in\n\nBody.\n",
    );
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    (config, findings)
}

/// The pass over `line` in `docs/notes.md`, spelled under `config.root` as the
/// walk spells every file it hands the pass. `load_config` canonicalizes the root,
/// so on Windows the raw fixture path shares no prefix with it, and a file never
/// written cannot be canonicalized back under it.
fn wrapped(config: &Config, findings: &Catalog, line: &str) -> String {
    wrap_markdown_links(
        line,
        &config.root.join("docs/notes.md"),
        config,
        findings,
        None,
        None,
    )
}

/// §FS-citations.wrap: the three readings on one line. The stale wrap is
/// re-derived (§FS-fmt.6.3.1), the citation in the author's link keeps every byte
/// of that link (§FS-citations.wrap.enclosing), and the bare one is wrapped.
#[test]
fn a_wrap_is_rederived_an_authored_label_is_left_and_a_bare_citation_is_wrapped() {
    let (config, findings) =
        project("a_wrap_is_rederived_an_authored_label_is_left_and_a_bare_citation_is_wrapped");
    let line = "A [\u{a7}FS-001-login](stale.md) wrap, [the \u{a7}FS-001-login spec](specs/FS-001-login.md) \
                link, and \u{a7}FS-001-login bare.";
    assert_eq!(
        wrapped(&config, &findings, line),
        format!(
            "A [\u{a7}FS-001-login]({CANON}) wrap, [the \u{a7}FS-001-login spec](specs/FS-001-login.md) \
             link, and [\u{a7}FS-001-login]({CANON}) bare."
        )
    );
}

/// §FS-citations.wrap.enclosing.shape: every shape that encloses keeps its bytes —
/// escaped and balanced brackets and a code span in the text, an escaped and a
/// balanced parenthesis and an empty destination, and a citation repeated in one
/// text.
#[test]
fn every_enclosing_shape_keeps_its_bytes() {
    let (config, findings) = project("every_enclosing_shape_keeps_its_bytes");
    for line in [
        "See [\u{a7}FS-001-login: logs in \\] for good](specs/FS-001-login.md).",
        "See [\u{a7}FS-001-login: logs in [draft]](specs/FS-001-login.md).",
        "See [\u{a7}FS-001-login: run `grund check` then `]`](specs/FS-001-login.md).",
        "See [the \u{a7}FS-001-login spec](specs/FS-001-login\\).md).",
        "See [the \u{a7}FS-001-login spec](specs/FS-001-login(1).md).",
        "See [the \u{a7}FS-001-login spec]().",
        "See [\u{a7}FS-001-login, that is \u{a7}FS-001-login](specs/FS-001-login.md).",
    ] {
        assert_eq!(wrapped(&config, &findings, line), line, "`{line}` encloses");
    }
}

/// §FS-citations.wrap.enclosing.shape: each occurrence is read by its own
/// position, so the same citation outside the author's link is still wrapped.
#[test]
fn the_same_citation_outside_the_link_is_still_wrapped() {
    let (config, findings) = project("the_same_citation_outside_the_link_is_still_wrapped");
    assert_eq!(
        wrapped(
            &config,
            &findings,
            "See [the \u{a7}FS-001-login spec](specs/FS-001-login.md), and \u{a7}FS-001-login itself."
        ),
        format!(
            "See [the \u{a7}FS-001-login spec](specs/FS-001-login.md), and [\u{a7}FS-001-login]({CANON}) itself."
        )
    );
}

/// §FS-citations.wrap.enclosing.shape: what does not open or close a link leaves the
/// citation unwrapped, so the pass wraps it fresh — an escaped opener, an opener in
/// a code span, a destination that never closes.
#[test]
fn what_encloses_nothing_is_wrapped_fresh() {
    let (config, findings) = project("what_encloses_nothing_is_wrapped_fresh");
    for (line, expected) in [
        (
            "See \\[the \u{a7}FS-001-login spec](specs/FS-001-login.md).",
            format!("See \\[the [\u{a7}FS-001-login]({CANON}) spec](specs/FS-001-login.md)."),
        ),
        (
            "See `[` \u{a7}FS-001-login](specs/FS-001-login.md).",
            format!("See `[` [\u{a7}FS-001-login]({CANON})](specs/FS-001-login.md)."),
        ),
        (
            "See [the \u{a7}FS-001-login spec](specs/FS-001-login.md",
            format!("See [the [\u{a7}FS-001-login]({CANON}) spec](specs/FS-001-login.md"),
        ),
    ] {
        assert_eq!(wrapped(&config, &findings, line), expected, "`{line}`");
    }
}

/// §FS-citations.wrap.enclosing.images: an image's description encloses nothing,
/// so a citation in its alt text is wrapped as it always was.
#[test]
fn a_citation_in_an_image_description_is_wrapped() {
    let (config, findings) = project("a_citation_in_an_image_description_is_wrapped");
    assert_eq!(
        wrapped(
            &config,
            &findings,
            "See ![a diagram of \u{a7}FS-001-login](diagram.png)."
        ),
        format!("See ![a diagram of [\u{a7}FS-001-login]({CANON})](diagram.png).")
    );
}

/// §FS-citations.wrap.enclosing.wrap-wins: on a line an older `fmt` nested, the
/// inner wrap is still a wrap and is re-derived; the outer brackets are text, so
/// nothing more is nested and nothing is unwrapped.
#[test]
fn an_already_nested_line_rederives_only_its_inner_wrap() {
    let (config, findings) = project("an_already_nested_line_rederives_only_its_inner_wrap");
    let canonical = format!("See [the [\u{a7}FS-001-login]({CANON}) spec](specs/FS-001-login.md).");
    assert_eq!(
        wrapped(&config, &findings, &canonical),
        canonical,
        "a fixed point"
    );
    assert_eq!(
        wrapped(
            &config,
            &findings,
            "See [the [\u{a7}FS-001-login](specs/old.md#gone) spec](specs/FS-001-login.md)."
        ),
        canonical,
        "the stale inner wrap is re-derived"
    );
}

/// §FS-citations.wrap.enclosing.scope: every form a link is written in encloses, so
/// the pass leaves the text of a reference link, a collapsed reference whether or
/// not a definition matches it, a definition label, an autolink and a raw HTML `<a>`
/// as written (§FS-citations.zones.authored-markup, §FS-citations.zones.link-destination),
/// and the citation-only image with its destination (§FS-citations.wrap.enclosing.images).
/// A `[` that nothing closes before its paragraph ends opens no link, so the bare
/// citation after it is still wrapped (§FS-citations.wrap.enclosing.shape).
#[test]
fn every_link_form_the_scope_lists_keeps_its_bytes() {
    let (config, findings) = project("every_link_form_the_scope_lists_keeps_its_bytes");
    for line in [
        "See [the \u{a7}FS-001-login rationale][pg].",
        "See [\u{a7}FS-001-login][pg].",
        "See [the \u{a7}FS-001-login spec][].",
        "[\u{a7}FS-001-login]: specs/FS-001-login.md",
        "[the \u{a7}FS-001-login spec]: specs/FS-001-login.md \"the \u{a7}FS-001-login spec\"",
        "See <https://example.com/\u{a7}FS-001-login>.",
        "See <a href=\"specs/FS-001-login.md\">the \u{a7}FS-001-login spec</a>.",
        "See <a title=\"\u{a7}FS-001-login\" href=\"specs/FS-001-login.md\">it</a>.",
        "See ![\u{a7}FS-001-login](diagram.png).",
    ] {
        assert_eq!(
            wrapped(&config, &findings, line),
            line,
            "`{line}` keeps its bytes"
        );
    }
    assert_eq!(
        wrapped(&config, &findings, "See [the \u{a7}FS-001-login"),
        format!("See [the [\u{a7}FS-001-login]({CANON})"),
        "an opener nothing closes encloses nothing"
    );
}
