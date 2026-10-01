//! Black-box contract for the declaration a chapter-scoped citation rule cannot
//! reach (§FS-rules.checks.unreached-declaration) and for the ramp that carries
//! it on the warnings channel until `0.16.0` (§FS-rules.7.7).

use super::support::{absent_chapter, assert_run, run};

const RAMP_RELEASE: &str = "0.16.0";

fn version(text: &str) -> Vec<u32> {
    text.trim_end_matches("-dev")
        .split('.')
        .map(|part| part.parse::<u32>().expect("numeric version"))
        .collect()
}

/// §FS-rules.7.7: the warning promises its own promotion, so the release it
/// names is held ahead of the running version — the bump that reaches `0.16.0`
/// fails here rather than shipping a message the binary is already past
/// (§FS-distribution.4.2.1).
#[test]
fn the_ramp_release_is_ahead_of_the_running_version() {
    assert!(
        version(env!("CARGO_PKG_VERSION")) < version(RAMP_RELEASE),
        "this tree reached {RAMP_RELEASE}; land the promotion instead of \
         shipping the warning past its deadline"
    );
}

/// §FS-rules.7.7: at the recommended level the absence follows the rule's own
/// level. It is a suggestion, so it is invisible without `--suggestions` and
/// carries no ramp clause when it is visible — a suggestion never moves the exit
/// status at any release, so it owes no promotion.
#[test]
fn a_recommended_chapter_rule_reports_the_absence_as_a_suggestion() {
    let sentence = "The requirements chapter of each FS should cite at least one REQ.";
    let root = absent_chapter("unreached-recommended");

    let quiet = ["check", ".", "--rule", sentence, "--format", "json"];
    assert_run(&run(&root, &quiet), 0, "", "");

    let shown = [
        "check",
        ".",
        "--rule",
        sentence,
        "--suggestions",
        "--format",
        "json",
    ];
    assert_run(
        &run(&root, &shown),
        0,
        "{\"channel\":\"suggestion\",\"path\":\"docs/fs/FS-demo.md\",\"line\":1,\
         \"code\":\"unreached-declaration\",\
         \"message\":\"FS-demo has no requirements chapter, so --rule cannot reach it; \
         add the chapter, or narrow the rule to the declarations that have one\",\
         \"sites\":null,\"authority\":[\"--rule\"]}\n",
        "",
    );
}

/// §FS-rules.checks.unreached-declaration: the code joins the public selector
/// vocabulary, so `--ignore unreached-declaration` is the opt-out for a
/// repository that wants the previous silence through the ramp window
/// (§FS-rules.7.6).
#[test]
fn the_absence_is_selectable_like_every_other_code() {
    let sentence = "The requirements chapter of each FS must cite at least one REQ.";
    let root = absent_chapter("unreached-selectable");
    let ignored = [
        "check",
        ".",
        "--rule",
        sentence,
        "--ignore",
        "unreached-declaration",
        "--format",
        "json",
    ];
    assert_run(&run(&root, &ignored), 0, "", "");
}
