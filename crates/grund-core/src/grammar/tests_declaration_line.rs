//! Test module: which ID a line in declaration position declares, read at the
//! seam every reader of a declaration line shares — [`declaration_id_on_line`],
//! which asks the declaration patterns first and the near-miss reading after
//! (§FS-declarations.line, §AR-scanner.2.1.1). The e2e cases
//! `*-section-suffixed-line-*` pin what `list`, `check` and `show` print; these
//! pin the one answer underneath them, in each shape a declaration line takes.

use super::{Grammar, declaration_id_on_line, near_miss_heading, render_id};
use crate::config::Config;
use crate::testing::{numbered_config, test_root};

/// Where a line sits, which is what decides the pattern it is read with.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// A heading in a `.md` file.
    Markdown,
    /// A comment-prefixed line in a source file.
    Comment,
    /// A bare line inside a Python docstring.
    Docstring,
}

impl Shape {
    fn flags(self) -> (bool, bool) {
        match self {
            Shape::Markdown => (false, true),
            Shape::Comment => (false, false),
            Shape::Docstring => (true, false),
        }
    }
}

/// The ID `line` declares, spelled as `list` prints it, or `None` for a line
/// that declares nothing.
fn declared(grammar: &Grammar, shape: Shape, line: &str) -> Option<String> {
    let (in_py_docstring, is_md) = shape.flags();
    declaration_id_on_line(grammar, line, in_py_docstring, is_md)
        .map(|(id, _)| render_id(grammar, &id))
}

/// The default `{kind}-{number}-{slug}` grammar with `edit` applied on top.
fn config_with(name: &str, edit: impl FnOnce(&mut Config)) -> Config {
    let mut config = numbered_config(test_root(name));
    edit(&mut config);
    config.rebuild_grammar().expect("rebuild the test grammar");
    config
}

/// Every row must declare nothing: no declaration, and no near miss either.
fn assert_declares_nothing(grammar: &Grammar, rows: &[(Shape, &str)]) {
    let declared = rows
        .iter()
        .filter_map(|&(shape, line)| declared(grammar, shape, line).map(|id| (shape, line, id)))
        .collect::<Vec<_>>();
    assert!(
        declared.is_empty(),
        "these lines open with a section-suffixed ID and must declare nothing, \
         but each was read as a declaration of the ID on the right:\n{declared:#?}"
    );
}

/// §FS-declarations.line.section-suffix: the report's three shapes, and the
/// `///` doc-comment beside the plain `//` one. Each opens with an ID followed
/// directly by `.` and a numbered section, and today each declares the parent
/// ID with the title `.2 …`.
#[test]
fn a_section_suffixed_id_declares_nothing_in_any_shape() {
    let config = config_with(
        "a_section_suffixed_id_declares_nothing_in_any_shape",
        |_| {},
    );
    assert_declares_nothing(
        &config.grammar,
        &[
            (
                Shape::Docstring,
                "    FS-042-user-login.2 is named here with a section suffix and no colon.",
            ),
            (
                Shape::Comment,
                "    // FS-042-user-login.2 / a citation note whose marker was dropped",
            ),
            (
                Shape::Comment,
                "/// FS-042-user-login.2 names the session section",
            ),
            (
                Shape::Markdown,
                "# FS-042-user-login.2 and what a session must outlive",
            ),
            (
                Shape::Docstring,
                "    FS-999-nowhere.3 is only mentioned; nothing declares it anywhere.",
            ),
            (Shape::Docstring, "FS-042-user-login.2.1"),
        ],
    );
}

/// §FS-declarations.line.section-suffix.3: a named path is a section only where
/// the project enables named sections, and where it does, an ID followed by one
/// declares nothing.
#[test]
fn a_named_section_suffix_declares_nothing_where_named_sections_are_on() {
    let config = config_with(
        "a_named_section_suffix_declares_nothing_where_named_sections_are_on",
        |config| config.named_sections = true,
    );
    assert_declares_nothing(
        &config.grammar,
        &[
            (
                Shape::Docstring,
                "    FS-042-user-login.session is named here by its name.",
            ),
            (
                Shape::Comment,
                "// FS-042-user-login.session / a dropped marker",
            ),
            (
                Shape::Markdown,
                "## FS-042-user-login.goals.latency and what it bounds",
            ),
        ],
    );
}

