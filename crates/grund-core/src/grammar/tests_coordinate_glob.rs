//! Test module: the coordinate-glob reader and matcher (§FS-coordinate-globs.grammar,
//! §FS-coordinate-globs.matching). The e2e cases pin each refusal's line and each
//! selection on a tree; these pin the reader's boundaries one input at a time, where a
//! fixture tree per boundary would hide which rule a case is about.

use super::*;
use crate::config::{DEFAULT_MARKER, DEFAULT_SECTION_SEPARATOR};

/// The launch stage, and the stage of a project with the built-in marker and separator.
const LAUNCH: GlobStage<'static> = GlobStage::new(&[DEFAULT_MARKER], &[DEFAULT_SECTION_SEPARATOR]);

/// The refusal a `.`-separated project makes: the reading before any config, then its
/// stage's checks, which wait for the config (§FS-coordinate-globs.errors.shape).
fn refused(pattern: &str) -> GlobRefusal {
    CoordinateGlob::parse(pattern, LAUNCH)
        .and_then(|glob| glob.check(LAUNCH))
        .expect_err("the pattern is refused")
}

/// The checks of a project with this outer separator and the built-in marker.
fn check_under(glob: &CoordinateGlob, separator: &str) -> Result<(), GlobRefusal> {
    glob.check(GlobStage::new(&[DEFAULT_MARKER], &[separator]))
}

fn line(pattern: &str) -> String {
    refused(pattern).to_string()
}

