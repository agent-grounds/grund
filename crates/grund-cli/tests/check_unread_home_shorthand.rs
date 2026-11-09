//! §FS-check.2.4: a number-only shorthand matching no declaration earns no finding while this
//! run's own `error:` lines hold a failure at or under the home of its kind's `fetch`, as a full
//! ID absent from that home does (§FS-check.3.13.3).
//!
//! The case is an e2e fixture held out of the sweep while it is red, because the sweep has no
//! per-case skip and the commit hook runs it. The fix moves every directory under
//! `tests/e2e/held-scan-failure-wording/` to `tests/e2e/cases/` and deletes this file.

use std::path::PathBuf;

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind, assert_every_case_passed, run_case};

#[test]
#[ignore = "red until the shorthand pass withholds the unknown form; the fix removes this"]
fn a_shorthand_matching_nothing_in_an_unread_home_withholds() {
    let name = "check-unread-home-shorthand-withholds";
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let case = root.join("tests/e2e/held-scan-failure-wording").join(name);
    assert_every_case_passed(name, &[run_case(&root, &case, CaseKind::E2e)]);
}
