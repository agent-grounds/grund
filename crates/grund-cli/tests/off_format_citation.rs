//! Black-box: a marked token that opens with a configured kind, that its kind's format does not
//! spell and that nothing declares is an off-format citation (§FS-check.1.1.12,
//! §FS-citations.recognized.off-format), reported as `off-format-citation` where it was dropped
//! (§FS-check.checks.off-format-citation).
//! Ports the #1073 reproducer. Source files are `off_format_citation_sources.rs`, and the readings
//! that keep their own verdict are `off_format_citation_controls.rs`.

#[path = "support/off_format_citation.rs"]
mod support;

use support::*;

/// The issue's own run: both off-format tokens are reported beside the dangling one.
#[test]
fn the_issue_reports_both_off_format_tokens_beside_the_dangling_one() {
    let fx = Fixture::issue();
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(PAGE, 3, "error", &off_format("REQ-12a", REQ_FORMAT)),
            text(PAGE, 3, "error", &off_format("REQ-999", REQ_FORMAT)),
            text(PAGE, 3, "error", "unknown reference REQ-a12"),
            UNUSED_FS_A.to_owned(),
        ]
        .concat()
    );
    assert_eq!(run.stderr, "");
}

/// JSON carries the same findings with the code, in the global order of §FS-errors.4.1.
#[test]
fn the_issue_in_json_carries_the_off_format_code() {
    let fx = Fixture::issue();
    let run = fx.expect(1, &["check", "--format", "json"]);
    assert_eq!(
        run.stdout,
        [
            json(
                "warning",
                PAGE,
                1,
                "unused",
                "declared but never cited: FS-a; cite it"
            ),
            json("error", PAGE, 3, CODE, &off_format("REQ-12a", REQ_FORMAT)),
            json("error", PAGE, 3, CODE, &off_format("REQ-999", REQ_FORMAT)),
            json("error", PAGE, 3, "dangling", "unknown reference REQ-a12"),
        ]
        .concat()
    );
}

/// With the dangling token deleted the tree passed; the two off-format tokens now fail it.
#[test]
fn without_the_dangling_token_the_tree_no_longer_passes() {
    let fx = Fixture::page("# FS-a: A\n\nSee \u{a7}REQ-999 and \u{a7}REQ-12a.\n");
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(PAGE, 3, "error", &off_format("REQ-12a", REQ_FORMAT)),
            text(PAGE, 3, "error", &off_format("REQ-999", REQ_FORMAT)),
            UNUSED_FS_A.to_owned(),
        ]
        .concat()
    );
}

/// A section path is part of the token, one trailing `.` is punctuation, and inline code is read
/// like prose.
#[test]
fn a_section_path_is_part_of_the_token_and_a_trailing_dot_is_not() {
    let fx = Fixture::page(
        "# FS-a: A\n\nSections \u{a7}REQ-999.2 and \u{a7}REQ-12a.3.1.\n\n\
         A sentence ends at \u{a7}REQ-999.\n\nInline code `\u{a7}REQ-7` is read too.\n",
    );
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(PAGE, 3, "error", &off_format("REQ-12a.3.1", REQ_FORMAT)),
            text(PAGE, 3, "error", &off_format("REQ-999.2", REQ_FORMAT)),
            text(PAGE, 5, "error", &off_format("REQ-999", REQ_FORMAT)),
            text(PAGE, 7, "error", &off_format("REQ-7", REQ_FORMAT)),
            UNUSED_FS_A.to_owned(),
        ]
        .concat()
    );
}

/// Under a numbered format a token with no number, or off the slug pattern, is reported, and a
/// number-only shorthand keeps its own verdict (§FS-citations.shorthand.full-id-wins).
#[test]
fn a_numbered_format_reports_what_it_does_not_spell() {
    let page = "docs/FS/FS-001-a.md";
    let fx = Fixture::with(&[
        ("grund.toml", NUMBERED),
        (
            page,
            "# FS-001-a: A\n\nShorthand \u{a7}FS-042, no number \u{a7}FS-login, off-pattern \
             \u{a7}FS-042-User-Login and \u{a7}FS-042-User-Login.1.\n",
        ),
    ]);
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(page, 3, "error", &off_format("FS-042-User-Login.1", FS_NUMBERED)),
            text(page, 3, "error", &off_format("FS-042-User-Login", FS_NUMBERED)),
            text(page, 3, "error", &off_format("FS-login", FS_NUMBERED)),
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

/// A kind's own format names its own key and only the component its template holds, beside an
/// in-format token that dangles.
#[test]
fn a_kinds_own_format_names_its_own_key() {
    let fx = Fixture::with(&[
        ("grund.toml", TICKETS),
        ("docs/TICKET/TICKET-7.md", "# TICKET-7: Seven\n"),
        (
            PAGE,
            "# FS-a: A\n\nDecided in \u{a7}TICKET-7.\nThen \u{a7}TICKET-abc and \u{a7}TICKET-8.\n",
        ),
    ]);
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(PAGE, 4, "error", &off_format("TICKET-abc", TICKET_FORMAT)),
            text(
                PAGE,
                4,
                "error",
                "unknown reference TICKET-8; did you mean TICKET-7?"
            ),
            UNUSED_FS_A.to_owned(),
        ]
        .concat()
    );
}