fn glob(pattern: &str) -> CoordinateGlob {
    CoordinateGlob::parse(pattern, LAUNCH).unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn matches(pattern: &str, separator: &str, id: &str, section: Option<&str>) -> bool {
    glob(pattern).matcher(separator).matches(id, section)
}

/// §FS-coordinate-globs.grammar: the body's operators are the citation guard's, read from
/// one table, so a token the guard reads whole as a pattern is one `--glob` reads too.
#[test]
fn operator_table_is_shared_with_glob_candidate() {
    for operator in GlobOperator::ALL {
        let written = format!(
            "FS-x{}a{}",
            operator.opener(),
            operator.closer().map(String::from).unwrap_or_default()
        );
        assert_eq!(
            read_loose_candidate(&written, 0),
            CandidateReading::Pattern(written.len()),
            "the guard reads {written} whole"
        );
        assert_eq!(glob(&written).literal_prefix("."), "FS-x", "{written}");
        assert_eq!(GlobOperator::of(operator.opener()), Some(operator));
    }
    for ch in ['!', '^', ']', '}', ',', '-'] {
        assert_eq!(GlobOperator::of(ch), None, "{ch:?} is not an operator");
    }
    // §FS-citations.recognized.globs: a `{` before the separator opens a group for both.
    let brace = "FS-x{.1,.2}";
    assert_eq!(
        read_loose_candidate(brace, 0),
        CandidateReading::Pattern(brace.len())
    );
    assert_eq!(glob(brace).literal_prefix("."), "FS-x");
}

/// §FS-coordinate-globs.grammar.globstar: a run of two or more stars is a whole component
/// or it is refused, rewritten to `*` (with the subtree beside it at the end) or to `**`.
#[test]
fn star_run_must_be_a_whole_component() {
    assert_eq!(
        line("FS-**"),
        "invalid coordinate glob \"FS-**\" at column 4: \"**\" must be a whole component; \
         accepted globs: \"FS-*\" or \"FS-*{,.**}\""
    );
    assert_eq!(
        line("FS-**.terms"),
        "invalid coordinate glob \"FS-**.terms\" at column 4: \"**\" must be a whole \
         component; accepted glob: \"FS-*.terms\""
    );
    assert_eq!(
        line("FS-config.***"),
        "invalid coordinate glob \"FS-config.***\" at column 11: \"**\" must be a whole \
         component; accepted glob: \"FS-config.**\""
    );
    // A run whole in one alternative but not in another is refused.
    assert_eq!(refused("X.**{,a}").column, 3);
    for valid in ["**", "**.terms", "X.**.1", "X{,.**}", "FS-*{,.*}"] {
        glob(valid);
    }
}

/// §FS-coordinate-globs.errors.reasons: an empty component is refused where it is closed,
/// and rewritten only where dropping one `.` mends it.
#[test]
fn empty_component_is_refused_and_rewritten_where_one_dot_mends_it() {
    assert_eq!(
        line("FS-config..requirements"),
        "invalid coordinate glob \"FS-config..requirements\" at column 11: empty component; \
         accepted glob: \"FS-config.requirements\""
    );
    assert_eq!(
        line("FS-config."),
        "invalid coordinate glob \"FS-config.\" at column 11: empty component; accepted \
         glob: \"FS-config\""
    );
    assert_eq!(
        line(""),
        "invalid coordinate glob \"\" at column 1: empty component"
    );
    assert_eq!(refused(".FS-x").accepted, ["FS-x"]);
    // Dropping one dot leaves `a..b` still empty, so nothing is offered.
    assert!(refused("a...b").accepted.is_empty());
}

/// §FS-coordinate-globs.grammar.braces: a `{` inside a group is refused at itself.
#[test]
fn braces_do_not_nest() {
    let refusal = refused("FS-{a,{b,c}}");
    assert_eq!(
        (refusal.column, refusal.reason.as_str()),
        (7, "braces do not nest")
    );
    assert!(refusal.accepted.is_empty());
    // Outside a group, `,` and `}` are ordinary characters.
    assert!(matches("a,b}", ".", "a,b}", None));
}

/// §FS-coordinate-globs.errors.reasons: an unclosed bracket points at its opener.
#[test]
fn unclosed_brace_and_class_point_at_the_opener() {
    assert_eq!(
        line("FS-{a,b"),
        "invalid coordinate glob \"FS-{a,b\" at column 4: \"{\" is never closed"
    );
    assert_eq!(
        line("FS-[a-z"),
        "invalid coordinate glob \"FS-[a-z\" at column 4: \"[\" is never closed"
    );
    // A leading `]` is a member, so `[]` and `[!]` are still open.
    assert_eq!(refused("x[]").column, 2);
    assert_eq!(refused("x[!]").column, 2);
}

/// §FS-coordinate-globs.grammar.braces: `{N..M}` is a range, which a group never takes.
#[test]
fn braces_take_no_range() {
    assert_eq!(
        line("FS-{1..3}"),
        "invalid coordinate glob \"FS-{1..3}\" at column 4: braces take no range; list each \
         alternative"
    );
    // With a comma it is two alternatives, the first of them holding an empty component.
    assert_eq!(refused("FS-x.{1..3,4}").reason, "empty component");
}

/// §FS-coordinate-globs.errors.reasons: a backwards range offers the forward one.
#[test]
fn backwards_class_range_names_the_forward_one() {
    assert_eq!(
        line("FS-[z-a]*"),
        "invalid coordinate glob \"FS-[z-a]*\" at column 5: the range \"z-a\" runs \
         backwards; accepted glob: \"FS-[a-z]*\""
    );
}

/// §FS-coordinate-globs.grammar.operators: a `\` that ends the pattern escapes nothing.
#[test]
fn dangling_escape_escapes_nothing() {
    assert_eq!(
        line("FS-a\\"),
        "invalid coordinate glob \"FS-a\\\" at column 5: \"\\\" escapes nothing"
    );
    assert_eq!(refused("FS-[a\\").column, 6);
    assert!(matches("FS-\\*", ".", "FS-*", None));
    assert!(!matches("FS-\\*", ".", "FS-x", None));
}

/// §FS-coordinate-globs.grammar.components: the qualifier is `*` or one exact alias.
#[test]
fn qualifier_is_one_alias_or_star() {
    assert_eq!(
        line("group/*/FS-*"),
        "invalid coordinate glob \"group/*/FS-*\" at column 7: a project qualifier is one \
         alias or \"*\"; accepted glob: \"*/FS-*\""
    );
    assert_eq!(glob("*/FS-*").qualifier(), &GlobQualifier::Every);
    assert_eq!(
        glob("group/alpha/FS-*").qualifier(),
        &GlobQualifier::Alias("group/alpha".to_string())
    );
    assert_eq!(glob("FS-*").qualifier(), &GlobQualifier::Current);
    assert_eq!(
        glob("FS-\\/x").qualifier(),
        &GlobQualifier::Current,
        "an escaped `/` ends no qualifier"
    );
}

/// §FS-coordinate-globs.errors.marker: a pattern opening with the marker is refused at
/// column 1, and the accepted glob is the pattern without it.
#[test]
fn citation_marker_is_refused() {
    // `\u{a7}` is the marker, spelled so this file cites nothing.
    assert_eq!(
        line("\u{a7}FS-*"),
        "invalid coordinate glob \"\u{a7}FS-*\" at column 1: coordinate globs do not take a \
         citation marker; accepted glob: \"FS-*\""
    );
    assert!(glob("FS-*").check(LAUNCH).is_ok());
}

/// §FS-coordinate-globs.errors.shape: a pattern is refused in two stages, what the grammar
/// alone decides before discovery and its components once the separator is known, and the
/// first stage that refuses it is reported; within a stage, the leftmost refusal is.
#[test]
fn first_refused_pattern_wins() {
    assert_eq!(refused("FS-**.a..b").column, 4);
    assert_eq!(refused("a..b.**x").column, 3);
    assert_eq!(refused("group/*/FS-{a").column, 7);
    // The unclosed brace is the grammar's, so it is reported before the `**` left of it.
    let refusal = refused("FS-**.{a,b");
    assert_eq!(
        (refusal.column, refusal.reason.as_str()),
        (7, "\"{\" is never closed")
    );
    // A refusal under any alternative refuses the whole pattern.
    assert_eq!(refused("FS-x{.a,..b}").reason, "empty component");
}

/// §FS-coordinate-globs.errors.shape: a component check waits for the config, because the
/// loader takes any literal but `/` as a separator under some config (§FS-config.3.2.2), and
/// a pattern refused under `.` can be valid under it. A pattern with no literal reads the
/// same under every separator, and the grammar's own refusals never wait.
#[test]
fn separator_checks_wait_for_the_config() {
    for (pattern, separator) in [("**_1", "_"), ("**-1", "-"), ("**X1", "X"), ("FS-**", "-")] {
        let read = glob(pattern);
        assert!(
            check_under(&read, separator).is_ok(),
            "{pattern} under {separator}"
        );
        assert!(check_under(&read, ".").is_err(), "{pattern} under .");
    }
    assert!(CoordinateGlob::parse("**..*", LAUNCH).is_err());
    assert!(CoordinateGlob::parse("FS-{a,b", LAUNCH).is_err());
}

/// §FS-coordinate-globs.errors.shape, §FS-rule-refusals.requirements.typed: every glob a
/// refusal offers is accepted when it is pasted back, read through the same stages as the
/// pattern it mends — one refused at launch as the `.`-separated reading with the built-in
/// marker, one refused once the config is read under that stage's markers and separators.
#[test]
fn every_offer_is_accepted_when_pasted_back() {
    let marked = |rest: &str| format!("{DEFAULT_MARKER}{rest}");
    let mut patterns: Vec<String> = [
        // The rows of §FS-coordinate-globs.errors.reasons, and the empty pattern.
        "FS-**",
        "FS-**.terms",
        "FS-config.***",
        "FS-config..requirements",
        "FS-config.",
        "FS-{a,{b,c}}",
        "FS-{a,b",
        "FS-[a-z",
        "FS-{1..3}",
        "FS-[z-a]*",
        "FS-a\\",
        "group/*/FS-*",
        "charlie/FS-*",
        "",
        // Offers whose rewrite is refused for another reason.
        "group/*/FS-**",
        "FS-[z-a]**",
        "FS-[z-a]*.",
        "*/@FS-**",
        "@FS-[z-a]*",
        "**_.1",
        "FS-x.{1..3,4}",
    ]
    .map(String::from)
    .into();
    patterns.extend(["FS-*", "FS-**", "FS-[z-a]*", "FS-x..y"].map(marked));
    let stages = [
        LAUNCH,
        GlobStage::new(&[DEFAULT_MARKER], &["#"]),
        GlobStage::new(&[DEFAULT_MARKER], &["_"]),
        GlobStage::new(&[DEFAULT_MARKER, "@"], &[".", "-"]),
    ];
    let mut offers = 0;
    for pattern in &patterns {
        for stage in stages {
            let (refusal, pasted_under) = match CoordinateGlob::parse(pattern, LAUNCH) {
                Err(refusal) => (refusal, LAUNCH),
                Ok(read) => match read.check(stage) {
                    Err(refusal) => (refusal, stage),
                    Ok(()) => continue,
                },
            };
            for offer in &refusal.accepted {
                let pasted =
                    CoordinateGlob::parse(offer, LAUNCH).and_then(|glob| glob.check(pasted_under));
                assert!(
                    pasted.is_ok(),
                    "{pattern:?} under {stage:?} offers {offer:?}, which is refused: {}",
                    pasted.unwrap_err()
                );
                offers += 1;
            }
        }
    }
    assert!(offers >= 50, "only {offers} offers were pasted back");
    // An offer its stage refuses is not made; the pinned ones stand.
    for pattern in [marked("FS-**"), "group/*/FS-**".into(), "FS-[z-a]**".into()] {
        assert!(refused(&pattern).accepted.is_empty(), "{pattern}");
    }
    assert!(refused(&marked("FS-[z-a]*")).accepted.is_empty());
    assert_eq!(refused("group/*/FS-*").accepted, ["*/FS-*"]);
    assert_eq!(refused(&marked("FS-*")).accepted, ["FS-*"]);
    assert_eq!(refused("FS-[z-a]*").accepted, ["FS-[a-z]*"]);
    assert_eq!(refused("FS-**").accepted, ["FS-*", "FS-*{,.**}"]);
}

/// §FS-coordinate-globs.errors.shape: once the config is read, the leftmost refusal under
/// any project the pattern reads is reported, its markers before its components.
#[test]
fn config_stage_reports_the_leftmost_refusal() {
    // Under `.` the `**x` at column 9 is refused; under `_` the empty component at 6.
    let read = glob("FS-a_.b.**x");
    for separators in [[".", "_"], ["_", "."]] {
        let refusal = read
            .check(GlobStage::new(&[DEFAULT_MARKER], &separators))
            .expect_err("refused under both");
        assert_eq!(
            (refusal.column, refusal.reason.as_str()),
            (6, "empty component")
        );
    }
    let refusal = glob("@FS-**")
        .check(GlobStage::new(&[DEFAULT_MARKER, "@"], &["."]))
        .expect_err("a marked pattern");
    assert_eq!(
        (refusal.column, refusal.reason.as_str()),
        (1, "coordinate globs do not take a citation marker")
    );
}

/// §FS-coordinate-globs.matching.anchored: `?` and a class each match one code point.
#[test]
fn question_and_classes_match_one_code_point() {
    assert!(matches("FS-caf?", ".", "FS-café", None));
    assert!(!matches("FS-caf?", ".", "FS-cafés", None));
    assert!(matches("FS-caf[à-ê]", ".", "FS-café", None));
    assert!(!matches("FS-caf[à-ê]", ".", "FS-cafe", None));
    assert!(matches("FS-c[!o]*", ".", "FS-check", None));
    assert!(!matches("FS-c[^o]*", ".", "FS-cover", None));
    assert!(matches("FS-[]x]", ".", "FS-]", None));
    assert!(matches("FS-[a-]", ".", "FS--", None));
    assert!(!matches("FS-?", ".", "FS-ab", None), "anchored");
    assert!(!matches("fs-*", ".", "FS-a", None), "case-sensitive");
}

/// §FS-coordinate-globs.grammar.globstar: `**` matches zero or more whole components where
/// it leads or stands between two, and one or more where it ends the pattern.
#[test]
fn globstar_matches_zero_or_more_whole_components() {
    let section = |path| Some(path);
    assert!(matches("**", ".", "FS-x", None));
    assert!(matches("**", ".", "FS-x", section("1.2")));
    assert!(matches("**.terms", ".", "FS-x", section("terms")));
    assert!(matches("**.terms", ".", "FS-x", section("a.terms")));
    assert!(!matches("**.terms", ".", "FS-x", section("aterms")));
    assert!(matches("X.**.1", ".", "X", section("1")));
    assert!(matches("X.**.1", ".", "X", section("a.b.1")));
    assert!(!matches("X.**.1", ".", "X", section("10")));
    assert!(
        !matches("X.**", ".", "X", None),
        "an ending `**` needs one component"
    );
    assert!(matches("X.**", ".", "X", section("a")));
    assert!(matches("X{,.**}", ".", "X", None));
    assert!(matches("X{,.**}", ".", "X", section("a.b")));
    assert!(
        !matches("X.*", ".", "X", section("a.b")),
        "`*` stays in one component"
    );
}

/// §FS-coordinate-globs.matching.bounded: a group product is walked, never expanded.
#[test]
fn brace_product_is_never_expanded() {
    let pattern = format!("FS-{}*", "{a,b}".repeat(24));
    let started = std::time::Instant::now();
    let glob = glob(&pattern);
    let matcher = glob.matcher(".");
    assert!(matcher.matches(&format!("FS-{}z", "ab".repeat(12)), None));
    assert!(!matcher.matches("FS-check", Some("terms")));
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
}

/// §FS-coordinate-globs.grammar.components, §FS-coordinate-globs.workspace.escapes: under
/// `#` the first unescaped `#` ends the ID atom, with none the first unescaped `.` does,
/// and an escaped dot stays in the ID.
#[test]
fn hash_separator_splits_and_escaped_dots_stay_in_the_id() {
    let (id, section) = ("RFC.042.draft", Some("3.1"));
    assert!(matches("RFC.042.draft#3.1", "#", id, section));
    assert!(matches("RFC\\.042\\.draft.3.1", "#", id, section));
    assert!(!matches("RFC.042.draft.3.1", "#", id, section));
    assert!(matches("RFC.*#3{,.**}", "#", id, Some("3")));
    assert!(matches("RFC.*#3{,.**}", "#", id, section));
    // `#` is an ordinary character where the separator is `.`.
    assert!(!matches("FS-config#terms", ".", "FS-config", Some("terms")));
    assert!(matches("FS-config#terms", ".", "FS-config#terms", None));
    // Each alternative is split on its own.
    assert!(matches("RFC.042.draft{#3,.x}", "#", id, Some("3")));
    // A pattern refused under `.` can be valid where `#` ends the atom.
    assert!(check_under(&glob("RFC..x#3"), "#").is_ok());
    assert!(check_under(&glob("RFC..x#3"), ".").is_err());
    assert_eq!(glob("RFC.042#3").literal_prefix("#"), "RFC.042");
    assert_eq!(glob("RFC.042#3").literal_prefix("."), "RFC");
}
