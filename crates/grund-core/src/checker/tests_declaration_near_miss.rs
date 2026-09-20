//! Test module: the declaration near-miss warning (§FS-check.4.6) — a heading
//! that opens like a declaration and parses as none.

use std::fs;
use std::path::PathBuf;

use crate::api::{FmtOpts, check, format_references, show};
use crate::queries::{ShowMode, ShowOpts};
use crate::testing::{check_run, codes, findings, only, test_root, write};

fn near_miss_repo(name: &str, heading: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
             [scan]\ninclude = [\"docs\"]\n",
    );
    write(&root.join("docs/spec.md"), heading);
    root
}

/// §FS-check.4.6.5 / §RM-off-grammar-declaration-error: the classic stumble
/// stays a warning with a named deadline before 0.15.0.
#[test]
fn off_grammar_heading_missing_the_number_is_reported() {
    let root = near_miss_repo(
        "a_heading_missing_the_number_is_reported",
        "# FS-login: Users can log in\n\nBody.\n",
    );
    let run = check_run(&root, false);
    let finding = only(&run, "declaration-near-miss");
    assert_eq!(
        finding.message,
        "`FS-login` resolves for compatibility but does not match \
             [id] format = \"{kind}-{number}-{slug}\" — rename it or change the \
             effective format; this warning becomes an error in grund 0.15.0"
    );
    assert_eq!(finding.line, Some(1));

    let version = |text: &str| {
        text.trim_end_matches("-dev")
            .split('.')
            .map(|part| part.parse::<u32>().expect("numeric version"))
            .collect::<Vec<_>>()
    };
    assert!(
        version(env!("CARGO_PKG_VERSION")) < version("0.15.0"),
        "this tree reached 0.15.0; land §RM-off-grammar-declaration-error \
             instead of shipping the warning past its deadline"
    );
}

/// §FS-config.3.2.5 / §FS-check.1.1.1 / §FS-show.1.1: a mismatch remains a
/// readable declaration, and only an exact marked candidate backed by that
/// catalog entry is promoted. Bare and unbacked malformed tokens stay text.
#[test]
fn an_off_grammar_declaration_and_its_exact_marked_citation_remain_readable() {
    let root = near_miss_repo(
        "an_off_grammar_declaration_and_its_exact_marked_citation_remain_readable",
        "# FS-security-providers: Security providers\n\nLead.\n\n\
             ## 1. Contract\n\nStable.\n",
    );
    write(
        &root.join("docs/notes.md"),
        "Backed \u{a7}FS-security-providers.1.\n\
             Bare FS-security-providers is prose.\n\
             Unbacked \u{a7}FS-not-declared stays text.\n",
    );

    let shown = show(
        "FS-security-providers.1",
        ShowOpts {
            path: root.clone(),
            mode: ShowMode::Full,
            ..ShowOpts::default()
        },
    )
    .expect("exact persisted declaration resolves");
    assert!(shown.body.contains("Stable."), "{}", shown.body);

    let report = check(&root).expect("check fixture");
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(
        report
            .warnings
            .iter()
            .filter(|finding| finding.code == "declaration-near-miss")
            .count(),
        1
    );
    assert!(
        report
            .warnings
            .iter()
            .all(|finding| !finding.message.contains("never cited")),
        "the backed citation must count as inbound use: {:?}",
        report.warnings
    );
}

/// §FS-check.4.6.1 read from the other side: a heading that *does* match
/// gets none.
#[test]
fn a_heading_that_matches_is_not_reported() {
    let root = near_miss_repo(
        "a_heading_that_matches_is_not_reported",
        "# FS-001-login: Users can log in\n\nBody.\n",
    );
    let run = check_run(&root, false);
    assert!(
        !codes(&run).contains(&"declaration-near-miss".to_string()),
        "a declaration is not a near miss: {:?}",
        findings(&run)
    );
}

/// §FS-config.3.2.5 / §FS-check.4.6: both recognition and the displayed
/// template come from the candidate kind's effective grammar. A persisted
/// spelling accepted only by the repository default remains readable when
/// that kind's authoritative override rejects it.
#[test]
fn off_grammar_kind_override_owns_its_near_miss_shape_and_message() {
    let root = test_root("a_kind_override_owns_its_near_miss_shape_and_message");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\
             [id]\nformat = \"{kind}-{slug}\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\nindex = false\n\n\
             [[kinds]]\nkind = \"TICKET\"\nfile = \"docs/tickets.md\"\n\
             format = \"{kind}_{number}\"\n\n\
             [scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/tickets.md"),
        "# Tickets\n\n## TICKET-old: Persisted default-shaped ticket\n\n\
             Ticket body cites \u{a7}TICKET-old.\n",
    );

    let shown = show(
        "TICKET-old",
        ShowOpts {
            path: root.clone(),
            mode: ShowMode::Full,
            ..ShowOpts::default()
        },
    )
    .expect("persisted override-rejected declaration resolves");
    assert!(shown.body.contains("Ticket body"), "{}", shown.body);

    let run = check_run(&root, false);
    assert!(run.report.errors.is_empty(), "got {:?}", findings(&run));
    let near_misses = run
        .report
        .warnings
        .iter()
        .filter(|finding| finding.code == "declaration-near-miss")
        .collect::<Vec<_>>();
    assert_eq!(near_misses.len(), 1, "got {:?}", findings(&run));
    assert_eq!(near_misses[0].line, Some(3));
    assert_eq!(
        near_misses[0].message,
        "`TICKET-old` resolves for compatibility but does not match \
             [id] format = \"{kind}_{number}\" — rename it or change the \
             effective format; this warning becomes an error in grund 0.15.0"
    );
}

