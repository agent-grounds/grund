//! Test module: which ID a line in declaration position declares, read at the
//! seam every reader of a declaration line shares — [`declaration_id_on_line`],
//! which asks the declaration patterns first and the near-miss reading after
//! (§FS-declarations.line, §AR-scanner.2.1.1). The e2e cases
//! `*-section-suffixed-line-*` pin what `list`, `check` and `show` print; these
//! pin the one answer underneath them, in each shape a declaration line takes.

use super::{
    Grammar, declaration_captures, declaration_id_on_line, near_miss_heading, parse_id,
    parse_id_arg, render_id,
};
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

/// §FS-declarations.line.configured-slug: punctuation admitted by the slug
/// grammar keeps the query's identity in every declaration position.
#[test]
fn configured_star_slug_declarations_keep_canonical_identity() {
    let config = config_with(
        "configured_star_slug_declarations_keep_canonical_identity",
        |c| {
            c.id_format = "{kind}-{slug}".into();
            c.slug_pattern = "[a-z*][a-z0-9*-]*".into();
        },
    );
    let mut disagreements = Vec::new();
    for token in ["FS-*", "FS-tail*"] {
        let expected = parse_id_arg(token, &config.grammar).unwrap().0;
        for (shape, prefix) in [
            (Shape::Markdown, "# "),
            (Shape::Comment, "/// "),
            (Shape::Docstring, "    "),
        ] {
            let line = format!("{prefix}{token}: Literal star");
            let (in_docstring, is_md) = shape.flags();
            let actual = declaration_id_on_line(&config.grammar, &line, in_docstring, is_md);
            if actual.as_ref().map(|(id, _)| id) != Some(&expected) {
                disagreements.push(format!(
                    "{shape:?} {token}: expected {expected:?}, got {actual:?}"
                ));
            }
            if let Some((_, end)) = actual {
                assert_eq!(&line[end..], ": Literal star", "{shape:?} {token}");
            }
        }
    }
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n"));
}

/// §FS-declarations.line.configured-slug: preserving punctuation never grants
/// a canonical prefix to an invalid longer token or a section coordinate.
#[test]
fn configured_star_slug_invalid_extended_tokens_keep_exact_spelling() {
    let config = config_with(
        "configured_star_slug_preserves_complete_token_and_section_guards",
        |c| {
            c.id_format = "{kind}-{slug}".into();
            c.slug_pattern = "[a-z*][a-z0-9*-]*".into();
        },
    );
    for (shape, prefix) in [
        (Shape::Markdown, "# "),
        (Shape::Comment, "/// "),
        (Shape::Docstring, "    "),
    ] {
        let (in_docstring, is_md) = shape.flags();
        let invalid = format!("{prefix}FS-tail*!: Invalid extension");
        assert!(declaration_captures(&config.grammar, &invalid, in_docstring, is_md).is_none());
        assert_eq!(
            declared(&config.grammar, shape, &invalid).as_deref(),
            Some("FS-tail*!")
        );
        assert_eq!(
            near_miss_heading(&config.grammar, &invalid, in_docstring, is_md).map(|hit| hit.0),
            Some("FS-tail*!")
        );
    }
}

