//! Black-box contract for the declaration a chapter-scoped citation rule cannot
//! reach (§FS-rules.checks.unreached-declaration), which is an error on the
//! ordinary `must` channel from `0.16.0` (§FS-rules.7).
//!
//! The ramp that carried it on the warnings channel until then held its own
//! deadline here, as the pending half of a ramp may
//! (§FS-distribution.4.2.1). The landed half has no unit test to hold: what
//! refuses a cut below `0.16.0` now is `scripts/check_release_ramps.py`, on
//! every publication path (§FS-distribution.4.2.6).

use super::support::{absent_chapter, assert_run, run};

/// §FS-rules.checks.unreached-declaration: at the recommended level the
/// absence follows the rule's own level. It is a suggestion, so it is
/// invisible without `--suggestions` and carries no landed clause when it is
/// visible — a suggestion never moved the exit status at any release, so it
/// owed no promotion.
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
/// vocabulary, so `--ignore unreached-declaration` drops the finding and the
/// exit code with it, for a repository that wants the absence reported by
/// nothing (§FS-rules.7.6).
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