/// §FS-declarations.line.section-suffix.3: the separator is the configured one.
/// Under `section_separator = ":"` the `:` after the ID is the separator in
/// front of a section, so these lines declare nothing and are no near miss
/// either: the near-miss reading must not take that `:` for a declaration colon
/// (§FS-declarations.line.section-suffix.1).
#[test]
fn the_configured_separator_is_the_one_that_counts() {
    let config = config_with(
        "the_configured_separator_is_the_one_that_counts",
        |config| {
            config.section_separator = ":".to_string();
        },
    );
    let rows = [
        (
            Shape::Docstring,
            "    FS-042-user-login:2 is named here, after the separator.",
        ),
        (Shape::Comment, "// FS-042-user-login:2 / a dropped marker"),
        (
            Shape::Markdown,
            "# FS-042-user-login:2 and what a session must outlive",
        ),
    ];
    assert_declares_nothing(&config.grammar, &rows);
    for (shape, line) in rows {
        let (in_py_docstring, is_md) = shape.flags();
        assert_eq!(
            near_miss_heading(&config.grammar, line, in_py_docstring, is_md).map(|hit| hit.0),
            None,
            "{shape:?} line {line:?} is no near miss: its `:` is the section separator"
        );
    }
}

/// §FS-declarations.line.section-suffix: what the point leaves alone. A
/// declaration written the way §FS-show.2.3 writes one still declares its ID,
/// in every shape and under either separator, the declaration colon followed by
/// its title. This holds today; it is here so the fix cannot over-reach.
#[test]
fn a_declaration_with_its_colon_still_declares() {
    let dotted = config_with("a_declaration_with_its_colon_still_declares", |_| {});
    let colon = config_with(
        "a_declaration_with_its_colon_still_declares_colon",
        |config| {
            config.section_separator = ":".to_string();
        },
    );
    for config in [&dotted, &colon] {
        for (shape, line) in [
            (Shape::Markdown, "# FS-042-user-login: the user logs in"),
            (Shape::Comment, "/// FS-042-user-login: the user logs in"),
            (Shape::Docstring, "    FS-042-user-login: the user logs in"),
        ] {
            assert_eq!(
                declared(&config.grammar, shape, line).as_deref(),
                Some("FS-042-user-login"),
                "{shape:?} line {line:?} under separator {:?}",
                config.section_separator
            );
        }
    }
}

/// §FS-declarations.line.section-suffix.2: with the declaration colon, the
/// token ends at the colon and is all of `FS-042-user-login.2`. The grammar
/// rejects it, so it is the off-grammar declaration §FS-config.3.2.5 retains
/// under its exact spelling, and a near miss — never the parent ID. This holds
/// today; it is here so the fix cannot turn it into a declaration of the
/// parent, or into nothing.
#[test]
fn with_the_colon_the_whole_token_is_the_near_miss() {
    let config = config_with("with_the_colon_the_whole_token_is_the_near_miss", |_| {});
    for (shape, line) in [
        (
            Shape::Markdown,
            "## FS-042-user-login.2: a heading with the colon",
        ),
        (
            Shape::Comment,
            "/// FS-042-user-login.2: a doc-comment with the colon",
        ),
        (
            Shape::Docstring,
            "    FS-042-user-login.2: a docstring line with the colon",
        ),
    ] {
        assert_eq!(
            declared(&config.grammar, shape, line).as_deref(),
            Some("FS-042-user-login.2"),
            "{shape:?} line {line:?}"
        );
        let (in_py_docstring, is_md) = shape.flags();
        assert_eq!(
            near_miss_heading(&config.grammar, line, in_py_docstring, is_md).map(|hit| hit.0),
            Some("FS-042-user-login.2"),
            "{shape:?} line {line:?} is reported as a near miss"
        );
    }
}