/// §FS-check.4.6.2: the declaration colon is the discriminator. The same
/// ID-shaped token opens the same heading in both halves here; the one written
/// `<KIND>-…: <title>` is the near miss and the one without a colon is not.
/// A line opening with an ID-shaped token and no colon is prose far more often
/// than it is a declaration attempt — a comment wrapped across lines whose
/// continuation begins with one is the case that proved it — so the rule reads
/// exactly the shape a declaration attempt has and says nothing about the rest.
/// A near miss written without a title is therefore not reported: that is the
/// cost, and it buys a rule that stays quiet on prose.
#[test]
fn an_off_grammar_heading_without_a_colon_is_not_a_near_miss() {
    let with_colon = near_miss_repo(
        "an_off_grammar_heading_with_a_colon_is_a_near_miss",
        "# FS-login: Users can log in\n\nBody.\n",
    );
    assert_eq!(
        only(&check_run(&with_colon, false), "declaration-near-miss").line,
        Some(1),
        "the colon-bearing half is the near miss this one is measured against"
    );

    let without_colon = near_miss_repo(
        "an_off_grammar_heading_without_a_colon_is_not_a_near_miss",
        "# FS-login Users can log in\n\nBody.\n",
    );
    let run = check_run(&without_colon, false);
    assert!(
        !codes(&run).contains(&"declaration-near-miss".to_string()),
        "no colon, so not the shape a declaration attempt has: {:?}",
        findings(&run)
    );
}

/// §FS-check.4.6.3: never in inline code, prose, or a fenced block. One fixture
/// holds all three exempt contexts — a heading whose token is stopped by a
/// backtick, a bare `<KIND>-<slug>: <title>` line in Markdown prose, and the
/// same heading inside a fence — and none of them is read as a near miss,
/// because the position rules are the declaration rules exactly.
#[test]
fn an_off_grammar_heading_is_never_read_in_code_prose_or_a_fence() {
    let root = near_miss_repo(
        "an_off_grammar_heading_is_never_read_in_code_prose_or_a_fence",
        "# `FS-login`: The token stops at the backtick\n\n\
             FS-login: a bare line in Markdown prose is not a declaration.\n\n\
             ```\n# FS-login: Fenced, so it is an illustration\n```\n",
    );
    let run = check_run(&root, false);
    assert!(
        !codes(&run).contains(&"declaration-near-miss".to_string()),
        "inline code, prose and a fence are all exempt: {:?}",
        findings(&run)
    );
}

/// §FS-check.4.6.6: no opt-out, no rewrite. `fmt` proposes nothing on a near
/// miss and a write leaves the heading byte-identical, so migration stays the
/// repository author's choice; and the one line-oriented directive grund has,
/// the `fmt` off/on region, does not suppress the finding either — the position
/// and colon rules are the whole of what bounds recognition.
#[test]
fn an_off_grammar_heading_is_neither_rewritten_nor_suppressible() {
    let root = near_miss_repo(
        "an_off_grammar_heading_is_neither_rewritten_nor_suppressible",
        "# FS-login: Users can log in\n\nBody.\n",
    );
    let heading = root.join("docs/spec.md");
    let before = fs::read_to_string(&heading).expect("read the near-miss file");

    let dry = format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        ..FmtOpts::default()
    })
    .expect("fmt --check the near miss");
    assert!(
        dry.changes.is_empty(),
        "there is no automatic rewrite to propose: {:?}",
        dry.changes
    );

    let written = format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        ..FmtOpts::default()
    })
    .expect("fmt --write the near miss");
    assert!(written.changes.is_empty(), "{:?}", written.changes);
    assert_eq!(
        before,
        fs::read_to_string(&heading).expect("read the file back"),
        "a write leaves the heading byte-identical"
    );

    let guarded = near_miss_repo(
        "an_off_grammar_heading_is_not_suppressed_by_a_fmt_region",
        "<!-- grund:fmt off -->\n\n# FS-login: Users can log in\n\nBody.\n\n\
             <!-- grund:fmt on -->\n",
    );
    assert_eq!(
        only(&check_run(&guarded, false), "declaration-near-miss").line,
        Some(3),
        "the fmt directive region silences rewrites, not this warning"
    );
}