/// §FS-declarations.line.configured-slug: a section heading cannot declare
/// either its parent ID or a shorter canonical prefix of that parent.
#[test]
fn configured_star_slug_section_coordinates_never_declare_a_prefix() {
    let config = config_with(
        "configured_star_slug_section_coordinates_never_declare_a_prefix",
        |c| {
            c.id_format = "{kind}-{slug}".into();
            c.slug_pattern = "[a-z*][a-z0-9*-]*".into();
        },
    );
    for (shape, prefix) in [
        (Shape::Markdown, "# "),
        (Shape::Comment, "/// "),
        (Shape::Docstring, "    "),
    ] {
        let (in_docstring, is_md) = shape.flags();
        let coordinate = format!("{prefix}FS-tail*.2 names a section");
        assert_eq!(declared(&config.grammar, shape, &coordinate), None);
        let with_colon = format!("{prefix}FS-tail*.2: Section coordinate");
        assert!(declaration_captures(&config.grammar, &with_colon, in_docstring, is_md).is_none());
        assert_eq!(
            declared(&config.grammar, shape, &with_colon).as_deref(),
            Some("FS-tail*.2")
        );
    }
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

/// §FS-declarations.line.configured-literals: the full canonical identity and
/// the body reader's boundary must survive an internal colon in every shape.
#[test]
fn colon_format_declaration_is_canonical_in_every_shape() {
    let config = config_with(
        "colon_format_declaration_is_canonical_in_every_shape",
        |c| {
            c.id_format = "{kind}:{slug}".into();
        },
    );
    let expected = parse_id_arg("FS:login", &config.grammar).unwrap().0;
    for (shape, line) in [
        (Shape::Markdown, "# FS:login: Login"),
        (Shape::Comment, "/// FS:login: Login"),
        (Shape::Docstring, "    FS:login: Login"),
    ] {
        let (in_docstring, is_md) = shape.flags();
        let captures = declaration_captures(&config.grammar, line, in_docstring, is_md)
            .unwrap_or_else(|| {
                panic!("{shape:?}: {line:?} lost its canonical declaration capture")
            });
        assert_eq!(captures.name("id").unwrap().as_str(), "FS:login");
        let (actual, end) =
            declaration_id_on_line(&config.grammar, line, in_docstring, is_md).unwrap();
        assert_eq!(
            actual, expected,
            "{shape:?}: declaration and query identities"
        );
        assert_eq!(&line[end..], ": Login", "{shape:?}: complete ID boundary");
    }
}

/// §FS-declarations.line.configured-literals: a conforming heading must never
/// be diagnosed as a near miss, even though its spelling can be cataloged today.
#[test]
fn colon_format_declaration_is_not_a_near_miss() {
    let config = config_with("colon_format_declaration_is_not_a_near_miss", |c| {
        c.id_format = "{kind}:{slug}".into();
    });
    for (shape, line) in [
        (Shape::Markdown, "# FS:login: Login"),
        (Shape::Comment, "/// FS:login: Login"),
        (Shape::Docstring, "    FS:login: Login"),
    ] {
        let (in_docstring, is_md) = shape.flags();
        assert!(
            near_miss_heading(&config.grammar, line, in_docstring, is_md).is_none(),
            "{shape:?}: {line:?} is conforming, not a declaration-near-miss"
        );
    }
}

/// §FS-config.3.2.5: the colon-format correction must preserve the guard that
/// retains a longer off-grammar token instead of claiming its canonical prefix.
#[test]
fn colon_format_fix_preserves_longer_off_grammar_tokens() {
    let config = config_with(
        "colon_format_fix_preserves_longer_off_grammar_tokens",
        |c| {
            c.id_format = "{kind}-{slug}".into();
            c.slug_pattern = "[a-z]+".into();
        },
    );
    for (shape, line) in [
        (Shape::Markdown, "# FS-legacy-2: Login"),
        (Shape::Comment, "/// FS-legacy-2: Login"),
        (Shape::Docstring, "    FS-legacy-2: Login"),
    ] {
        let (in_docstring, is_md) = shape.flags();
        assert!(declaration_captures(&config.grammar, line, in_docstring, is_md).is_none());
        assert_eq!(
            declared(&config.grammar, shape, line).as_deref(),
            Some("FS-legacy-2")
        );
        assert_eq!(
            near_miss_heading(&config.grammar, line, in_docstring, is_md).map(|hit| hit.0),
            Some("FS-legacy-2")
        );
    }
}

/// §FS-declarations.line.section-suffix: an internal colon does not make a
/// section coordinate a canonical declaration, or declare its parent ID.
#[test]
fn colon_format_section_suffix_does_not_declare_the_parent() {
    let config = config_with(
        "colon_format_section_suffix_does_not_declare_the_parent",
        |c| {
            c.id_format = "{kind}:{slug}".into();
        },
    );
    for (shape, line) in [
        (Shape::Markdown, "# FS:login.2 names a section"),
        (Shape::Comment, "/// FS:login.2 names a section"),
        (Shape::Docstring, "    FS:login.2 names a section"),
    ] {
        let (in_docstring, is_md) = shape.flags();
        assert!(declaration_captures(&config.grammar, line, in_docstring, is_md).is_none());
        assert_eq!(declared(&config.grammar, shape, line), None);
    }
}

/// §FS-config.3.2.5: a syntactic capture whose number exceeds u32 remains a
/// readable declaration under its exact spelling, with a near-miss diagnostic.
#[test]
fn numeric_overflow_preserves_the_compatibility_declaration() {
    let config = config_with(
        "numeric_overflow_preserves_the_compatibility_declaration",
        |c| {
            c.slug_pattern = "[a-z]+".into();
        },
    );
    let token = "FS-4294967296-login";
    assert!(parse_id_arg(token, &config.grammar).is_err());
    for (shape, line) in [
        (Shape::Markdown, "# FS-4294967296-login: Login"),
        (Shape::Comment, "/// FS-4294967296-login: Login"),
        (Shape::Docstring, "    FS-4294967296-login: Login"),
    ] {
        let (in_docstring, is_md) = shape.flags();
        let captures = declaration_captures(&config.grammar, line, in_docstring, is_md)
            .expect("numeric overflow still matches the configured declaration regex");
        assert_eq!(captures.name("id").unwrap().as_str(), token);
        assert!(parse_id(&captures, &config.grammar).is_none());
        assert_eq!(
            near_miss_heading(&config.grammar, line, in_docstring, is_md),
            Some((token, "{kind}-{number}-{slug}", "FS")),
            "{shape:?}: the rejected canonical capture must remain a near miss"
        );
        assert_eq!(
            declared(&config.grammar, shape, line).as_deref(),
            Some(token)
        );
        let (_, end) = declaration_id_on_line(&config.grammar, line, in_docstring, is_md).unwrap();
        assert_eq!(&line[end..], ": Login", "{shape:?}: exact ID boundary");
    }
}
