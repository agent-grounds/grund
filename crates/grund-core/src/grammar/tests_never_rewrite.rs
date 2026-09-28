//! Test module: the escape-position carve-out of §FS-check.1.1.9, the one
//! never-rewrite zone that depends on neither the strict mode nor the host
//! language (§AR-scanner.2.3.1).

use super::*;

/// §FS-check.1.1.9: the predicate is asked of the *token's own start column*, and
/// answers for the bytes immediately before it — the configured marker wrapped in
/// `<` and `>`. A space between the escape and the ID escapes nothing, and a line
/// with no escape on it is never in an escape position.
#[test]
fn an_escape_position_is_the_bytes_immediately_before_the_token() {
    let line = "The shape is <§>FS-001-login in prose.";
    let start = line.find("FS-001-login").expect("the illustrated token");
    assert!(
        in_escape_position(line, start, "§"),
        "a token opening immediately after `<§>` is in an escape position"
    );

    let spaced = "The shape is <§> FS-001-login in prose.";
    let spaced_start = spaced.find("FS-001-login").expect("the spaced token");
    assert!(
        !in_escape_position(spaced, spaced_start, "§"),
        "`<§> ` with a space escapes nothing — the token does not begin after the `>`"
    );

    let live = "The rule is §FS-001-login here.";
    let live_start = live.find("FS-001-login").expect("the live token");
    assert!(
        !in_escape_position(live, live_start, "§"),
        "an ordinary marked citation is not in an escape position"
    );

    let bare = "The rule is FS-001-login here.";
    let bare_start = bare.find("FS-001-login").expect("the bare token");
    assert!(
        !in_escape_position(bare, bare_start, "§"),
        "a bare token with no escape in front of it is not in an escape position"
    );

    // Asked of the start column only: one byte further in, the escape is no
    // longer immediately behind the position.
    assert!(
        !in_escape_position(line, start + 1, "§"),
        "the question is about the token's start, not about being after an escape"
    );
}

/// §FS-check.1.1.9: the escape is spelled with the **configured** marker, so under
/// `marker = "@"` the escape is `<@>` and a `<§>`-wrapped token is an ordinary bare
/// one. An empty marker configures no escape at all.
#[test]
fn the_escape_is_spelled_with_the_configured_marker() {
    let at_escape = "The shape is <@>FS-001-login in prose.";
    let at_start = at_escape.find("FS-001-login").expect("the token");
    assert!(
        in_escape_position(at_escape, at_start, "@"),
        "`<@>` is the escape where `@` is the configured marker"
    );
    assert!(
        !in_escape_position(at_escape, at_start, "§"),
        "`<@>` escapes nothing in a project whose marker is `§`"
    );

    let section_escape = "The shape is <§>FS-001-login in prose.";
    let section_start = section_escape.find("FS-001-login").expect("the token");
    assert!(
        !in_escape_position(section_escape, section_start, "@"),
        "`<§>` is an ordinary prefix in a project whose marker is `@`"
    );
    assert!(
        !in_escape_position(section_escape, section_start, ""),
        "an empty marker configures no escape, so no position is an escape position"
    );
}

/// §AR-scanner.2.3.1: the escape is the third zone `bare_token_in_never_rewrite_zone`
/// answers for, and it is the one that holds in Markdown and in source alike —
/// the other two are chosen by `is_md`.
#[test]
fn the_shared_zone_predicate_answers_for_the_escape_in_both_host_languages() {
    let line = "The shape is <§>FS-001-login in prose.";
    let start = line.find("FS-001-login").expect("the illustrated token");
    for is_md in [true, false] {
        assert!(
            bare_token_in_never_rewrite_zone(line, is_md, start, "§"),
            "the escape position is a never-rewrite zone with is_md = {is_md}"
        );
    }

    let bare = "The rule is FS-001-login here.";
    let bare_start = bare.find("FS-001-login").expect("the bare token");
    for is_md in [true, false] {
        assert!(
            !bare_token_in_never_rewrite_zone(bare, is_md, bare_start, "§"),
            "an ordinary bare token in prose is in no zone with is_md = {is_md}"
        );
    }
}