/// A qualified token is judged by the project its alias names and named with its alias, while an
/// unknown alias stays an unknown project and nothing else (§FS-check.3.8).
#[test]
fn a_qualified_token_is_judged_by_the_project_its_alias_names() {
    let page = "app/docs/FS/FS-b.md";
    let fx = Fixture::with(&[
        (
            "grund.toml",
            "grund_config_version = 2\nproject_name = \"ws\"\n\n[workspace]\nmembers = [\"req\", \"app\"]\n",
        ),
        ("req/grund.toml", &CONFIG.replace("\"fixture\"", "\"req\"")),
        ("req/docs/FS/FS-a.md", "# FS-a: A\n\nLocal.\n"),
        (
            "app/grund.toml",
            "grund_config_version = 2\nproject_name = \"app\"\n\n[schema.kinds.FS]\n\
             folders = [\"docs/FS\"]\nindex = false\n",
        ),
        (
            page,
            "# FS-b: B\n\nSee \u{a7}req/FS-a and \u{a7}req/REQ-999 and \u{a7}req/REQ-zzz and \
             \u{a7}nope/REQ-999.\n",
        ),
    ]);
    // stderr carries the root's own nothing-to-scan warning, which this rule does not touch.
    let run = fx.expect(1, &["check"]);
    assert_eq!(
        run.stdout,
        [
            text(page, 3, "error", &off_format("req/REQ-999", REQ_FORMAT)),
            text(page, 3, "error", "unknown project alias nope"),
            text(page, 3, "error", "unknown reference req/REQ-zzz"),
            text(
                page,
                1,
                "warning",
                "declared but never cited: FS-b; cite it"
            ),
        ]
        .concat()
    );
}

/// A file outside the default scope, which a plain run does not read.
fn out_of_scope() -> Fixture {
    let config = format!(
        "{CONFIG}\n[schema.kinds.archive]\ncitable = false\nscan = false\nfolders = [\"archive\"]\n"
    );
    Fixture::with(&[
        ("grund.toml", &config),
        (PAGE, "# FS-a: A\n\nText.\n"),
        (
            "archive/old.md",
            "Old, cites \u{a7}FS-a and \u{a7}REQ-999 and \u{a7}REQ-a12.\n",
        ),
    ])
}

/// Guard: a plain run reads nothing outside the default scope, so it reports nothing there.
#[test]
fn a_plain_run_reports_nothing_outside_the_default_scope() {
    let run = out_of_scope().expect(0, &["check"]);
    assert_eq!(run.stdout, UNUSED_FS_A);
}

/// Under `--full` an out-of-scope site takes the compound code (§FS-check.3.14.5), its message
/// after the scope, in text and in JSON.
#[test]
fn under_full_an_out_of_scope_site_takes_the_compound_code() {
    let fx = out_of_scope();
    let scope = "outside configured places and source scope: ";
    let off = format!("{scope}{}", off_format("REQ-999", REQ_FORMAT));
    let dangling = format!("{scope}unknown reference REQ-a12");
    let run = fx.expect(1, &["check", "--full"]);
    assert_eq!(
        run.stdout,
        [
            text("archive/old.md", 1, "error", &off),
            text("archive/old.md", 1, "error", &dangling),
            UNUSED_FS_A.to_owned(),
        ]
        .concat()
    );
    let run = fx.expect(1, &["check", "--full", "--format", "json"]);
    assert_eq!(
        run.stdout,
        [
            json(
                "error",
                "archive/old.md",
                1,
                "out-of-scope-off-format-citation",
                &off
            ),
            json(
                "error",
                "archive/old.md",
                1,
                "out-of-scope-dangling",
                &dangling
            ),
            json(
                "warning",
                PAGE,
                1,
                "unused",
                "declared but never cited: FS-a; cite it"
            ),
        ]
        .concat()
    );
}

/// `--only` and `--ignore` select the code by the ordinary rules, and an ignored finding takes
/// its exit code with it (§FS-check.1.4).
#[test]
fn only_and_ignore_select_the_off_format_code() {
    let fx = Fixture::page("# FS-a: A\n\nSee \u{a7}REQ-999 and \u{a7}REQ-12a.\n");
    let run = fx.expect(1, &["check", "--only", CODE]);
    assert_eq!(
        run.stdout,
        [
            text(PAGE, 3, "error", &off_format("REQ-12a", REQ_FORMAT)),
            text(PAGE, 3, "error", &off_format("REQ-999", REQ_FORMAT)),
        ]
        .concat()
    );
    let run = fx.expect(0, &["check", "--ignore", CODE]);
    assert_eq!(run.stdout, UNUSED_FS_A);
}

/// `config explain` answers both codes from their rows in the catalog of §FS-errors.5.5.
#[test]
fn config_explain_answers_both_codes() {
    let fx = Fixture::issue();
    let fix = "Fix: cite an ID in that format, or write {escape} for an illustration\n";
    assert_eq!(
        fx.expect(0, &["config", "explain", CODE]).stdout,
        format!(
            "grund.toml is read as version 2.\n\noff-format-citation\nA marked citation opens \
             with a configured kind, but its kind's ID format does not spell it and nothing \
             declares it.\n\nSeverity: error, fixed.\nOn: always.\n{fix}"
        )
    );
    assert_eq!(
        fx.expect(
            0,
            &["config", "explain", "out-of-scope-off-format-citation"]
        )
        .stdout,
        format!(
            "grund.toml is read as version 2.\n\nout-of-scope-off-format-citation\nUnder --full, \
             a file outside the default scope writes a marked citation its kind's ID format does \
             not spell and nothing declares.\n\nSeverity: error, fixed.\nOn: when --full is \
             passed (a flag)\n{fix}"
        )
    );
}
