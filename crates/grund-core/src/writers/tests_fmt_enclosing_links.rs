//! Test module: the cross-reference pass's three-way reading of a citation
//! (§FS-citations.wrap) — a wrap is re-derived, a citation inside a link its author
//! wrote is left with its link (§FS-citations.wrap.enclosing), and an unwrapped
//! citation is wrapped fresh. The reading has one owner in the grammar; these
//! cases pin it through the pass that writes, line by line.

use std::path::{Path, PathBuf};

use super::wrap_markdown_links;
use crate::config::{Config, load_config};
use crate::model::Catalog;
use crate::testing::{scan_findings, test_root, write};

/// The canonical destination of `FS-001-login` from `docs/notes.md`.
const CANON: &str = "specs/FS-001-login.md#fs-001-login-a-user-logs-in";

/// A project whose `FS` kind keeps `FS-001-login` under `docs/specs`, loaded and
/// scanned: what the pass resolves a citation against.
fn project(name: &str) -> (PathBuf, Config, Catalog) {
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
    (root, config, findings)
}

fn wrapped(root: &Path, config: &Config, findings: &Catalog, line: &str) -> String {
    wrap_markdown_links(
        line,
        &root.join("docs/notes.md"),
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
    let (root, config, findings) =
        project("a_wrap_is_rederived_an_authored_label_is_left_and_a_bare_citation_is_wrapped");
    let line = "A [§FS-001-login](stale.md) wrap, [the §FS-001-login spec](specs/FS-001-login.md) \
                link, and §FS-001-login bare.";
    assert_eq!(
        wrapped(&root, &config, &findings, line),
        format!(
            "A [§FS-001-login]({CANON}) wrap, [the §FS-001-login spec](specs/FS-001-login.md) \
             link, and [§FS-001-login]({CANON}) bare."
        )
    );
}

/// §FS-citations.wrap.enclosing.shape: every shape that encloses keeps its bytes —
/// escaped and balanced brackets and a code span in the text, an escaped and a
/// balanced parenthesis and an empty destination, and a citation repeated in one
/// text.
#[test]
fn every_enclosing_shape_keeps_its_bytes() {
    let (root, config, findings) = project("every_enclosing_shape_keeps_its_bytes");
    for line in [
        r"See [§FS-001-login: logs in \] for good](specs/FS-001-login.md).",
        "See [§FS-001-login: logs in [draft]](specs/FS-001-login.md).",
        "See [§FS-001-login: run `grund check` then `]`](specs/FS-001-login.md).",
        r"See [the §FS-001-login spec](specs/FS-001-login\).md).",
        "See [the §FS-001-login spec](specs/FS-001-login(1).md).",
        "See [the §FS-001-login spec]().",
        "See [§FS-001-login, that is §FS-001-login](specs/FS-001-login.md).",
    ] {
        assert_eq!(
            wrapped(&root, &config, &findings, line),
            line,
            "`{line}` encloses"
        );
    }
}

/// §FS-citations.wrap.enclosing.shape: each occurrence is read by its own
/// position, so the same citation outside the author's link is still wrapped.
#[test]
fn the_same_citation_outside_the_link_is_still_wrapped() {
    let (root, config, findings) = project("the_same_citation_outside_the_link_is_still_wrapped");
    assert_eq!(
        wrapped(
            &root,
            &config,
            &findings,
            "See [the §FS-001-login spec](specs/FS-001-login.md), and §FS-001-login itself."
        ),
        format!(
            "See [the §FS-001-login spec](specs/FS-001-login.md), and [§FS-001-login]({CANON}) itself."
        )
    );
}

/// §FS-citations.wrap.enclosing.shape: what does not open or close a link leaves the
/// citation unwrapped, so the pass wraps it fresh — an escaped opener, an opener in
/// a code span, a destination that never closes.
#[test]
fn what_encloses_nothing_is_wrapped_fresh() {
    let (root, config, findings) = project("what_encloses_nothing_is_wrapped_fresh");
    for (line, expected) in [
        (
            r"See \[the §FS-001-login spec](specs/FS-001-login.md).",
            format!(r"See \[the [§FS-001-login]({CANON}) spec](specs/FS-001-login.md)."),
        ),
        (
            "See `[` §FS-001-login](specs/FS-001-login.md).",
            format!("See `[` [§FS-001-login]({CANON})](specs/FS-001-login.md)."),
        ),
        (
            "See [the §FS-001-login spec](specs/FS-001-login.md",
            format!("See [the [§FS-001-login]({CANON}) spec](specs/FS-001-login.md"),
        ),
    ] {
        assert_eq!(
            wrapped(&root, &config, &findings, line),
            expected,
            "`{line}`"
        );
    }
}

/// §FS-citations.wrap.enclosing.images: an image's description encloses nothing,
/// so a citation in its alt text is wrapped as it always was.
#[test]
fn a_citation_in_an_image_description_is_wrapped() {
    let (root, config, findings) = project("a_citation_in_an_image_description_is_wrapped");
    assert_eq!(
        wrapped(
            &root,
            &config,
            &findings,
            "See ![a diagram of §FS-001-login](diagram.png)."
        ),
        format!("See ![a diagram of [§FS-001-login]({CANON})](diagram.png).")
    );
}

/// §FS-citations.wrap.enclosing.wrap-wins: on a line an older `fmt` nested, the
/// inner wrap is still a wrap and is re-derived; the outer brackets are text, so
/// nothing more is nested and nothing is unwrapped.
#[test]
fn an_already_nested_line_rederives_only_its_inner_wrap() {
    let (root, config, findings) = project("an_already_nested_line_rederives_only_its_inner_wrap");
    let canonical = format!("See [the [§FS-001-login]({CANON}) spec](specs/FS-001-login.md).");
    assert_eq!(
        wrapped(&root, &config, &findings, &canonical),
        canonical,
        "a fixed point"
    );
    assert_eq!(
        wrapped(
            &root,
            &config,
            &findings,
            "See [the [§FS-001-login](specs/old.md#gone) spec](specs/FS-001-login.md)."
        ),
        canonical,
        "the stale inner wrap is re-derived"
    );
}

/// §FS-citations.wrap.enclosing.scope: only a same-line inline link encloses for
/// now, so the text of a reference link, a collapsed reference, a definition
/// label, a raw HTML `<a>` and link text that runs onto the next line are read as
/// they were before, and the pass wraps the citation in each.
#[test]
fn what_the_scope_leaves_out_is_read_as_before() {
    let (root, config, findings) = project("what_the_scope_leaves_out_is_read_as_before");
    let wrap = format!("[§FS-001-login]({CANON})");
    for (line, expected) in [
        (
            "See [the §FS-001-login rationale][pg].".to_string(),
            format!("See [the {wrap} rationale][pg]."),
        ),
        (
            "See [the §FS-001-login spec][].".to_string(),
            format!("See [the {wrap} spec][]."),
        ),
        (
            "[§FS-001-login]: specs/FS-001-login.md".to_string(),
            format!("[{wrap}]: specs/FS-001-login.md"),
        ),
        (
            r#"See <a href="specs/FS-001-login.md">the §FS-001-login spec</a>."#.to_string(),
            format!(r#"See <a href="specs/FS-001-login.md">the {wrap} spec</a>."#),
        ),
        (
            "See [the §FS-001-login".to_string(),
            format!("See [the {wrap}"),
        ),
    ] {
        assert_eq!(
            wrapped(&root, &config, &findings, &line),
            expected,
            "`{line}`"
        );
    }
}
