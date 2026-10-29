//! Black-box guards that run today and must still hold once off-format citations are reported:
//! each reading §FS-citations.recognized.off-format leaves alone keeps its own verdict, and an
//! off-format token stays outside the graph, so `show`, `refs` and `fmt` treat it as they did.

#[path = "support/off_format_citation.rs"]
mod support;

use support::*;

/// A kind nothing configures, an unmarked or escaped token, a prefix mention and a fence stay
/// text (§FS-citations.recognized.off-format, §FS-citations.recognized.contexts).
#[test]
fn what_opens_with_no_configured_kind_or_is_not_read_stays_text() {
    let fx = Fixture::page(
        "# FS-a: A\n\nUnconfigured \u{a7}XYZ-1, unmarked REQ-999, escaped <\u{a7}>REQ-999.\n\
         Prefix mentions \u{a7}REQ- and \u{a7}REQ-<slug> and \u{a7}REQ-\u{2026}.\n\n\
         ```text\n\u{a7}REQ-999 in a fence\n```\n",
    );
    let run = fx.expect(0, &["check"]);
    assert_eq!(run.stdout, UNUSED_FS_A);
    assert_eq!(run.stderr, "");
}

/// An in-format token nothing declares still dangles, and only dangles (§FS-check.3.1).
#[test]
fn an_in_format_token_still_dangles() {
    let fx = Fixture::page("# FS-a: A\n\nSee \u{a7}REQ-a12.\n");
    let run = fx.expect(1, &["check"]);
    let dangling = text(PAGE, 3, "error", "unknown reference REQ-a12");
    assert_eq!(run.stdout, format!("{dangling}{UNUSED_FS_A}"));
}

/// A heading's citation of its own in-format ID stays dropped
/// (§FS-citations.recognized.title-lines).
#[test]
fn a_headings_citation_of_its_own_id_stays_dropped() {
    let page = "docs/FS/FS-001-alpha.md";
    let fx = Fixture::with(&[
        ("grund.toml", NUMBERED),
        (
            page,
            "# FS-001-alpha: Alpha, see \u{a7}FS-001-alpha\n\nBody.\n",
        ),
    ]);
    let run = fx.expect(0, &["check"]);
    assert_eq!(
        run.stdout,
        text(
            page,
            1,
            "warning",
            "declared but never cited: FS-001-alpha; cite it"
        )
    );
}

/// A number-only shorthand keeps its own verdict and earns no second (§FS-citations.shorthand).
#[test]
fn a_number_only_shorthand_keeps_its_own_verdict() {
    let page = "docs/FS/FS-001-a.md";
    let fx = Fixture::with(&[
        ("grund.toml", NUMBERED),
        (page, "# FS-001-a: A\n\nShorthand \u{a7}FS-042.\n"),
    ]);
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(
                page,
                3,
                "error",
                "shorthand citation \u{a7}FS-042 matches no declaration; write the full ID it means",
            ),
            text(page, 1, "warning", "declared but never cited: FS-001-a; cite it"),
        ]
        .concat()
    );
}

/// A pattern keeps its glob verdict (§FS-check.checks.glob-citation).
#[test]
fn a_pattern_keeps_its_glob_verdict() {
    let fx = Fixture::page("# FS-a: A\n\nSee \u{a7}FS-* for all of them.\n");
    let run = fx.expect(1, &["check"]);
    let glob = text(
        PAGE,
        3,
        "error",
        "glob citation \u{a7}FS-*: a citation names one exact point, not a pattern; cite the \
         point, or escape it as <\u{a7}>FS-*; this became an error in grund 0.19.0",
    );
    assert_eq!(run.stdout, format!("{glob}{UNUSED_FS_A}"));
}

/// A token whose alias names no project is an unknown project and nothing else (§FS-check.3.8).
#[test]
fn an_unknown_alias_is_an_unknown_project_and_nothing_else() {
    let fx = Fixture::page("# FS-a: A\n\nSee \u{a7}nope/REQ-999.\n");
    let run = fx.expect(1, &["check"]);
    let unknown = text(PAGE, 3, "error", "unknown project alias nope");
    assert_eq!(run.stdout, format!("{unknown}{UNUSED_FS_A}"));
}

/// Once a declaration of exactly that spelling exists the token cites it, and the only finding
/// is the declaration's (§FS-citations.recognized.off-grammar).
#[test]
fn a_backed_token_cites_its_declaration() {
    let fx = Fixture::with(&[
        ("grund.toml", CONFIG),
        ("docs/REQ/REQ-999.md", "# REQ-999: Nine\n"),
        (PAGE, "# FS-a: A\n\nSee \u{a7}REQ-999.\n"),
    ]);
    let run = fx.expect(1, &["check"]);
    let near_miss = text(
        "docs/REQ/REQ-999.md",
        1,
        "error",
        "`REQ-999` resolves for compatibility but does not match [schema] id_format = \
         \"{kind}-{slug}\" — rename it or change the effective format; this became an error in \
         grund 0.16.0",
    );
    assert_eq!(run.stdout, format!("{near_miss}{UNUSED_FS_A}"));
    let refs = fx.expect(0, &["refs", "REQ-999"]);
    assert_eq!(refs.stdout, "docs/FS/FS-a.md:3: \u{a7}REQ-999\n");
}

/// `show` and `refs` still refuse an off-format token as an invalid ID (§FS-show.3.5.1).
#[test]
fn show_and_refs_still_refuse_an_off_format_token() {
    let fx = Fixture::issue();
    let refusal = "invalid ID `REQ-12a`\nhint: this repo's [schema] id_format is `{kind}-{slug}` \
                   (run `grund config show`); `grund list` shows the IDs that exist\n";
    for command in ["show", "refs"] {
        let run = fx.expect(1, &[command, "REQ-12a"]);
        assert_eq!(run.stdout, "", "{command}");
        assert_eq!(run.stderr, refusal, "{command}");
    }
}

/// `grund fmt` rewrites nothing in an off-format token (§FS-fmt.2.4).
#[test]
fn fmt_rewrites_nothing_in_an_off_format_token() {
    let fx = Fixture::issue();
    let run = fx.expect(0, &["fmt", "--write"]);
    assert_eq!(run.stdout, "rewrote 0 lines\n");
    assert_eq!(fx.read(PAGE), ISSUE);
}
